use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub user_id: Uuid,
    pub content: String,
    pub parent_message_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedMessages {
    pub messages: Vec<Message>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

impl Message {
    /// Create a new message in a channel
    pub async fn create_channel_message(
        pool: &PgPool,
        channel_id: Uuid,
        user_id: Uuid,
        content: &str,
        parent_message_id: Option<Uuid>,
    ) -> ApiResult<Message> {
        let message = sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO messages (id, channel_id, user_id, content, parent_message_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(channel_id)
        .bind(user_id)
        .bind(content)
        .bind(parent_message_id)
        .fetch_one(pool)
        .await?;

        Ok(message)
    }

    /// Create a new message in a DM
    pub async fn create_dm_message(
        pool: &PgPool,
        dm_id: Uuid,
        user_id: Uuid,
        content: &str,
        parent_message_id: Option<Uuid>,
    ) -> ApiResult<Message> {
        let message = sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO messages (id, dm_id, user_id, content, parent_message_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(dm_id)
        .bind(user_id)
        .bind(content)
        .bind(parent_message_id)
        .fetch_one(pool)
        .await?;

        Ok(message)
    }

    /// List messages for a channel with cursor-based pagination
    /// Returns messages in descending order (newest first)
    pub async fn list_by_channel(
        pool: &PgPool,
        channel_id: Uuid,
        limit: i64,
        cursor: Option<String>,
    ) -> ApiResult<PaginatedMessages> {
        let limit = limit.min(100); // Cap at 100 messages per request

        let messages = if let Some(cursor_str) = cursor {
            // Parse cursor: format is "timestamp_id"
            let parts: Vec<&str> = cursor_str.split('_').collect();
            if parts.len() != 2 {
                return Err(crate::errors::ApiError::BadRequest("Invalid cursor format".to_string()));
            }

            let cursor_time = parts[0].parse::<i64>()
                .map_err(|_| crate::errors::ApiError::BadRequest("Invalid cursor timestamp".to_string()))?;
            let cursor_id = Uuid::parse_str(parts[1])
                .map_err(|_| crate::errors::ApiError::BadRequest("Invalid cursor ID".to_string()))?;

            let cursor_datetime = DateTime::<Utc>::from_timestamp(cursor_time, 0)
                .ok_or_else(|| crate::errors::ApiError::BadRequest("Invalid cursor timestamp".to_string()))?;

            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages
                WHERE channel_id = $1
                    AND deleted_at IS NULL
                    AND (created_at < $2 OR (created_at = $2 AND id < $3))
                ORDER BY created_at DESC, id DESC
                LIMIT $4
                "#,
            )
            .bind(channel_id)
            .bind(cursor_datetime)
            .bind(cursor_id)
            .bind(limit + 1)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages
                WHERE channel_id = $1 AND deleted_at IS NULL
                ORDER BY created_at DESC, id DESC
                LIMIT $2
                "#,
            )
            .bind(channel_id)
            .bind(limit + 1)
            .fetch_all(pool)
            .await?
        };

        let has_more = messages.len() > limit as usize;
        let mut messages = messages;

        if has_more {
            messages.pop(); // Remove the extra message used for has_more check
        }

        let next_cursor = if has_more && !messages.is_empty() {
            let last = messages.last().unwrap();
            Some(format!("{}_{}", last.created_at.timestamp(), last.id))
        } else {
            None
        };

        Ok(PaginatedMessages {
            messages,
            has_more,
            next_cursor,
        })
    }

    /// List messages for a DM with cursor-based pagination
    /// Will be used in Phase 7 (Direct Messages)
    #[allow(dead_code)]
    pub async fn list_by_dm(
        pool: &PgPool,
        dm_id: Uuid,
        limit: i64,
        cursor: Option<String>,
    ) -> ApiResult<PaginatedMessages> {
        let limit = limit.min(100); // Cap at 100 messages per request

        let messages = if let Some(cursor_str) = cursor {
            // Parse cursor: format is "timestamp_id"
            let parts: Vec<&str> = cursor_str.split('_').collect();
            if parts.len() != 2 {
                return Err(crate::errors::ApiError::BadRequest("Invalid cursor format".to_string()));
            }

            let cursor_time = parts[0].parse::<i64>()
                .map_err(|_| crate::errors::ApiError::BadRequest("Invalid cursor timestamp".to_string()))?;
            let cursor_id = Uuid::parse_str(parts[1])
                .map_err(|_| crate::errors::ApiError::BadRequest("Invalid cursor ID".to_string()))?;

            let cursor_datetime = DateTime::<Utc>::from_timestamp(cursor_time, 0)
                .ok_or_else(|| crate::errors::ApiError::BadRequest("Invalid cursor timestamp".to_string()))?;

            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages
                WHERE dm_id = $1
                    AND deleted_at IS NULL
                    AND (created_at < $2 OR (created_at = $2 AND id < $3))
                ORDER BY created_at DESC, id DESC
                LIMIT $4
                "#,
            )
            .bind(dm_id)
            .bind(cursor_datetime)
            .bind(cursor_id)
            .bind(limit + 1)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages
                WHERE dm_id = $1 AND deleted_at IS NULL
                ORDER BY created_at DESC, id DESC
                LIMIT $2
                "#,
            )
            .bind(dm_id)
            .bind(limit + 1)
            .fetch_all(pool)
            .await?
        };

        let has_more = messages.len() > limit as usize;
        let mut messages = messages;

        if has_more {
            messages.pop(); // Remove the extra message used for has_more check
        }

        let next_cursor = if has_more && !messages.is_empty() {
            let last = messages.last().unwrap();
            Some(format!("{}_{}", last.created_at.timestamp(), last.id))
        } else {
            None
        };

        Ok(PaginatedMessages {
            messages,
            has_more,
            next_cursor,
        })
    }

    /// Get a message by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<Message>> {
        let message = sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(message)
    }

    /// Update a message (edit)
    /// This function saves the old content to message_edits table before updating
    pub async fn update(pool: &PgPool, id: Uuid, content: &str, user_id: Uuid) -> ApiResult<Message> {
        // Start a transaction to ensure atomicity
        let mut tx = pool.begin().await?;

        // Fetch the current message content before updating
        let old_message = sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        // Save the old content to message_edits table
        sqlx::query(
            r#"
            INSERT INTO message_edits (message_id, old_content, edited_by)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(id)
        .bind(&old_message.content)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // Update the message with new content
        let message = sqlx::query_as::<_, Message>(
            r#"
            UPDATE messages
            SET content = $1,
                edited_at = NOW()
            WHERE id = $2 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(content)
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        // Commit the transaction
        tx.commit().await?;

        Ok(message)
    }

    /// Soft delete a message
    pub async fn soft_delete(pool: &PgPool, id: Uuid) -> ApiResult<Message> {
        let message = sqlx::query_as::<_, Message>(
            r#"
            UPDATE messages
            SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(message)
    }

    /// Get thread messages (replies) for a parent message
    pub async fn list_thread_messages(
        pool: &PgPool,
        parent_message_id: Uuid,
    ) -> ApiResult<Vec<Message>> {
        let messages = sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages
            WHERE parent_message_id = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC
            "#,
        )
        .bind(parent_message_id)
        .fetch_all(pool)
        .await?;

        Ok(messages)
    }

    /// Count replies for multiple messages (batch operation)
    pub async fn count_replies_batch(
        pool: &PgPool,
        message_ids: &[Uuid],
    ) -> ApiResult<std::collections::HashMap<Uuid, i64>> {
        if message_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        #[derive(sqlx::FromRow)]
        struct ReplyCount {
            parent_message_id: Uuid,
            count: i64,
        }

        let results = sqlx::query_as::<_, ReplyCount>(
            r#"
            SELECT parent_message_id, COUNT(*) as count
            FROM messages
            WHERE parent_message_id = ANY($1) AND deleted_at IS NULL
            GROUP BY parent_message_id
            "#,
        )
        .bind(message_ids)
        .fetch_all(pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| (r.parent_message_id, r.count))
            .collect())
    }

    /// Get first reply for multiple messages (batch operation)
    /// Returns a map of parent_message_id -> first_reply
    pub async fn get_first_replies_batch(
        pool: &PgPool,
        message_ids: &[Uuid],
    ) -> ApiResult<std::collections::HashMap<Uuid, Message>> {
        if message_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        // Use DISTINCT ON to get the first reply for each parent message
        let results = sqlx::query_as::<_, Message>(
            r#"
            SELECT DISTINCT ON (parent_message_id) *
            FROM messages
            WHERE parent_message_id = ANY($1) AND deleted_at IS NULL
            ORDER BY parent_message_id, created_at ASC
            "#,
        )
        .bind(message_ids)
        .fetch_all(pool)
        .await?;

        Ok(results
            .into_iter()
            .filter_map(|msg| msg.parent_message_id.map(|pid| (pid, msg)))
            .collect())
    }

    /// Get messages with details for channel subscription (includes user names and reply counts)
    /// Returns messages in descending order (newest first) with a limit
    pub async fn get_messages_with_details_for_channel(
        pool: &PgPool,
        channel_id: Uuid,
        limit: i64,
    ) -> ApiResult<Vec<crate::websocket::messages::MessageWithDetails>> {
        #[derive(sqlx::FromRow)]
        struct MessageRow {
            id: Uuid,
            channel_id: Option<Uuid>,
            dm_id: Option<Uuid>,
            user_id: Uuid,
            user_name: String,
            content: String,
            parent_message_id: Option<Uuid>,
            created_at: DateTime<Utc>,
            edited_at: Option<DateTime<Utc>>,
            reply_count: i64,
        }

        let rows = sqlx::query_as::<_, MessageRow>(
            r#"
            SELECT
                m.id,
                m.channel_id,
                m.dm_id,
                m.user_id,
                COALESCE(u.full_name, u.email) as user_name,
                m.content,
                m.parent_message_id,
                m.created_at,
                m.edited_at,
                COALESCE(
                    (SELECT COUNT(*)::bigint FROM messages replies
                     WHERE replies.parent_message_id = m.id AND replies.deleted_at IS NULL),
                    0
                ) as reply_count
            FROM messages m
            LEFT JOIN users u ON m.user_id = u.id
            WHERE m.channel_id = $1 AND m.deleted_at IS NULL
            ORDER BY m.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(channel_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| crate::websocket::messages::MessageWithDetails {
                id: row.id,
                channel_id: row.channel_id,
                dm_id: row.dm_id,
                user_id: row.user_id,
                user_name: row.user_name,
                content: row.content,
                parent_message_id: row.parent_message_id,
                created_at: row.created_at,
                edited_at: row.edited_at,
                reply_count: row.reply_count,
            })
            .collect())
    }
}
