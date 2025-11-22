use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::{
        device::{DeviceSession},
        user::User,
    },
    services::{device_pairing, tv_api::{TokenClaims, TvApiClient}},
};

#[derive(Debug, Serialize)]
pub struct GenerateCodeResponse {
    pub code: String,
    pub expires_in: i64, // seconds
}

#[derive(Debug, Deserialize)]
pub struct VerifyCodeRequest {
    pub code: String,
    pub device_name: Option<String>,
    pub device_fingerprint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VerifyCodeResponse {
    pub access_token: String,
    pub user: UserInfo,
    pub device_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub org_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeviceSessionResponse {
    pub id: Uuid,
    pub device_type: String,
    pub device_name: Option<String>,
    pub last_active_at: String,
    pub created_at: String,
}

impl From<DeviceSession> for DeviceSessionResponse {
    fn from(session: DeviceSession) -> Self {
        Self {
            id: session.id,
            device_type: session.device_type,
            device_name: session.device_name,
            last_active_at: session.last_active_at.to_rfc3339(),
            created_at: session.created_at.to_rfc3339(),
        }
    }
}

/// POST /api/auth/device/generate-code
/// Generate a pairing code for desktop app authentication
/// Requires: Valid JWT from web user
pub async fn generate_code(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Get claims from request extensions (set by auth middleware)
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get the user
    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Generate pairing code
    let pairing_code =
        device_pairing::generate_pairing_code(pool.get_ref(), user.id, claims.org_id).await?;

    // Calculate seconds until expiration
    let expires_in = (pairing_code.expires_at - chrono::Utc::now()).num_seconds();

    Ok(HttpResponse::Ok().json(GenerateCodeResponse {
        code: pairing_code.code,
        expires_in,
    }))
}

/// POST /api/auth/device/verify-code
/// Verify a pairing code and create a device session
/// Public endpoint (no authentication required)
pub async fn verify_code(
    pool: web::Data<PgPool>,
    tv_api_client: web::Data<TvApiClient>,
    body: web::Json<VerifyCodeRequest>,
) -> ApiResult<HttpResponse> {
    // Verify the pairing code and create device session
    let (user, device_session) = device_pairing::verify_pairing_code(
        pool.get_ref(),
        tv_api_client.get_ref(),
        &body.code,
        body.device_name.clone(),
        body.device_fingerprint.clone(),
    )
    .await?;

    // Generate a JWT token for the desktop app
    // For now, we'll use a simple approach: we need to get the original token
    // In a production system, you'd want to generate a new token specifically for the device
    // TODO: Implement proper token generation for device sessions
    // For now, we'll return a placeholder that the client needs to handle

    // Note: The desktop app will need to implement its own token management
    // This is a simplified version - in production, you'd want to:
    // 1. Generate a new JWT token with device-specific claims
    // 2. Set appropriate expiration (e.g., longer for desktop devices)
    // 3. Include device_id in the token claims

    let response = VerifyCodeResponse {
        access_token: format!("device_session_{}", device_session.id),
        user: UserInfo {
            id: user.id,
            org_id: user.org_id,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
        },
        device_id: device_session.id,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/auth/device/sessions
/// Get all device sessions for the current user
/// Requires: Valid JWT
pub async fn get_sessions(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Get claims from request extensions
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get the user
    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Get all device sessions for the user
    let sessions = DeviceSession::get_by_user(pool.get_ref(), user.id).await?;

    let response: Vec<DeviceSessionResponse> = sessions
        .into_iter()
        .map(DeviceSessionResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// DELETE /api/auth/device/sessions/:id
/// Revoke a device session
/// Requires: Valid JWT
pub async fn revoke_session(
    pool: web::Data<PgPool>,
    session_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Get claims from request extensions
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get the user
    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Delete the device session (only if it belongs to the user)
    let deleted = DeviceSession::delete(pool.get_ref(), *session_id, user.id).await?;

    if !deleted {
        return Err(ApiError::NotFound("Device session not found".to_string()));
    }

    Ok(HttpResponse::NoContent().finish())
}
