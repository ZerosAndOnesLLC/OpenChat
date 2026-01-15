// Rate limiting using Redis
// Implements token bucket algorithm for smooth rate limiting

use redis::AsyncCommands;
use uuid::Uuid;

use crate::db::RedisPool;
use crate::errors::ApiResult;

const RATE_LIMIT_PREFIX: &str = "openchat:ratelimit";

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed
    pub max_requests: u32,
    /// Time window in seconds
    pub window_seconds: u64,
}

impl RateLimitConfig {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
        }
    }
}

/// Rate limit types
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum RateLimitType {
    /// API requests (200/second)
    ApiRequest,
    /// Messages (30/second)
    Message,
    /// WebSocket messages (1000/minute)
    WebSocket,
    /// Device pairing code generation (5/minute)
    DevicePairingGenerate,
    /// Device pairing code verification (10/minute per IP)
    DevicePairingVerify,
}

impl RateLimitType {
    pub fn config(&self) -> RateLimitConfig {
        match self {
            // 200 requests per second - handles page loads with multiple API calls
            RateLimitType::ApiRequest => RateLimitConfig::new(200, 1),
            // 30 messages per second - generous for even fast typers
            RateLimitType::Message => RateLimitConfig::new(30, 1),
            // 1000 WebSocket messages per minute
            RateLimitType::WebSocket => RateLimitConfig::new(1000, 60),
            // Device pairing stays conservative for security
            RateLimitType::DevicePairingGenerate => RateLimitConfig::new(5, 60),
            RateLimitType::DevicePairingVerify => RateLimitConfig::new(10, 60),
        }
    }

    pub fn key_suffix(&self) -> &str {
        match self {
            RateLimitType::ApiRequest => "api",
            RateLimitType::Message => "message",
            RateLimitType::WebSocket => "ws",
            RateLimitType::DevicePairingGenerate => "device_pair_gen",
            RateLimitType::DevicePairingVerify => "device_pair_verify",
        }
    }
}

/// Build cache key for rate limit
fn rate_limit_cache_key(user_id: Uuid, limit_type: &RateLimitType) -> String {
    format!(
        "{}:{}:{}",
        RATE_LIMIT_PREFIX,
        limit_type.key_suffix(),
        user_id
    )
}

/// Build cache key for IP-based rate limit
fn rate_limit_cache_key_by_ip(ip_addr: &str, limit_type: &RateLimitType) -> String {
    format!(
        "{}:{}:ip:{}",
        RATE_LIMIT_PREFIX,
        limit_type.key_suffix(),
        ip_addr
    )
}

/// Check if request should be rate limited
/// Returns (allowed, remaining, reset_time_seconds)
/// Rate limiting fails open - if Redis is unavailable, requests are allowed
pub async fn check_rate_limit(
    redis_pool: &RedisPool,
    user_id: Uuid,
    limit_type: RateLimitType,
) -> ApiResult<(bool, u32, u64)> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            // Fail open - allow request if Redis is unavailable
            tracing::warn!("Redis pool error in rate limit check, allowing request: {}", e);
            let config = limit_type.config();
            return Ok((true, config.max_requests, config.window_seconds));
        }
    };

    let config = limit_type.config();
    let key = rate_limit_cache_key(user_id, &limit_type);

    // Get current count
    let current: Option<u32> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error in rate limit check, allowing request: {}", e);
            return Ok((true, config.max_requests, config.window_seconds));
        }
    };

    let (count, ttl) = match current {
        Some(count) if count >= config.max_requests => {
            // Rate limit exceeded
            let ttl: i64 = conn.ttl(&key).await.unwrap_or(0);
            return Ok((false, 0, ttl.max(0) as u64));
        }
        Some(_count) => {
            // Increment count
            let new_count: u32 = conn.incr(&key, 1).await?;
            let ttl: i64 = conn.ttl(&key).await.unwrap_or(0);
            (new_count, ttl.max(0) as u64)
        }
        None => {
            // First request in window, set with expiry
            let _: () = conn
                .set_ex(&key, 1u32, config.window_seconds)
                .await?;
            (1, config.window_seconds)
        }
    };

    let remaining = config.max_requests.saturating_sub(count);
    Ok((true, remaining, ttl))
}

/// Check if request should be rate limited by IP address
/// Returns (allowed, remaining, reset_time_seconds)
/// Used for public endpoints like device pairing verification
/// Rate limiting fails open - if Redis is unavailable, requests are allowed
pub async fn check_rate_limit_by_ip(
    redis_pool: &RedisPool,
    ip_addr: &str,
    limit_type: RateLimitType,
) -> ApiResult<(bool, u32, u64)> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            // Fail open - allow request if Redis is unavailable
            tracing::warn!("Redis pool error in IP rate limit check, allowing request: {}", e);
            let config = limit_type.config();
            return Ok((true, config.max_requests, config.window_seconds));
        }
    };

    let config = limit_type.config();
    let key = rate_limit_cache_key_by_ip(ip_addr, &limit_type);

    // Get current count
    let current: Option<u32> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error in IP rate limit check, allowing request: {}", e);
            return Ok((true, config.max_requests, config.window_seconds));
        }
    };

    let (count, ttl) = match current {
        Some(count) if count >= config.max_requests => {
            // Rate limit exceeded
            let ttl: i64 = conn.ttl(&key).await.unwrap_or(0);
            return Ok((false, 0, ttl.max(0) as u64));
        }
        Some(_count) => {
            // Increment count
            let new_count: u32 = conn.incr(&key, 1).await?;
            let ttl: i64 = conn.ttl(&key).await.unwrap_or(0);
            (new_count, ttl.max(0) as u64)
        }
        None => {
            // First request in window, set with expiry
            let _: () = conn
                .set_ex(&key, 1u32, config.window_seconds)
                .await?;
            (1, config.window_seconds)
        }
    };

    let remaining = config.max_requests.saturating_sub(count);
    Ok((true, remaining, ttl))
}

/// Reset rate limit for a user (useful for testing or admin override)
#[allow(dead_code)]
pub async fn reset_rate_limit(
    redis_pool: &RedisPool,
    user_id: Uuid,
    limit_type: RateLimitType,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let key = rate_limit_cache_key(user_id, &limit_type);
    let _: () = conn.del(&key).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis to be running
    async fn test_rate_limiting() {
        // Test would need to be updated to use RedisPool
        // For now, just a placeholder
    }
}
