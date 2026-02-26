use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelSection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub position: i32,
    pub collapsed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelSectionItem {
    pub id: Uuid,
    pub section_id: Uuid,
    pub channel_id: Uuid,
    pub position: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateChannelSection {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelSection {
    pub name: Option<String>,
    pub collapsed: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderSection {
    pub id: Uuid,
    pub position: i32,
}

#[derive(Debug, Deserialize)]
pub struct ReorderSectionItem {
    pub channel_id: Uuid,
    pub position: i32,
}

impl ChannelSection {
    pub async fn list_by_user(
        pool: &PgPool,
        user_id: Uuid,
        org_id: Uuid,
    ) -> ApiResult<Vec<ChannelSection>> {
        let sections = sqlx::query_as::<_, ChannelSection>(
            r#"
            SELECT * FROM channel_sections
            WHERE user_id = $1 AND org_id = $2
            ORDER BY position
            "#,
        )
        .bind(user_id)
        .bind(org_id)
        .fetch_all(pool)
        .await?;

        Ok(sections)
    }

    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        org_id: Uuid,
        data: CreateChannelSection,
    ) -> ApiResult<ChannelSection> {
        let max_position = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT MAX(position) FROM channel_sections WHERE user_id = $1 AND org_id = $2",
        )
        .bind(user_id)
        .bind(org_id)
        .fetch_one(pool)
        .await?;

        let position = max_position.unwrap_or(-1) + 1;

        let section = sqlx::query_as::<_, ChannelSection>(
            r#"
            INSERT INTO channel_sections (id, user_id, org_id, name, position)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(org_id)
        .bind(&data.name)
        .bind(position)
        .fetch_one(pool)
        .await?;

        Ok(section)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        data: UpdateChannelSection,
    ) -> ApiResult<Option<ChannelSection>> {
        let section = sqlx::query_as::<_, ChannelSection>(
            r#"
            UPDATE channel_sections
            SET name = COALESCE($3, name),
                collapsed = COALESCE($4, collapsed)
            WHERE id = $1 AND user_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(data.name)
        .bind(data.collapsed)
        .fetch_optional(pool)
        .await?;

        Ok(section)
    }

    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query(
            "DELETE FROM channel_sections WHERE id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn bulk_reorder(
        pool: &PgPool,
        user_id: Uuid,
        org_id: Uuid,
        order: Vec<ReorderSection>,
    ) -> ApiResult<()> {
        for item in order {
            sqlx::query(
                "UPDATE channel_sections SET position = $3 WHERE id = $1 AND user_id = $2 AND org_id = $4",
            )
            .bind(item.id)
            .bind(user_id)
            .bind(item.position)
            .bind(org_id)
            .execute(pool)
            .await?;
        }

        Ok(())
    }
}

impl ChannelSectionItem {
    pub async fn list_by_section(pool: &PgPool, section_id: Uuid) -> ApiResult<Vec<ChannelSectionItem>> {
        let items = sqlx::query_as::<_, ChannelSectionItem>(
            r#"
            SELECT * FROM channel_section_items
            WHERE section_id = $1
            ORDER BY position
            "#,
        )
        .bind(section_id)
        .fetch_all(pool)
        .await?;

        Ok(items)
    }

    pub async fn add(
        pool: &PgPool,
        section_id: Uuid,
        channel_id: Uuid,
        position: Option<i32>,
    ) -> ApiResult<ChannelSectionItem> {
        let pos = match position {
            Some(p) => p,
            None => {
                let max = sqlx::query_scalar::<_, Option<i32>>(
                    "SELECT MAX(position) FROM channel_section_items WHERE section_id = $1",
                )
                .bind(section_id)
                .fetch_one(pool)
                .await?;
                max.unwrap_or(-1) + 1
            }
        };

        let item = sqlx::query_as::<_, ChannelSectionItem>(
            r#"
            INSERT INTO channel_section_items (id, section_id, channel_id, position)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (section_id, channel_id) DO UPDATE SET position = EXCLUDED.position
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(section_id)
        .bind(channel_id)
        .bind(pos)
        .fetch_one(pool)
        .await?;

        Ok(item)
    }

    pub async fn remove(pool: &PgPool, section_id: Uuid, channel_id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query(
            "DELETE FROM channel_section_items WHERE section_id = $1 AND channel_id = $2",
        )
        .bind(section_id)
        .bind(channel_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn bulk_reorder(
        pool: &PgPool,
        section_id: Uuid,
        order: Vec<ReorderSectionItem>,
    ) -> ApiResult<()> {
        for item in order {
            sqlx::query(
                "UPDATE channel_section_items SET position = $3 WHERE section_id = $1 AND channel_id = $2",
            )
            .bind(section_id)
            .bind(item.channel_id)
            .bind(item.position)
            .execute(pool)
            .await?;
        }

        Ok(())
    }
}
