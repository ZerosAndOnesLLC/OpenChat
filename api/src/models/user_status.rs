use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[allow(dead_code)]
pub enum StatusType {
    Online,
    Away,
    Dnd,
    Offline,
}

impl std::fmt::Display for StatusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusType::Online => write!(f, "online"),
            StatusType::Away => write!(f, "away"),
            StatusType::Dnd => write!(f, "dnd"),
            StatusType::Offline => write!(f, "offline"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserStatus {
    pub user_id: Uuid,
    pub status: String,
    pub custom_message: Option<String>,
    pub emoji: Option<String>,
    pub clear_at: Option<DateTime<Utc>>,
    pub back_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateStatusRequest {
    pub status: String,
    pub custom_message: Option<String>,
    pub emoji: Option<String>,
    pub clear_after_minutes: Option<i32>, // Convert to clear_at
    pub back_at: Option<String>,          // ISO 8601 datetime for when user will be back
}

impl UserStatus {
    /// Update or insert user status
    pub async fn upsert(
        pool: &PgPool,
        user_id: Uuid,
        status: &str,
        custom_message: Option<&str>,
        emoji: Option<&str>,
        clear_at: Option<DateTime<Utc>>,
        back_at: Option<DateTime<Utc>>,
    ) -> ApiResult<UserStatus> {
        let user_status = sqlx::query_as::<_, UserStatus>(
            r#"
            INSERT INTO user_status (user_id, status, custom_message, emoji, clear_at, back_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id)
            DO UPDATE SET
                status = EXCLUDED.status,
                custom_message = EXCLUDED.custom_message,
                emoji = EXCLUDED.emoji,
                clear_at = EXCLUDED.clear_at,
                back_at = EXCLUDED.back_at,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(status)
        .bind(custom_message)
        .bind(emoji)
        .bind(clear_at)
        .bind(back_at)
        .fetch_one(pool)
        .await?;

        Ok(user_status)
    }

    /// Get user status by user ID
    pub async fn get_by_user_id(pool: &PgPool, user_id: Uuid) -> ApiResult<Option<UserStatus>> {
        let status = sqlx::query_as::<_, UserStatus>(
            r#"
            SELECT * FROM user_status
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(status)
    }

    /// Get multiple user statuses by user IDs
    #[allow(dead_code)]
    pub async fn get_by_user_ids(
        pool: &PgPool,
        user_ids: &[Uuid],
    ) -> ApiResult<Vec<UserStatus>> {
        let statuses = sqlx::query_as::<_, UserStatus>(
            r#"
            SELECT * FROM user_status
            WHERE user_id = ANY($1)
            "#,
        )
        .bind(user_ids)
        .fetch_all(pool)
        .await?;

        Ok(statuses)
    }

    /// Get all online/away/dnd users in an organization
    pub async fn get_active_users(pool: &PgPool, org_id: Uuid) -> ApiResult<Vec<UserStatus>> {
        let statuses = sqlx::query_as::<_, UserStatus>(
            r#"
            SELECT us.* FROM user_status us
            JOIN users u ON us.user_id = u.id
            WHERE u.org_id = $1 AND us.status IN ('online', 'away', 'dnd')
            ORDER BY us.updated_at DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(pool)
        .await?;

        Ok(statuses)
    }

    /// Set user status to offline
    pub async fn set_offline(pool: &PgPool, user_id: Uuid) -> ApiResult<UserStatus> {
        Self::upsert(pool, user_id, "offline", None, None, None, None).await
    }

    /// Set user status to online
    pub async fn set_online(pool: &PgPool, user_id: Uuid) -> ApiResult<UserStatus> {
        Self::upsert(pool, user_id, "online", None, None, None, None).await
    }

    /// Set user status to away
    pub async fn set_away(pool: &PgPool, user_id: Uuid) -> ApiResult<UserStatus> {
        Self::upsert(pool, user_id, "away", None, None, None, None).await
    }

    /// Clear expired custom statuses (run periodically)
    pub async fn clear_expired_statuses(pool: &PgPool) -> ApiResult<u64> {
        let result = sqlx::query(
            r#"
            UPDATE user_status
            SET custom_message = NULL, emoji = NULL, clear_at = NULL
            WHERE clear_at IS NOT NULL AND clear_at <= NOW()
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Auto-set users to 'away' if they've been inactive for more than 15 minutes
    /// Returns list of user IDs that were set to away
    pub async fn auto_away_inactive_users(pool: &PgPool) -> ApiResult<Vec<Uuid>> {
        // Set users to 'away' if their status is 'online' and they haven't updated in 15+ minutes
        let updated_users = sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE user_status
            SET status = 'away', updated_at = NOW()
            WHERE status = 'online'
            AND updated_at < NOW() - INTERVAL '15 minutes'
            RETURNING user_id
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(updated_users)
    }

    /// Update the last activity timestamp for a user (without changing their status)
    /// This is called on WebSocket heartbeat and user actions
    pub async fn touch_activity(pool: &PgPool, user_id: Uuid) -> ApiResult<()> {
        sqlx::query(
            r#"
            UPDATE user_status
            SET updated_at = NOW()
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
