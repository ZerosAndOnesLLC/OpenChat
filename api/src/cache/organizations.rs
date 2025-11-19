// Cache functions for Redis integration
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::{errors::ApiResult, models::organization::Organization};

const ORG_CACHE_TTL: u64 = 3600; // 1 hour in seconds
const ORG_CACHE_PREFIX: &str = "openchat:org";

/// Build cache key for an organization
fn org_cache_key(org_id: Uuid) -> String {
    format!("{}:{}", ORG_CACHE_PREFIX, org_id)
}

/// Get organization from cache
pub async fn get_org_from_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    org_id: Uuid,
) -> ApiResult<Option<Organization>> {
    let key = org_cache_key(org_id);

    let cached: Option<String> = redis.get(&key).await?;

    match cached {
        Some(json) => {
            super::metrics::record_hit(redis, super::metrics::CacheType::Organizations).await;
            let org: Organization = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(org))
        }
        None => {
            super::metrics::record_miss(redis, super::metrics::CacheType::Organizations).await;
            Ok(None)
        }
    }
}

/// Store organization in cache
pub async fn set_org_in_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    org: &Organization,
) -> ApiResult<()> {
    let key = org_cache_key(org.id);
    let json = serde_json::to_string(org)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = redis.set_ex(&key, json, ORG_CACHE_TTL).await?;

    Ok(())
}

/// Invalidate organization cache
pub async fn invalidate_org_cache(
    redis: &mut redis::aio::MultiplexedConnection,
    org_id: Uuid,
) -> ApiResult<()> {
    let key = org_cache_key(org_id);
    let _: () = redis.del(&key).await?;

    Ok(())
}
