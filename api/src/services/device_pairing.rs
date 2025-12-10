use chrono::{Duration, Utc};
use rand::{thread_rng, Rng};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{ApiError, ApiResult};
use crate::models::device::{DevicePairingCode, DeviceSession};
use crate::models::user::User;
use crate::services::tv_api::TvApiClient;

const CODE_LENGTH: usize = 6;
const CODE_EXPIRY_MINUTES: i64 = 5;
const MAX_RETRY_ATTEMPTS: usize = 5;

// Characters to use for pairing codes (excluding ambiguous characters like 0/O, 1/I/l)
const CODE_CHARS: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";

/// Generate a random pairing code
pub fn generate_code() -> String {
    let mut rng = thread_rng();
    (0..CODE_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CODE_CHARS.len());
            CODE_CHARS[idx] as char
        })
        .collect()
}

/// Generate a unique pairing code for device authentication
/// Returns the code and its expiration timestamp
pub async fn generate_pairing_code(
    pool: &PgPool,
    user_id: Uuid,
    org_id: Uuid,
    roles: Vec<String>,
) -> ApiResult<DevicePairingCode> {
    let expires_at = Utc::now() + Duration::minutes(CODE_EXPIRY_MINUTES);

    // Try to generate a unique code (retry if collision occurs)
    for _ in 0..MAX_RETRY_ATTEMPTS {
        let code = generate_code();

        // Check if code already exists
        if DevicePairingCode::get_by_code(pool, &code)
            .await?
            .is_some()
        {
            continue; // Try another code
        }

        // Create the pairing code
        let pairing_code =
            DevicePairingCode::create(pool, code, user_id, org_id, expires_at, roles.clone()).await?;

        return Ok(pairing_code);
    }

    Err(ApiError::Internal(
        "Failed to generate unique pairing code".to_string(),
    ))
}

/// Verify a pairing code and create a device session
/// Returns the user info, device session, and roles from the pairing code
pub async fn verify_pairing_code(
    pool: &PgPool,
    _tv_api_client: &TvApiClient,
    code: &str,
    device_name: Option<String>,
    device_fingerprint: Option<String>,
) -> ApiResult<(User, DeviceSession, Vec<String>)> {
    // Normalize code to uppercase
    let code = code.to_uppercase();

    // Get the pairing code
    let pairing_code = DevicePairingCode::get_by_code(pool, &code)
        .await?
        .ok_or_else(|| ApiError::BadRequest("Invalid pairing code".to_string()))?;

    // Validate the code
    if !pairing_code.is_valid() {
        if pairing_code.used {
            return Err(ApiError::BadRequest(
                "Pairing code has already been used".to_string(),
            ));
        } else {
            return Err(ApiError::BadRequest(
                "Pairing code has expired".to_string(),
            ));
        }
    }

    // Mark the code as used
    DevicePairingCode::mark_as_used(pool, pairing_code.id).await?;

    // Get the user
    let user = User::get_by_id(pool, pairing_code.user_id)
        .await?
        .ok_or_else(|| ApiError::Internal("User not found".to_string()))?;

    // Create a device session
    let device_session = DeviceSession::create(
        pool,
        pairing_code.user_id,
        pairing_code.org_id,
        "desktop".to_string(),
        device_name,
        device_fingerprint,
    )
    .await?;

    Ok((user, device_session, pairing_code.roles))
}

/// Cleanup expired pairing codes
/// This should be run periodically (e.g., every 5 minutes)
#[allow(dead_code)]
pub async fn cleanup_expired_codes(pool: &PgPool) -> ApiResult<u64> {
    DevicePairingCode::delete_expired(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code() {
        let code = generate_code();
        assert_eq!(code.len(), CODE_LENGTH);

        // Check that all characters are from our allowed set
        for c in code.chars() {
            assert!(CODE_CHARS.contains(&(c as u8)));
        }
    }

    #[test]
    fn test_generate_code_uniqueness() {
        // Generate multiple codes and check they're not all the same
        let codes: Vec<String> = (0..100).map(|_| generate_code()).collect();
        let unique_codes: std::collections::HashSet<_> = codes.iter().collect();

        // With 32^6 possibilities, we should have very high uniqueness
        assert!(unique_codes.len() > 95);
    }
}
