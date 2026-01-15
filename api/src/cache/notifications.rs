// Cache functions for notification counts
use redis::AsyncCommands;
use uuid::Uuid;

use crate::db::RedisPool;
use crate::errors::ApiResult;

const NOTIFICATION_CACHE_TTL: u64 = 300; // 5 minutes in seconds
const NOTIFICATION_CACHE_PREFIX: &str = "openchat:org";

/// Build cache key for notification count: openchat:org:{org_id}:notification_count:{user_id}
fn notification_count_cache_key(org_id: Uuid, user_id: Uuid) -> String {
    format!("{}:{}:notification_count:{}", NOTIFICATION_CACHE_PREFIX, org_id, user_id)
}

/// Get notification count from cache
pub async fn get_notification_count_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<Option<i32>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching notification count cache: {}", e);
            return Ok(None);
        }
    };

    let key = notification_count_cache_key(org_id, user_id);
    let count: Option<i32> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching notification count cache: {}", e);
            return Ok(None);
        }
    };
    Ok(count)
}

/// Set notification count in cache
pub async fn set_notification_count_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
    count: i32,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = notification_count_cache_key(org_id, user_id);
    let _: () = conn.set_ex(&key, count, NOTIFICATION_CACHE_TTL).await?;
    Ok(())
}

/// Increment notification count in cache
#[allow(dead_code)]
pub async fn increment_notification_count_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<i32> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = notification_count_cache_key(org_id, user_id);
    let new_count: i32 = conn.incr(&key, 1).await?;
    // Reset TTL when incrementing
    let _: bool = conn.expire(&key, NOTIFICATION_CACHE_TTL as i64).await?;
    Ok(new_count)
}

/// Decrement notification count in cache
pub async fn decrement_notification_count_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<i32> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = notification_count_cache_key(org_id, user_id);
    let new_count: i32 = conn.decr(&key, 1).await?;
    // Ensure count doesn't go negative
    if new_count < 0 {
        let _: () = conn.set(&key, 0).await?;
        Ok(0)
    } else {
        // Reset TTL when decrementing
        let _: bool = conn.expire(&key, NOTIFICATION_CACHE_TTL as i64).await?;
        Ok(new_count)
    }
}

/// Clear notification count from cache
#[allow(dead_code)]
pub async fn clear_notification_count_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = notification_count_cache_key(org_id, user_id);
    let _: () = conn.del(&key).await?;
    Ok(())
}

/// Invalidate notification count cache for a user
#[allow(dead_code)]
pub async fn invalidate_notification_count_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<()> {
    clear_notification_count_from_cache(redis_pool, org_id, user_id).await
}

/// Invalidate all notification caches for an organization
#[allow(dead_code)]
pub async fn invalidate_org_notification_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let pattern = format!("{}:{}:notification_count:*", NOTIFICATION_CACHE_PREFIX, org_id);
    let keys: Vec<String> = conn.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = conn.del(&keys).await?;
    }

    Ok(())
}
