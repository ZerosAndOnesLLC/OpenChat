// Cache functions for Redis integration
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::db::RedisPool;
use crate::{errors::ApiResult, models::organization::Organization};

const ORG_CACHE_TTL: u64 = 3600; // 1 hour in seconds
const ORG_CACHE_PREFIX: &str = "openchat:org";

/// Build cache key for an organization
fn org_cache_key(org_id: Uuid) -> String {
    format!("{}:{}", ORG_CACHE_PREFIX, org_id)
}

/// Get organization from cache
pub async fn get_org_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
) -> ApiResult<Option<Organization>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching org cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    let key = org_cache_key(org_id);

    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching org cache, falling back to DB: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis_pool, super::metrics::CacheType::Organizations).await;
            let org: Organization = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(org))
        }
        None => {
            super::metrics::record_miss(redis_pool, super::metrics::CacheType::Organizations).await;
            Ok(None)
        }
    }
}

/// Store organization in cache
pub async fn set_org_in_cache(
    redis_pool: &RedisPool,
    org: &Organization,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = org_cache_key(org.id);
    let json = serde_json::to_string(org)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, ORG_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate organization cache
#[allow(dead_code)]
pub async fn invalidate_org_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = org_cache_key(org_id);
    let _: () = conn.del(&key).await?;

    Ok(())
}
