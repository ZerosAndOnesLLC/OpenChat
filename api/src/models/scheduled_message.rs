use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScheduledMessage {
    pub id: Uuid,
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub content: String,
    pub parent_message_id: Option<Uuid>,
    pub scheduled_at: DateTime<Utc>,
    pub sent: bool,
    pub created_at: DateTime<Utc>,
}

impl ScheduledMessage {
    pub async fn create(
        pool: &PgPool,
        org_id: Uuid,
        user_id: Uuid,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        content: &str,
        parent_message_id: Option<Uuid>,
        scheduled_at: DateTime<Utc>,
    ) -> ApiResult<ScheduledMessage> {
        let row = sqlx::query_as::<_, ScheduledMessage>(
            r#"INSERT INTO scheduled_messages (org_id, user_id, channel_id, dm_id, content, parent_message_id, scheduled_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(org_id)
        .bind(user_id)
        .bind(channel_id)
        .bind(dm_id)
        .bind(content)
        .bind(parent_message_id)
        .bind(scheduled_at)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<ScheduledMessage>> {
        let row = sqlx::query_as::<_, ScheduledMessage>(
            "SELECT * FROM scheduled_messages WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    pub async fn list_pending_by_user(pool: &PgPool, user_id: Uuid) -> ApiResult<Vec<ScheduledMessage>> {
        let rows = sqlx::query_as::<_, ScheduledMessage>(
            r#"SELECT * FROM scheduled_messages
               WHERE user_id = $1 AND sent = false
               ORDER BY scheduled_at ASC"#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        content: Option<&str>,
        scheduled_at: Option<DateTime<Utc>>,
    ) -> ApiResult<ScheduledMessage> {
        let row = sqlx::query_as::<_, ScheduledMessage>(
            r#"UPDATE scheduled_messages
               SET content = COALESCE($1, content),
                   scheduled_at = COALESCE($2, scheduled_at)
               WHERE id = $3 AND sent = false
               RETURNING *"#,
        )
        .bind(content)
        .bind(scheduled_at)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> ApiResult<()> {
        sqlx::query("DELETE FROM scheduled_messages WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn mark_sent(pool: &PgPool, id: Uuid) -> ApiResult<()> {
        sqlx::query("UPDATE scheduled_messages SET sent = true WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn list_due(pool: &PgPool) -> ApiResult<Vec<ScheduledMessage>> {
        let rows = sqlx::query_as::<_, ScheduledMessage>(
            r#"SELECT * FROM scheduled_messages
               WHERE scheduled_at <= NOW() AND sent = false
               LIMIT 50"#,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }
}
