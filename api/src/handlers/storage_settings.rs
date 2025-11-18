use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::models::{storage_settings::{
    StorageSettings, StorageSettingsResponse, UpdateStorageSettingsRequest,
}, user::User};
use crate::services::{audit_logger::AuditLogger, tv_api::TokenClaims};

// Simple encryption/decryption for credentials (in production, use proper encryption)
// TODO: Implement proper encryption with a secure key management system
fn encrypt_credential(credential: &str) -> String {
    // For now, we'll use base64 encoding as a placeholder
    // In production, use proper encryption like AES-256-GCM with AWS KMS or similar
    base64::encode(credential)
}

fn decrypt_credential(encrypted: &str) -> Result<String, ApiError> {
    base64::decode(encrypted)
        .map_err(|e| ApiError::Internal(format!("Failed to decrypt credential: {}", e)))
        .and_then(|bytes| {
            String::from_utf8(bytes)
                .map_err(|e| ApiError::Internal(format!("Invalid credential format: {}", e)))
        })
}

pub async fn get_storage_settings(
    req: HttpRequest,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // TODO: Add admin permission check
    // For now, we'll allow any authenticated user to view their org's settings
    let org_id = claims.org_id;

    let settings: Option<StorageSettings> = sqlx::query_as(
        "SELECT * FROM storage_settings WHERE org_id = $1",
    )
    .bind(org_id)
    .fetch_optional(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch storage settings: {}", e)))?;

    if let Some(settings) = settings {
        Ok(HttpResponse::Ok().json(StorageSettingsResponse::from(settings)))
    } else {
        // Return default settings if none exist
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "org_id": org_id,
            "storage_type": "local",
            "s3_bucket": null,
            "s3_region": null,
            "s3_endpoint": null
        })))
    }
}

pub async fn update_storage_settings(
    req: HttpRequest,
    db: web::Data<PgPool>,
    payload: web::Json<UpdateStorageSettingsRequest>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // TODO: Add admin permission check
    // For now, we'll allow any authenticated user to update their org's settings
    let org_id = claims.org_id;

    // Get current user for audit logging
    let current_user = User::get_by_tv_user_id(db.get_ref(), claims.user_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get current user: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Get old settings for audit log
    let old_settings: Option<StorageSettings> = sqlx::query_as(
        "SELECT * FROM storage_settings WHERE org_id = $1",
    )
    .bind(org_id)
    .fetch_optional(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch old storage settings: {}", e)))?;

    // Validate storage_type
    if payload.storage_type != "local" && payload.storage_type != "s3" {
        return Err(ApiError::BadRequest(
            "storage_type must be 'local' or 's3'".to_string(),
        ));
    }

    // If S3, validate required fields
    if payload.storage_type == "s3" {
        if payload.s3_bucket.is_none() || payload.s3_region.is_none() {
            return Err(ApiError::BadRequest(
                "s3_bucket and s3_region are required for S3 storage".to_string(),
            ));
        }
        if payload.s3_access_key_id.is_none() || payload.s3_secret_key.is_none() {
            return Err(ApiError::BadRequest(
                "s3_access_key_id and s3_secret_key are required for S3 storage".to_string(),
            ));
        }
    }

    // Encrypt credentials if provided
    let s3_access_key_id_encrypted = payload
        .s3_access_key_id
        .as_ref()
        .map(|key| encrypt_credential(key));
    let s3_secret_key_encrypted = payload
        .s3_secret_key
        .as_ref()
        .map(|secret| encrypt_credential(secret));

    // Upsert storage settings
    let settings: StorageSettings = sqlx::query_as(
        "INSERT INTO storage_settings
            (id, org_id, storage_type, s3_bucket, s3_region, s3_access_key_id_encrypted,
             s3_secret_key_encrypted, s3_endpoint, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
         ON CONFLICT (org_id)
         DO UPDATE SET
            storage_type = EXCLUDED.storage_type,
            s3_bucket = EXCLUDED.s3_bucket,
            s3_region = EXCLUDED.s3_region,
            s3_access_key_id_encrypted = EXCLUDED.s3_access_key_id_encrypted,
            s3_secret_key_encrypted = EXCLUDED.s3_secret_key_encrypted,
            s3_endpoint = EXCLUDED.s3_endpoint,
            updated_at = NOW()
         RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(org_id)
    .bind(&payload.storage_type)
    .bind(&payload.s3_bucket)
    .bind(&payload.s3_region)
    .bind(s3_access_key_id_encrypted)
    .bind(s3_secret_key_encrypted)
    .bind(&payload.s3_endpoint)
    .fetch_one(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to update storage settings: {}", e)))?;

    // Log storage settings update in audit log
    if let Err(e) = AuditLogger::log_settings_updated(
        db.get_ref(),
        current_user.id,
        "storage",
        json!({
            "storage_type": old_settings.as_ref().map(|s| &s.storage_type),
            "s3_bucket": old_settings.as_ref().and_then(|s| s.s3_bucket.as_ref()),
            "s3_region": old_settings.as_ref().and_then(|s| s.s3_region.as_ref()),
        }),
        json!({
            "storage_type": &settings.storage_type,
            "s3_bucket": &settings.s3_bucket,
            "s3_region": &settings.s3_region,
        }),
        Some(&req),
    )
    .await
    {
        tracing::warn!("Failed to create audit log for storage settings update: {}", e);
    }

    // Invalidate storage factory cache so it picks up new settings
    // TODO: Implement cache invalidation

    Ok(HttpResponse::Ok().json(StorageSettingsResponse::from(settings)))
}
