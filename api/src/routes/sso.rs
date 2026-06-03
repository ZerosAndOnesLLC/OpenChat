use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};

use crate::errors::ApiError;

// JWKS structures for fetching public keys
#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: Option<String>,
    #[allow(dead_code)]
    kty: String,
    n: String,
    e: String,
}

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
    pub id_token: Option<String>,
}

// OpenID Connect ID Token claims
#[derive(Debug, Serialize, Deserialize)]
pub struct IDTokenClaims {
    pub sub: String,  // User ID
    pub email: Option<String>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub org_id: Option<String>,
    pub org_name: Option<String>,
    pub iss: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
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

    // Get OAuth configuration from environment
    let client_id = std::env::var("OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| "openchat-api".to_string());

    let client_secret = std::env::var("OAUTH_CLIENT_SECRET")
        .unwrap_or_else(|_| "web-ui-secret".to_string());

    let redirect_uri = std::env::var("OAUTH_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:3000/sso/callback".to_string());

    // OAuth 2.0 token endpoint requires application/x-www-form-urlencoded
    let form_params = [
        ("grant_type", "authorization_code"),
        ("code", &payload.code),
        ("client_id", &client_id),
        ("client_secret", &client_secret),
        ("redirect_uri", &redirect_uri),
    ];

    tracing::debug!("Token request: client_id={}, code={}", client_id, payload.code);

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

    // Decode and validate id_token if present (OpenID Connect with proper signature verification)
    let user_claims = if let Some(ref id_token) = token_response.id_token {
        tracing::debug!("Validating and decoding ID token with JWKS");

        match validate_id_token(id_token, &tv_api_url, &client_id).await {
            Ok(claims) => {
                tracing::info!("Successfully validated ID token for user: {}", claims.sub);
                Some(claims)
            }
            Err(e) => {
                tracing::warn!("Failed to validate ID token: {}, falling back to userinfo endpoint", e);
                None
            }
        }
    } else {
        tracing::debug!("No ID token in response, client will need to call /userinfo");
        None
    };

    // Return token response with decoded user claims
    #[derive(Serialize)]
    struct SSOResponse {
        access_token: String,
        token_type: String,
        expires_in: i64,
        refresh_token: Option<String>,
        id_token: Option<String>,
        user_claims: Option<IDTokenClaims>,
    }

    Ok(HttpResponse::Ok().json(SSOResponse {
        access_token: token_response.access_token,
        token_type: token_response.token_type,
        expires_in: token_response.expires_in,
        refresh_token: token_response.refresh_token,
        id_token: token_response.id_token,
        user_claims,
    }))
}

/// Validates an ID token by fetching JWKS and verifying the signature
async fn validate_id_token(
    id_token: &str,
    tv_api_url: &str,
    expected_audience: &str,
) -> Result<IDTokenClaims, ApiError> {
    // Decode the header to get the key ID (kid)
    let header = decode_header(id_token)
        .map_err(|e| ApiError::Internal(format!("Failed to decode JWT header: {}", e)))?;

    let kid = header.kid
        .ok_or_else(|| ApiError::Internal("ID token missing 'kid' in header".to_string()))?;

    // Fetch JWKS from TitaniumVault
    let jwks_url = format!("{}/.well-known/jwks.json", tv_api_url.trim_end_matches('/'));
    tracing::debug!("Fetching JWKS from: {}", jwks_url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| ApiError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let jwks: Jwks = client
        .get(&jwks_url)
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to fetch JWKS: {}", e)))?
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to parse JWKS: {}", e)))?;

    // Find the matching key
    let jwk = jwks
        .keys
        .iter()
        .find(|k| k.kid.as_ref() == Some(&kid))
        .ok_or_else(|| ApiError::Internal(format!("No matching key found for kid: {}", kid)))?;

    // Construct the RSA public key from n and e
    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|e| ApiError::Internal(format!("Failed to create decoding key: {}", e)))?;

    // Set up validation
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[expected_audience]); // Validate audience matches our client_id
    validation.set_issuer(&[tv_api_url]); // Validate issuer is TV API
    validation.validate_exp = true; // Validate expiration

    // Decode and validate the token
    let token_data = decode::<IDTokenClaims>(id_token, &decoding_key, &validation)
        .map_err(|e| ApiError::Internal(format!("Failed to validate ID token: {}", e)))?;

    Ok(token_data.claims)
}

/// POST /api/sso/userinfo
/// Proxies the /userinfo request to TitaniumVault to avoid CORS issues
pub async fn get_userinfo(
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Proxying userinfo request to TitaniumVault");

    let tv_api_url = std::env::var("TV_API_URL")
        .unwrap_or_else(|_| "https://api.titanium-vault.com".to_string());
    // Shared secret that lets TitaniumVault trust the client IP we assert below.
    let service_token = std::env::var("SERVICE_TOKEN").unwrap_or_default();

    // Extract the Authorization header from the incoming request
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(ApiError::Authentication("Missing Authorization header".to_string()))?;

    // The end user's connection terminates at OpenChat's load balancer, so by the time we call
    // TitaniumVault it would only see our egress IP. Forward the observed client IP (leftmost
    // X-Forwarded-For entry) so TV can log/rate-limit the real user; gated by the service token.
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|xff| xff.split(',').next())
        .map(|ip| ip.trim().to_string())
        .filter(|ip| !ip.is_empty());

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| ApiError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    // Call TitaniumVault's /userinfo endpoint
    let userinfo_endpoint = format!("{}/userinfo", tv_api_url.trim_end_matches('/'));
    tracing::debug!("Calling userinfo endpoint: {}", userinfo_endpoint);

    let mut request = client
        .get(&userinfo_endpoint)
        .header("Authorization", auth_header);
    if !service_token.is_empty() {
        request = request.header("X-Service-Token", service_token);
        if let Some(ip) = &client_ip {
            request = request.header("X-Original-Client-IP", ip);
        }
    }

    let response = request.send().await.map_err(|e| {
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
