// Cache functions for user status
#![allow(dead_code)]

use redis::AsyncCommands;
use uuid::Uuid;

use crate::db::RedisPool;
use crate::errors::ApiResult;
use crate::handlers::user_status::UserStatusResponse;

const STATUS_CACHE_TTL: u64 = 300; // 5 minutes in seconds
const STATUS_CACHE_PREFIX: &str = "openchat:org";

/// Build cache key for a user status: openchat:org:{org_id}:user_status:{user_id}
fn status_cache_key(org_id: Uuid, user_id: Uuid) -> String {
    format!("{}:{}:user_status:{}", STATUS_CACHE_PREFIX, org_id, user_id)
}

/// Get user status from cache
pub async fn get_status(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<Option<UserStatusResponse>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching user status cache: {}", e);
            return Ok(None);
        }
    };

    let key = status_cache_key(org_id, user_id);
    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching user status cache: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            let status: UserStatusResponse = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(status))
        }
        None => Ok(None),
    }
}

/// Store user status in cache
pub async fn set_status(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
    status: &UserStatusResponse,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = status_cache_key(org_id, user_id);
    let json = serde_json::to_string(status)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, STATUS_CACHE_TTL).await?;
    Ok(())
}

/// Invalidate user status cache
pub async fn invalidate_status(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = status_cache_key(org_id, user_id);
    let _: () = conn.del(&key).await?;
    Ok(())
}

/// Invalidate all user status caches for an organization
pub async fn invalidate_org_user_status_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let pattern = format!("{}:{}:user_status:*", STATUS_CACHE_PREFIX, org_id);
    let keys: Vec<String> = conn.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = conn.del(&keys).await?;
    }

    Ok(())
}
