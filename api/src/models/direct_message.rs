use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DirectMessage {
    pub id: Uuid,
    pub org_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DmParticipant {
    pub id: Uuid,
    pub dm_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    #[serde(default)]
    pub hidden: bool,
}

impl DirectMessage {
    /// List all DMs for a user in their organization (excludes hidden)
    pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> ApiResult<Vec<DirectMessage>> {
        let dms = sqlx::query_as::<_, DirectMessage>(
            r#"
            SELECT DISTINCT dm.*
            FROM direct_messages dm
            INNER JOIN dm_participants dp ON dm.id = dp.dm_id
            WHERE dp.user_id = $1 AND dp.hidden = FALSE
            ORDER BY dm.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(dms)
    }

    /// Get a DM by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<DirectMessage>> {
        let dm = sqlx::query_as::<_, DirectMessage>(
            r#"
            SELECT * FROM direct_messages
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(dm)
    }

    /// Create a new DM
    pub async fn create(
        pool: &PgPool,
        org_id: Uuid,
        created_by: Uuid,
        participant_ids: &[Uuid],
    ) -> ApiResult<DirectMessage> {
        // Start a transaction
        let mut tx = pool.begin().await?;

        // Create the DM
        let dm = sqlx::query_as::<_, DirectMessage>(
            r#"
            INSERT INTO direct_messages (id, org_id, created_by)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // Add all participants (including the creator)
        for user_id in participant_ids {
            sqlx::query(
                r#"
                INSERT INTO dm_participants (id, dm_id, user_id)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(dm.id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        }

        // Commit the transaction
        tx.commit().await?;

        Ok(dm)
    }

    /// Find an existing DM between specific participants
    /// Returns None if no exact match is found
    pub async fn find_by_participants(
        pool: &PgPool,
        participant_ids: &[Uuid],
    ) -> ApiResult<Option<DirectMessage>> {
        if participant_ids.is_empty() {
            return Ok(None);
        }

        // Find DMs that have exactly the same participants
        let dm = sqlx::query_as::<_, DirectMessage>(
            r#"
            SELECT dm.*
            FROM direct_messages dm
            WHERE dm.id IN (
                SELECT dp.dm_id
                FROM dm_participants dp
                WHERE dp.user_id = ANY($1)
                GROUP BY dp.dm_id
                HAVING COUNT(DISTINCT dp.user_id) = $2
                    AND COUNT(DISTINCT dp.user_id) = (
                        SELECT COUNT(*)
                        FROM dm_participants
                        WHERE dm_id = dp.dm_id
                    )
            )
            LIMIT 1
            "#,
        )
        .bind(participant_ids)
        .bind(participant_ids.len() as i64)
        .fetch_optional(pool)
        .await?;

        Ok(dm)
    }

    /// Check if a user is a participant in a DM
    pub async fn is_participant(pool: &PgPool, dm_id: Uuid, user_id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM dm_participants WHERE dm_id = $1 AND user_id = $2)",
        )
        .bind(dm_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Get DM metadata with unread counts for a user (for initial WebSocket state)
    /// Excludes hidden DMs
    pub async fn get_metadata_for_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> ApiResult<Vec<crate::websocket::messages::DmMetadata>> {
        let metadata = sqlx::query_as::<_, crate::websocket::messages::DmMetadata>(
            r#"
            SELECT
                dm.id,
                other_user.tv_user_id as other_user_id,
                other_user.display_name as other_user_name,
                COALESCE(
                    (
                        SELECT COUNT(*)::int
                        FROM messages m
                        WHERE m.dm_id = dm.id
                        AND m.created_at > COALESCE(
                            (SELECT last_read_at FROM dm_read_status WHERE dm_id = dm.id AND user_id = $1),
                            '1970-01-01'::timestamp
                        )
                    ),
                    0
                ) as unread_count,
                (
                    SELECT content
                    FROM messages
                    WHERE dm_id = dm.id
                    ORDER BY created_at DESC
                    LIMIT 1
                ) as last_message_preview,
                (
                    SELECT created_at
                    FROM messages
                    WHERE dm_id = dm.id
                    ORDER BY created_at DESC
                    LIMIT 1
                ) as last_message_at
            FROM direct_messages dm
            INNER JOIN dm_participants dp1 ON dm.id = dp1.dm_id AND dp1.user_id = $1 AND dp1.hidden = FALSE
            INNER JOIN dm_participants dp2 ON dm.id = dp2.dm_id AND dp2.user_id != $1
            INNER JOIN users other_user ON dp2.user_id = other_user.id
            ORDER BY dm.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(metadata)
    }
}

impl DmParticipant {
    /// Hide a DM for a user
    pub async fn hide_dm(pool: &PgPool, dm_id: Uuid, user_id: Uuid) -> ApiResult<()> {
        sqlx::query(
            r#"
            UPDATE dm_participants
            SET hidden = TRUE
            WHERE dm_id = $1 AND user_id = $2
            "#,
        )
        .bind(dm_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Unhide a DM for a user (called when a new message is received)
    #[allow(dead_code)]
    pub async fn unhide_dm(pool: &PgPool, dm_id: Uuid, user_id: Uuid) -> ApiResult<()> {
        sqlx::query(
            r#"
            UPDATE dm_participants
            SET hidden = FALSE
            WHERE dm_id = $1 AND user_id = $2
            "#,
        )
        .bind(dm_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// List all participants of a DM
    pub async fn list_by_dm(pool: &PgPool, dm_id: Uuid) -> ApiResult<Vec<DmParticipant>> {
        let participants = sqlx::query_as::<_, DmParticipant>(
            r#"
            SELECT * FROM dm_participants
            WHERE dm_id = $1
            ORDER BY joined_at
            "#,
        )
        .bind(dm_id)
        .fetch_all(pool)
        .await?;

        Ok(participants)
    }
}
