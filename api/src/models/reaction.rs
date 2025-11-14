use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Reaction {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReactionCount {
    pub emoji: String,
    pub count: i64,
    pub user_ids: Vec<Uuid>,
}

impl Reaction {
    /// Add a reaction to a message
    /// Uses ON CONFLICT to handle duplicate reactions (same user, same emoji on same message)
    pub async fn add(
        pool: &PgPool,
        message_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> ApiResult<Reaction> {
        let reaction = sqlx::query_as::<_, Reaction>(
            r#"
            INSERT INTO reactions (id, message_id, user_id, emoji)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (message_id, user_id, emoji) DO NOTHING
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .fetch_one(pool)
        .await?;

        Ok(reaction)
    }

    /// Remove a reaction from a message
    pub async fn remove(
        pool: &PgPool,
        message_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> ApiResult<()> {
        sqlx::query(
            "DELETE FROM reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3"
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// List all reactions for a message
    pub async fn list_by_message(pool: &PgPool, message_id: Uuid) -> ApiResult<Vec<Reaction>> {
        let reactions = sqlx::query_as::<_, Reaction>(
            r#"
            SELECT * FROM reactions
            WHERE message_id = $1
            ORDER BY created_at
            "#,
        )
        .bind(message_id)
        .fetch_all(pool)
        .await?;

        Ok(reactions)
    }

    /// Get reaction counts (aggregated by emoji) for a message
    pub async fn count_by_message(pool: &PgPool, message_id: Uuid) -> ApiResult<Vec<ReactionCount>> {
        let counts = sqlx::query_as::<_, ReactionCount>(
            r#"
            SELECT
                emoji,
                COUNT(*) as count,
                ARRAY_AGG(user_id) as user_ids
            FROM reactions
            WHERE message_id = $1
            GROUP BY emoji
            ORDER BY count DESC, emoji
            "#,
        )
        .bind(message_id)
        .fetch_all(pool)
        .await?;

        Ok(counts)
    }
}
