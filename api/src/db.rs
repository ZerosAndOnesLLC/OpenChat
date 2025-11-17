use sqlx::postgres::{PgPool, PgPoolOptions};
use uuid::Uuid;

use crate::errors::ApiResult;

/// Initialize database connection pool
pub async fn init_pool(database_url: &str) -> ApiResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(database_url)
        .await?;

    Ok(pool)
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
