use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use redis::aio::MultiplexedConnection;
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use tracing::{error, warn};

use crate::{
    cache::rate_limit::{check_rate_limit, RateLimitType},
    services::tv_api::TokenClaims,
};

/// Rate limiting middleware
/// Applies per-user rate limits using Redis
#[derive(Clone)]
pub struct RateLimitMiddleware {
    limit_type: RateLimitType,
    enabled: bool,
}

impl RateLimitMiddleware {
    /// Create a new rate limit middleware for API requests
    pub fn api_request(enabled: bool) -> Self {
        Self {
            limit_type: RateLimitType::ApiRequest,
            enabled,
        }
    }

    /// Create a new rate limit middleware for messages
    pub fn message(enabled: bool) -> Self {
        Self {
            limit_type: RateLimitType::Message,
            enabled,
        }
    }

    /// Create a new rate limit middleware for WebSocket messages
    pub fn websocket(enabled: bool) -> Self {
        Self {
            limit_type: RateLimitType::WebSocket,
            enabled,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Rc::new(service),
            limit_type: self.limit_type.clone(),
            enabled: self.enabled,
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: Rc<S>,
    limit_type: RateLimitType,
    enabled: bool,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let limit_type = self.limit_type.clone();
        let enabled = self.enabled;

        Box::pin(async move {
            // If rate limiting is disabled, allow all requests through
            if !enabled {
                let res = service.call(req).await?;
                return Ok(res.map_into_left_body());
            }

            // Extract user claims from request extensions (set by AuthMiddleware)
            let claims = req.extensions().get::<TokenClaims>().cloned();

            let user_id = match claims {
                Some(c) => c.user_id,
                None => {
                    // If no claims, authentication hasn't run yet or failed
                    // Let the request through - auth middleware will handle it
                    warn!("Rate limit middleware: No user claims found, skipping rate limit check");
                    let res = service.call(req).await?;
                    return Ok(res.map_into_left_body());
                }
            };

            // Get Redis connection
            let redis_conn = req
                .app_data::<actix_web::web::Data<MultiplexedConnection>>()
                .ok_or_else(|| {
                    error!("Redis connection not found in app data");
                    actix_web::error::ErrorInternalServerError("Redis not configured")
                })?;

            // Clone the connection for async use
            let mut redis = redis_conn.as_ref().clone();

            // Check rate limit
            let (allowed, remaining, reset_time) =
                match check_rate_limit(&mut redis, user_id, limit_type).await {
                    Ok(result) => result,
                    Err(e) => {
                        error!("Rate limit check failed: {}", e);
                        // On error, allow the request through to avoid blocking legitimate traffic
                        warn!("Allowing request due to rate limit check error");
                        let res = service.call(req).await?;
                        return Ok(res.map_into_left_body());
                    }
                };

            if !allowed {
                // Rate limit exceeded
                warn!(
                    "Rate limit exceeded for user {}: {} requests remaining, resets in {}s",
                    user_id, remaining, reset_time
                );

                // Create error response with custom headers
                let response = HttpResponse::TooManyRequests()
                    .insert_header(("X-RateLimit-Limit", format!("{}", limit_type.config().max_requests)))
                    .insert_header(("X-RateLimit-Remaining", "0"))
                    .insert_header(("X-RateLimit-Reset", format!("{}", reset_time)))
                    .insert_header(("Retry-After", format!("{}", reset_time)))
                    .json(serde_json::json!({
                        "error": "Rate limit exceeded",
                        "message": format!("Too many requests. Please try again in {} seconds.", reset_time),
                        "retry_after": reset_time,
                    }));

                let (http_req, _) = req.into_parts();
                return Ok(ServiceResponse::new(http_req, response).map_into_right_body());
            }

            // Allow the request and add rate limit headers
            let mut res = service.call(req).await?;

            let headers = res.headers_mut();
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-ratelimit-limit"),
                actix_web::http::header::HeaderValue::from_str(&format!("{}", limit_type.config().max_requests))
                    .unwrap(),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-ratelimit-remaining"),
                actix_web::http::header::HeaderValue::from_str(&format!("{}", remaining))
                    .unwrap(),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-ratelimit-reset"),
                actix_web::http::header::HeaderValue::from_str(&format!("{}", reset_time))
                    .unwrap(),
            );

            Ok(res.map_into_left_body())
        })
    }
}
