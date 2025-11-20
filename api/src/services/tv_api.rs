use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::errors::{ApiError, ApiResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub user_id: Uuid,
    pub email: String,
    pub org_id: Uuid,
    pub org_name: String,
    pub display_name: String,
    pub roles: Vec<String>,
}

// JWT payload structure from TitaniumVault access tokens
#[derive(Debug, Deserialize)]
struct AccessTokenClaims {
    sub: String,           // User ID
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    nickname: Option<String>,     // User's display name/nickname
    #[serde(default)]
    name: Option<String>,         // User's full name
    org_id: Option<String>,
    org_name: Option<String>,
    #[serde(default)]
    roles: Option<Vec<String>>,  // User roles
    #[allow(dead_code)]
    exp: i64,
    #[allow(dead_code)]
    iat: i64,
    #[allow(dead_code)]
    iss: String,
    #[allow(dead_code)]
    aud: String,
}

// JWKS structures for fetching public keys
#[derive(Debug, Clone, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    kid: Option<String>,
    #[allow(dead_code)]
    kty: String,
    n: String,
    e: String,
}

#[allow(dead_code)]
pub struct TvApiClient {
    base_url: String,
    client: Client,
    jwks_cache: Arc<RwLock<Option<(Jwks, Instant)>>>,
    jwks_ttl: Duration,
}

#[allow(dead_code)]
impl TvApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
            jwks_cache: Arc::new(RwLock::new(None)),
            jwks_ttl: Duration::from_secs(3600), // Cache JWKS for 1 hour
        }
    }

    /// Verify a JWT access token by validating its signature with JWKS
    /// Returns the token claims if valid, error otherwise
    pub async fn verify_token(&self, token: &str) -> ApiResult<TokenClaims> {
        // Decode the header to get the key ID (kid)
        let header = decode_header(token)
            .map_err(|e| ApiError::Authentication(format!("Failed to decode JWT header: {}", e)))?;

        let kid = header.kid
            .ok_or_else(|| ApiError::Authentication("Access token missing 'kid' in header".to_string()))?;

        // Get JWKS (from cache or fetch)
        let jwks = self.get_jwks().await?;

        // Find the matching key
        let jwk = jwks
            .keys
            .iter()
            .find(|k| k.kid.as_ref() == Some(&kid))
            .ok_or_else(|| ApiError::Authentication(format!("No matching key found for kid: {}", kid)))?;

        // Construct the RSA public key from n and e
        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|e| ApiError::Authentication(format!("Failed to create decoding key: {}", e)))?;

        // Set up validation - we validate issuer but not audience (tokens may be for different apps)
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.base_url]);
        validation.validate_exp = true;
        validation.validate_aud = false; // Don't validate audience - allow tokens for any app

        // Decode and validate the token
        let token_data = decode::<AccessTokenClaims>(token, &decoding_key, &validation)
            .map_err(|e| ApiError::Authentication(format!("Failed to validate access token: {}", e)))?;

        let jwt_claims = token_data.claims;

        // Parse user_id from sub
        let user_id = Uuid::parse_str(&jwt_claims.sub)
            .map_err(|_| ApiError::Authentication("Invalid user ID in token".to_string()))?;

        // Parse org_id from token
        let org_id = jwt_claims.org_id
            .ok_or_else(|| ApiError::Authentication("Token missing org_id claim".to_string()))
            .and_then(|id| Uuid::parse_str(&id)
                .map_err(|_| ApiError::Authentication("Invalid org_id in token".to_string())))?;

        // Extract email - use email claim if available, otherwise use sub
        let email = jwt_claims.email
            .unwrap_or_else(|| format!("{}@unknown", jwt_claims.sub));

        // Extract roles from JWT (now included in token by TV API)
        let roles: Vec<String> = jwt_claims.roles
            .unwrap_or_default();

        // Use org_name from token, or default to org_id as string
        let org_name = jwt_claims.org_name.clone()
            .unwrap_or_else(|| format!("org-{}", org_id));

        // Use nickname first, then name, then email prefix as display_name
        let display_name = jwt_claims.nickname
            .or(jwt_claims.name)
            .unwrap_or_else(|| email.split('@').next().unwrap_or("User").to_string());

        let claims = TokenClaims {
            user_id,
            email,
            org_id,
            org_name,
            display_name,
            roles,
        };

        Ok(claims)
    }

    /// Get JWKS from cache or fetch from TitaniumVault
    async fn get_jwks(&self) -> ApiResult<Jwks> {
        // Check cache first
        {
            let cache = self.jwks_cache.read().await;
            if let Some((jwks, cached_at)) = cache.as_ref() {
                if cached_at.elapsed() < self.jwks_ttl {
                    tracing::debug!("JWKS cache hit");
                    return Ok(jwks.clone());
                }
            }
        }

        // Cache miss or expired, fetch from TitaniumVault
        tracing::debug!("JWKS cache miss, fetching from TitaniumVault");
        let jwks = self.fetch_jwks().await?;

        // Update cache
        {
            let mut cache = self.jwks_cache.write().await;
            *cache = Some((jwks.clone(), Instant::now()));
        }

        Ok(jwks)
    }

    /// Fetch JWKS from TitaniumVault
    async fn fetch_jwks(&self) -> ApiResult<Jwks> {
        let jwks_url = format!("{}/.well-known/jwks.json", self.base_url.trim_end_matches('/'));

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| ApiError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        let jwks: Jwks = client
            .get(&jwks_url)
            .send()
            .await
            .map_err(|e| ApiError::Authentication(format!("Failed to fetch JWKS: {}", e)))?
            .json()
            .await
            .map_err(|e| ApiError::Authentication(format!("Failed to parse JWKS: {}", e)))?;

        Ok(jwks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires TV-API to be running
    async fn test_verify_token() {
        let client = TvApiClient::new("https://api.titanium-vault.com".to_string());
        let result = client.verify_token("invalid_token").await;
        assert!(result.is_err());
    }
}
