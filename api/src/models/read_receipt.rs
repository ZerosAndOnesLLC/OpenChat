use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageReadReceipt {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub read_at: DateTime<Utc>,
}

impl MessageReadReceipt {
    /// Record that a user has read a message
    pub async fn record(
        pool: &PgPool,
        message_id: Uuid,
        user_id: Uuid,
    ) -> ApiResult<MessageReadReceipt> {
        let receipt = sqlx::query_as::<_, MessageReadReceipt>(
            r#"
            INSERT INTO message_read_receipts (id, message_id, user_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (message_id, user_id) DO UPDATE
            SET read_at = NOW()
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(message_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(receipt)
    }

    /// Record multiple read receipts at once (batch operation)
    pub async fn record_batch(
        pool: &PgPool,
        message_ids: Vec<Uuid>,
        user_id: Uuid,
    ) -> ApiResult<Vec<MessageReadReceipt>> {
        let receipts = sqlx::query_as::<_, MessageReadReceipt>(
            r#"
            INSERT INTO message_read_receipts (id, message_id, user_id)
            SELECT gen_random_uuid(), unnest($1::UUID[]), $2
            ON CONFLICT (message_id, user_id) DO UPDATE
            SET read_at = NOW()
            RETURNING *
            "#,
        )
        .bind(&message_ids)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(receipts)
    }

    /// Get all receipts for a specific message
    pub async fn get_by_message(
        pool: &PgPool,
        message_id: Uuid,
    ) -> ApiResult<Vec<MessageReadReceipt>> {
        let receipts = sqlx::query_as::<_, MessageReadReceipt>(
            r#"
            SELECT * FROM message_read_receipts
            WHERE message_id = $1
            ORDER BY read_at DESC
            "#,
        )
        .bind(message_id)
        .fetch_all(pool)
        .await?;

        Ok(receipts)
    }

    /// Get all receipts by a specific user
    pub async fn get_by_user(pool: &PgPool, user_id: Uuid) -> ApiResult<Vec<MessageReadReceipt>> {
        let receipts = sqlx::query_as::<_, MessageReadReceipt>(
            r#"
            SELECT * FROM message_read_receipts
            WHERE user_id = $1
            ORDER BY read_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(receipts)
    }

    /// Check if a user has read a specific message
    pub async fn has_read(pool: &PgPool, message_id: Uuid, user_id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM message_read_receipts WHERE message_id = $1 AND user_id = $2)",
        )
        .bind(message_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Get count of how many users have read a message
    pub async fn count_by_message(pool: &PgPool, message_id: Uuid) -> ApiResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM message_read_receipts WHERE message_id = $1",
        )
        .bind(message_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    /// Get receipts for a message with user details
    pub async fn get_with_user_details(
        pool: &PgPool,
        message_id: Uuid,
    ) -> ApiResult<Vec<ReadReceiptWithUser>> {
        let receipts = sqlx::query_as::<_, ReadReceiptWithUser>(
            r#"
            SELECT
                mrr.id,
                mrr.message_id,
                mrr.user_id,
                mrr.read_at,
                u.display_name,
                u.avatar_url
            FROM message_read_receipts mrr
            INNER JOIN users u ON mrr.user_id = u.id
            WHERE mrr.message_id = $1
            ORDER BY mrr.read_at DESC
            "#,
        )
        .bind(message_id)
        .fetch_all(pool)
        .await?;

        Ok(receipts)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReadReceiptWithUser {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub read_at: DateTime<Utc>,
    pub display_name: String,
    pub avatar_url: Option<String>,
}
