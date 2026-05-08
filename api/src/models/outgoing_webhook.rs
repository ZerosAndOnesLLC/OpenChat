use chrono::{DateTime, Utc};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OutgoingWebhook {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub url: String,
    pub signing_secret: String,
    pub event_types: Vec<String>,
    pub enabled: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event_type: String,
    pub payload: JsonValue,
    pub status: String,
    pub response_code: Option<i32>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub attempt: i32,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl OutgoingWebhook {
    fn generate_signing_secret() -> String {
        let mut bytes = [0u8; 32];
        rand::rng().fill(&mut bytes);
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub async fn create(
        pool: &PgPool,
        org_id: Uuid,
        name: &str,
        url: &str,
        event_types: &[String],
        created_by: Uuid,
    ) -> ApiResult<OutgoingWebhook> {
        let signing_secret = Self::generate_signing_secret();

        let webhook = sqlx::query_as::<_, OutgoingWebhook>(
            r#"
            INSERT INTO outgoing_webhooks (org_id, name, url, signing_secret, event_types, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(name)
        .bind(url)
        .bind(&signing_secret)
        .bind(event_types)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(webhook)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<OutgoingWebhook>> {
        let webhook = sqlx::query_as::<_, OutgoingWebhook>(
            "SELECT * FROM outgoing_webhooks WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(webhook)
    }

    pub async fn list_by_org(pool: &PgPool, org_id: Uuid) -> ApiResult<Vec<OutgoingWebhook>> {
        let webhooks = sqlx::query_as::<_, OutgoingWebhook>(
            r#"
            SELECT * FROM outgoing_webhooks
            WHERE org_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(pool)
        .await?;

        Ok(webhooks)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        name: Option<&str>,
        url: Option<&str>,
        event_types: Option<&[String]>,
        enabled: Option<bool>,
    ) -> ApiResult<OutgoingWebhook> {
        let webhook = sqlx::query_as::<_, OutgoingWebhook>(
            r#"
            UPDATE outgoing_webhooks
            SET name = COALESCE($2, name),
                url = COALESCE($3, url),
                event_types = COALESCE($4, event_types),
                enabled = COALESCE($5, enabled),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(url)
        .bind(event_types)
        .bind(enabled)
        .fetch_one(pool)
        .await?;

        Ok(webhook)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query("DELETE FROM outgoing_webhooks WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_for_event(
        pool: &PgPool,
        org_id: Uuid,
        event_type: &str,
    ) -> ApiResult<Vec<OutgoingWebhook>> {
        let webhooks = sqlx::query_as::<_, OutgoingWebhook>(
            r#"
            SELECT * FROM outgoing_webhooks
            WHERE org_id = $1 AND enabled = TRUE AND $2 = ANY(event_types)
            "#,
        )
        .bind(org_id)
        .bind(event_type)
        .fetch_all(pool)
        .await?;

        Ok(webhooks)
    }
}

impl WebhookDelivery {
    pub async fn create(
        pool: &PgPool,
        webhook_id: Uuid,
        event_type: &str,
        payload: &JsonValue,
    ) -> ApiResult<WebhookDelivery> {
        let delivery = sqlx::query_as::<_, WebhookDelivery>(
            r#"
            INSERT INTO webhook_deliveries (webhook_id, event_type, payload)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(webhook_id)
        .bind(event_type)
        .bind(payload)
        .fetch_one(pool)
        .await?;

        Ok(delivery)
    }

    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: &str,
        response_code: Option<i32>,
        response_body: Option<&str>,
        error_message: Option<&str>,
        attempt: i32,
    ) -> ApiResult<()> {
        let delivered_at = if status == "delivered" {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query(
            r#"
            UPDATE webhook_deliveries
            SET status = $2, response_code = $3, response_body = $4,
                error_message = $5, attempt = $6, delivered_at = COALESCE($7, delivered_at)
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(response_code)
        .bind(response_body)
        .bind(error_message)
        .bind(attempt)
        .bind(delivered_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn list_by_webhook(
        pool: &PgPool,
        webhook_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> ApiResult<Vec<WebhookDelivery>> {
        let deliveries = sqlx::query_as::<_, WebhookDelivery>(
            r#"
            SELECT * FROM webhook_deliveries
            WHERE webhook_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(webhook_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(deliveries)
    }
}
