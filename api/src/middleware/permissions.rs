use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use redis::AsyncCommands;
use sqlx::PgPool;
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use tracing::error;

use crate::{
    errors::ApiError,
    models::role::user_has_permission,
    services::tv_api::TokenClaims,
};

const PERMISSION_CACHE_TTL: u64 = 300; // 5 minutes in seconds
const PERMISSION_CACHE_PREFIX: &str = "openchat:perm";

/// Build cache key for permission check (case-insensitive)
fn permission_cache_key(roles: &[String], permission: &str) -> String {
    // Normalize roles to lowercase for consistent cache keys
    let lowercase_roles: Vec<String> = roles.iter().map(|r| r.to_lowercase()).collect();
    let roles_hash = format!("{:?}", lowercase_roles);
    format!("{}:{}:{}", PERMISSION_CACHE_PREFIX, roles_hash, permission)
}

#[derive(Clone)]
pub struct PermissionMiddleware {
    required_permission: String,
}

impl PermissionMiddleware {
    /// Create a new permission middleware that requires a specific permission
    pub fn new(required_permission: String) -> Self {
        Self {
            required_permission,
        }
    }

    /// Helper to create middleware for common permissions
    pub fn require(permission: &str) -> Self {
        Self::new(permission.to_string())
    }
}

impl<S, B> Transform<S, ServiceRequest> for PermissionMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PermissionMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PermissionMiddlewareService {
            service: Rc::new(service),
            required_permission: self.required_permission.clone(),
        }))
    }
}

pub struct PermissionMiddlewareService<S> {
    service: Rc<S>,
    required_permission: String,
}

impl<S, B> Service<ServiceRequest> for PermissionMiddlewareService<S>
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
        let required_permission = self.required_permission.clone();

        Box::pin(async move {
            // Extract claims from request extensions (set by AuthMiddleware)
            let claims = req
                .extensions()
                .get::<TokenClaims>()
                .ok_or_else(|| {
                    error!("TokenClaims not found in request extensions. Ensure AuthMiddleware runs before PermissionMiddleware.");
                    actix_web::error::ErrorUnauthorized(ApiError::Authentication(
                        "Not authenticated".to_string(),
                    ))
                })?
                .clone();

            // Get database pool and Redis connection
            let pool = req
                .app_data::<actix_web::web::Data<PgPool>>()
                .ok_or_else(|| {
                    error!("Database pool not found in app data");
                    actix_web::error::ErrorInternalServerError("Database pool not configured")
                })?;

            let redis_pool = req
                .app_data::<actix_web::web::Data<redis::Client>>()
                .ok_or_else(|| {
                    error!("Redis client not found in app data");
                    actix_web::error::ErrorInternalServerError("Redis client not configured")
                })?;

            let mut redis_conn = redis_pool.get_multiplexed_tokio_connection().await.map_err(|e| {
                error!("Failed to get Redis connection: {}", e);
                actix_web::error::ErrorInternalServerError("Redis connection failed")
            })?;

            // Check permission (with caching)
            let has_perm = check_permission_cached(
                &mut redis_conn,
                pool.get_ref(),
                &claims.roles,
                &required_permission,
            )
            .await
            .map_err(|e| {
                error!("Permission check failed: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            })?;

            if !has_perm {
                error!(
                    "User {} does not have required permission: {}. User roles: {:?}",
                    claims.email, required_permission, claims.roles
                );
                return Err(actix_web::error::ErrorForbidden(ApiError::Authorization(
                    format!("Missing required permission: {}", required_permission),
                )));
            }

            // Permission granted, continue to the next service
            service.call(req).await
        })
    }
}

/// Check if user has permission, with Redis caching
async fn check_permission_cached(
    redis: &mut redis::aio::MultiplexedConnection,
    pool: &PgPool,
    roles: &[String],
    permission: &str,
) -> Result<bool, ApiError> {
    let cache_key = permission_cache_key(roles, permission);

    // Try to get from cache first
    let cached: Option<String> = redis
        .get(&cache_key)
        .await
        .map_err(|e| ApiError::Internal(format!("Redis error: {}", e)))?;

    if let Some(cached_value) = cached {
        // Cache hit
        return Ok(cached_value == "1");
    }

    // Cache miss - check database
    let has_perm = user_has_permission(pool, roles, permission).await?;

    // Store in cache
    let cache_value = if has_perm { "1" } else { "0" };
    let _: () = redis
        .set_ex(&cache_key, cache_value, PERMISSION_CACHE_TTL)
        .await
        .map_err(|e| ApiError::Internal(format!("Redis error: {}", e)))?;

    Ok(has_perm)
}

/// Helper function to invalidate permission cache when roles or permissions are updated
#[allow(dead_code)]
pub async fn invalidate_permission_cache(
    redis: &mut redis::aio::MultiplexedConnection,
) -> Result<(), ApiError> {
    // Delete all permission cache keys
    let pattern = format!("{}:*", PERMISSION_CACHE_PREFIX);
    let keys: Vec<String> = redis
        .keys(&pattern)
        .await
        .map_err(|e| ApiError::Internal(format!("Redis error: {}", e)))?;

    if !keys.is_empty() {
        let _: () = redis
            .del(&keys)
            .await
            .map_err(|e| ApiError::Internal(format!("Redis error: {}", e)))?;
    }

    Ok(())
}
