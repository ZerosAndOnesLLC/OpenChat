use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::{ApiError, ApiResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TokenClaims {
    pub user_id: Uuid,
    pub email: String,
    pub org_id: Uuid,
    pub display_name: String,
    pub roles: Vec<String>,
}

#[allow(dead_code)]
pub struct TvApiClient {
    base_url: String,
    client: Client,
}

#[allow(dead_code)]
impl TvApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    /// Verify a JWT token with the TitaniumVault API
    /// Returns the token claims if valid, error otherwise
    pub async fn verify_token(&self, token: &str) -> ApiResult<TokenClaims> {
        let url = format!("{}/api/auth/verify", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ApiError::Authentication(format!(
                "Token verification failed: {}",
                response.status()
            )));
        }

        let claims = response.json::<TokenClaims>().await?;
        Ok(claims)
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
