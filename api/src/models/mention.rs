use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum MentionType {
    User,
    Channel,
    Here,
    Everyone,
    Group,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Mention {
    pub id: Uuid,
    pub message_id: Uuid,
    pub mentioned_user_id: Option<Uuid>,
    pub mention_type: MentionType,
    pub mentioned_group_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMention {
    pub message_id: Uuid,
    pub mentioned_user_id: Option<Uuid>,
    pub mention_type: MentionType,
    pub mentioned_group_id: Option<Uuid>,
}

impl Mention {
    /// Create a new mention
    pub async fn create(pool: &PgPool, data: CreateMention) -> ApiResult<Mention> {
        let mention = sqlx::query_as::<_, Mention>(
            r#"
            INSERT INTO mentions (id, message_id, mentioned_user_id, mention_type, mentioned_group_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(data.message_id)
        .bind(data.mentioned_user_id)
        .bind(data.mention_type)
        .bind(data.mentioned_group_id)
        .fetch_one(pool)
        .await?;

        Ok(mention)
    }

    /// Create multiple mentions in a batch
    pub async fn create_batch(pool: &PgPool, mentions: Vec<CreateMention>) -> ApiResult<Vec<Mention>> {
        let mut created_mentions = Vec::new();

        for mention_data in mentions {
            let mention = Self::create(pool, mention_data).await?;
            created_mentions.push(mention);
        }

        Ok(created_mentions)
    }

    /// Get all mentions for a user (for showing "you were mentioned")
    pub async fn list_by_user(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> ApiResult<Vec<Mention>> {
        let mentions = sqlx::query_as::<_, Mention>(
            r#"
            SELECT m.* FROM mentions m
            WHERE m.mentioned_user_id = $1
            ORDER BY m.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(mentions)
    }

    /// Get mentions for a specific message
    #[allow(dead_code)]
    pub async fn list_by_message(pool: &PgPool, message_id: Uuid) -> ApiResult<Vec<Mention>> {
        let mentions = sqlx::query_as::<_, Mention>(
            r#"
            SELECT * FROM mentions
            WHERE message_id = $1
            ORDER BY created_at
            "#,
        )
        .bind(message_id)
        .fetch_all(pool)
        .await?;

        Ok(mentions)
    }

    /// Count unread mentions for a user (mentions in messages they haven't read)
    pub async fn count_unread_by_user(pool: &PgPool, user_id: Uuid) -> ApiResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(DISTINCT m.id)
            FROM mentions m
            INNER JOIN messages msg ON m.message_id = msg.id
            LEFT JOIN channel_read_status crs ON msg.channel_id = crs.channel_id AND crs.user_id = $1
            LEFT JOIN dm_read_status drs ON msg.dm_id = drs.dm_id AND drs.user_id = $1
            WHERE (
                m.mentioned_user_id = $1
                OR (m.mentioned_group_id IS NOT NULL AND EXISTS (
                    SELECT 1 FROM user_group_members ugm
                    WHERE ugm.group_id = m.mentioned_group_id AND ugm.user_id = $1
                ))
            )
            AND (
                (msg.channel_id IS NOT NULL AND (crs.last_read_message_id IS NULL OR msg.created_at > crs.last_read_at))
                OR (msg.dm_id IS NOT NULL AND (drs.last_read_message_id IS NULL OR msg.created_at > drs.last_read_at))
            )
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }
}
