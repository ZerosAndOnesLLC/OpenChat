// Cache functions for search results
use redis::AsyncCommands;
use serde_json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::db::RedisPool;
use crate::{errors::ApiResult, handlers::search::SearchResult};

const SEARCH_CACHE_TTL: u64 = 60; // 1 minute in seconds (short TTL for search results)
const SEARCH_CACHE_PREFIX: &str = "openchat:search";

/// Build cache key for search results
/// Uses a hash of the query parameters to create a unique key
fn search_cache_key(org_id: &str, query: &str, scope: &str, channel_id: Option<&str>, dm_id: Option<&str>) -> String {
    let mut hasher = DefaultHasher::new();
    org_id.hash(&mut hasher);
    query.hash(&mut hasher);
    scope.hash(&mut hasher);
    channel_id.hash(&mut hasher);
    dm_id.hash(&mut hasher);
    let hash = hasher.finish();

    format!("{}:{}:{}", SEARCH_CACHE_PREFIX, org_id, hash)
}

/// Get search results from cache
pub async fn get_search_results_from_cache(
    redis_pool: &RedisPool,
    org_id: &str,
    query: &str,
    scope: &str,
    channel_id: Option<&str>,
    dm_id: Option<&str>,
) -> ApiResult<Option<SearchResult>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching search cache: {}", e);
            return Ok(None);
        }
    };

    let key = search_cache_key(org_id, query, scope, channel_id, dm_id);
    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching search cache: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            let results: SearchResult = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(results))
        }
        None => Ok(None),
    }
}

/// Store search results in cache
pub async fn set_search_results_in_cache(
    redis_pool: &RedisPool,
    org_id: &str,
    query: &str,
    scope: &str,
    channel_id: Option<&str>,
    dm_id: Option<&str>,
    results: &SearchResult,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = search_cache_key(org_id, query, scope, channel_id, dm_id);
    let json = serde_json::to_string(results)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, SEARCH_CACHE_TTL).await?;

    Ok(())
}
