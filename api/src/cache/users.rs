// Cache functions for future Redis integration (Phase 9-11)
#![allow(dead_code)]

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
            let user: User = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(user))
        }
        None => Ok(None),
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
