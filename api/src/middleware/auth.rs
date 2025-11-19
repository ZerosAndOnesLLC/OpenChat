use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use tracing::{debug, error};

use crate::{
    cache, db,
    errors::ApiError,
    models::{organization::Organization, user::User},
    services::tv_api::TvApiClient,
};

#[derive(Clone)]
pub struct AuthMiddleware {
    tv_api_client: Rc<TvApiClient>,
    required_role: Option<String>,
}

impl AuthMiddleware {
    /// Create a new auth middleware that requires a specific role
    pub fn new(tv_api_url: String, required_role: Option<String>) -> Self {
        Self {
            tv_api_client: Rc::new(TvApiClient::new(tv_api_url)),
            required_role,
        }
    }

    /// Create auth middleware that requires the "openchat" role
    pub fn with_openchat_role(tv_api_url: String) -> Self {
        Self::new(tv_api_url, Some("openchat".to_string()))
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
            required_role: self.required_role.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    tv_api_client: Rc<TvApiClient>,
    required_role: Option<String>,
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
        let required_role = self.required_role.clone();

        Box::pin(async move {
            // Extract token from Authorization header
            let token = extract_token(&req)?;

            // Verify token with TV-API
            let claims = tv_api_client
                .verify_token(&token)
                .await
                .map_err(|e| actix_web::error::ErrorUnauthorized(e))?;

            // Check if user has the required role
            if let Some(required) = &required_role {
                if !claims.roles.contains(required) {
                    error!(
                        "User {} does not have required role: {}. User roles: {:?}",
                        claims.email, required, claims.roles
                    );
                    return Err(actix_web::error::ErrorForbidden(ApiError::Authorization(
                        format!("Missing required role: {}", required),
                    )));
                }
            }

            // Get database pool
            let pool = req
                .app_data::<actix_web::web::Data<PgPool>>()
                .ok_or_else(|| {
                    error!("Database pool not found in app data");
                    actix_web::error::ErrorInternalServerError("Database pool not configured")
                })?;

            // Get Redis connection
            let redis_conn = req
                .app_data::<actix_web::web::Data<MultiplexedConnection>>()
                .ok_or_else(|| {
                    error!("Redis connection not found in app data");
                    actix_web::error::ErrorInternalServerError("Redis not configured")
                })?;

            let mut redis = redis_conn.as_ref().clone();

            // Set RLS context for this request
            db::set_rls_context(pool.get_ref(), claims.org_id)
                .await
                .map_err(|e| {
                    error!("Failed to set RLS context: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                })?;

            // Check cache for organization first
            let cached_org = cache::organizations::get_org_from_cache(&mut redis, claims.org_id)
                .await
                .ok()
                .flatten();

            if cached_org.is_none() {
                debug!("Organization cache miss for {}, upserting", claims.org_id);
                // Create or update organization (required foreign key for users)
                let org = Organization::upsert(pool.get_ref(), claims.org_id, &claims.org_name)
                    .await
                    .map_err(|e| {
                        error!("Failed to upsert organization: {}", e);
                        actix_web::error::ErrorInternalServerError(e)
                    })?;

                // Cache the organization
                if let Err(e) = cache::organizations::set_org_in_cache(&mut redis, &org).await {
                    error!("Failed to cache organization: {}", e);
                }
            } else {
                debug!("Organization cache hit for {}", claims.org_id);
            }

            // Check cache for user first
            let cached_user = cache::users::get_user_by_tv_id_from_cache(&mut redis, claims.user_id)
                .await
                .ok()
                .flatten();

            match cached_user {
                Some(user) => {
                    debug!("User cache hit for {}", claims.user_id);
                    // Check if user data needs updating (email or display name changed)
                    if user.email != claims.email || user.display_name != claims.display_name {
                        debug!("User data changed, upserting and invalidating cache");
                        let updated_user = User::upsert(
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

                        // Update cache with new user data
                        if let Err(e) = cache::users::set_user_with_tv_index_in_cache(&mut redis, &updated_user).await {
                            error!("Failed to cache updated user: {}", e);
                        }
                    }
                }
                None => {
                    debug!("User cache miss for {}, upserting", claims.user_id);
                    // Create or update user in database
                    let user = User::upsert(
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

                    // Cache the user
                    if let Err(e) = cache::users::set_user_with_tv_index_in_cache(&mut redis, &user).await {
                        error!("Failed to cache user: {}", e);
                    }
                }
            }

            // Store claims in request extensions for handlers to use
            req.extensions_mut().insert(claims);

            // Continue to the next service
            service.call(req).await
        })
    }
}

/// Extract JWT token from Authorization header
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
