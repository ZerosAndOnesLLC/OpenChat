use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::errors::ApiError;

#[derive(Debug, Deserialize)]
pub struct ExchangeCodeRequest {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
}

/// POST /api/sso/exchange
/// Exchanges an authorization code for an access token
pub async fn exchange_code(
    payload: web::Json<ExchangeCodeRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Exchanging authorization code for access token");

    let tv_api_url = std::env::var("TV_API_URL")
        .unwrap_or_else(|_| "https://api.titanium-vault.com".to_string());

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| ApiError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    // Call TitaniumVault's /oauth/token endpoint
    let token_endpoint = format!("{}/oauth/token", tv_api_url.trim_end_matches('/'));
    tracing::debug!("Calling token endpoint: {}", token_endpoint);

    // Get the client secret from environment
    let client_secret = std::env::var("OAUTH_CLIENT_SECRET")
        .unwrap_or_else(|_| "web-ui-secret".to_string());

    let redirect_uri = std::env::var("OAUTH_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:3000/sso/callback".to_string());

    // OAuth 2.0 token endpoint requires application/x-www-form-urlencoded
    let form_params = [
        ("grant_type", "authorization_code"),
        ("code", &payload.code),
        ("client_id", "openchat-ui"),
        ("client_secret", &client_secret),
        ("redirect_uri", &redirect_uri),
    ];

    tracing::debug!("Token request: client_id=openchat-ui, code={}", payload.code);

    let response = client
        .post(&token_endpoint)
        .form(&form_params)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Network error during token exchange: {}", e);
            ApiError::Internal(format!("Failed to exchange code: {}", e))
        })?;

    let status = response.status();
    tracing::debug!("Token exchange response status: {}", status);

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Token exchange failed with status {}: {}", status, error_text);
        return Err(ApiError::BadRequest(format!("Token exchange failed: {}", error_text)));
    }

    let token_response: TokenResponse = response
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to parse token response: {}", e)))?;

    tracing::info!("Successfully exchanged authorization code for access token");

    Ok(HttpResponse::Ok().json(token_response))
}

/// POST /api/sso/userinfo
/// Proxies the /userinfo request to TitaniumVault to avoid CORS issues
pub async fn get_userinfo(
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Proxying userinfo request to TitaniumVault");

    let tv_api_url = std::env::var("TV_API_URL")
        .unwrap_or_else(|_| "https://api.titanium-vault.com".to_string());

    // Extract the Authorization header from the incoming request
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(ApiError::Authentication("Missing Authorization header".to_string()))?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| ApiError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    // Call TitaniumVault's /userinfo endpoint
    let userinfo_endpoint = format!("{}/userinfo", tv_api_url.trim_end_matches('/'));
    tracing::debug!("Calling userinfo endpoint: {}", userinfo_endpoint);

    let response = client
        .get(&userinfo_endpoint)
        .header("Authorization", auth_header)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Network error during userinfo request: {}", e);
            ApiError::Internal(format!("Failed to fetch userinfo: {}", e))
        })?;

    let status = response.status();
    tracing::debug!("Userinfo response status: {}", status);

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Userinfo request failed with status {}: {}", status, error_text);
        return Err(ApiError::BadRequest(format!("Userinfo request failed: {}", error_text)));
    }

    let userinfo_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to parse userinfo response: {}", e)))?;

    tracing::info!("Successfully fetched userinfo from TitaniumVault");

    Ok(HttpResponse::Ok().json(userinfo_data))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/sso")
            .route("/exchange", web::post().to(exchange_code))
            .route("/userinfo", web::post().to(get_userinfo))
    );
}
