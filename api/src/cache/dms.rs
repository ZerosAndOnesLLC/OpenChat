// Cache functions for Redis integration
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::{errors::ApiResult, models::direct_message::{DirectMessage, DmParticipant}};

const DM_CACHE_TTL: u64 = 300; // 5 minutes in seconds
const DM_CACHE_PREFIX: &str = "openchat:dm";
const DM_PARTICIPANTS_PREFIX: &str = "openchat:dm_participants";
#[allow(dead_code)]
const USER_DMS_PREFIX: &str = "openchat:user_dms";

/// Build cache key for a DM
fn dm_cache_key(dm_id: Uuid) -> String {
    format!("{}:{}", DM_CACHE_PREFIX, dm_id)
}

/// Build cache key for DM participants
fn dm_participants_cache_key(dm_id: Uuid) -> String {
    format!("{}:{}", DM_PARTICIPANTS_PREFIX, dm_id)
}

/// Build cache key for user's DMs list
#[allow(dead_code)]
fn user_dms_cache_key(user_id: Uuid) -> String {
    format!("{}:{}", USER_DMS_PREFIX, user_id)
}

/// Get DM from cache
pub async fn get_dm_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    dm_id: Uuid,
) -> ApiResult<Option<DirectMessage>> {
    let key = dm_cache_key(dm_id);
    let cached: Option<String> = redis.get(&key).await?;

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis, super::metrics::CacheType::Dms).await;
            let dm: DirectMessage = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(dm))
        }
        None => {
            super::metrics::record_miss(redis, super::metrics::CacheType::Dms).await;
            Ok(None)
        }
    }
}

/// Store DM in cache
pub async fn set_dm_in_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    dm: &DirectMessage,
) -> ApiResult<()> {
    let key = dm_cache_key(dm.id);
    let json = serde_json::to_string(dm)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = redis.set_ex(&key, json, DM_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate DM cache
#[allow(dead_code)]
pub async fn invalidate_dm_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    dm_id: Uuid,
) -> ApiResult<()> {
    let key = dm_cache_key(dm_id);
    let _: () = redis.del(&key).await?;

    Ok(())
}

/// Get DM participants from cache
#[allow(dead_code)]
pub async fn get_dm_participants_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    dm_id: Uuid,
) -> ApiResult<Option<Vec<DmParticipant>>> {
    let key = dm_participants_cache_key(dm_id);
    let cached: Option<String> = redis.get(&key).await?;

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis, super::metrics::CacheType::DmParticipants).await;
            let participants: Vec<DmParticipant> = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(participants))
        }
        None => {
            super::metrics::record_miss(redis, super::metrics::CacheType::DmParticipants).await;
            Ok(None)
        }
    }
}

/// Store DM participants in cache
pub async fn set_dm_participants_in_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    dm_id: Uuid,
    participants: &[DmParticipant],
) -> ApiResult<()> {
    let key = dm_participants_cache_key(dm_id);
    let json = serde_json::to_string(participants)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = redis.set_ex(&key, json, DM_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate DM participants cache
#[allow(dead_code)]
pub async fn invalidate_dm_participants_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    dm_id: Uuid,
) -> ApiResult<()> {
    let key = dm_participants_cache_key(dm_id);
    let _: () = redis.del(&key).await?;

    Ok(())
}

/// Get user's DMs list from cache
#[allow(dead_code)]
pub async fn get_user_dms_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
) -> ApiResult<Option<Vec<DirectMessage>>> {
    let key = user_dms_cache_key(user_id);
    let cached: Option<String> = redis.get(&key).await?;

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
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    dms: &[DirectMessage],
) -> ApiResult<()> {
    let key = user_dms_cache_key(user_id);
    let json = serde_json::to_string(dms)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = redis.set_ex(&key, json, DM_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate user's DMs list cache
#[allow(dead_code)]
pub async fn invalidate_user_dms_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
) -> ApiResult<()> {
    let key = user_dms_cache_key(user_id);
    let _: () = redis.del(&key).await?;

    Ok(())
}
