use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::db::RedisPool;
use crate::errors::ApiResult;

const NOTIFICATION_PREFS_CACHE_TTL: u64 = 300; // 5 minutes
const NOTIFICATION_PREFS_PREFIX: &str = "openchat:org";

/// Cached version of a notification preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPref {
    pub preference: String,
    pub mute_until: Option<DateTime<Utc>>,
}

/// Build cache key: openchat:org:{org_id}:notif_prefs:{user_id}
fn cache_key(org_id: Uuid, user_id: Uuid) -> String {
    format!("{}:{}:notif_prefs:{}", NOTIFICATION_PREFS_PREFIX, org_id, user_id)
}

/// Get user notification prefs from cache.
/// Returns a map of channel_id/dm_id -> CachedPref.
pub async fn get_user_prefs_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<Option<HashMap<String, CachedPref>>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching notification prefs cache: {}", e);
            return Ok(None);
        }
    };

    let key = cache_key(org_id, user_id);
    let data: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching notification prefs cache: {}", e);
            return Ok(None);
        }
    };

    match data {
        Some(json_str) => {
            match serde_json::from_str(&json_str) {
                Ok(map) => Ok(Some(map)),
                Err(e) => {
                    tracing::warn!("Failed to parse notification prefs cache: {}", e);
                    Ok(None)
                }
            }
        }
        None => Ok(None),
    }
}

/// Set user notification prefs in cache.
pub async fn set_user_prefs_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
    prefs: &HashMap<String, CachedPref>,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = cache_key(org_id, user_id);
    let json_str = serde_json::to_string(prefs)
        .map_err(|e| crate::errors::ApiError::Internal(format!("JSON serialize error: {}", e)))?;

    let _: () = conn.set_ex(&key, json_str, NOTIFICATION_PREFS_CACHE_TTL).await?;
    Ok(())
}

/// Invalidate user notification prefs cache.
pub async fn invalidate_user_prefs_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = cache_key(org_id, user_id);
    let _: () = conn.del(&key).await?;
    Ok(())
}
