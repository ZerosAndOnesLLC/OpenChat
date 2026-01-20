use std::time::Duration;

use deadpool_redis::{Config as RedisConfig, Pool as DeadpoolRedisPool, PoolConfig, Runtime, Timeouts};
use redis::Client;
use sqlx::postgres::{PgPool, PgPoolOptions};
use uuid::Uuid;

use crate::errors::ApiResult;

/// Type alias for Redis connection pool
pub type RedisPool = DeadpoolRedisPool;

/// Initialize database connection pool
pub async fn init_pool(database_url: &str) -> ApiResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(database_url)
        .await?;

    Ok(pool)
}

/// Initialize Redis connection pool with deadpool for automatic reconnection
pub fn init_redis_pool(redis_url: &str) -> ApiResult<RedisPool> {
    let mut redis_cfg = RedisConfig::from_url(redis_url);
    redis_cfg.pool = Some(PoolConfig {
        max_size: 16,
        timeouts: Timeouts {
            wait: Some(Duration::from_secs(5)),
            create: Some(Duration::from_secs(5)),
            recycle: Some(Duration::from_secs(5)),
        },
        ..Default::default()
    });
    let pool = redis_cfg
        .create_pool(Some(Runtime::Tokio1))
        .map_err(|e| crate::errors::ApiError::Internal(format!("Failed to create Redis pool: {}", e)))?;
    Ok(pool)
}

/// Initialize Redis client (for pub/sub and other direct client needs)
pub fn init_redis_client(redis_url: &str) -> ApiResult<Client> {
    let client = Client::open(redis_url)?;
    Ok(client)
}

/// Set the RLS (Row Level Security) context for the current database connection
/// This ensures all queries are automatically filtered by org_id
/// Note: Uses SET instead of SET LOCAL since we're not in an explicit transaction
#[allow(dead_code)]
pub async fn set_rls_context(pool: &PgPool, org_id: Uuid) -> ApiResult<()> {
    sqlx::query(&format!("SET app.current_org_id = '{}'", org_id))
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires database to be running
    async fn test_pool_initialization() {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for tests");

        let pool = init_pool(&database_url).await;
        assert!(pool.is_ok());
    }
}
