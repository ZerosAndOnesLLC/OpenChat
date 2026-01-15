// Cache functions for unread counts
#![allow(dead_code)]

use redis::AsyncCommands;
use uuid::Uuid;

use crate::db::RedisPool;
use crate::errors::ApiResult;

const UNREAD_CACHE_TTL: u64 = 60; // 1 minute in seconds
const UNREAD_CACHE_PREFIX: &str = "openchat:org";

/// Build cache key for channel unread count: openchat:org:{org_id}:unread:channel:{user_id}:{channel_id}
fn channel_unread_cache_key(org_id: Uuid, user_id: Uuid, channel_id: Uuid) -> String {
    format!("{}:{}:unread:channel:{}:{}", UNREAD_CACHE_PREFIX, org_id, user_id, channel_id)
}

/// Build cache key for DM unread count: openchat:org:{org_id}:unread:dm:{user_id}:{dm_id}
fn dm_unread_cache_key(org_id: Uuid, user_id: Uuid, dm_id: Uuid) -> String {
    format!("{}:{}:unread:dm:{}:{}", UNREAD_CACHE_PREFIX, org_id, user_id, dm_id)
}

/// Get channel unread count from cache
pub async fn get_channel_unread_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<Option<i32>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching channel unread cache: {}", e);
            return Ok(None);
        }
    };

    let key = channel_unread_cache_key(org_id, user_id, channel_id);
    let cached: Option<i32> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching channel unread cache: {}", e);
            return Ok(None);
        }
    };
    Ok(cached)
}

/// Store channel unread count in cache
pub async fn set_channel_unread_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
    channel_id: Uuid,
    unread_count: i32,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = channel_unread_cache_key(org_id, user_id, channel_id);
    let _: () = conn.set_ex(&key, unread_count, UNREAD_CACHE_TTL).await?;
    Ok(())
}

/// Invalidate channel unread count cache for a user
pub async fn invalidate_channel_unread_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = channel_unread_cache_key(org_id, user_id, channel_id);
    let _: () = conn.del(&key).await?;
    Ok(())
}

/// Invalidate channel unread count cache for all members of a channel
pub async fn invalidate_channel_unread_cache_for_all_members(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    // Delete all keys matching the pattern
    let pattern = format!("{}:{}:unread:channel:*:{}", UNREAD_CACHE_PREFIX, org_id, channel_id);
    let keys: Vec<String> = conn.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = conn.del(keys).await?;
    }

    Ok(())
}

/// Get DM unread count from cache
pub async fn get_dm_unread_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
    dm_id: Uuid,
) -> ApiResult<Option<i32>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching DM unread cache: {}", e);
            return Ok(None);
        }
    };

    let key = dm_unread_cache_key(org_id, user_id, dm_id);
    let cached: Option<i32> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching DM unread cache: {}", e);
            return Ok(None);
        }
    };
    Ok(cached)
}

/// Store DM unread count in cache
pub async fn set_dm_unread_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
    dm_id: Uuid,
    unread_count: i32,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = dm_unread_cache_key(org_id, user_id, dm_id);
    let _: () = conn.set_ex(&key, unread_count, UNREAD_CACHE_TTL).await?;
    Ok(())
}

/// Invalidate DM unread count cache for a user
pub async fn invalidate_dm_unread_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
    dm_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = dm_unread_cache_key(org_id, user_id, dm_id);
    let _: () = conn.del(&key).await?;
    Ok(())
}

/// Invalidate DM unread count cache for all participants of a DM
pub async fn invalidate_dm_unread_cache_for_all_participants(
    redis_pool: &RedisPool,
    org_id: Uuid,
    dm_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    // Delete all keys matching the pattern
    let pattern = format!("{}:{}:unread:dm:*:{}", UNREAD_CACHE_PREFIX, org_id, dm_id);
    let keys: Vec<String> = conn.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = conn.del(keys).await?;
    }

    Ok(())
}

/// Invalidate all unread caches for an organization
pub async fn invalidate_org_unread_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let pattern = format!("{}:{}:unread:*", UNREAD_CACHE_PREFIX, org_id);
    let keys: Vec<String> = conn.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = conn.del(&keys).await?;
    }

    Ok(())
}
