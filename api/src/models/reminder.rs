use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Reminder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub message_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub remind_at: DateTime<Utc>,
    pub message_preview: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}

impl Reminder {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        org_id: Uuid,
        message_id: Uuid,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        remind_at: DateTime<Utc>,
        message_preview: &str,
    ) -> ApiResult<Reminder> {
        let row = sqlx::query_as::<_, Reminder>(
            r#"INSERT INTO reminders (user_id, org_id, message_id, channel_id, dm_id, remind_at, message_preview)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(org_id)
        .bind(message_id)
        .bind(channel_id)
        .bind(dm_id)
        .bind(remind_at)
        .bind(message_preview)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<Reminder>> {
        let row = sqlx::query_as::<_, Reminder>(
            "SELECT * FROM reminders WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    pub async fn list_pending_by_user(pool: &PgPool, user_id: Uuid) -> ApiResult<Vec<Reminder>> {
        let rows = sqlx::query_as::<_, Reminder>(
            r#"SELECT * FROM reminders
               WHERE user_id = $1 AND completed = false
               ORDER BY remind_at ASC"#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> ApiResult<()> {
        sqlx::query("DELETE FROM reminders WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn mark_completed(pool: &PgPool, id: Uuid) -> ApiResult<()> {
        sqlx::query("UPDATE reminders SET completed = true WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn list_due(pool: &PgPool) -> ApiResult<Vec<Reminder>> {
        let rows = sqlx::query_as::<_, Reminder>(
            r#"SELECT * FROM reminders
               WHERE remind_at <= NOW() AND completed = false
               LIMIT 50"#,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }
}
