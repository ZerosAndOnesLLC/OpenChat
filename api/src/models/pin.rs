use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PinnedMessage {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub message_id: Uuid,
    pub pinned_by: Uuid,
    pub pinned_at: DateTime<Utc>,
}

impl PinnedMessage {
    /// Pin a message in a channel
    pub async fn pin(
        pool: &PgPool,
        channel_id: Uuid,
        message_id: Uuid,
        user_id: Uuid,
    ) -> ApiResult<PinnedMessage> {
        let pinned = sqlx::query_as::<_, PinnedMessage>(
            r#"
            INSERT INTO pinned_messages (id, channel_id, message_id, pinned_by)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (channel_id, message_id) DO NOTHING
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(channel_id)
        .bind(message_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(pinned)
    }

    /// Unpin a message from a channel
    pub async fn unpin(pool: &PgPool, channel_id: Uuid, message_id: Uuid) -> ApiResult<()> {
        sqlx::query(
            "DELETE FROM pinned_messages WHERE channel_id = $1 AND message_id = $2",
        )
        .bind(channel_id)
        .bind(message_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Check if a message is pinned in a channel
    #[allow(dead_code)]
    pub async fn is_pinned(
        pool: &PgPool,
        channel_id: Uuid,
        message_id: Uuid,
    ) -> ApiResult<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pinned_messages WHERE channel_id = $1 AND message_id = $2)",
        )
        .bind(channel_id)
        .bind(message_id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// List all pinned messages for a channel
    pub async fn list_by_channel(pool: &PgPool, channel_id: Uuid) -> ApiResult<Vec<PinnedMessage>> {
        let pins = sqlx::query_as::<_, PinnedMessage>(
            r#"
            SELECT * FROM pinned_messages
            WHERE channel_id = $1
            ORDER BY pinned_at DESC
            "#,
        )
        .bind(channel_id)
        .fetch_all(pool)
        .await?;

        Ok(pins)
    }

    /// Get pin details for a specific message
    #[allow(dead_code)]
    pub async fn get_by_message(
        pool: &PgPool,
        channel_id: Uuid,
        message_id: Uuid,
    ) -> ApiResult<Option<PinnedMessage>> {
        let pin = sqlx::query_as::<_, PinnedMessage>(
            r#"
            SELECT * FROM pinned_messages
            WHERE channel_id = $1 AND message_id = $2
            "#,
        )
        .bind(channel_id)
        .bind(message_id)
        .fetch_optional(pool)
        .await?;

        Ok(pin)
    }

    /// Get pins with info for channel subscription
    pub async fn get_pins_for_channel(
        pool: &PgPool,
        channel_id: Uuid,
    ) -> ApiResult<Vec<crate::websocket::messages::PinnedMessageInfo>> {
        let pins = sqlx::query_as::<_, PinnedMessage>(
            r#"
            SELECT * FROM pinned_messages
            WHERE channel_id = $1
            ORDER BY pinned_at DESC
            "#,
        )
        .bind(channel_id)
        .fetch_all(pool)
        .await?;

        Ok(pins
            .into_iter()
            .map(|pin| crate::websocket::messages::PinnedMessageInfo {
                id: pin.id,
                message_id: pin.message_id,
                pinned_by: pin.pinned_by,
                pinned_at: pin.pinned_at,
            })
            .collect())
    }
}
