// Cache functions for Redis integration
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::{errors::ApiResult, models::user::User};

const USER_CACHE_TTL: u64 = 3600; // 1 hour in seconds
const USER_CACHE_PREFIX: &str = "openchat:user";

/// Build cache key for a user
fn user_cache_key(user_id: Uuid) -> String {
    format!("{}:{}", USER_CACHE_PREFIX, user_id)
}

/// Get user from cache
pub async fn get_user_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
) -> ApiResult<Option<User>> {
    let key = user_cache_key(user_id);

    let cached: Option<String> = redis.get(&key).await?;

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis, super::metrics::CacheType::Users).await;
            let user: User = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(user))
        }
        None => {
            super::metrics::record_miss(redis, super::metrics::CacheType::Users).await;
            Ok(None)
        }
    }
}

/// Store user in cache
pub async fn set_user_in_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    user: &User,
) -> ApiResult<()> {
    let key = user_cache_key(user.id);
    let json = serde_json::to_string(user)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = redis.set_ex(&key, json, USER_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate user cache
pub async fn invalidate_user_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
) -> ApiResult<()> {
    let key = user_cache_key(user_id);
    let _: () = redis.del(&key).await?;

    Ok(())
}

/// Invalidate all user caches for an organization
#[allow(dead_code)]
pub async fn invalidate_org_users_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    _org_id: Uuid,
) -> ApiResult<()> {
    // Pattern to match all user keys for the org
    // Note: This is a simplified approach. In production, you might want to
    // maintain a set of user IDs per org for more efficient invalidation
    let pattern = format!("{}:*", USER_CACHE_PREFIX);
    let keys: Vec<String> = redis.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = redis.del(&keys).await?;
    }

    Ok(())
}

/// Get user from cache by TitaniumVault user ID
/// This uses a secondary index pattern to map tv_user_id -> user_id -> user data
pub async fn get_user_by_tv_id_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    tv_user_id: Uuid,
) -> ApiResult<Option<User>> {
    let index_key = format!("{}:tv_index:{}", USER_CACHE_PREFIX, tv_user_id);

    // First, get the actual user_id from the index
    let user_id: Option<String> = redis.get(&index_key).await?;

    match user_id {
        Some(id_str) => {
            let user_id = Uuid::parse_str(&id_str)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Invalid UUID in cache: {}", e)))?;
            get_user_from_cache(redis, user_id).await
        }
        None => {
            super::metrics::record_miss(redis, super::metrics::CacheType::Users).await;
            Ok(None)
        }
    }
}

/// Store user in cache with TV user ID index
pub async fn set_user_with_tv_index_in_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    user: &User,
) -> ApiResult<()> {
    // Store the main user data
    set_user_in_cache(redis, user).await?;

    // Store the tv_user_id -> user_id index
    let index_key = format!("{}:tv_index:{}", USER_CACHE_PREFIX, user.tv_user_id);
    let _: () = redis.set_ex(&index_key, user.id.to_string(), USER_CACHE_TTL).await?;

    Ok(())
}
