// Cache functions for pinned messages
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::db::RedisPool;
use crate::{errors::ApiResult, websocket::messages::PinnedMessageInfo};

const PIN_CACHE_TTL: u64 = 300; // 5 minutes in seconds
const PIN_CACHE_PREFIX: &str = "openchat:org";

/// Build cache key for channel pins: openchat:org:{org_id}:channel_pins:{channel_id}
fn channel_pins_cache_key(org_id: Uuid, channel_id: Uuid) -> String {
    format!("{}:{}:channel_pins:{}", PIN_CACHE_PREFIX, org_id, channel_id)
}

/// Get pinned messages from cache
pub async fn get_pins_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<Option<Vec<PinnedMessageInfo>>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching pins cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    let key = channel_pins_cache_key(org_id, channel_id);
    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching pins cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis_pool, super::metrics::CacheType::Channels).await;
            let pins: Vec<PinnedMessageInfo> = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(pins))
        }
        None => {
            super::metrics::record_miss(redis_pool, super::metrics::CacheType::Channels).await;
            Ok(None)
        }
    }
}

/// Store pinned messages in cache
pub async fn set_pins_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
    pins: &[PinnedMessageInfo],
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = channel_pins_cache_key(org_id, channel_id);
    let json = serde_json::to_string(pins)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, PIN_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate pinned messages cache
pub async fn invalidate_pins_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = channel_pins_cache_key(org_id, channel_id);
    let _: () = conn.del(&key).await?;

    Ok(())
}

/// Add a single pin to cache (optimistic update)
#[allow(dead_code)]
pub async fn add_pin_to_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
    pin: PinnedMessageInfo,
) -> ApiResult<()> {
    // Try to get existing pins
    if let Some(mut pins) = get_pins_from_cache(redis_pool, org_id, channel_id).await? {
        // Add new pin
        pins.push(pin);
        // Store back
        set_pins_in_cache(redis_pool, org_id, channel_id, &pins).await?;
    } else {
        // No cache yet, just store this pin
        set_pins_in_cache(redis_pool, org_id, channel_id, &[pin]).await?;
    }

    Ok(())
}

/// Remove a single pin from cache (optimistic update)
#[allow(dead_code)]
pub async fn remove_pin_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
    message_id: Uuid,
) -> ApiResult<()> {
    // Try to get existing pins
    if let Some(mut pins) = get_pins_from_cache(redis_pool, org_id, channel_id).await? {
        // Remove the pin
        pins.retain(|p| p.message_id != message_id);
        // Store back
        set_pins_in_cache(redis_pool, org_id, channel_id, &pins).await?;
    }
    // If no cache, do nothing (cache will be populated on next request)

    Ok(())
}

/// Invalidate all pin cache for an organization
#[allow(dead_code)]
pub async fn invalidate_org_pins_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let pattern = format!("{}:{}:channel_pins:*", PIN_CACHE_PREFIX, org_id);
    let keys: Vec<String> = conn.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = conn.del(&keys).await?;
    }

    Ok(())
}
