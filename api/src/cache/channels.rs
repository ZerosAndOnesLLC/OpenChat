// Cache functions for Redis integration
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::db::RedisPool;
use crate::{errors::ApiResult, models::channel::{Channel, ChannelMember}};

const CHANNEL_CACHE_TTL: u64 = 300; // 5 minutes in seconds
const CHANNEL_CACHE_PREFIX: &str = "openchat:org";

/// Build cache key for a channel: openchat:org:{org_id}:channel:{channel_id}
fn channel_cache_key(org_id: Uuid, channel_id: Uuid) -> String {
    format!("{}:{}:channel:{}", CHANNEL_CACHE_PREFIX, org_id, channel_id)
}

/// Build cache key for channel members: openchat:org:{org_id}:channel_members:{channel_id}
fn channel_members_cache_key(org_id: Uuid, channel_id: Uuid) -> String {
    format!("{}:{}:channel_members:{}", CHANNEL_CACHE_PREFIX, org_id, channel_id)
}

/// Build cache key for channel membership: openchat:org:{org_id}:channel_membership:{channel_id}:{user_id}
fn channel_membership_cache_key(org_id: Uuid, channel_id: Uuid, user_id: Uuid) -> String {
    format!("{}:{}:channel_membership:{}:{}", CHANNEL_CACHE_PREFIX, org_id, channel_id, user_id)
}

/// Get channel from cache
/// Returns None on Redis errors to gracefully fall back to database
pub async fn get_channel_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<Option<Channel>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching channel cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    let key = channel_cache_key(org_id, channel_id);
    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching channel cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis_pool, super::metrics::CacheType::Channels).await;
            let channel: Channel = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(channel))
        }
        None => {
            super::metrics::record_miss(redis_pool, super::metrics::CacheType::Channels).await;
            Ok(None)
        }
    }
}

/// Store channel in cache
pub async fn set_channel_in_cache(
    redis_pool: &RedisPool,
    channel: &Channel,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = channel_cache_key(channel.org_id, channel.id);
    let json = serde_json::to_string(channel)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, CHANNEL_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate channel cache
pub async fn invalidate_channel_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = channel_cache_key(org_id, channel_id);
    let _: () = conn.del(&key).await?;

    Ok(())
}

/// Get channel members from cache
/// Returns None on Redis errors to gracefully fall back to database
pub async fn get_channel_members_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<Option<Vec<ChannelMember>>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching channel members cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    let key = channel_members_cache_key(org_id, channel_id);
    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching channel members cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis_pool, super::metrics::CacheType::ChannelMembers).await;
            let members: Vec<ChannelMember> = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(members))
        }
        None => {
            super::metrics::record_miss(redis_pool, super::metrics::CacheType::ChannelMembers).await;
            Ok(None)
        }
    }
}

/// Store channel members in cache
pub async fn set_channel_members_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
    members: &[ChannelMember],
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = channel_members_cache_key(org_id, channel_id);
    let json = serde_json::to_string(members)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, CHANNEL_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate channel members cache
pub async fn invalidate_channel_members_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = channel_members_cache_key(org_id, channel_id);
    let _: () = conn.del(&key).await?;

    Ok(())
}

/// Check if a user is a member of a channel (cached)
/// Returns None on Redis errors to gracefully fall back to database
#[allow(dead_code)]
pub async fn is_channel_member_cached(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
    user_id: Uuid,
) -> ApiResult<Option<bool>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error checking channel membership cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    let key = channel_membership_cache_key(org_id, channel_id, user_id);

    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error checking channel membership cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(val) => {
            super::metrics::record_hit(redis_pool, super::metrics::CacheType::ChannelMembers).await;
            Ok(Some(val == "1"))
        }
        None => {
            super::metrics::record_miss(redis_pool, super::metrics::CacheType::ChannelMembers).await;
            Ok(None)
        }
    }
}

/// Store channel membership check result in cache
#[allow(dead_code)]
pub async fn set_channel_membership_cached(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
    user_id: Uuid,
    is_member: bool,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = channel_membership_cache_key(org_id, channel_id, user_id);
    let value = if is_member { "1" } else { "0" };

    let _: () = conn.set_ex(&key, value, CHANNEL_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate all membership checks for a channel
#[allow(dead_code)]
pub async fn invalidate_channel_membership_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let pattern = format!("{}:{}:channel_membership:{}:*", CHANNEL_CACHE_PREFIX, org_id, channel_id);
    let keys: Vec<String> = conn.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = conn.del(&keys).await?;
    }

    Ok(())
}

/// Invalidate all cache for an organization (useful for org deletion/reset)
#[allow(dead_code)]
pub async fn invalidate_org_channel_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let pattern = format!("{}:{}:channel*", CHANNEL_CACHE_PREFIX, org_id);
    let keys: Vec<String> = conn.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = conn.del(&keys).await?;
    }

    Ok(())
}
