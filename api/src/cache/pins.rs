// Cache functions for pinned messages
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::{errors::ApiResult, websocket::messages::PinnedMessageInfo};

const PIN_CACHE_TTL: u64 = 300; // 5 minutes in seconds
const PIN_CACHE_PREFIX: &str = "openchat:channel_pins";

/// Build cache key for channel pins
fn channel_pins_cache_key(channel_id: Uuid) -> String {
    format!("{}:{}", PIN_CACHE_PREFIX, channel_id)
}

/// Get pinned messages from cache
pub async fn get_pins_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
) -> ApiResult<Option<Vec<PinnedMessageInfo>>> {
    let key = channel_pins_cache_key(channel_id);
    let cached: Option<String> = redis.get(&key).await?;

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis, super::metrics::CacheType::Channels).await;
            let pins: Vec<PinnedMessageInfo> = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(pins))
        }
        None => {
            super::metrics::record_miss(redis, super::metrics::CacheType::Channels).await;
            Ok(None)
        }
    }
}

/// Store pinned messages in cache
pub async fn set_pins_in_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
    pins: &[PinnedMessageInfo],
) -> ApiResult<()> {
    let key = channel_pins_cache_key(channel_id);
    let json = serde_json::to_string(pins)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = redis.set_ex(&key, json, PIN_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate pinned messages cache
pub async fn invalidate_pins_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
) -> ApiResult<()> {
    let key = channel_pins_cache_key(channel_id);
    let _: () = redis.del(&key).await?;

    Ok(())
}

/// Add a single pin to cache (optimistic update)
#[allow(dead_code)]
pub async fn add_pin_to_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
    pin: PinnedMessageInfo,
) -> ApiResult<()> {
    // Try to get existing pins
    if let Some(mut pins) = get_pins_from_cache(redis, channel_id).await? {
        // Add new pin
        pins.push(pin);
        // Store back
        set_pins_in_cache(redis, channel_id, &pins).await?;
    } else {
        // No cache yet, just store this pin
        set_pins_in_cache(redis, channel_id, &[pin]).await?;
    }

    Ok(())
}

/// Remove a single pin from cache (optimistic update)
#[allow(dead_code)]
pub async fn remove_pin_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
    message_id: Uuid,
) -> ApiResult<()> {
    // Try to get existing pins
    if let Some(mut pins) = get_pins_from_cache(redis, channel_id).await? {
        // Remove the pin
        pins.retain(|p| p.message_id != message_id);
        // Store back
        set_pins_in_cache(redis, channel_id, &pins).await?;
    }
    // If no cache, do nothing (cache will be populated on next request)

    Ok(())
}
