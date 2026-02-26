use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Call {
    pub id: Uuid,
    pub org_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub call_type: String,
    pub status: String,
    pub started_by: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub livekit_room_name: String,
    pub is_huddle: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CallParticipant {
    pub id: Uuid,
    pub call_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub muted: bool,
    pub video_off: bool,
}

impl Call {
    pub async fn create(
        pool: &PgPool,
        org_id: Uuid,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        call_type: &str,
        started_by: Uuid,
        livekit_room_name: &str,
        is_huddle: bool,
    ) -> ApiResult<Call> {
        let status = if is_huddle { "active" } else { "ringing" };
        let call = sqlx::query_as::<_, Call>(
            r#"
            INSERT INTO calls (org_id, channel_id, dm_id, call_type, status, started_by, livekit_room_name, is_huddle)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(channel_id)
        .bind(dm_id)
        .bind(call_type)
        .bind(status)
        .bind(started_by)
        .bind(livekit_room_name)
        .bind(is_huddle)
        .fetch_one(pool)
        .await?;

        Ok(call)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<Call>> {
        let call = sqlx::query_as::<_, Call>("SELECT * FROM calls WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(call)
    }

    pub async fn get_active_for_channel(pool: &PgPool, channel_id: Uuid) -> ApiResult<Option<Call>> {
        let call = sqlx::query_as::<_, Call>(
            "SELECT * FROM calls WHERE channel_id = $1 AND status != 'ended' ORDER BY created_at DESC LIMIT 1",
        )
        .bind(channel_id)
        .fetch_optional(pool)
        .await?;
        Ok(call)
    }

    pub async fn get_active_for_dm(pool: &PgPool, dm_id: Uuid) -> ApiResult<Option<Call>> {
        let call = sqlx::query_as::<_, Call>(
            "SELECT * FROM calls WHERE dm_id = $1 AND status != 'ended' ORDER BY created_at DESC LIMIT 1",
        )
        .bind(dm_id)
        .fetch_optional(pool)
        .await?;
        Ok(call)
    }

    pub async fn get_active_huddle_for_channel(pool: &PgPool, channel_id: Uuid) -> ApiResult<Option<Call>> {
        let call = sqlx::query_as::<_, Call>(
            "SELECT * FROM calls WHERE channel_id = $1 AND is_huddle = true AND status != 'ended' ORDER BY created_at DESC LIMIT 1",
        )
        .bind(channel_id)
        .fetch_optional(pool)
        .await?;
        Ok(call)
    }

    pub async fn set_active(pool: &PgPool, id: Uuid) -> ApiResult<()> {
        sqlx::query("UPDATE calls SET status = 'active' WHERE id = $1 AND status = 'ringing'")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn end_call(pool: &PgPool, id: Uuid) -> ApiResult<()> {
        sqlx::query("UPDATE calls SET status = 'ended', ended_at = NOW() WHERE id = $1 AND status != 'ended'")
            .bind(id)
            .execute(pool)
            .await?;
        // Mark all active participants as left
        sqlx::query("UPDATE call_participants SET left_at = NOW() WHERE call_id = $1 AND left_at IS NULL")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Get active calls in channels the user belongs to
    pub async fn list_active_for_user(pool: &PgPool, user_id: Uuid) -> ApiResult<Vec<ActiveCallInfo>> {
        let calls = sqlx::query_as::<_, ActiveCallInfo>(
            r#"
            SELECT c.id, c.channel_id, c.dm_id, c.call_type, c.status, c.started_by,
                   c.started_at, c.is_huddle, c.livekit_room_name,
                   COUNT(cp.id) FILTER (WHERE cp.left_at IS NULL) as participant_count
            FROM calls c
            LEFT JOIN call_participants cp ON c.id = cp.call_id
            WHERE c.status != 'ended'
              AND (
                c.channel_id IN (SELECT channel_id FROM channel_members WHERE user_id = $1)
                OR c.dm_id IN (SELECT dm_id FROM dm_participants WHERE user_id = $1)
              )
            GROUP BY c.id
            ORDER BY c.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        Ok(calls)
    }

    /// Find stale ringing calls (>60 seconds)
    pub async fn find_stale_ringing(pool: &PgPool) -> ApiResult<Vec<Call>> {
        let calls = sqlx::query_as::<_, Call>(
            "SELECT * FROM calls WHERE status = 'ringing' AND started_at < NOW() - INTERVAL '60 seconds'",
        )
        .fetch_all(pool)
        .await?;
        Ok(calls)
    }

    /// Find active calls with 0 participants (>30 seconds since last leave)
    pub async fn find_empty_active(pool: &PgPool) -> ApiResult<Vec<Call>> {
        let calls = sqlx::query_as::<_, Call>(
            r#"
            SELECT c.* FROM calls c
            WHERE c.status = 'active'
              AND NOT EXISTS (
                SELECT 1 FROM call_participants cp
                WHERE cp.call_id = c.id AND cp.left_at IS NULL
              )
              AND c.started_at < NOW() - INTERVAL '30 seconds'
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(calls)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActiveCallInfo {
    pub id: Uuid,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub call_type: String,
    pub status: String,
    pub started_by: Uuid,
    pub started_at: DateTime<Utc>,
    pub is_huddle: bool,
    pub livekit_room_name: String,
    pub participant_count: i64,
}

impl CallParticipant {
    pub async fn join(pool: &PgPool, call_id: Uuid, user_id: Uuid) -> ApiResult<CallParticipant> {
        let participant = sqlx::query_as::<_, CallParticipant>(
            r#"
            INSERT INTO call_participants (call_id, user_id)
            VALUES ($1, $2)
            ON CONFLICT (call_id, user_id) WHERE left_at IS NULL
            DO UPDATE SET joined_at = NOW()
            RETURNING *
            "#,
        )
        .bind(call_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(participant)
    }

    pub async fn leave(pool: &PgPool, call_id: Uuid, user_id: Uuid) -> ApiResult<()> {
        sqlx::query(
            "UPDATE call_participants SET left_at = NOW() WHERE call_id = $1 AND user_id = $2 AND left_at IS NULL",
        )
        .bind(call_id)
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list_active(pool: &PgPool, call_id: Uuid) -> ApiResult<Vec<CallParticipant>> {
        let participants = sqlx::query_as::<_, CallParticipant>(
            "SELECT * FROM call_participants WHERE call_id = $1 AND left_at IS NULL ORDER BY joined_at",
        )
        .bind(call_id)
        .fetch_all(pool)
        .await?;
        Ok(participants)
    }

    pub async fn count_active(pool: &PgPool, call_id: Uuid) -> ApiResult<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM call_participants WHERE call_id = $1 AND left_at IS NULL",
        )
        .bind(call_id)
        .fetch_one(pool)
        .await?;
        Ok(row.0)
    }
}
