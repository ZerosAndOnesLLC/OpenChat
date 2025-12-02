// Cache functions for Redis integration
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::{errors::ApiResult, models::channel::{Channel, ChannelMember}};

const CHANNEL_CACHE_TTL: u64 = 300; // 5 minutes in seconds
const CHANNEL_CACHE_PREFIX: &str = "openchat:channel";
const CHANNEL_MEMBERS_PREFIX: &str = "openchat:channel_members";

/// Build cache key for a channel
fn channel_cache_key(channel_id: Uuid) -> String {
    format!("{}:{}", CHANNEL_CACHE_PREFIX, channel_id)
}

/// Build cache key for channel members
fn channel_members_cache_key(channel_id: Uuid) -> String {
    format!("{}:{}", CHANNEL_MEMBERS_PREFIX, channel_id)
}

/// Get channel from cache
pub async fn get_channel_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
) -> ApiResult<Option<Channel>> {
    let key = channel_cache_key(channel_id);
    let cached: Option<String> = redis.get(&key).await?;

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis, super::metrics::CacheType::Channels).await;
            let channel: Channel = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(channel))
        }
        None => {
            super::metrics::record_miss(redis, super::metrics::CacheType::Channels).await;
            Ok(None)
        }
    }
}

/// Store channel in cache
pub async fn set_channel_in_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel: &Channel,
) -> ApiResult<()> {
    let key = channel_cache_key(channel.id);
    let json = serde_json::to_string(channel)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = redis.set_ex(&key, json, CHANNEL_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate channel cache
pub async fn invalidate_channel_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
) -> ApiResult<()> {
    let key = channel_cache_key(channel_id);
    let _: () = redis.del(&key).await?;

    Ok(())
}

/// Get channel members from cache
pub async fn get_channel_members_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
) -> ApiResult<Option<Vec<ChannelMember>>> {
    let key = channel_members_cache_key(channel_id);
    let cached: Option<String> = redis.get(&key).await?;

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis, super::metrics::CacheType::ChannelMembers).await;
            let members: Vec<ChannelMember> = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(members))
        }
        None => {
            super::metrics::record_miss(redis, super::metrics::CacheType::ChannelMembers).await;
            Ok(None)
        }
    }
}

/// Store channel members in cache
pub async fn set_channel_members_in_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
    members: &[ChannelMember],
) -> ApiResult<()> {
    let key = channel_members_cache_key(channel_id);
    let json = serde_json::to_string(members)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = redis.set_ex(&key, json, CHANNEL_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate channel members cache
pub async fn invalidate_channel_members_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
) -> ApiResult<()> {
    let key = channel_members_cache_key(channel_id);
    let _: () = redis.del(&key).await?;

    Ok(())
}

/// Check if a user is a member of a channel (cached)
#[allow(dead_code)]
pub async fn is_channel_member_cached(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
    user_id: Uuid,
) -> ApiResult<Option<bool>> {
    let key = format!("openchat:channel_membership:{}:{}", channel_id, user_id);

    let cached: Option<String> = redis.get(&key).await?;

    match cached {
        Some(val) => {
            super::metrics::record_hit(redis, super::metrics::CacheType::ChannelMembers).await;
            Ok(Some(val == "1"))
        }
        None => {
            super::metrics::record_miss(redis, super::metrics::CacheType::ChannelMembers).await;
            Ok(None)
        }
    }
}

/// Store channel membership check result in cache
#[allow(dead_code)]
pub async fn set_channel_membership_cached(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
    user_id: Uuid,
    is_member: bool,
) -> ApiResult<()> {
    let key = format!("openchat:channel_membership:{}:{}", channel_id, user_id);
    let value = if is_member { "1" } else { "0" };

    let _: () = redis.set_ex(&key, value, CHANNEL_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate all membership checks for a channel
#[allow(dead_code)]
pub async fn invalidate_channel_membership_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
) -> ApiResult<()> {
    let pattern = format!("openchat:channel_membership:{}:*", channel_id);
    let keys: Vec<String> = redis.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = redis.del(&keys).await?;
    }

    Ok(())
}
