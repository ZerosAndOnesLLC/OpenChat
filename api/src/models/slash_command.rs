use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SlashCommand {
    pub id: Uuid,
    pub org_id: Uuid,
    pub command_name: String,
    pub description: String,
    pub usage_hint: Option<String>,
    pub handler_type: String,
    pub webhook_url: Option<String>,
    pub response_type: String,
    pub created_by: Uuid,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl SlashCommand {
    pub async fn create(
        pool: &PgPool,
        org_id: Uuid,
        command_name: &str,
        description: &str,
        usage_hint: Option<&str>,
        handler_type: &str,
        webhook_url: Option<&str>,
        response_type: &str,
        created_by: Uuid,
    ) -> ApiResult<SlashCommand> {
        let command = sqlx::query_as::<_, SlashCommand>(
            r#"
            INSERT INTO slash_commands (id, org_id, command_name, description, usage_hint, handler_type, webhook_url, response_type, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(command_name)
        .bind(description)
        .bind(usage_hint)
        .bind(handler_type)
        .bind(webhook_url)
        .bind(response_type)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(command)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<SlashCommand>> {
        let command = sqlx::query_as::<_, SlashCommand>(
            "SELECT * FROM slash_commands WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(command)
    }

    pub async fn get_by_org_and_name(
        pool: &PgPool,
        org_id: Uuid,
        command_name: &str,
    ) -> ApiResult<Option<SlashCommand>> {
        let command = sqlx::query_as::<_, SlashCommand>(
            "SELECT * FROM slash_commands WHERE org_id = $1 AND command_name = $2",
        )
        .bind(org_id)
        .bind(command_name)
        .fetch_optional(pool)
        .await?;

        Ok(command)
    }

    pub async fn list_by_org(pool: &PgPool, org_id: Uuid) -> ApiResult<Vec<SlashCommand>> {
        let commands = sqlx::query_as::<_, SlashCommand>(
            "SELECT * FROM slash_commands WHERE org_id = $1 ORDER BY command_name",
        )
        .bind(org_id)
        .fetch_all(pool)
        .await?;

        Ok(commands)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        description: Option<&str>,
        usage_hint: Option<&str>,
        webhook_url: Option<&str>,
        response_type: Option<&str>,
        enabled: Option<bool>,
    ) -> ApiResult<SlashCommand> {
        let command = sqlx::query_as::<_, SlashCommand>(
            r#"
            UPDATE slash_commands
            SET description = COALESCE($1, description),
                usage_hint = COALESCE($2, usage_hint),
                webhook_url = COALESCE($3, webhook_url),
                response_type = COALESCE($4, response_type),
                enabled = COALESCE($5, enabled)
            WHERE id = $6
            RETURNING *
            "#,
        )
        .bind(description)
        .bind(usage_hint)
        .bind(webhook_url)
        .bind(response_type)
        .bind(enabled)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(command)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> ApiResult<()> {
        sqlx::query("DELETE FROM slash_commands WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
