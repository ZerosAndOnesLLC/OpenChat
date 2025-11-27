use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::{ApiError, ApiResult};

/// Device-specific JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceTokenClaims {
    /// Subject (user ID from TitaniumVault)
    pub sub: Uuid,
    /// User ID (OpenChat internal)
    pub user_id: Uuid,
    /// Organization ID
    pub org_id: Uuid,
    /// Device session ID
    pub device_id: Uuid,
    /// Device type
    pub device_type: String,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration (Unix timestamp) - 30 days for desktop devices
    pub exp: i64,
    /// Issuer
    pub iss: String,
}

impl DeviceTokenClaims {
    /// Create new device token claims
    pub fn new(
        tv_user_id: Uuid,
        user_id: Uuid,
        org_id: Uuid,
        device_id: Uuid,
        device_type: String,
    ) -> Self {
        let now = Utc::now();
        let exp = now + Duration::days(365); // 365 days expiration for device sessions (like Slack/Mattermost)

        Self {
            sub: tv_user_id,
            user_id,
            org_id,
            device_id,
            device_type,
            iat: now.timestamp(),
            exp: exp.timestamp(),
            iss: "openchat-api".to_string(),
        }
    }
}

/// Generate a JWT token for a device session
pub fn generate_device_token(
    tv_user_id: Uuid,
    user_id: Uuid,
    org_id: Uuid,
    device_id: Uuid,
    device_type: String,
    jwt_secret: &str,
) -> ApiResult<String> {
    let claims = DeviceTokenClaims::new(tv_user_id, user_id, org_id, device_id, device_type);

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(format!("Failed to generate device token: {}", e)))?;

    Ok(token)
}

/// Verify and decode a device token
pub fn verify_device_token(token: &str, jwt_secret: &str) -> ApiResult<DeviceTokenClaims> {
    let validation = Validation::default();

    let token_data = decode::<DeviceTokenClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|e| ApiError::Authentication(format!("Invalid device token: {}", e)))?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_device_token() {
        let secret = "test-secret-key";
        let tv_user_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();
        let device_type = "desktop".to_string();

        // Generate token
        let token = generate_device_token(
            tv_user_id,
            user_id,
            org_id,
            device_id,
            device_type.clone(),
            secret,
        )
        .unwrap();

        // Verify token
        let claims = verify_device_token(&token, secret).unwrap();

        assert_eq!(claims.sub, tv_user_id);
        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.org_id, org_id);
        assert_eq!(claims.device_id, device_id);
        assert_eq!(claims.device_type, device_type);
        assert_eq!(claims.iss, "openchat-api");
    }

    #[test]
    fn test_verify_with_wrong_secret() {
        let secret = "test-secret-key";
        let wrong_secret = "wrong-secret";
        let tv_user_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();

        let token = generate_device_token(
            tv_user_id,
            user_id,
            org_id,
            device_id,
            "desktop".to_string(),
            secret,
        )
        .unwrap();

        let result = verify_device_token(&token, wrong_secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_expiration() {
        let claims = DeviceTokenClaims::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "desktop".to_string(),
        );

        // Expiration should be 365 days from now
        let now = Utc::now().timestamp();
        let expected_exp = now + (365 * 24 * 60 * 60); // 365 days in seconds

        // Allow 5 second tolerance for test execution time
        assert!((claims.exp - expected_exp).abs() <= 5);
    }
}
