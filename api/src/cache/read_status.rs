// Cache functions for unread counts
#![allow(dead_code)]

use redis::AsyncCommands;
use uuid::Uuid;

use crate::errors::ApiResult;

const UNREAD_CACHE_TTL: u64 = 60; // 1 minute in seconds
const CHANNEL_UNREAD_PREFIX: &str = "openchat:unread:channel";
const DM_UNREAD_PREFIX: &str = "openchat:unread:dm";

/// Build cache key for channel unread count
fn channel_unread_cache_key(user_id: Uuid, channel_id: Uuid) -> String {
    format!("{}:{}:{}", CHANNEL_UNREAD_PREFIX, user_id, channel_id)
}

/// Build cache key for DM unread count
fn dm_unread_cache_key(user_id: Uuid, dm_id: Uuid) -> String {
    format!("{}:{}:{}", DM_UNREAD_PREFIX, user_id, dm_id)
}

/// Get channel unread count from cache
pub async fn get_channel_unread_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<Option<i32>> {
    let key = channel_unread_cache_key(user_id, channel_id);
    let cached: Option<i32> = redis.get(&key).await?;
    Ok(cached)
}

/// Store channel unread count in cache
pub async fn set_channel_unread_in_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    channel_id: Uuid,
    unread_count: i32,
) -> ApiResult<()> {
    let key = channel_unread_cache_key(user_id, channel_id);
    let _: () = redis.set_ex(&key, unread_count, UNREAD_CACHE_TTL).await?;
    Ok(())
}

/// Invalidate channel unread count cache for a user
pub async fn invalidate_channel_unread_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<()> {
    let key = channel_unread_cache_key(user_id, channel_id);
    let _: () = redis.del(&key).await?;
    Ok(())
}

/// Invalidate channel unread count cache for all members of a channel
pub async fn invalidate_channel_unread_cache_for_all_members(
    redis: &mut redis::aio::MultiplexedConnection,
    channel_id: Uuid,
) -> ApiResult<()> {
    // Delete all keys matching the pattern
    let pattern = format!("{}:*:{}", CHANNEL_UNREAD_PREFIX, channel_id);
    let keys: Vec<String> = redis.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = redis.del(keys).await?;
    }

    Ok(())
}

/// Get DM unread count from cache
pub async fn get_dm_unread_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    dm_id: Uuid,
) -> ApiResult<Option<i32>> {
    let key = dm_unread_cache_key(user_id, dm_id);
    let cached: Option<i32> = redis.get(&key).await?;
    Ok(cached)
}

/// Store DM unread count in cache
pub async fn set_dm_unread_in_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    dm_id: Uuid,
    unread_count: i32,
) -> ApiResult<()> {
    let key = dm_unread_cache_key(user_id, dm_id);
    let _: () = redis.set_ex(&key, unread_count, UNREAD_CACHE_TTL).await?;
    Ok(())
}

/// Invalidate DM unread count cache for a user
pub async fn invalidate_dm_unread_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    dm_id: Uuid,
) -> ApiResult<()> {
    let key = dm_unread_cache_key(user_id, dm_id);
    let _: () = redis.del(&key).await?;
    Ok(())
}

/// Invalidate DM unread count cache for all participants of a DM
pub async fn invalidate_dm_unread_cache_for_all_participants(
    redis: &mut redis::aio::MultiplexedConnection,
    dm_id: Uuid,
) -> ApiResult<()> {
    // Delete all keys matching the pattern
    let pattern = format!("{}:*:{}", DM_UNREAD_PREFIX, dm_id);
    let keys: Vec<String> = redis.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = redis.del(keys).await?;
    }

    Ok(())
}
