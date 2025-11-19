use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::ApiResult,
    models::audit_log::{AuditLog, AuditLogFilters},
};

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct ListAuditLogsRequest {
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ListAuditLogsResponse {
    pub logs: Vec<AuditLogResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl From<AuditLog> for AuditLogResponse {
    fn from(log: AuditLog) -> Self {
        Self {
            id: log.id,
            user_id: log.user_id,
            action: log.action,
            resource_type: log.resource_type,
            resource_id: log.resource_id,
            metadata: log.metadata,
            ip_address: log.ip_address,
            user_agent: log.user_agent,
            timestamp: log.timestamp,
        }
    }
}

/// GET /api/audit-logs - List audit logs with filters (admin only)
/// Requires: org.view_audit_logs permission
pub async fn list_audit_logs(
    pool: web::Data<PgPool>,
    query: web::Query<ListAuditLogsRequest>,
    _req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let filters = AuditLogFilters {
        user_id: query.user_id,
        action: query.action.clone(),
        resource_type: query.resource_type.clone(),
        resource_id: query.resource_id,
        start_date: query.start_date,
        end_date: query.end_date,
        limit: query.limit,
        offset: query.offset,
    };

    let logs = AuditLog::list(pool.get_ref(), filters.clone()).await?;
    let total = AuditLog::count(pool.get_ref(), &filters).await?;

    let response = ListAuditLogsResponse {
        logs: logs.into_iter().map(|l| l.into()).collect(),
        total,
        limit: filters.limit.unwrap_or(100),
        offset: filters.offset.unwrap_or(0),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/audit-logs/export - Export audit logs to CSV (admin only)
/// Requires: org.view_audit_logs permission
pub async fn export_audit_logs(
    pool: web::Data<PgPool>,
    query: web::Query<ListAuditLogsRequest>,
    _req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let filters = AuditLogFilters {
        user_id: query.user_id,
        action: query.action.clone(),
        resource_type: query.resource_type.clone(),
        resource_id: query.resource_id,
        start_date: query.start_date,
        end_date: query.end_date,
        limit: Some(10000), // Max 10k rows for export
        offset: query.offset,
    };

    let logs = AuditLog::list(pool.get_ref(), filters).await?;

    // Generate CSV
    let mut csv = String::from("ID,User ID,Action,Resource Type,Resource ID,IP Address,User Agent,Timestamp,Metadata\n");

    for log in logs {
        let metadata_str = serde_json::to_string(&log.metadata).unwrap_or_default();
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            log.id,
            log.user_id.map(|id| id.to_string()).unwrap_or_default(),
            escape_csv(&log.action),
            escape_csv(&log.resource_type),
            log.resource_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            log.ip_address.as_deref().unwrap_or(""),
            escape_csv(log.user_agent.as_deref().unwrap_or("")),
            log.timestamp.to_rfc3339(),
            escape_csv(&metadata_str),
        ));
    }

    Ok(HttpResponse::Ok()
        .content_type("text/csv")
        .insert_header((
            "Content-Disposition",
            "attachment; filename=\"audit-logs.csv\"",
        ))
        .body(csv))
}

/// GET /api/audit-logs/actions - Get list of all unique actions
/// Useful for filtering in the UI
pub async fn list_actions(pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
    let actions = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT action
        FROM audit_logs
        ORDER BY action
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(actions))
}

/// GET /api/audit-logs/resource-types - Get list of all unique resource types
/// Useful for filtering in the UI
pub async fn list_resource_types(pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
    let resource_types = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT resource_type
        FROM audit_logs
        ORDER BY resource_type
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(resource_types))
}

/// Helper function to escape CSV fields
fn escape_csv(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}
