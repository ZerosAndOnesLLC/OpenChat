// Cache functions for Redis integration
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::db::RedisPool;
use crate::{errors::ApiResult, models::message::PaginatedMessages};

const MESSAGE_CACHE_TTL: u64 = 120; // 2 minutes in seconds
const MESSAGE_CACHE_PREFIX: &str = "openchat:org";

/// Build cache key for channel messages: openchat:org:{org_id}:channel_messages:{channel_id}
fn channel_messages_cache_key(org_id: Uuid, channel_id: Uuid) -> String {
    format!("{}:{}:channel_messages:{}", MESSAGE_CACHE_PREFIX, org_id, channel_id)
}

/// Build cache key for DM messages: openchat:org:{org_id}:dm_messages:{dm_id}
fn dm_messages_cache_key(org_id: Uuid, dm_id: Uuid) -> String {
    format!("{}:{}:dm_messages:{}", MESSAGE_CACHE_PREFIX, org_id, dm_id)
}

/// Get channel messages from cache (first page only)
pub async fn get_channel_messages_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<Option<PaginatedMessages>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching channel messages cache: {}", e);
            return Ok(None);
        }
    };

    let key = channel_messages_cache_key(org_id, channel_id);
    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching channel messages cache: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            let messages: PaginatedMessages = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(messages))
        }
        None => Ok(None),
    }
}

/// Store channel messages in cache (first page only)
pub async fn set_channel_messages_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
    messages: &PaginatedMessages,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = channel_messages_cache_key(org_id, channel_id);
    let json = serde_json::to_string(messages)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, MESSAGE_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate channel messages cache
pub async fn invalidate_channel_messages_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = channel_messages_cache_key(org_id, channel_id);
    let _: () = conn.del(&key).await?;

    Ok(())
}

/// Get DM messages from cache (first page only)
pub async fn get_dm_messages_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    dm_id: Uuid,
) -> ApiResult<Option<PaginatedMessages>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching DM messages cache: {}", e);
            return Ok(None);
        }
    };

    let key = dm_messages_cache_key(org_id, dm_id);
    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching DM messages cache: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            let messages: PaginatedMessages = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(messages))
        }
        None => Ok(None),
    }
}

/// Store DM messages in cache (first page only)
pub async fn set_dm_messages_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    dm_id: Uuid,
    messages: &PaginatedMessages,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = dm_messages_cache_key(org_id, dm_id);
    let json = serde_json::to_string(messages)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, MESSAGE_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate DM messages cache
pub async fn invalidate_dm_messages_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    dm_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = dm_messages_cache_key(org_id, dm_id);
    let _: () = conn.del(&key).await?;

    Ok(())
}

/// Invalidate all message cache for an organization
#[allow(dead_code)]
pub async fn invalidate_org_messages_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let pattern = format!("{}:{}:*_messages:*", MESSAGE_CACHE_PREFIX, org_id);
    let keys: Vec<String> = conn.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = conn.del(&keys).await?;
    }

    Ok(())
}
