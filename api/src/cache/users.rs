// Cache functions for Redis integration
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::db::RedisPool;
use crate::{errors::ApiResult, models::user::User};

const USER_CACHE_TTL: u64 = 3600; // 1 hour in seconds
const USER_CACHE_PREFIX: &str = "openchat:org";

/// Build cache key for a user: openchat:org:{org_id}:user:{user_id}
fn user_cache_key(org_id: Uuid, user_id: Uuid) -> String {
    format!("{}:{}:user:{}", USER_CACHE_PREFIX, org_id, user_id)
}

/// Build cache key for TV user index: openchat:org:{org_id}:tv_user:{tv_user_id}
fn tv_user_index_key(org_id: Uuid, tv_user_id: Uuid) -> String {
    format!("{}:{}:tv_user:{}", USER_CACHE_PREFIX, org_id, tv_user_id)
}

/// Get user from cache
pub async fn get_user_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<Option<User>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching user cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    let key = user_cache_key(org_id, user_id);

    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching user cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis_pool, super::metrics::CacheType::Users).await;
            let user: User = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(user))
        }
        None => {
            super::metrics::record_miss(redis_pool, super::metrics::CacheType::Users).await;
            Ok(None)
        }
    }
}

/// Store user in cache
pub async fn set_user_in_cache(
    redis_pool: &RedisPool,
    user: &User,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = user_cache_key(user.org_id, user.id);
    let json = serde_json::to_string(user)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, USER_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate user cache
pub async fn invalidate_user_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = user_cache_key(org_id, user_id);
    let _: () = conn.del(&key).await?;

    Ok(())
}

/// Invalidate all user caches for an organization
#[allow(dead_code)]
pub async fn invalidate_org_users_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let pattern = format!("{}:{}:user:*", USER_CACHE_PREFIX, org_id);
    let keys: Vec<String> = conn.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = conn.del(&keys).await?;
    }

    // Also clear TV user index
    let tv_pattern = format!("{}:{}:tv_user:*", USER_CACHE_PREFIX, org_id);
    let tv_keys: Vec<String> = conn.keys(&tv_pattern).await?;

    if !tv_keys.is_empty() {
        let _: () = conn.del(&tv_keys).await?;
    }

    Ok(())
}

/// Get user from cache by TitaniumVault user ID
/// This uses a secondary index pattern to map tv_user_id -> user_id -> user data
pub async fn get_user_by_tv_id_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    tv_user_id: Uuid,
) -> ApiResult<Option<User>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching user by TV ID cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    let index_key = tv_user_index_key(org_id, tv_user_id);

    // First, get the actual user_id from the index
    let user_id: Option<String> = match conn.get(&index_key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching user TV index, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    match user_id {
        Some(id_str) => {
            let user_id = Uuid::parse_str(&id_str)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Invalid UUID in cache: {}", e)))?;
            get_user_from_cache(redis_pool, org_id, user_id).await
        }
        None => {
            super::metrics::record_miss(redis_pool, super::metrics::CacheType::Users).await;
            Ok(None)
        }
    }
}

/// Store user in cache with TV user ID index
pub async fn set_user_with_tv_index_in_cache(
    redis_pool: &RedisPool,
    user: &User,
) -> ApiResult<()> {
    // Store the main user data
    set_user_in_cache(redis_pool, user).await?;

    // Store the tv_user_id -> user_id index
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let index_key = tv_user_index_key(user.org_id, user.tv_user_id);
    let _: () = conn.set_ex(&index_key, user.id.to_string(), USER_CACHE_TTL).await?;

    Ok(())
}
