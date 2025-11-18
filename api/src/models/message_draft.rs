use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct MessageDraft {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MessageDraft {
    /// Save or update a draft for a channel
    pub async fn save_for_channel(
        pool: &PgPool,
        user_id: Uuid,
        channel_id: Uuid,
        content: String,
    ) -> ApiResult<Self> {
        let draft = sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO message_drafts (user_id, channel_id, content)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, channel_id)
            DO UPDATE SET
                content = EXCLUDED.content,
                updated_at = now()
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(channel_id)
        .bind(content)
        .fetch_one(pool)
        .await?;

        Ok(draft)
    }

    /// Save or update a draft for a DM
    pub async fn save_for_dm(
        pool: &PgPool,
        user_id: Uuid,
        dm_id: Uuid,
        content: String,
    ) -> ApiResult<Self> {
        let draft = sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO message_drafts (user_id, dm_id, content)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, dm_id)
            DO UPDATE SET
                content = EXCLUDED.content,
                updated_at = now()
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(dm_id)
        .bind(content)
        .fetch_one(pool)
        .await?;

        Ok(draft)
    }

    /// Get draft for a channel
    pub async fn get_for_channel(
        pool: &PgPool,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> ApiResult<Option<Self>> {
        let draft = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM message_drafts
            WHERE user_id = $1 AND channel_id = $2
            "#,
        )
        .bind(user_id)
        .bind(channel_id)
        .fetch_optional(pool)
        .await?;

        Ok(draft)
    }

    /// Get draft for a DM
    pub async fn get_for_dm(
        pool: &PgPool,
        user_id: Uuid,
        dm_id: Uuid,
    ) -> ApiResult<Option<Self>> {
        let draft = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM message_drafts
            WHERE user_id = $1 AND dm_id = $2
            "#,
        )
        .bind(user_id)
        .bind(dm_id)
        .fetch_optional(pool)
        .await?;

        Ok(draft)
    }

    /// Get all drafts for a user
    pub async fn get_all_for_user(pool: &PgPool, user_id: Uuid) -> ApiResult<Vec<Self>> {
        let drafts = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM message_drafts
            WHERE user_id = $1
            ORDER BY updated_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(drafts)
    }

    /// Delete draft for a channel
    pub async fn delete_for_channel(
        pool: &PgPool,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> ApiResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM message_drafts
            WHERE user_id = $1 AND channel_id = $2
            "#,
        )
        .bind(user_id)
        .bind(channel_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete draft for a DM
    pub async fn delete_for_dm(pool: &PgPool, user_id: Uuid, dm_id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM message_drafts
            WHERE user_id = $1 AND dm_id = $2
            "#,
        )
        .bind(user_id)
        .bind(dm_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete all drafts for a user
    pub async fn delete_all_for_user(pool: &PgPool, user_id: Uuid) -> ApiResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM message_drafts
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
