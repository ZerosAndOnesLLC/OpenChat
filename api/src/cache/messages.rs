// Cache functions for future Redis integration (Phase 9-11)
#![allow(dead_code)]

use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::{errors::ApiResult, models::message::PaginatedMessages};

const MESSAGE_CACHE_TTL: u64 = 120; // 2 minutes in seconds
const MESSAGE_CACHE_PREFIX: &str = "openchat:channel_messages";
const DM_MESSAGE_CACHE_PREFIX: &str = "openchat:dm_messages";

/// Build cache key for channel messages
fn channel_messages_cache_key(channel_id: Uuid) -> String {
    format!("{}:{}", MESSAGE_CACHE_PREFIX, channel_id)
}

/// Build cache key for DM messages
fn dm_messages_cache_key(dm_id: Uuid) -> String {
    format!("{}:{}", DM_MESSAGE_CACHE_PREFIX, dm_id)
}

/// Get channel messages from cache (first page only)
pub async fn get_channel_messages_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
) -> ApiResult<Option<PaginatedMessages>> {
    let key = channel_messages_cache_key(channel_id);
    let cached: Option<String> = redis.get(&key).await?;

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
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
    messages: &PaginatedMessages,
) -> ApiResult<()> {
    let key = channel_messages_cache_key(channel_id);
    let json = serde_json::to_string(messages)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = redis.set_ex(&key, json, MESSAGE_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate channel messages cache
pub async fn invalidate_channel_messages_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
) -> ApiResult<()> {
    let key = channel_messages_cache_key(channel_id);
    let _: () = redis.del(&key).await?;

    Ok(())
}

/// Get DM messages from cache (first page only)
pub async fn get_dm_messages_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    dm_id: Uuid,
) -> ApiResult<Option<PaginatedMessages>> {
    let key = dm_messages_cache_key(dm_id);
    let cached: Option<String> = redis.get(&key).await?;

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
    redis: &mut redis::aio::MultiplexedConnection,
    dm_id: Uuid,
    messages: &PaginatedMessages,
) -> ApiResult<()> {
    let key = dm_messages_cache_key(dm_id);
    let json = serde_json::to_string(messages)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = redis.set_ex(&key, json, MESSAGE_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate DM messages cache
pub async fn invalidate_dm_messages_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    dm_id: Uuid,
) -> ApiResult<()> {
    let key = dm_messages_cache_key(dm_id);
    let _: () = redis.del(&key).await?;

    Ok(())
}
