use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelReadStatus {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub last_read_message_id: Option<Uuid>,
    pub last_read_at: DateTime<Utc>,
    pub unread_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DmReadStatus {
    pub id: Uuid,
    pub user_id: Uuid,
    pub dm_id: Uuid,
    pub last_read_message_id: Option<Uuid>,
    pub last_read_at: DateTime<Utc>,
    pub unread_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ChannelReadStatus {
    /// Mark a channel as read for a user
    pub async fn mark_as_read(
        pool: &PgPool,
        user_id: Uuid,
        channel_id: Uuid,
        last_message_id: Option<Uuid>,
    ) -> ApiResult<ChannelReadStatus> {
        let read_status = sqlx::query_as::<_, ChannelReadStatus>(
            r#"
            INSERT INTO channel_read_status (id, user_id, channel_id, last_read_message_id, unread_count)
            VALUES ($1, $2, $3, $4, 0)
            ON CONFLICT (user_id, channel_id)
            DO UPDATE SET
                last_read_message_id = EXCLUDED.last_read_message_id,
                last_read_at = NOW(),
                unread_count = 0,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(channel_id)
        .bind(last_message_id)
        .fetch_one(pool)
        .await?;

        Ok(read_status)
    }

    /// Get unread count for a channel
    pub async fn get_unread_count(
        pool: &PgPool,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> ApiResult<i32> {
        let result = sqlx::query_scalar::<_, i32>(
            r#"
            SELECT COALESCE(
                (SELECT COUNT(*)::int
                FROM messages m
                WHERE m.channel_id = $2
                    AND m.deleted_at IS NULL
                    AND (
                        NOT EXISTS (SELECT 1 FROM channel_read_status crs WHERE crs.user_id = $1 AND crs.channel_id = $2)
                        OR m.created_at > (SELECT last_read_at FROM channel_read_status WHERE user_id = $1 AND channel_id = $2)
                    )
                    AND m.user_id != $1
                ),
                0
            )
            "#,
        )
        .bind(user_id)
        .bind(channel_id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }
}

impl DmReadStatus {
    /// Mark a DM as read for a user
    pub async fn mark_as_read(
        pool: &PgPool,
        user_id: Uuid,
        dm_id: Uuid,
        last_message_id: Option<Uuid>,
    ) -> ApiResult<DmReadStatus> {
        let read_status = sqlx::query_as::<_, DmReadStatus>(
            r#"
            INSERT INTO dm_read_status (id, user_id, dm_id, last_read_message_id, unread_count)
            VALUES ($1, $2, $3, $4, 0)
            ON CONFLICT (user_id, dm_id)
            DO UPDATE SET
                last_read_message_id = EXCLUDED.last_read_message_id,
                last_read_at = NOW(),
                unread_count = 0,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(dm_id)
        .bind(last_message_id)
        .fetch_one(pool)
        .await?;

        Ok(read_status)
    }

    /// Get unread count for a DM
    pub async fn get_unread_count(
        pool: &PgPool,
        user_id: Uuid,
        dm_id: Uuid,
    ) -> ApiResult<i32> {
        let result = sqlx::query_scalar::<_, i32>(
            r#"
            SELECT COALESCE(
                (SELECT COUNT(*)::int
                FROM messages m
                WHERE m.dm_id = $2
                    AND m.deleted_at IS NULL
                    AND (
                        NOT EXISTS (SELECT 1 FROM dm_read_status drs WHERE drs.user_id = $1 AND drs.dm_id = $2)
                        OR m.created_at > (SELECT last_read_at FROM dm_read_status WHERE user_id = $1 AND dm_id = $2)
                    )
                    AND m.user_id != $1
                ),
                0
            )
            "#,
        )
        .bind(user_id)
        .bind(dm_id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }
}
