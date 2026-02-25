use redis::AsyncCommands;
use uuid::Uuid;

use crate::db::RedisPool;
use crate::errors::ApiResult;
use crate::models::channel_section::ChannelSection;

const CHANNEL_SECTIONS_CACHE_TTL: u64 = 300; // 5 minutes
const CACHE_PREFIX: &str = "openchat:org";

fn cache_key(org_id: Uuid, user_id: Uuid) -> String {
    format!("{}:{}:channel_sections:{}", CACHE_PREFIX, org_id, user_id)
}

pub async fn get_sections_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
) -> ApiResult<Option<Vec<ChannelSection>>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching channel sections cache: {}", e);
            return Ok(None);
        }
    };

    let key = cache_key(org_id, user_id);
    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching channel sections cache: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            let sections: Vec<ChannelSection> = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(sections))
        }
        None => Ok(None),
    }
}

pub async fn set_sections_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    user_id: Uuid,
    sections: &[ChannelSection],
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = cache_key(org_id, user_id);
    let json = serde_json::to_string(sections)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, CHANNEL_SECTIONS_CACHE_TTL).await?;
    Ok(())
}

pub async fn invalidate_sections_cache(
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
