use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bookmark {
    pub id: Uuid,
    pub user_id: Uuid,
    pub message_id: Uuid,
    pub bookmarked_at: DateTime<Utc>,
}

impl Bookmark {
    /// Bookmark a message for a user
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        message_id: Uuid,
    ) -> ApiResult<Bookmark> {
        let bookmark = sqlx::query_as::<_, Bookmark>(
            r#"
            INSERT INTO bookmarks (id, user_id, message_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, message_id) DO NOTHING
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(message_id)
        .fetch_one(pool)
        .await?;

        Ok(bookmark)
    }

    /// Remove a bookmark
    pub async fn delete(pool: &PgPool, user_id: Uuid, message_id: Uuid) -> ApiResult<()> {
        sqlx::query(
            "DELETE FROM bookmarks WHERE user_id = $1 AND message_id = $2",
        )
        .bind(user_id)
        .bind(message_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Check if a message is bookmarked by a user
    pub async fn is_bookmarked(
        pool: &PgPool,
        user_id: Uuid,
        message_id: Uuid,
    ) -> ApiResult<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM bookmarks WHERE user_id = $1 AND message_id = $2)",
        )
        .bind(user_id)
        .bind(message_id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// List all bookmarks for a user
    pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> ApiResult<Vec<Bookmark>> {
        let bookmarks = sqlx::query_as::<_, Bookmark>(
            r#"
            SELECT * FROM bookmarks
            WHERE user_id = $1
            ORDER BY bookmarked_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(bookmarks)
    }

    /// Get a specific bookmark
    pub async fn get(
        pool: &PgPool,
        user_id: Uuid,
        message_id: Uuid,
    ) -> ApiResult<Option<Bookmark>> {
        let bookmark = sqlx::query_as::<_, Bookmark>(
            r#"
            SELECT * FROM bookmarks
            WHERE user_id = $1 AND message_id = $2
            "#,
        )
        .bind(user_id)
        .bind(message_id)
        .fetch_optional(pool)
        .await?;

        Ok(bookmark)
    }
}
