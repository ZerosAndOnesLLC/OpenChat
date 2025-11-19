use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageEdit {
    pub id: Uuid,
    pub message_id: Uuid,
    pub old_content: String,
    pub edited_by: Uuid,
    pub edited_at: DateTime<Utc>,
}

impl MessageEdit {
    /// Create a new message edit record
    pub async fn create(
        pool: &PgPool,
        message_id: Uuid,
        old_content: &str,
        edited_by: Uuid,
    ) -> ApiResult<MessageEdit> {
        let edit = sqlx::query_as::<_, MessageEdit>(
            r#"
            INSERT INTO message_edits (message_id, old_content, edited_by)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(message_id)
        .bind(old_content)
        .bind(edited_by)
        .fetch_one(pool)
        .await?;

        Ok(edit)
    }

    /// Get edit history for a message
    pub async fn list_by_message(
        pool: &PgPool,
        message_id: Uuid,
    ) -> ApiResult<Vec<MessageEdit>> {
        let edits = sqlx::query_as::<_, MessageEdit>(
            r#"
            SELECT * FROM message_edits
            WHERE message_id = $1
            ORDER BY edited_at DESC
            "#,
        )
        .bind(message_id)
        .fetch_all(pool)
        .await?;

        Ok(edits)
    }
}
