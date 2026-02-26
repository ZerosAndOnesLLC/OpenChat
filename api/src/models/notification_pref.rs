use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NotificationPref {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub preference: String,
    pub mute_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertNotificationPref {
    pub preference: String,
    pub mute_until: Option<DateTime<Utc>>,
}

impl NotificationPref {
    pub async fn get_for_channel(
        pool: &PgPool,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> ApiResult<Option<NotificationPref>> {
        let pref = sqlx::query_as::<_, NotificationPref>(
            "SELECT * FROM notification_preferences WHERE user_id = $1 AND channel_id = $2",
        )
        .bind(user_id)
        .bind(channel_id)
        .fetch_optional(pool)
        .await?;

        Ok(pref)
    }

    pub async fn get_for_dm(
        pool: &PgPool,
        user_id: Uuid,
        dm_id: Uuid,
    ) -> ApiResult<Option<NotificationPref>> {
        let pref = sqlx::query_as::<_, NotificationPref>(
            "SELECT * FROM notification_preferences WHERE user_id = $1 AND dm_id = $2",
        )
        .bind(user_id)
        .bind(dm_id)
        .fetch_optional(pool)
        .await?;

        Ok(pref)
    }

    pub async fn list_by_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> ApiResult<Vec<NotificationPref>> {
        let prefs = sqlx::query_as::<_, NotificationPref>(
            "SELECT * FROM notification_preferences WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(prefs)
    }

    /// Batch-load notification prefs for a list of users for a specific channel.
    /// Returns a map of user_id -> NotificationPref.
    pub async fn list_for_channel_users(
        pool: &PgPool,
        channel_id: Uuid,
        user_ids: &[Uuid],
    ) -> ApiResult<std::collections::HashMap<Uuid, NotificationPref>> {
        let prefs = sqlx::query_as::<_, NotificationPref>(
            "SELECT * FROM notification_preferences WHERE channel_id = $1 AND user_id = ANY($2)",
        )
        .bind(channel_id)
        .bind(user_ids)
        .fetch_all(pool)
        .await?;

        let map = prefs.into_iter().map(|p| (p.user_id, p)).collect();
        Ok(map)
    }

    /// Batch-load notification prefs for a list of users for a specific DM.
    pub async fn list_for_dm_users(
        pool: &PgPool,
        dm_id: Uuid,
        user_ids: &[Uuid],
    ) -> ApiResult<std::collections::HashMap<Uuid, NotificationPref>> {
        let prefs = sqlx::query_as::<_, NotificationPref>(
            "SELECT * FROM notification_preferences WHERE dm_id = $1 AND user_id = ANY($2)",
        )
        .bind(dm_id)
        .bind(user_ids)
        .fetch_all(pool)
        .await?;

        let map = prefs.into_iter().map(|p| (p.user_id, p)).collect();
        Ok(map)
    }

    pub async fn upsert_channel(
        pool: &PgPool,
        user_id: Uuid,
        channel_id: Uuid,
        data: UpsertNotificationPref,
    ) -> ApiResult<NotificationPref> {
        let pref = sqlx::query_as::<_, NotificationPref>(
            r#"
            INSERT INTO notification_preferences (id, user_id, channel_id, preference, mute_until)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id, channel_id) WHERE channel_id IS NOT NULL
            DO UPDATE SET preference = EXCLUDED.preference,
                          mute_until = EXCLUDED.mute_until,
                          updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(channel_id)
        .bind(&data.preference)
        .bind(data.mute_until)
        .fetch_one(pool)
        .await?;

        Ok(pref)
    }

    pub async fn upsert_dm(
        pool: &PgPool,
        user_id: Uuid,
        dm_id: Uuid,
        data: UpsertNotificationPref,
    ) -> ApiResult<NotificationPref> {
        let pref = sqlx::query_as::<_, NotificationPref>(
            r#"
            INSERT INTO notification_preferences (id, user_id, dm_id, preference, mute_until)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id, dm_id) WHERE dm_id IS NOT NULL
            DO UPDATE SET preference = EXCLUDED.preference,
                          mute_until = EXCLUDED.mute_until,
                          updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(dm_id)
        .bind(&data.preference)
        .bind(data.mute_until)
        .fetch_one(pool)
        .await?;

        Ok(pref)
    }

    pub async fn delete_channel(
        pool: &PgPool,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> ApiResult<bool> {
        let result = sqlx::query(
            "DELETE FROM notification_preferences WHERE user_id = $1 AND channel_id = $2",
        )
        .bind(user_id)
        .bind(channel_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_dm(
        pool: &PgPool,
        user_id: Uuid,
        dm_id: Uuid,
    ) -> ApiResult<bool> {
        let result = sqlx::query(
            "DELETE FROM notification_preferences WHERE user_id = $1 AND dm_id = $2",
        )
        .bind(user_id)
        .bind(dm_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Determine if a notification should be sent based on the user's preference.
    /// - `None` or `"all"` → true
    /// - `"mentions"` → true only if `is_direct_mention`
    /// - `"nothing"` → false
    /// - If `mute_until` is set and not expired → false
    pub fn should_notify(pref: Option<&NotificationPref>, is_direct_mention: bool) -> bool {
        let Some(pref) = pref else {
            return true;
        };

        // Check mute_until first — if muted and not expired, suppress
        if let Some(mute_until) = pref.mute_until {
            if Utc::now() < mute_until {
                return false;
            }
        }

        match pref.preference.as_str() {
            "all" => true,
            "mentions" => is_direct_mention,
            "nothing" => false,
            _ => true, // unknown preference defaults to all
        }
    }
}
