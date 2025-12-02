use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub metadata: JsonValue,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditLog {
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub metadata: Option<JsonValue>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogFilters {
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl AuditLog {
    /// Create a new audit log entry
    pub async fn create(pool: &PgPool, create: CreateAuditLog) -> ApiResult<AuditLog> {
        let metadata = create.metadata.unwrap_or(serde_json::json!({}));

        let audit_log = sqlx::query_as::<_, AuditLog>(
            r#"
            INSERT INTO audit_logs (user_id, action, resource_type, resource_id, metadata, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6::inet, $7)
            RETURNING id, user_id, action, resource_type, resource_id, metadata,
                      host(ip_address) as ip_address, user_agent, timestamp
            "#,
        )
        .bind(create.user_id)
        .bind(&create.action)
        .bind(&create.resource_type)
        .bind(create.resource_id)
        .bind(&metadata)
        .bind(create.ip_address)
        .bind(create.user_agent)
        .fetch_one(pool)
        .await?;

        Ok(audit_log)
    }

    /// List audit logs with filters
    pub async fn list(pool: &PgPool, filters: AuditLogFilters) -> ApiResult<Vec<AuditLog>> {
        let limit = filters.limit.unwrap_or(100).min(1000); // Max 1000
        let offset = filters.offset.unwrap_or(0);

        let mut query = String::from(
            r#"
            SELECT id, user_id, action, resource_type, resource_id, metadata,
                   host(ip_address) as ip_address, user_agent, timestamp
            FROM audit_logs
            WHERE 1=1
            "#,
        );

        let mut bindings: Vec<String> = Vec::new();
        let mut param_count = 0;

        if let Some(user_id) = filters.user_id {
            param_count += 1;
            query.push_str(&format!(" AND user_id = ${}", param_count));
            bindings.push(user_id.to_string());
        }

        if let Some(ref action) = filters.action {
            param_count += 1;
            query.push_str(&format!(" AND action = ${}", param_count));
            bindings.push(action.clone());
        }

        if let Some(ref resource_type) = filters.resource_type {
            param_count += 1;
            query.push_str(&format!(" AND resource_type = ${}", param_count));
            bindings.push(resource_type.clone());
        }

        if let Some(resource_id) = filters.resource_id {
            param_count += 1;
            query.push_str(&format!(" AND resource_id = ${}", param_count));
            bindings.push(resource_id.to_string());
        }

        if let Some(start_date) = filters.start_date {
            param_count += 1;
            query.push_str(&format!(" AND timestamp >= ${}", param_count));
            bindings.push(start_date.to_rfc3339());
        }

        if let Some(end_date) = filters.end_date {
            param_count += 1;
            query.push_str(&format!(" AND timestamp <= ${}", param_count));
            bindings.push(end_date.to_rfc3339());
        }

        query.push_str(" ORDER BY timestamp DESC");
        param_count += 1;
        query.push_str(&format!(" LIMIT ${}", param_count));
        bindings.push(limit.to_string());

        param_count += 1;
        query.push_str(&format!(" OFFSET ${}", param_count));
        bindings.push(offset.to_string());

        // Build the query dynamically
        let mut sqlx_query = sqlx::query_as::<_, AuditLog>(&query);

        if let Some(user_id) = filters.user_id {
            sqlx_query = sqlx_query.bind(user_id);
        }
        if let Some(action) = filters.action {
            sqlx_query = sqlx_query.bind(action);
        }
        if let Some(resource_type) = filters.resource_type {
            sqlx_query = sqlx_query.bind(resource_type);
        }
        if let Some(resource_id) = filters.resource_id {
            sqlx_query = sqlx_query.bind(resource_id);
        }
        if let Some(start_date) = filters.start_date {
            sqlx_query = sqlx_query.bind(start_date);
        }
        if let Some(end_date) = filters.end_date {
            sqlx_query = sqlx_query.bind(end_date);
        }
        sqlx_query = sqlx_query.bind(limit).bind(offset);

        let logs = sqlx_query.fetch_all(pool).await?;

        Ok(logs)
    }

    /// Count audit logs matching filters
    pub async fn count(pool: &PgPool, filters: &AuditLogFilters) -> ApiResult<i64> {
        let mut query = String::from("SELECT COUNT(*) as count FROM audit_logs WHERE 1=1");

        let mut sqlx_query = sqlx::query_scalar::<_, i64>(&query);

        if let Some(user_id) = filters.user_id {
            query.push_str(" AND user_id = $1");
            sqlx_query = sqlx::query_scalar::<_, i64>(&query).bind(user_id);
        }

        if let Some(ref action) = filters.action {
            sqlx_query = sqlx_query.bind(action);
        }

        if let Some(ref resource_type) = filters.resource_type {
            sqlx_query = sqlx_query.bind(resource_type);
        }

        if let Some(resource_id) = filters.resource_id {
            sqlx_query = sqlx_query.bind(resource_id);
        }

        if let Some(start_date) = filters.start_date {
            sqlx_query = sqlx_query.bind(start_date);
        }

        if let Some(end_date) = filters.end_date {
            sqlx_query = sqlx_query.bind(end_date);
        }

        let count = sqlx_query.fetch_one(pool).await?;
        Ok(count)
    }
}

// Common audit action constants
#[allow(dead_code)]
pub mod actions {
    // Message actions
    pub const MESSAGE_CREATED: &str = "message.created";
    pub const MESSAGE_UPDATED: &str = "message.updated";
    pub const MESSAGE_DELETED: &str = "message.deleted";

    // Channel actions
    pub const CHANNEL_CREATED: &str = "channel.created";
    pub const CHANNEL_UPDATED: &str = "channel.updated";
    pub const CHANNEL_DELETED: &str = "channel.deleted";
    pub const CHANNEL_MEMBER_ADDED: &str = "channel.member_added";
    pub const CHANNEL_MEMBER_REMOVED: &str = "channel.member_removed";

    // Permission actions
    pub const PERMISSION_GRANTED: &str = "permission.granted";
    pub const PERMISSION_REVOKED: &str = "permission.revoked";
    pub const ROLE_ASSIGNED: &str = "role.assigned";
    pub const ROLE_UNASSIGNED: &str = "role.unassigned";
    pub const ROLE_CREATED: &str = "role.created";
    pub const ROLE_UPDATED: &str = "role.updated";
    pub const ROLE_DELETED: &str = "role.deleted";

    // Settings actions
    pub const SETTINGS_UPDATED: &str = "settings.updated";
    pub const STORAGE_SETTINGS_UPDATED: &str = "storage_settings.updated";

    // Auth actions
    pub const USER_LOGIN: &str = "user.login";
    pub const USER_LOGOUT: &str = "user.logout";
    pub const USER_LOGIN_FAILED: &str = "user.login_failed";
}

// Resource type constants
#[allow(dead_code)]
pub mod resource_types {
    pub const MESSAGE: &str = "message";
    pub const CHANNEL: &str = "channel";
    pub const USER: &str = "user";
    pub const ROLE: &str = "role";
    pub const PERMISSION: &str = "permission";
    pub const SETTINGS: &str = "settings";
    pub const ORGANIZATION: &str = "organization";
}
