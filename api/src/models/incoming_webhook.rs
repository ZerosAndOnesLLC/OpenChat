use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IncomingWebhook {
    pub id: Uuid,
    pub org_id: Uuid,
    pub channel_id: Uuid,
    pub token: String,
    pub display_name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub username: Option<String>,
    pub enabled: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response struct that includes the webhook URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingWebhookWithUrl {
    #[serde(flatten)]
    pub webhook: IncomingWebhook,
    pub url: String,
}

impl IncomingWebhook {
    /// Generate a secure random token for webhooks
    fn generate_token() -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        // Use base16 encoding (hex) - manual implementation to avoid extra deps
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Create a new incoming webhook
    pub async fn create(
        pool: &PgPool,
        org_id: Uuid,
        channel_id: Uuid,
        display_name: &str,
        description: Option<&str>,
        icon_url: Option<&str>,
        username: Option<&str>,
        created_by: Uuid,
    ) -> ApiResult<IncomingWebhook> {
        let token = Self::generate_token();

        let webhook = sqlx::query_as::<_, IncomingWebhook>(
            r#"
            INSERT INTO incoming_webhooks (org_id, channel_id, token, display_name, description, icon_url, username, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(channel_id)
        .bind(&token)
        .bind(display_name)
        .bind(description)
        .bind(icon_url)
        .bind(username)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(webhook)
    }

    /// Get a webhook by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<IncomingWebhook>> {
        let webhook = sqlx::query_as::<_, IncomingWebhook>(
            "SELECT * FROM incoming_webhooks WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(webhook)
    }

    /// Get a webhook by token (for public endpoint)
    pub async fn get_by_token(pool: &PgPool, token: &str) -> ApiResult<Option<IncomingWebhook>> {
        let webhook = sqlx::query_as::<_, IncomingWebhook>(
            "SELECT * FROM incoming_webhooks WHERE token = $1 AND enabled = true",
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;

        Ok(webhook)
    }

    /// List all webhooks for an organization
    pub async fn list_by_org(pool: &PgPool, org_id: Uuid) -> ApiResult<Vec<IncomingWebhook>> {
        let webhooks = sqlx::query_as::<_, IncomingWebhook>(
            r#"
            SELECT * FROM incoming_webhooks
            WHERE org_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(pool)
        .await?;

        Ok(webhooks)
    }

    /// List webhooks for a specific channel
    pub async fn list_by_channel(pool: &PgPool, channel_id: Uuid) -> ApiResult<Vec<IncomingWebhook>> {
        let webhooks = sqlx::query_as::<_, IncomingWebhook>(
            r#"
            SELECT * FROM incoming_webhooks
            WHERE channel_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(channel_id)
        .fetch_all(pool)
        .await?;

        Ok(webhooks)
    }

    /// Update a webhook
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        display_name: Option<&str>,
        description: Option<&str>,
        icon_url: Option<&str>,
        username: Option<&str>,
        channel_id: Option<Uuid>,
        enabled: Option<bool>,
    ) -> ApiResult<IncomingWebhook> {
        let webhook = sqlx::query_as::<_, IncomingWebhook>(
            r#"
            UPDATE incoming_webhooks
            SET display_name = COALESCE($2, display_name),
                description = COALESCE($3, description),
                icon_url = COALESCE($4, icon_url),
                username = COALESCE($5, username),
                channel_id = COALESCE($6, channel_id),
                enabled = COALESCE($7, enabled),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(display_name)
        .bind(description)
        .bind(icon_url)
        .bind(username)
        .bind(channel_id)
        .bind(enabled)
        .fetch_one(pool)
        .await?;

        Ok(webhook)
    }

    /// Delete a webhook
    pub async fn delete(pool: &PgPool, id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query("DELETE FROM incoming_webhooks WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Regenerate the webhook token
    pub async fn regenerate_token(pool: &PgPool, id: Uuid) -> ApiResult<IncomingWebhook> {
        let new_token = Self::generate_token();

        let webhook = sqlx::query_as::<_, IncomingWebhook>(
            r#"
            UPDATE incoming_webhooks
            SET token = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&new_token)
        .fetch_one(pool)
        .await?;

        Ok(webhook)
    }
}
