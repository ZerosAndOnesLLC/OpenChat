use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::{
        channel::{Channel, ChannelMember},
        crypto_device::CryptoDevice,
        encrypted_channel::EncryptedChannel,
        user::User,
    },
    services::tv_api::TokenClaims,
};

// -- Request/Response types --

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_id: String,
    pub display_name: Option<String>,
    pub identity_key: String,
    pub signing_key: String,
    pub one_time_keys: Option<serde_json::Value>,
    pub fallback_key: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: String,
    pub display_name: Option<String>,
    pub identity_key: String,
    pub signing_key: String,
    pub one_time_key_count: usize,
    pub has_fallback_key: bool,
    pub verified: bool,
    pub last_seen_at: String,
    pub created_at: String,
}

impl From<CryptoDevice> for DeviceResponse {
    fn from(d: CryptoDevice) -> Self {
        let otk_count = d.one_time_keys.as_object().map(|o| o.len()).unwrap_or(0);
        Self {
            id: d.id,
            user_id: d.user_id,
            device_id: d.device_id,
            display_name: d.display_name,
            identity_key: d.identity_key,
            signing_key: d.signing_key,
            one_time_key_count: otk_count,
            has_fallback_key: d.fallback_key.is_some(),
            verified: d.verified,
            last_seen_at: d.last_seen_at.to_rfc3339(),
            created_at: d.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UploadKeysRequest {
    pub one_time_keys: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ClaimKeysRequest {
    pub devices: Vec<ClaimKeyTarget>,
}

#[derive(Debug, Deserialize)]
pub struct ClaimKeyTarget {
    pub user_id: Uuid,
    pub device_id: String,
}

#[derive(Debug, Serialize)]
pub struct ClaimKeysResponse {
    pub one_time_keys: Vec<ClaimedKey>,
}

#[derive(Debug, Serialize)]
pub struct ClaimedKey {
    pub user_id: Uuid,
    pub device_id: String,
    pub key_id: String,
    pub key: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct QueryKeysRequest {
    pub user_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct QueryKeysResponse {
    pub device_keys: Vec<DeviceKeyInfo>,
}

#[derive(Debug, Serialize)]
pub struct DeviceKeyInfo {
    pub user_id: Uuid,
    pub device_id: String,
    pub identity_key: String,
    pub signing_key: String,
    pub one_time_key_count: usize,
    pub verified: bool,
}

#[derive(Debug, Serialize)]
pub struct EncryptionStatusResponse {
    pub encryption_enabled: bool,
    pub algorithm: Option<String>,
    pub rotation_period_msgs: Option<i32>,
    pub rotation_period_ms: Option<i64>,
}

// -- Handlers --

/// POST /api/crypto/devices — register a device
pub async fn register_device(
    pool: web::Data<PgPool>,
    body: web::Json<RegisterDeviceRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id).await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    if body.device_id.is_empty() || body.device_id.len() > 64 {
        return Err(ApiError::BadRequest("device_id must be 1-64 characters".to_string()));
    }
    if body.identity_key.is_empty() || body.signing_key.is_empty() {
        return Err(ApiError::BadRequest("identity_key and signing_key are required".to_string()));
    }

    let device = CryptoDevice::create(
        pool.get_ref(),
        user.id,
        &body.device_id,
        body.display_name.as_deref(),
        &body.identity_key,
        &body.signing_key,
    ).await?;

    // Upload one-time keys if provided
    if let Some(ref otk) = body.one_time_keys {
        if otk.is_object() && !otk.as_object().unwrap().is_empty() {
            CryptoDevice::upload_one_time_keys(pool.get_ref(), user.id, &body.device_id, otk.clone()).await?;
        }
    }

    // Set fallback key if provided
    if let Some(ref fallback) = body.fallback_key {
        sqlx::query(
            r#"
            UPDATE user_crypto_devices
            SET fallback_key = $3
            WHERE user_id = $1 AND device_id = $2
            "#,
        )
        .bind(user.id)
        .bind(&body.device_id)
        .bind(fallback)
        .execute(pool.get_ref())
        .await?;
    }

    // Re-fetch to get updated state
    let device = CryptoDevice::get_by_user_and_device(pool.get_ref(), user.id, &body.device_id).await?
        .unwrap_or(device);

    Ok(HttpResponse::Created().json(DeviceResponse::from(device)))
}

/// GET /api/crypto/devices — list my devices
pub async fn list_my_devices(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id).await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let devices = CryptoDevice::list_by_user(pool.get_ref(), user.id).await?;
    let responses: Vec<DeviceResponse> = devices.into_iter().map(DeviceResponse::from).collect();

    Ok(HttpResponse::Ok().json(responses))
}

/// GET /api/crypto/devices/{user_id} — list a user's devices
pub async fn list_user_devices(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let _claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let target_user_id = path.into_inner();
    let devices = CryptoDevice::list_by_user(pool.get_ref(), target_user_id).await?;
    let responses: Vec<DeviceResponse> = devices.into_iter().map(DeviceResponse::from).collect();

    Ok(HttpResponse::Ok().json(responses))
}

/// DELETE /api/crypto/devices/{device_id} — remove a device
pub async fn remove_device(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id).await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let device_id = path.into_inner();
    CryptoDevice::delete(pool.get_ref(), user.id, &device_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/crypto/devices/{device_id}/verify — verify a device
pub async fn verify_device(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id).await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let device_id = path.into_inner();
    let device = CryptoDevice::get_by_user_and_device(pool.get_ref(), user.id, &device_id).await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    let device = CryptoDevice::set_verified(pool.get_ref(), device.id, true).await?;

    Ok(HttpResponse::Ok().json(DeviceResponse::from(device)))
}

/// POST /api/crypto/keys/upload — upload one-time keys for a device
pub async fn upload_keys(
    pool: web::Data<PgPool>,
    body: web::Json<UploadKeysRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id).await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Device ID should be in the query or header; for simplicity, require it in the body
    // The client should know its own device_id
    let device_id = req.headers().get("X-Device-Id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::BadRequest("X-Device-Id header required".to_string()))?;

    let device = CryptoDevice::upload_one_time_keys(
        pool.get_ref(),
        user.id,
        device_id,
        body.one_time_keys.clone(),
    ).await?;

    Ok(HttpResponse::Ok().json(DeviceResponse::from(device)))
}

/// POST /api/crypto/keys/claim — claim one-time keys for devices
pub async fn claim_keys(
    pool: web::Data<PgPool>,
    body: web::Json<ClaimKeysRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let _claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let mut claimed_keys = Vec::new();

    for target in &body.devices {
        if let Ok(Some((key_id, key))) = CryptoDevice::claim_one_time_key(
            pool.get_ref(),
            target.user_id,
            &target.device_id,
        ).await {
            claimed_keys.push(ClaimedKey {
                user_id: target.user_id,
                device_id: target.device_id.clone(),
                key_id,
                key,
            });
        }
    }

    Ok(HttpResponse::Ok().json(ClaimKeysResponse { one_time_keys: claimed_keys }))
}

/// POST /api/crypto/keys/query — query device keys for users
pub async fn query_keys(
    pool: web::Data<PgPool>,
    body: web::Json<QueryKeysRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let _claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let devices = CryptoDevice::list_by_users(pool.get_ref(), &body.user_ids).await?;

    let device_keys: Vec<DeviceKeyInfo> = devices.into_iter().map(|d| {
        let otk_count = d.one_time_keys.as_object().map(|o| o.len()).unwrap_or(0);
        DeviceKeyInfo {
            user_id: d.user_id,
            device_id: d.device_id,
            identity_key: d.identity_key,
            signing_key: d.signing_key,
            one_time_key_count: otk_count,
            verified: d.verified,
        }
    }).collect();

    Ok(HttpResponse::Ok().json(QueryKeysResponse { device_keys }))
}

/// POST /api/channels/{id}/encryption — enable encryption on a channel (admin, irreversible)
pub async fn enable_channel_encryption(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id).await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let channel_id = path.into_inner();

    // Verify channel exists
    let channel = Channel::get_by_id(pool.get_ref(), channel_id).await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Verify user is a member and is admin/owner
    let member = sqlx::query_as::<_, ChannelMember>(
        r#"
        SELECT * FROM channel_members
        WHERE channel_id = $1 AND user_id = $2
        "#,
    )
    .bind(channel_id)
    .bind(user.id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| ApiError::Authorization("Not a channel member".to_string()))?;

    if member.role != "admin" && member.role != "owner" && channel.created_by != user.id {
        return Err(ApiError::Authorization("Only channel admins can enable encryption".to_string()));
    }

    let ec = EncryptedChannel::enable_encryption(pool.get_ref(), channel_id).await?;

    Ok(HttpResponse::Ok().json(EncryptionStatusResponse {
        encryption_enabled: ec.encryption_enabled,
        algorithm: Some(ec.algorithm),
        rotation_period_msgs: Some(ec.rotation_period_msgs),
        rotation_period_ms: Some(ec.rotation_period_ms),
    }))
}

/// GET /api/channels/{id}/encryption — get encryption status
pub async fn get_channel_encryption(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let _claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let channel_id = path.into_inner();

    let ec = EncryptedChannel::get(pool.get_ref(), channel_id).await?;

    match ec {
        Some(ec) => Ok(HttpResponse::Ok().json(EncryptionStatusResponse {
            encryption_enabled: ec.encryption_enabled,
            algorithm: Some(ec.algorithm),
            rotation_period_msgs: Some(ec.rotation_period_msgs),
            rotation_period_ms: Some(ec.rotation_period_ms),
        })),
        None => Ok(HttpResponse::Ok().json(EncryptionStatusResponse {
            encryption_enabled: false,
            algorithm: None,
            rotation_period_msgs: None,
            rotation_period_ms: None,
        })),
    }
}
