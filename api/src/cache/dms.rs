// Cache functions for Redis integration
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::db::RedisPool;
use crate::{errors::ApiResult, models::direct_message::{DirectMessage, DmParticipant}};

const DM_CACHE_TTL: u64 = 300; // 5 minutes in seconds
const DM_CACHE_PREFIX: &str = "openchat:org";

/// Build cache key for a DM: openchat:org:{org_id}:dm:{dm_id}
fn dm_cache_key(org_id: Uuid, dm_id: Uuid) -> String {
    format!("{}:{}:dm:{}", DM_CACHE_PREFIX, org_id, dm_id)
}

/// Build cache key for DM participants: openchat:org:{org_id}:dm_participants:{dm_id}
fn dm_participants_cache_key(org_id: Uuid, dm_id: Uuid) -> String {
    format!("{}:{}:dm_participants:{}", DM_CACHE_PREFIX, org_id, dm_id)
}

/// Build cache key for user's DMs list: openchat:org:{org_id}:user_dms:{user_id}
fn user_dms_cache_key(org_id: Uuid, user_id: Uuid) -> String {
    format!("{}:{}:user_dms:{}", DM_CACHE_PREFIX, org_id, user_id)
}

/// Get DM from cache
pub async fn get_dm_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    dm_id: Uuid,
) -> ApiResult<Option<DirectMessage>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching DM cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    let key = dm_cache_key(org_id, dm_id);
    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching DM cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis_pool, super::metrics::CacheType::Dms).await;
            let dm: DirectMessage = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(dm))
        }
        None => {
            super::metrics::record_miss(redis_pool, super::metrics::CacheType::Dms).await;
            Ok(None)
        }
    }
}

/// Store DM in cache
pub async fn set_dm_in_cache(
    redis_pool: &RedisPool,
    dm: &DirectMessage,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = dm_cache_key(dm.org_id, dm.id);
    let json = serde_json::to_string(dm)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, DM_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate DM cache
#[allow(dead_code)]
pub async fn invalidate_dm_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    dm_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = dm_cache_key(org_id, dm_id);
    let _: () = conn.del(&key).await?;

    Ok(())
}

/// Get DM participants from cache
#[allow(dead_code)]
pub async fn get_dm_participants_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    dm_id: Uuid,
) -> ApiResult<Option<Vec<DmParticipant>>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching DM participants cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    let key = dm_participants_cache_key(org_id, dm_id);
    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching DM participants cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis_pool, super::metrics::CacheType::DmParticipants).await;
            let participants: Vec<DmParticipant> = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(participants))
        }
        None => {
            super::metrics::record_miss(redis_pool, super::metrics::CacheType::DmParticipants).await;
            Ok(None)
        }
    }
}

/// Store DM participants in cache
pub async fn set_dm_participants_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    dm_id: Uuid,
    participants: &[DmParticipant],
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = dm_participants_cache_key(org_id, dm_id);
    let json = serde_json::to_string(participants)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, DM_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate DM participants cache
#[allow(dead_code)]
pub async fn invalidate_dm_participants_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    dm_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = dm_participants_cache_key(org_id, dm_id);
    let _: () = conn.del(&key).await?;

    Ok(())
}

/// Get user's DMs list from cache
#[allow(dead_code)]
pub async fn get_user_dms_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<Option<Vec<DirectMessage>>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching user DMs cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    let key = user_dms_cache_key(org_id, user_id);
    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching user DMs cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            let dms: Vec<DirectMessage> = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(dms))
        }
        None => Ok(None),
    }
}

/// Store user's DMs list in cache
#[allow(dead_code)]
pub async fn set_user_dms_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
    dms: &[DirectMessage],
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = user_dms_cache_key(org_id, user_id);
    let json = serde_json::to_string(dms)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, DM_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate user's DMs list cache
#[allow(dead_code)]
pub async fn invalidate_user_dms_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = user_dms_cache_key(org_id, user_id);
    let _: () = conn.del(&key).await?;

    Ok(())
}

/// Invalidate all DM cache for an organization
#[allow(dead_code)]
pub async fn invalidate_org_dm_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let pattern = format!("{}:{}:dm*", DM_CACHE_PREFIX, org_id);
    let keys: Vec<String> = conn.keys(&pattern).await?;

    if !keys.is_empty() {
        let _: () = conn.del(&keys).await?;
    }

    Ok(())
}
