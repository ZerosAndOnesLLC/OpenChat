use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{ApiError, ApiResult};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EncryptedChannel {
    pub channel_id: Uuid,
    pub encryption_enabled: bool,
    pub algorithm: String,
    pub rotation_period_msgs: i32,
    pub rotation_period_ms: i64,
    pub created_at: DateTime<Utc>,
}

impl EncryptedChannel {
    pub async fn enable_encryption(
        pool: &PgPool,
        channel_id: Uuid,
    ) -> ApiResult<EncryptedChannel> {
        let ec = sqlx::query_as::<_, EncryptedChannel>(
            r#"
            INSERT INTO encrypted_channels (channel_id)
            VALUES ($1)
            ON CONFLICT (channel_id) DO NOTHING
            RETURNING *
            "#,
        )
        .bind(channel_id)
        .fetch_optional(pool)
        .await?;

        match ec {
            Some(ec) => Ok(ec),
            None => {
                // Already exists, fetch it
                Self::get(pool, channel_id)
                    .await?
                    .ok_or_else(|| ApiError::Internal("Failed to enable encryption".to_string()))
            }
        }
    }

    pub async fn is_encrypted(pool: &PgPool, channel_id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM encrypted_channels
                WHERE channel_id = $1 AND encryption_enabled = TRUE
            )
            "#,
        )
        .bind(channel_id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    pub async fn get(pool: &PgPool, channel_id: Uuid) -> ApiResult<Option<EncryptedChannel>> {
        let ec = sqlx::query_as::<_, EncryptedChannel>(
            r#"
            SELECT * FROM encrypted_channels
            WHERE channel_id = $1
            "#,
        )
        .bind(channel_id)
        .fetch_optional(pool)
        .await?;

        Ok(ec)
    }
}
