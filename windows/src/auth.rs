use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc, Duration};

const KEYRING_SERVICE: &str = "openchat-desktop";
const KEYRING_USERNAME: &str = "auth-credentials";
const API_BASE_URL: &str = "https://openchat-api.zerosandones.us:9876";
const TOKEN_VALIDITY_DAYS: i64 = 365;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub user: User,
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub org_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredentials {
    pub access_token: String,
    pub device_id: String,
    pub user: User,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifyCodeRequest {
    code: String,
    device_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeepLinkPayload {
    token: String,
    user_id: String,
    org_id: String,
    exp: i64,
}

pub struct AppState {
    pub credentials: Mutex<Option<StoredCredentials>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            credentials: Mutex::new(None),
        }
    }
}

#[tauri::command]
pub async fn verify_pairing_code(
    code: String,
    device_name: String,
    state: State<'_, AppState>,
) -> Result<AuthResponse, String> {
    let client = reqwest::Client::new();

    let request_body = VerifyCodeRequest {
        code: code.clone(),
        device_name,
    };

    let response = client
        .post(&format!("{}/api/auth/device/verify-code", API_BASE_URL))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Verification failed ({}): {}", status, error_text));
    }

    let auth_response: AuthResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Store full credentials with 365-day expiry
    store_credentials(
        auth_response.access_token.clone(),
        auth_response.device_id.clone(),
        auth_response.user.clone(),
        state,
    ).await?;

    Ok(auth_response)
}

/// Retrieves stored credentials if they exist and haven't expired.
/// Returns None if no credentials exist or if they have expired (365-day expiry).
#[tauri::command]
pub async fn get_stored_credentials(state: State<'_, AppState>) -> Result<Option<StoredCredentials>, String> {
    // First check in-memory state
    let in_memory_result = {
        if let Ok(creds_guard) = state.credentials.lock() {
            if let Some(ref creds) = *creds_guard {
                if Utc::now() >= creds.expires_at {
                    Some(Err("expired_memory"))
                } else {
                    Some(Ok(creds.clone()))
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    match in_memory_result {
        Some(Ok(creds)) => return Ok(Some(creds)),
        Some(Err(_)) => {
            // Credentials expired in memory, clear them
            clear_credentials_internal(&state)?;
            return Ok(None);
        }
        None => {}
    }

    // Then check OS keychain
    match keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME) {
        Ok(entry) => match entry.get_password() {
            Ok(json_str) => {
                // Parse stored credentials
                let creds: StoredCredentials = serde_json::from_str(&json_str)
                    .map_err(|e| format!("Failed to parse stored credentials: {}", e))?;

                // Check if credentials have expired
                if Utc::now() >= creds.expires_at {
                    clear_credentials_internal(&state)?;
                    return Ok(None);
                }

                // Update in-memory state
                if let Ok(mut creds_guard) = state.credentials.lock() {
                    *creds_guard = Some(creds.clone());
                }
                Ok(Some(creds))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(format!("Keychain error: {}", e)),
        },
        Err(e) => Err(format!("Failed to access keychain: {}", e)),
    }
}

/// Internal function to clear credentials without requiring async
fn clear_credentials_internal(state: &State<'_, AppState>) -> Result<(), String> {
    // Clear from OS keychain
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
        .map_err(|e| format!("Failed to access keychain: {}", e))?;

    match entry.delete_credential() {
        Ok(_) => {}
        Err(keyring::Error::NoEntry) => {}
        Err(e) => return Err(format!("Failed to clear credentials: {}", e)),
    }

    // Clear in-memory state
    if let Ok(mut creds_guard) = state.credentials.lock() {
        *creds_guard = None;
    }

    Ok(())
}

/// Stores credentials securely with a 365-day expiry.
#[tauri::command]
pub async fn store_credentials(
    access_token: String,
    device_id: String,
    user: User,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let credentials = StoredCredentials {
        access_token,
        device_id,
        user,
        expires_at: Utc::now() + Duration::days(TOKEN_VALIDITY_DAYS),
    };

    // Serialize to JSON
    let json_str = serde_json::to_string(&credentials)
        .map_err(|e| format!("Failed to serialize credentials: {}", e))?;

    // Store in OS keychain
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
        .map_err(|e| format!("Failed to access keychain: {}", e))?;

    entry
        .set_password(&json_str)
        .map_err(|e| format!("Failed to store credentials: {}", e))?;

    // Update in-memory state
    if let Ok(mut creds_guard) = state.credentials.lock() {
        *creds_guard = Some(credentials);
    }

    Ok(())
}

/// Clears stored credentials (logout).
#[tauri::command]
pub async fn clear_credentials(state: State<'_, AppState>) -> Result<(), String> {
    clear_credentials_internal(&state)
}

#[tauri::command]
pub async fn validate_token(token: String) -> Result<bool, String> {
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/api/sso/userinfo", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    Ok(response.status().is_success())
}

#[tauri::command]
pub async fn process_deep_link_payload(
    encrypted_payload: String,
    state: State<'_, AppState>,
) -> Result<AuthResponse, String> {
    // Decode base64 payload
    let decoded = general_purpose::STANDARD.decode(&encrypted_payload)
        .map_err(|e| format!("Failed to decode payload: {}", e))?;

    let payload_str = String::from_utf8(decoded)
        .map_err(|e| format!("Invalid payload format: {}", e))?;

    let payload: DeepLinkPayload = serde_json::from_str(&payload_str)
        .map_err(|e| format!("Failed to parse payload: {}", e))?;

    // Check expiration
    let now = Utc::now().timestamp();
    if payload.exp < now {
        return Err("Deep link has expired".to_string());
    }

    // Validate the token with backend
    let is_valid = validate_token(payload.token.clone()).await?;
    if !is_valid {
        return Err("Invalid token in deep link".to_string());
    }

    // Fetch user info
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/sso/userinfo", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", payload.token))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user info: {}", e))?;

    let user_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse user info: {}", e))?;

    // Extract user info from the response
    let user = User {
        id: user_json.get("sub")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'sub' in userinfo")?.to_string(),
        email: user_json.get("email")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'email' in userinfo")?.to_string(),
        name: user_json.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        org_id: user_json.get("org_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'org_id' in userinfo")?.to_string(),
    };

    let device_id = uuid::Uuid::new_v4().to_string();

    // Store full credentials with 365-day expiry
    store_credentials(
        payload.token.clone(),
        device_id.clone(),
        user.clone(),
        state,
    ).await?;

    Ok(AuthResponse {
        access_token: payload.token,
        user,
        device_id,
    })
}
