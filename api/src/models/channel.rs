use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Channel {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub channel_type: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelMember {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

impl Channel {
    /// List all channels for an organization
    pub async fn list_by_org(pool: &PgPool, org_id: Uuid) -> ApiResult<Vec<Channel>> {
        let channels = sqlx::query_as::<_, Channel>(
            r#"
            SELECT * FROM channels
            WHERE org_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(pool)
        .await?;

        Ok(channels)
    }

    /// List channels where the user is a member
    pub async fn list_by_user_membership(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> ApiResult<Vec<Channel>> {
        let channels = sqlx::query_as::<_, Channel>(
            r#"
            SELECT c.* FROM channels c
            INNER JOIN channel_members cm ON c.id = cm.channel_id
            WHERE c.org_id = $1 AND cm.user_id = $2
            ORDER BY c.created_at DESC
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(channels)
    }

    /// List public channels in an organization (for browsing/discovery)
    /// Excludes channels the user is already a member of
    pub async fn list_public_channels(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> ApiResult<Vec<Channel>> {
        let channels = sqlx::query_as::<_, Channel>(
            r#"
            SELECT c.* FROM channels c
            WHERE c.org_id = $1
            AND c.channel_type = 'public'
            AND NOT EXISTS (
                SELECT 1 FROM channel_members cm
                WHERE cm.channel_id = c.id AND cm.user_id = $2
            )
            ORDER BY c.created_at DESC
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(channels)
    }

    /// Get a channel by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<Channel>> {
        let channel = sqlx::query_as::<_, Channel>(
            r#"
            SELECT * FROM channels
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(channel)
    }

    /// Create a new channel
    pub async fn create(
        pool: &PgPool,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        channel_type: &str,
        created_by: Uuid,
    ) -> ApiResult<Channel> {
        let channel = sqlx::query_as::<_, Channel>(
            r#"
            INSERT INTO channels (id, org_id, name, description, channel_type, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(name)
        .bind(description)
        .bind(channel_type)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(channel)
    }

    /// Update a channel
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
    ) -> ApiResult<Channel> {
        let channel = sqlx::query_as::<_, Channel>(
            r#"
            UPDATE channels
            SET name = COALESCE($1, name),
                description = COALESCE($2, description),
                updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(channel)
    }

    /// Delete a channel
    pub async fn delete(pool: &PgPool, id: Uuid) -> ApiResult<()> {
        sqlx::query("DELETE FROM channels WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

impl ChannelMember {
    /// List all members of a channel
    pub async fn list_by_channel(pool: &PgPool, channel_id: Uuid) -> ApiResult<Vec<ChannelMember>> {
        let members = sqlx::query_as::<_, ChannelMember>(
            r#"
            SELECT * FROM channel_members
            WHERE channel_id = $1
            ORDER BY joined_at
            "#,
        )
        .bind(channel_id)
        .fetch_all(pool)
        .await?;

        Ok(members)
    }

    /// Add a member to a channel
    pub async fn add(
        pool: &PgPool,
        channel_id: Uuid,
        user_id: Uuid,
        role: &str,
    ) -> ApiResult<ChannelMember> {
        let member = sqlx::query_as::<_, ChannelMember>(
            r#"
            INSERT INTO channel_members (id, channel_id, user_id, role)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(channel_id)
        .bind(user_id)
        .bind(role)
        .fetch_one(pool)
        .await?;

        Ok(member)
    }

    /// Remove a member from a channel
    pub async fn remove(pool: &PgPool, channel_id: Uuid, user_id: Uuid) -> ApiResult<()> {
        sqlx::query("DELETE FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Check if a user is a member of a channel
    pub async fn is_member(pool: &PgPool, channel_id: Uuid, user_id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
        )
        .bind(channel_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }
}
