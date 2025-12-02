// Rate limiting using Redis
// Implements token bucket algorithm for smooth rate limiting

use redis::AsyncCommands;
use uuid::Uuid;

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
    /// API requests (20/second)
    ApiRequest,
    /// Messages (5/second)
    Message,
    /// WebSocket messages (100/minute)
    WebSocket,
    /// Device pairing code generation (3/minute)
    DevicePairingGenerate,
    /// Device pairing code verification (5/minute per IP)
    DevicePairingVerify,
}

impl RateLimitType {
    pub fn config(&self) -> RateLimitConfig {
        match self {
            RateLimitType::ApiRequest => RateLimitConfig::new(20, 1),
            RateLimitType::Message => RateLimitConfig::new(5, 1),
            RateLimitType::WebSocket => RateLimitConfig::new(100, 60),
            RateLimitType::DevicePairingGenerate => RateLimitConfig::new(3, 60),
            RateLimitType::DevicePairingVerify => RateLimitConfig::new(5, 60),
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
pub async fn check_rate_limit(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    limit_type: RateLimitType,
) -> ApiResult<(bool, u32, u64)> {
    let config = limit_type.config();
    let key = rate_limit_cache_key(user_id, &limit_type);

    // Get current count
    let current: Option<u32> = redis.get(&key).await?;

    let (count, ttl) = match current {
        Some(count) if count >= config.max_requests => {
            // Rate limit exceeded
            let ttl: i64 = redis.ttl(&key).await?;
            return Ok((false, 0, ttl.max(0) as u64));
        }
        Some(_count) => {
            // Increment count
            let new_count: u32 = redis.incr(&key, 1).await?;
            let ttl: i64 = redis.ttl(&key).await?;
            (new_count, ttl.max(0) as u64)
        }
        None => {
            // First request in window, set with expiry
            let _: () = redis
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
pub async fn check_rate_limit_by_ip(
    redis: &mut redis::aio::MultiplexedConnection,
    ip_addr: &str,
    limit_type: RateLimitType,
) -> ApiResult<(bool, u32, u64)> {
    let config = limit_type.config();
    let key = rate_limit_cache_key_by_ip(ip_addr, &limit_type);

    // Get current count
    let current: Option<u32> = redis.get(&key).await?;

    let (count, ttl) = match current {
        Some(count) if count >= config.max_requests => {
            // Rate limit exceeded
            let ttl: i64 = redis.ttl(&key).await?;
            return Ok((false, 0, ttl.max(0) as u64));
        }
        Some(_count) => {
            // Increment count
            let new_count: u32 = redis.incr(&key, 1).await?;
            let ttl: i64 = redis.ttl(&key).await?;
            (new_count, ttl.max(0) as u64)
        }
        None => {
            // First request in window, set with expiry
            let _: () = redis
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
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    limit_type: RateLimitType,
) -> ApiResult<()> {
    let key = rate_limit_cache_key(user_id, &limit_type);
    let _: () = redis.del(&key).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis to be running
    async fn test_rate_limiting() {
        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set for tests");
        let client = redis::Client::open(redis_url).unwrap();
        let mut conn = client.get_multiplexed_async_connection().await.unwrap();

        let user_id = Uuid::new_v4();
        let limit_type = RateLimitType::Message; // 5/second

        // Reset before test
        reset_rate_limit(&mut conn, user_id, RateLimitType::Message)
            .await
            .unwrap();

        // First 5 requests should succeed
        for i in 1..=5 {
            let (allowed, remaining, _) = check_rate_limit(
                &mut conn,
                user_id,
                RateLimitType::Message,
            )
            .await
            .unwrap();

            assert!(allowed, "Request {} should be allowed", i);
            assert_eq!(remaining, 5 - i, "Remaining should be {}", 5 - i);
        }

        // 6th request should be rate limited
        let (allowed, remaining, reset_time) = check_rate_limit(
            &mut conn,
            user_id,
            RateLimitType::Message,
        )
        .await
        .unwrap();

        assert!(!allowed, "Request 6 should be rate limited");
        assert_eq!(remaining, 0, "No requests should remain");
        assert!(reset_time > 0, "Reset time should be set");

        // Clean up
        reset_rate_limit(&mut conn, user_id, RateLimitType::Message)
            .await
            .unwrap();
    }
}
