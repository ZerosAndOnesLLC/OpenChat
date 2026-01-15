use redis::AsyncCommands;
use sha2::{Digest, Sha256};
use tracing::debug;

use crate::db::RedisPool;
use crate::{errors::ApiResult, services::tv_api::TokenClaims};

/// Get cached token claims from Redis
/// Uses SHA256 hash of token as cache key for security
pub async fn get_cached_token_claims(
    redis_pool: &RedisPool,
    token: &str,
) -> ApiResult<Option<TokenClaims>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool error fetching token cache: {}", e);
            return Ok(None);
        }
    };

    let hash = hash_token(token);
    let key = format!("openchat:token:{}", hash);

    let cached: Option<String> = match conn.get(&key).await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("Redis error fetching token cache: {}", e);
            return Ok(None);
        }
    };

    match cached {
        Some(json) => {
            debug!("Token cache hit");
            let claims: TokenClaims = serde_json::from_str(&json)
                .map_err(|e| crate::errors::ApiError::Internal(format!("Token cache deserialization error: {}", e)))?;
            Ok(Some(claims))
        }
        None => {
            debug!("Token cache miss");
            Ok(None)
        }
    }
}

/// Cache token claims in Redis with TTL
/// TTL of 300 seconds (5 minutes) balances security and performance
pub async fn cache_token_claims(
    redis_pool: &RedisPool,
    token: &str,
    claims: &TokenClaims,
    ttl_seconds: u64,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let hash = hash_token(token);
    let key = format!("openchat:token:{}", hash);
    let value = serde_json::to_string(claims)
        .map_err(|e| crate::errors::ApiError::Internal(format!("Token cache serialization error: {}", e)))?;

    let _: () = conn.set_ex(&key, value, ttl_seconds).await?;

    debug!("Token cached successfully with TTL: {}s", ttl_seconds);
    Ok(())
}

/// Invalidate a cached token (e.g., on logout)
#[allow(dead_code)]
pub async fn invalidate_token_cache(
    redis_pool: &RedisPool,
    token: &str,
) -> ApiResult<()> {
    let mut conn = redis_pool.get().await
        .map_err(|e| crate::errors::ApiError::Internal(format!("Redis pool error: {}", e)))?;

    let hash = hash_token(token);
    let key = format!("openchat:token:{}", hash);

    let _: () = conn.del(&key).await?;

    debug!("Token cache invalidated");
    Ok(())
}

/// Hash token using SHA256 for secure cache key
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_token() {
        let token = "test_token_123";
        let hash = hash_token(token);
        // Hash should be deterministic
        assert_eq!(hash, hash_token(token));
        // Hash should be 64 characters (SHA256 hex)
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_different_tokens_different_hashes() {
        let token1 = "token1";
        let token2 = "token2";
        assert_ne!(hash_token(token1), hash_token(token2));
    }
}
