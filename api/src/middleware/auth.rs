use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use sqlx::PgPool;
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use tracing::error;

use crate::{
    db,
    errors::ApiError,
    models::user::User,
    services::tv_api::TvApiClient,
};

#[allow(dead_code)]
pub struct AuthMiddleware {
    tv_api_client: Rc<TvApiClient>,
}

impl AuthMiddleware {
    #[allow(dead_code)]
    pub fn new(tv_api_url: String) -> Self {
        Self {
            tv_api_client: Rc::new(TvApiClient::new(tv_api_url)),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            tv_api_client: self.tv_api_client.clone(),
        }))
    }
}

#[allow(dead_code)]
pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    tv_api_client: Rc<TvApiClient>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let tv_api_client = self.tv_api_client.clone();

        Box::pin(async move {
            // Extract token from Authorization header
            let token = extract_token(&req)?;

            // Verify token with TV-API
            let claims = tv_api_client
                .verify_token(&token)
                .await
                .map_err(|e| actix_web::error::ErrorUnauthorized(e))?;

            // Get database pool
            let pool = req
                .app_data::<actix_web::web::Data<PgPool>>()
                .ok_or_else(|| {
                    error!("Database pool not found in app data");
                    actix_web::error::ErrorInternalServerError("Database pool not configured")
                })?;

            // Set RLS context for this request
            db::set_rls_context(pool.get_ref(), claims.org_id)
                .await
                .map_err(|e| {
                    error!("Failed to set RLS context: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                })?;

            // Create or update user in database
            User::upsert(
                pool.get_ref(),
                claims.user_id,
                &claims.org_id,
                &claims.email,
                &claims.display_name,
            )
            .await
            .map_err(|e| {
                error!("Failed to upsert user: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            })?;

            // Store claims in request extensions for handlers to use
            req.extensions_mut().insert(claims);

            // Continue to the next service
            service.call(req).await
        })
    }
}

/// Extract JWT token from Authorization header
#[allow(dead_code)]
fn extract_token(req: &ServiceRequest) -> Result<String, Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| {
            actix_web::error::ErrorUnauthorized(ApiError::Authentication(
                "Missing Authorization header".to_string(),
            ))
        })?
        .to_str()
        .map_err(|_| {
            actix_web::error::ErrorUnauthorized(ApiError::Authentication(
                "Invalid Authorization header".to_string(),
            ))
        })?;

    if !auth_header.starts_with("Bearer ") {
        return Err(actix_web::error::ErrorUnauthorized(
            ApiError::Authentication("Authorization header must start with 'Bearer '".to_string()),
        ));
    }

    Ok(auth_header[7..].to_string())
}
