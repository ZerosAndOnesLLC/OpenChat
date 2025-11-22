use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;
use base64::{Engine as _, engine::general_purpose};

const KEYRING_SERVICE: &str = "openchat-desktop";
const KEYRING_USERNAME: &str = "auth-token";
const API_BASE_URL: &str = "https://api.openchat.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub user: User,
    pub device_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub org_id: String,
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
    pub current_token: Mutex<Option<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_token: Mutex::new(None),
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

    store_token(auth_response.access_token.clone(), state).await?;

    Ok(auth_response)
}

#[tauri::command]
pub async fn get_stored_token(state: State<'_, AppState>) -> Result<Option<String>, String> {
    // First check in-memory state
    if let Ok(token_guard) = state.current_token.lock() {
        if token_guard.is_some() {
            return Ok(token_guard.clone());
        }
    }

    // Then check OS keychain
    match keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME) {
        Ok(entry) => match entry.get_password() {
            Ok(token) => {
                // Update in-memory state
                if let Ok(mut token_guard) = state.current_token.lock() {
                    *token_guard = Some(token.clone());
                }
                Ok(Some(token))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(format!("Keychain error: {}", e)),
        },
        Err(e) => Err(format!("Failed to access keychain: {}", e)),
    }
}

#[tauri::command]
pub async fn store_token(token: String, state: State<'_, AppState>) -> Result<(), String> {
    // Store in OS keychain
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
        .map_err(|e| format!("Failed to access keychain: {}", e))?;

    entry
        .set_password(&token)
        .map_err(|e| format!("Failed to store token: {}", e))?;

    // Update in-memory state
    if let Ok(mut token_guard) = state.current_token.lock() {
        *token_guard = Some(token);
    }

    Ok(())
}

#[tauri::command]
pub async fn clear_token(state: State<'_, AppState>) -> Result<(), String> {
    // Clear from OS keychain
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
        .map_err(|e| format!("Failed to access keychain: {}", e))?;

    match entry.delete_credential() {
        Ok(_) => {}
        Err(keyring::Error::NoEntry) => {} // Already cleared
        Err(e) => return Err(format!("Failed to clear token: {}", e)),
    }

    // Clear in-memory state
    if let Ok(mut token_guard) = state.current_token.lock() {
        *token_guard = None;
    }

    Ok(())
}

#[tauri::command]
pub async fn validate_token(token: String) -> Result<bool, String> {
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/api/auth/me", API_BASE_URL))
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
    let now = chrono::Utc::now().timestamp();
    if payload.exp < now {
        return Err("Deep link has expired".to_string());
    }

    // Validate the token with backend
    let is_valid = validate_token(payload.token.clone()).await?;
    if !is_valid {
        return Err("Invalid token in deep link".to_string());
    }

    // Store the token
    store_token(payload.token.clone(), state).await?;

    // Fetch user info
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/api/auth/me", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", payload.token))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user info: {}", e))?;

    let user: User = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse user info: {}", e))?;

    Ok(AuthResponse {
        access_token: payload.token,
        user,
        device_id: uuid::Uuid::new_v4().to_string(),
    })
}
