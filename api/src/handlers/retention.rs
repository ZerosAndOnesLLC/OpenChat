use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::models::{
    retention::{
        CreateLegalHoldRequest, LegalHold, RetentionPolicy, RetentionPolicyResponse,
        UpdateRetentionPolicyRequest,
    },
    user::User,
};
use crate::services::{audit_logger::AuditLogger, tv_api::TokenClaims};

pub async fn get_retention_policies(
    req: HttpRequest,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let org_id = claims.org_id;

    let policies: Vec<RetentionPolicy> = sqlx::query_as(
        "SELECT * FROM retention_policies WHERE org_id = $1 ORDER BY policy_type",
    )
    .bind(org_id)
    .fetch_all(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch retention policies: {}", e)))?;

    Ok(HttpResponse::Ok().json(RetentionPolicyResponse { policies }))
}

pub async fn update_retention_policy(
    req: HttpRequest,
    db: web::Data<PgPool>,
    payload: web::Json<UpdateRetentionPolicyRequest>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let org_id = claims.org_id;

    // Get current user for audit logging
    let current_user = User::get_by_tv_user_id(db.get_ref(), claims.user_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get current user: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Validate policy_type
    if payload.policy_type != "messages" && payload.policy_type != "files" {
        return Err(ApiError::BadRequest(
            "policy_type must be 'messages' or 'files'".to_string(),
        ));
    }

    // Validate retention_days
    if payload.retention_days <= 0 {
        return Err(ApiError::BadRequest(
            "retention_days must be greater than 0".to_string(),
        ));
    }

    // Get old policy for audit log
    let old_policy: Option<RetentionPolicy> = sqlx::query_as(
        "SELECT * FROM retention_policies WHERE org_id = $1 AND policy_type = $2",
    )
    .bind(org_id)
    .bind(&payload.policy_type)
    .fetch_optional(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch old retention policy: {}", e)))?;

    // Upsert retention policy
    let policy: RetentionPolicy = sqlx::query_as(
        "INSERT INTO retention_policies
            (id, org_id, policy_type, retention_days, enabled, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
         ON CONFLICT (org_id, policy_type)
         DO UPDATE SET
            retention_days = EXCLUDED.retention_days,
            enabled = EXCLUDED.enabled,
            updated_at = NOW()
         RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(org_id)
    .bind(&payload.policy_type)
    .bind(payload.retention_days)
    .bind(payload.enabled)
    .fetch_one(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to update retention policy: {}", e)))?;

    // Log retention policy update in audit log
    if let Err(e) = AuditLogger::log_settings_updated(
        db.get_ref(),
        current_user.id,
        "retention_policy",
        json!({
            "policy_type": &payload.policy_type,
            "old_retention_days": old_policy.as_ref().map(|p| p.retention_days),
            "old_enabled": old_policy.as_ref().map(|p| p.enabled),
        }),
        json!({
            "policy_type": &payload.policy_type,
            "retention_days": payload.retention_days,
            "enabled": payload.enabled,
        }),
        Some(&req),
    )
    .await
    {
        tracing::warn!(
            "Failed to create audit log for retention policy update: {}",
            e
        );
    }

    Ok(HttpResponse::Ok().json(policy))
}

pub async fn create_legal_hold(
    req: HttpRequest,
    db: web::Data<PgPool>,
    channel_id: web::Path<Uuid>,
    payload: web::Json<CreateLegalHoldRequest>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let org_id = claims.org_id;
    let channel_id = channel_id.into_inner();

    // Get current user for audit logging
    let current_user = User::get_by_tv_user_id(db.get_ref(), claims.user_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get current user: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Verify channel exists and belongs to org
    let channel_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM channels WHERE id = $1 AND org_id = $2)",
    )
    .bind(channel_id)
    .bind(org_id)
    .fetch_one(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to verify channel: {}", e)))?;

    if !channel_exists {
        return Err(ApiError::NotFound("Channel not found".to_string()));
    }

    // Check if there's already an active legal hold
    let existing_hold: Option<LegalHold> = sqlx::query_as(
        "SELECT * FROM legal_holds WHERE channel_id = $1 AND enabled = TRUE",
    )
    .bind(channel_id)
    .fetch_optional(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to check existing legal hold: {}", e)))?;

    if existing_hold.is_some() {
        return Err(ApiError::BadRequest(
            "Channel already has an active legal hold".to_string(),
        ));
    }

    // Create legal hold
    let legal_hold: LegalHold = sqlx::query_as(
        "INSERT INTO legal_holds
            (id, org_id, channel_id, reason, enabled, created_by, created_at)
         VALUES ($1, $2, $3, $4, TRUE, $5, NOW())
         RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(org_id)
    .bind(channel_id)
    .bind(&payload.reason)
    .bind(current_user.id)
    .fetch_one(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to create legal hold: {}", e)))?;

    // Log legal hold creation in audit log
    if let Err(e) = AuditLogger::log(
        db.get_ref(),
        Some(current_user.id),
        "legal_hold_created",
        "channel",
        Some(channel_id),
        json!({
            "channel_id": channel_id,
            "reason": &payload.reason,
        }),
        Some(&req),
    )
    .await
    {
        tracing::warn!("Failed to create audit log for legal hold creation: {}", e);
    }

    Ok(HttpResponse::Created().json(legal_hold))
}

pub async fn get_legal_hold(
    req: HttpRequest,
    db: web::Data<PgPool>,
    channel_id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let org_id = claims.org_id;
    let channel_id = channel_id.into_inner();

    // Verify channel exists and belongs to org
    let channel_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM channels WHERE id = $1 AND org_id = $2)",
    )
    .bind(channel_id)
    .bind(org_id)
    .fetch_one(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to verify channel: {}", e)))?;

    if !channel_exists {
        return Err(ApiError::NotFound("Channel not found".to_string()));
    }

    let legal_hold: Option<LegalHold> = sqlx::query_as(
        "SELECT * FROM legal_holds WHERE channel_id = $1 AND enabled = TRUE",
    )
    .bind(channel_id)
    .fetch_optional(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch legal hold: {}", e)))?;

    if let Some(hold) = legal_hold {
        Ok(HttpResponse::Ok().json(hold))
    } else {
        Ok(HttpResponse::Ok().json(json!({ "enabled": false })))
    }
}

pub async fn disable_legal_hold(
    req: HttpRequest,
    db: web::Data<PgPool>,
    channel_id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let org_id = claims.org_id;
    let channel_id = channel_id.into_inner();

    // Get current user for audit logging
    let current_user = User::get_by_tv_user_id(db.get_ref(), claims.user_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get current user: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Verify channel exists and belongs to org
    let channel_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM channels WHERE id = $1 AND org_id = $2)",
    )
    .bind(channel_id)
    .bind(org_id)
    .fetch_one(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to verify channel: {}", e)))?;

    if !channel_exists {
        return Err(ApiError::NotFound("Channel not found".to_string()));
    }

    // Disable legal hold
    let result = sqlx::query(
        "UPDATE legal_holds
         SET enabled = FALSE, disabled_at = NOW(), disabled_by = $1
         WHERE channel_id = $2 AND enabled = TRUE",
    )
    .bind(current_user.id)
    .bind(channel_id)
    .execute(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to disable legal hold: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound(
            "No active legal hold found for this channel".to_string(),
        ));
    }

    // Log legal hold removal in audit log
    if let Err(e) = AuditLogger::log(
        db.get_ref(),
        Some(current_user.id),
        "legal_hold_disabled",
        "channel",
        Some(channel_id),
        json!({
            "channel_id": channel_id,
        }),
        Some(&req),
    )
    .await
    {
        tracing::warn!("Failed to create audit log for legal hold removal: {}", e);
    }

    Ok(HttpResponse::NoContent().finish())
}
