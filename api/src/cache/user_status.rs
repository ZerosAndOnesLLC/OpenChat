#![allow(dead_code)]

use redis::AsyncCommands;
use uuid::Uuid;

use crate::handlers::user_status::UserStatusResponse;

const STATUS_CACHE_TTL: u64 = 300; // 5 minutes in seconds
const STATUS_CACHE_PREFIX: &str = "openchat:user_status";

/// Build cache key for a user status
fn status_cache_key(user_id: Uuid) -> String {
    format!("{}:{}", STATUS_CACHE_PREFIX, user_id)
}

/// Get user status from cache
pub async fn get_status(redis: &redis::Client, user_id: Uuid) -> Option<UserStatusResponse> {
    let mut conn = match redis.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(_) => return None,
    };

    let key = status_cache_key(user_id);
    let cached: Result<Option<String>, _> = conn.get(&key).await;

    match cached {
        Ok(Some(json)) => serde_json::from_str(&json).ok(),
        _ => None,
    }
}

/// Store user status in cache
pub async fn set_status(redis: &redis::Client, user_id: Uuid, status: &UserStatusResponse) {
    let mut conn = match redis.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(_) => return,
    };

    let key = status_cache_key(user_id);
    if let Ok(json) = serde_json::to_string(status) {
        let _: Result<(), _> = conn.set_ex(&key, json, STATUS_CACHE_TTL).await;
    }
}

/// Invalidate user status cache
pub async fn invalidate_status(redis: &redis::Client, user_id: Uuid) {
    let mut conn = match redis.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(_) => return,
    };

    let key = status_cache_key(user_id);
    let _: Result<(), _> = conn.del(&key).await;
}
