use redis::AsyncCommands;
use uuid::Uuid;

use crate::db::RedisPool;
use crate::errors::ApiResult;
use crate::models::user_group::UserGroup;

const USER_GROUPS_CACHE_TTL: u64 = 300; // 5 minutes
const CACHE_PREFIX: &str = "openchat:org";

fn cache_key(org_id: Uuid) -> String {
    format!("{}:{}:user_groups", CACHE_PREFIX, org_id)
}

pub async fn get_groups_from_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
) -> ApiResult<Option<Vec<UserGroup>>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching user groups cache: {}", e);
            return Ok(None);
        }
    };

    let key = cache_key(org_id);
    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching user groups cache: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            let groups: Vec<UserGroup> = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Cache deserialization error: {}", e)))?;
            Ok(Some(groups))
        }
        None => Ok(None),
    }
}

pub async fn set_groups_in_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
    groups: &[UserGroup],
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = cache_key(org_id);
    let json = serde_json::to_string(groups)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, json, USER_GROUPS_CACHE_TTL).await?;
    Ok(())
}

pub async fn invalidate_groups_cache(
    redis_pool: &RedisPool,
    org_id: Uuid,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = cache_key(org_id);
    let _: () = conn.del(&key).await?;
    Ok(())
}
