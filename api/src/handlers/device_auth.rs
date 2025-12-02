use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    cache::rate_limit::{check_rate_limit, check_rate_limit_by_ip, RateLimitType},
    config::Config,
    errors::{ApiError, ApiResult},
    models::{
        device::{DeviceSession},
        user::User,
    },
    services::{
        device_pairing,
        device_token::generate_device_token,
        tv_api::{TokenClaims, TvApiClient}
    },
    utils::crypto::{derive_key_from_secret, encrypt},
};

/// Extract IP address from request, checking X-Forwarded-For header first (for ALB)
fn get_client_ip(req: &HttpRequest) -> String {
    // Check X-Forwarded-For header first (for ALB)
    if let Some(forwarded_for) = req.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            // Take the first IP in the chain
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    // Fallback to peer address
    req.peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

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
    /// API base URL for the desktop client to use for future requests
    pub api_url: String,
    /// Web UI URL for the desktop client to load after login
    pub webui_url: String,
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

#[derive(Debug, Serialize, Deserialize)]
struct DeepLinkPayload {
    /// JWT token for authentication
    token: String,
    /// User ID
    user_id: Uuid,
    /// Organization ID
    org_id: Uuid,
    /// Timestamp when payload was created
    created_at: i64,
    /// API base URL for the desktop client to use
    api_url: String,
    /// Web UI URL for the desktop client to load
    webui_url: String,
}

#[derive(Debug, Serialize)]
pub struct GenerateDeepLinkResponse {
    /// Encrypted payload (base64-encoded)
    pub payload: String,
    /// Full deep link URL
    pub deep_link: String,
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
    redis: web::Data<MultiplexedConnection>,
    _config: web::Data<Config>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Get claims from request extensions (set by auth middleware)
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Check rate limit (3 requests per minute per user)
    let mut redis_conn = redis.as_ref().clone();
    let (allowed, remaining, reset_time) = check_rate_limit(
        &mut redis_conn,
        claims.user_id,
        RateLimitType::DevicePairingGenerate,
    )
    .await?;

    if !allowed {
        return Err(ApiError::TooManyRequests(format!(
            "Rate limit exceeded. Please try again in {} seconds.",
            reset_time
        )));
    }

    // Get the user
    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Generate pairing code
    let pairing_code =
        device_pairing::generate_pairing_code(pool.get_ref(), user.id, claims.org_id).await?;

    // Calculate seconds until expiration
    let expires_in = (pairing_code.expires_at - chrono::Utc::now()).num_seconds();

    let mut response = HttpResponse::Ok();
    response.insert_header(("X-RateLimit-Limit", "3"));
    response.insert_header(("X-RateLimit-Remaining", remaining.to_string()));
    response.insert_header(("X-RateLimit-Reset", reset_time.to_string()));

    Ok(response.json(GenerateCodeResponse {
        code: pairing_code.code,
        expires_in,
    }))
}

/// POST /api/auth/device/verify-code
/// Verify a pairing code and create a device session
/// Public endpoint (no authentication required)
pub async fn verify_code(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    tv_api_client: web::Data<Arc<TvApiClient>>,
    config: web::Data<Config>,
    body: web::Json<VerifyCodeRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Check rate limit by IP address (5 requests per minute per IP)
    let client_ip = get_client_ip(&req);
    let mut redis_conn = redis.as_ref().clone();
    let (allowed, remaining, reset_time) = check_rate_limit_by_ip(
        &mut redis_conn,
        &client_ip,
        RateLimitType::DevicePairingVerify,
    )
    .await?;

    if !allowed {
        return Err(ApiError::TooManyRequests(format!(
            "Too many verification attempts. Please try again in {} seconds.",
            reset_time
        )));
    }

    // Verify the pairing code and create device session
    let (user, device_session) = device_pairing::verify_pairing_code(
        pool.get_ref(),
        tv_api_client.get_ref(),
        &body.code,
        body.device_name.clone(),
        body.device_fingerprint.clone(),
    )
    .await?;

    // Generate a JWT token for the device session
    // This token includes device-specific claims and has a 30-day expiration
    let access_token = generate_device_token(
        user.tv_user_id,
        user.id,
        user.org_id,
        device_session.id,
        device_session.device_type.clone(),
        &config.jwt_secret,
    )?;

    let response = VerifyCodeResponse {
        access_token,
        user: UserInfo {
            id: user.id,
            org_id: user.org_id,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
        },
        device_id: device_session.id,
        api_url: config.api_base_url.clone(),
        webui_url: config.webui_url.clone(),
    };

    let mut response_builder = HttpResponse::Ok();
    response_builder.insert_header(("X-RateLimit-Limit", "5"));
    response_builder.insert_header(("X-RateLimit-Remaining", remaining.to_string()));
    response_builder.insert_header(("X-RateLimit-Reset", reset_time.to_string()));

    Ok(response_builder.json(response))
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

/// POST /api/auth/device/generate-deep-link
/// Generate an encrypted deep link for desktop app login
/// Requires: Valid JWT from web user
pub async fn generate_deep_link(
    config: web::Data<Config>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Get token from Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::Authentication("Missing authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Authentication("Invalid authorization format".to_string()))?
        .to_string();

    // Get claims from request extensions
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Create payload
    let payload = DeepLinkPayload {
        token,
        user_id: claims.user_id,
        org_id: claims.org_id,
        created_at: chrono::Utc::now().timestamp(),
        api_url: config.api_base_url.clone(),
        webui_url: config.webui_url.clone(),
    };

    // Serialize payload
    let payload_json = serde_json::to_string(&payload)
        .map_err(|e| ApiError::Internal(format!("Failed to serialize payload: {}", e)))?;

    // Encrypt payload
    let encryption_key = derive_key_from_secret(&config.encryption_secret);
    let encrypted = encrypt(payload_json.as_bytes(), &encryption_key)?;

    // Encode as base64 URL-safe string
    let encoded_payload = encrypted.encode()?;

    // Generate deep link URL
    let deep_link = format!("openchat://login?payload={}", encoded_payload);

    Ok(HttpResponse::Ok().json(GenerateDeepLinkResponse {
        payload: encoded_payload,
        deep_link,
    }))
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
