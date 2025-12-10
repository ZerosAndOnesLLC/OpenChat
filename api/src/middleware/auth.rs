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
use uuid::Uuid;

use crate::{
    cache::{self, metrics::{CacheType, record_hit, record_miss}},
    config::Config,
    db,
    errors::ApiError,
    models::{organization::Organization, user::User},
    services::{device_token::verify_device_token, tv_api::TvApiClient},
};

/// Verified token claims that can come from either TitaniumVault or device tokens
#[derive(Clone)]
pub struct VerifiedClaims {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub org_name: String,
    pub roles: Vec<String>,
    /// Whether this came from a device token (vs TitaniumVault token)
    pub is_device_token: bool,
}

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

            // Get Redis connection
            let redis_conn = req
                .app_data::<actix_web::web::Data<MultiplexedConnection>>()
                .ok_or_else(|| {
                    error!("Redis connection not found in app data");
                    actix_web::error::ErrorInternalServerError("Redis not configured")
                })?;

            let mut redis = redis_conn.as_ref().clone();

            // Get database pool (needed for device token user lookup)
            let pool = req
                .app_data::<actix_web::web::Data<PgPool>>()
                .ok_or_else(|| {
                    error!("Database pool not found in app data");
                    actix_web::error::ErrorInternalServerError("Database pool not configured")
                })?;

            // Get config (needed for device token verification)
            let config = req
                .app_data::<actix_web::web::Data<Config>>()
                .ok_or_else(|| {
                    error!("Config not found in app data");
                    actix_web::error::ErrorInternalServerError("Config not configured")
                })?;

            // Try to get cached token claims first (only for TitaniumVault tokens)
            let verified_claims: VerifiedClaims = match cache::tokens::get_cached_token_claims(&mut redis, &token)
                .await
                .ok()
                .flatten()
            {
                Some(claims) => {
                    debug!("Token cache hit");
                    record_hit(&mut redis, CacheType::Tokens).await;
                    VerifiedClaims {
                        user_id: claims.user_id,
                        org_id: claims.org_id,
                        email: claims.email,
                        display_name: claims.display_name,
                        org_name: claims.org_name,
                        roles: claims.roles,
                        is_device_token: false,
                    }
                }
                None => {
                    record_miss(&mut redis, CacheType::Tokens).await;

                    // Try TitaniumVault token first
                    match tv_api_client.verify_token(&token).await {
                        Ok(claims) => {
                            debug!("TitaniumVault token verified");
                            // Cache the token claims for 5 minutes
                            if let Err(e) = cache::tokens::cache_token_claims(&mut redis, &token, &claims, 300).await {
                                error!("Failed to cache token claims: {}", e);
                            }
                            VerifiedClaims {
                                user_id: claims.user_id,
                                org_id: claims.org_id,
                                email: claims.email,
                                display_name: claims.display_name,
                                org_name: claims.org_name,
                                roles: claims.roles,
                                is_device_token: false,
                            }
                        }
                        Err(_) => {
                            // Try device token verification
                            debug!("TitaniumVault verification failed, trying device token");
                            let device_claims = verify_device_token(&token, &config.jwt_secret)
                                .map_err(|e| {
                                    error!("Device token verification failed: {}", e);
                                    actix_web::error::ErrorUnauthorized(ApiError::Authentication(
                                        "Invalid token".to_string(),
                                    ))
                                })?;

                            // Look up user from database to get email/display_name
                            let user = User::get_by_tv_user_id(pool.get_ref(), device_claims.sub)
                                .await
                                .map_err(|e| {
                                    error!("Failed to look up user for device token: {}", e);
                                    actix_web::error::ErrorInternalServerError(e)
                                })?
                                .ok_or_else(|| {
                                    error!("User not found for device token");
                                    actix_web::error::ErrorUnauthorized(ApiError::Authentication(
                                        "User not found".to_string(),
                                    ))
                                })?;

                            debug!("Device token verified for user {}", user.id);
                            VerifiedClaims {
                                user_id: device_claims.sub,
                                org_id: device_claims.org_id,
                                email: user.email,
                                display_name: user.display_name,
                                org_name: format!("org-{}", device_claims.org_id),
                                roles: device_claims.roles,
                                is_device_token: true,
                            }
                        }
                    }
                }
            };

            // Check if user has the required role (device tokens always pass since they were validated at pairing)
            if let Some(required) = &required_role {
                if !verified_claims.roles.contains(required) {
                    error!(
                        "User {} does not have required role: {}. User roles: {:?}",
                        verified_claims.email, required, verified_claims.roles
                    );
                    return Err(actix_web::error::ErrorForbidden(ApiError::Authorization(
                        format!("Missing required role: {}", required),
                    )));
                }
            }

            // Set RLS context for this request
            db::set_rls_context(pool.get_ref(), verified_claims.org_id)
                .await
                .map_err(|e| {
                    error!("Failed to set RLS context: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                })?;

            // Check cache for organization first
            let cached_org = cache::organizations::get_org_from_cache(&mut redis, verified_claims.org_id)
                .await
                .ok()
                .flatten();

            if cached_org.is_none() {
                debug!("Organization cache miss for {}, upserting", verified_claims.org_id);
                // Create or update organization (required foreign key for users)
                let org = Organization::upsert(pool.get_ref(), verified_claims.org_id, &verified_claims.org_name)
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
                debug!("Organization cache hit for {}", verified_claims.org_id);
            }

            // For device tokens, we already looked up the user, so skip upsert logic
            if !verified_claims.is_device_token {
                // Check cache for user first
                let cached_user = cache::users::get_user_by_tv_id_from_cache(&mut redis, verified_claims.user_id)
                    .await
                    .ok()
                    .flatten();

                match cached_user {
                    Some(user) => {
                        debug!("User cache hit for {}", verified_claims.user_id);
                        // Check if user data needs updating (email or display name changed)
                        if user.email != verified_claims.email || user.display_name != verified_claims.display_name {
                            debug!("User data changed, upserting and invalidating cache");
                            let updated_user = User::upsert(
                                pool.get_ref(),
                                verified_claims.user_id,
                                &verified_claims.org_id,
                                &verified_claims.email,
                                &verified_claims.display_name,
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
                        debug!("User cache miss for {}, upserting", verified_claims.user_id);
                        // Create or update user in database
                        let user = User::upsert(
                            pool.get_ref(),
                            verified_claims.user_id,
                            &verified_claims.org_id,
                            &verified_claims.email,
                            &verified_claims.display_name,
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
            }

            // Store claims in request extensions for handlers to use
            // Convert to TokenClaims for backwards compatibility with existing handlers
            let token_claims = crate::services::tv_api::TokenClaims {
                user_id: verified_claims.user_id,
                email: verified_claims.email,
                org_id: verified_claims.org_id,
                org_name: verified_claims.org_name,
                display_name: verified_claims.display_name,
                roles: verified_claims.roles,
            };
            req.extensions_mut().insert(token_claims);

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
