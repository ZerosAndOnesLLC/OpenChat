use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum NotificationType {
    Mention,
    Dm,
    ThreadReply,
    ChannelInvite,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub message_id: Option<Uuid>,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotification {
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub message_id: Option<Uuid>,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
}

impl Notification {
    /// Create a new notification
    pub async fn create(pool: &PgPool, data: CreateNotification) -> ApiResult<Notification> {
        let notification = sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications (id, user_id, notification_type, message_id, channel_id, dm_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(data.user_id)
        .bind(data.notification_type)
        .bind(data.message_id)
        .bind(data.channel_id)
        .bind(data.dm_id)
        .fetch_one(pool)
        .await?;

        Ok(notification)
    }

    /// Create multiple notifications in a batch
    #[allow(dead_code)]
    pub async fn create_batch(pool: &PgPool, notifications: Vec<CreateNotification>) -> ApiResult<Vec<Notification>> {
        let mut created_notifications = Vec::new();

        for notification_data in notifications {
            let notification = Self::create(pool, notification_data).await?;
            created_notifications.push(notification);
        }

        Ok(created_notifications)
    }

    /// Get all notifications for a user
    pub async fn list_by_user(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> ApiResult<Vec<Notification>> {
        let notifications = sqlx::query_as::<_, Notification>(
            r#"
            SELECT * FROM notifications
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(notifications)
    }

    /// Get unread notifications for a user
    pub async fn list_unread_by_user(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> ApiResult<Vec<Notification>> {
        let notifications = sqlx::query_as::<_, Notification>(
            r#"
            SELECT * FROM notifications
            WHERE user_id = $1 AND read = false
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(notifications)
    }

    /// Count unread notifications for a user
    pub async fn count_unread_by_user(pool: &PgPool, user_id: Uuid) -> ApiResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND read = false",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    /// Mark a notification as read
    pub async fn mark_as_read(pool: &PgPool, notification_id: Uuid, user_id: Uuid) -> ApiResult<()> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET read = true
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(notification_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Mark all notifications as read for a user
    pub async fn mark_all_as_read(pool: &PgPool, user_id: Uuid) -> ApiResult<i64> {
        let result = sqlx::query(
            "UPDATE notifications SET read = true WHERE user_id = $1 AND read = false",
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Delete old read notifications (cleanup job)
    #[allow(dead_code)]
    pub async fn delete_old_read(pool: &PgPool, days: i64) -> ApiResult<i64> {
        let result = sqlx::query(
            r#"
            DELETE FROM notifications
            WHERE read = true AND created_at < NOW() - $1::interval
            "#,
        )
        .bind(format!("{} days", days))
        .execute(pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}
