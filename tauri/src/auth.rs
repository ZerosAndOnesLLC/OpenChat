use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;
use std::path::PathBuf;
use std::fs;
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc, Duration};

const KEYRING_SERVICE: &str = "openchat-desktop";
const KEYRING_USERNAME: &str = "auth-credentials";
const API_BASE_URL: &str = "https://openchat-api.zerosandones.us:9876";
const TOKEN_VALIDITY_DAYS: i64 = 365;
const CREDENTIALS_FILE: &str = "credentials.dat";

/// Gets the app data directory for storing credentials
fn get_credentials_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("OpenChat").join(CREDENTIALS_FILE))
}

/// Stores credentials to a file (fallback when keyring fails)
fn store_credentials_to_file(json_str: &str) -> Result<(), String> {
    let path = get_credentials_path()
        .ok_or_else(|| "Could not determine app data directory".to_string())?;

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    }

    // Encode as base64 for basic obfuscation
    let encoded = general_purpose::STANDARD.encode(json_str.as_bytes());

    fs::write(&path, encoded)
        .map_err(|e| format!("Failed to write credentials file: {}", e))?;

    log::info!("store_credentials_to_file: saved to {:?}", path);
    Ok(())
}

/// Reads credentials from file (fallback when keyring fails)
fn read_credentials_from_file() -> Option<String> {
    let path = get_credentials_path()?;

    if !path.exists() {
        log::info!("read_credentials_from_file: file does not exist at {:?}", path);
        return None;
    }

    let encoded = fs::read_to_string(&path).ok()?;
    let decoded = general_purpose::STANDARD.decode(encoded.trim()).ok()?;
    let json_str = String::from_utf8(decoded).ok()?;

    log::info!("read_credentials_from_file: successfully read from {:?}", path);
    Some(json_str)
}

/// Deletes the credentials file
fn delete_credentials_file() {
    if let Some(path) = get_credentials_path() {
        if path.exists() {
            let _ = fs::remove_file(&path);
            log::info!("delete_credentials_file: removed {:?}", path);
        }
    }
}

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
/// Checks in-memory cache first, then OS keychain, then file fallback.
#[tauri::command]
pub async fn get_stored_credentials(state: State<'_, AppState>) -> Result<Option<StoredCredentials>, String> {
    log::info!("get_stored_credentials: checking for stored credentials");

    // First check in-memory state
    let in_memory_result = {
        if let Ok(creds_guard) = state.credentials.lock() {
            if let Some(ref creds) = *creds_guard {
                if Utc::now() >= creds.expires_at {
                    log::info!("get_stored_credentials: in-memory credentials expired");
                    Some(Err("expired_memory"))
                } else {
                    log::info!("get_stored_credentials: found valid credentials in memory");
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
    log::info!("get_stored_credentials: checking OS keychain");
    if let Some(creds) = read_from_keychain() {
        if Utc::now() >= creds.expires_at {
            log::info!("get_stored_credentials: keychain credentials expired");
            clear_credentials_internal(&state)?;
            return Ok(None);
        }

        log::info!("get_stored_credentials: found valid credentials in keychain");
        // Update in-memory state
        if let Ok(mut creds_guard) = state.credentials.lock() {
            *creds_guard = Some(creds.clone());
        }
        return Ok(Some(creds));
    }

    // Finally check file fallback (Windows keychain can be unreliable)
    log::info!("get_stored_credentials: checking file fallback");
    if let Some(json_str) = read_credentials_from_file() {
        match serde_json::from_str::<StoredCredentials>(&json_str) {
            Ok(creds) => {
                if Utc::now() >= creds.expires_at {
                    log::info!("get_stored_credentials: file credentials expired");
                    clear_credentials_internal(&state)?;
                    return Ok(None);
                }

                log::info!("get_stored_credentials: found valid credentials in file fallback");
                // Update in-memory state
                if let Ok(mut creds_guard) = state.credentials.lock() {
                    *creds_guard = Some(creds.clone());
                }
                return Ok(Some(creds));
            }
            Err(e) => {
                log::error!("get_stored_credentials: failed to parse file credentials: {}", e);
            }
        }
    }

    log::info!("get_stored_credentials: no valid credentials found");
    Ok(None)
}

/// Internal function to clear credentials without requiring async
fn clear_credentials_internal(state: &State<'_, AppState>) -> Result<(), String> {
    log::info!("clear_credentials_internal: clearing all stored credentials");

    // Clear from OS keychain
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME) {
        match entry.delete_credential() {
            Ok(_) => log::info!("clear_credentials_internal: cleared keychain"),
            Err(keyring::Error::NoEntry) => log::info!("clear_credentials_internal: no keychain entry to clear"),
            Err(e) => log::warn!("clear_credentials_internal: failed to clear keychain: {}", e),
        }
    }

    // Clear file fallback
    delete_credentials_file();

    // Clear in-memory state
    if let Ok(mut creds_guard) = state.credentials.lock() {
        *creds_guard = None;
    }

    log::info!("clear_credentials_internal: all credentials cleared");
    Ok(())
}

/// Stores credentials securely with a 365-day expiry.
/// Uses OS keychain as primary storage with file-based fallback for Windows compatibility.
#[tauri::command]
pub async fn store_credentials(
    access_token: String,
    device_id: String,
    user: User,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("store_credentials: storing credentials for user {}", user.email);

    let credentials = StoredCredentials {
        access_token,
        device_id,
        user,
        expires_at: Utc::now() + Duration::days(TOKEN_VALIDITY_DAYS),
    };

    // Serialize to JSON
    let json_str = serde_json::to_string(&credentials)
        .map_err(|e| format!("Failed to serialize credentials: {}", e))?;

    // Try to store in OS keychain first
    let keychain_result = store_in_keychain(&json_str);

    // Always store to file as well (Windows keychain can be unreliable across restarts)
    if let Err(e) = store_credentials_to_file(&json_str) {
        log::warn!("store_credentials: file fallback failed: {}", e);
    }

    // Log keychain result but don't fail if file storage worked
    match keychain_result {
        Ok(_) => {
            log::info!("store_credentials: successfully stored in keychain, expires_at: {}", credentials.expires_at);
        }
        Err(e) => {
            log::warn!("store_credentials: keychain storage failed (using file fallback): {}", e);
        }
    }

    // Update in-memory state
    if let Ok(mut creds_guard) = state.credentials.lock() {
        *creds_guard = Some(credentials);
    }

    log::info!("store_credentials: updated in-memory cache");
    Ok(())
}

/// Helper to store in OS keychain
fn store_in_keychain(json_str: &str) -> Result<(), String> {
    log::info!("store_in_keychain: accessing keychain service '{}' with username '{}'", KEYRING_SERVICE, KEYRING_USERNAME);
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
        .map_err(|e| {
            log::error!("store_in_keychain: failed to access keychain: {}", e);
            format!("Failed to access keychain: {}", e)
        })?;

    entry
        .set_password(json_str)
        .map_err(|e| {
            log::error!("store_in_keychain: failed to store in keychain: {}", e);
            format!("Failed to store credentials: {}", e)
        })?;

    Ok(())
}

/// Clears stored credentials (logout).
#[tauri::command]
pub async fn clear_credentials(state: State<'_, AppState>) -> Result<(), String> {
    clear_credentials_internal(&state)
}

/// Gets just the stored token (for web UI compatibility)
/// Checks in-memory cache first, then OS keychain, then file fallback.
#[tauri::command]
pub async fn get_stored_token(state: State<'_, AppState>) -> Result<Option<String>, String> {
    log::info!("get_stored_token: checking for stored credentials");

    // First check in-memory state
    {
        if let Ok(creds_guard) = state.credentials.lock() {
            if let Some(ref creds) = *creds_guard {
                if chrono::Utc::now() < creds.expires_at {
                    log::info!("get_stored_token: found valid token in memory cache");
                    return Ok(Some(creds.access_token.clone()));
                } else {
                    log::info!("get_stored_token: in-memory token expired");
                }
            }
        }
    }

    // Then check OS keychain
    log::info!("get_stored_token: checking OS keychain");
    if let Some(creds) = read_from_keychain() {
        if chrono::Utc::now() < creds.expires_at {
            log::info!("get_stored_token: keychain token valid, expires_at: {}", creds.expires_at);
            // Update in-memory state
            if let Ok(mut creds_guard) = state.credentials.lock() {
                *creds_guard = Some(creds.clone());
            }
            return Ok(Some(creds.access_token));
        } else {
            log::info!("get_stored_token: keychain token expired at {}", creds.expires_at);
        }
    }

    // Finally check file fallback
    log::info!("get_stored_token: checking file fallback");
    if let Some(json_str) = read_credentials_from_file() {
        match serde_json::from_str::<StoredCredentials>(&json_str) {
            Ok(creds) => {
                if chrono::Utc::now() < creds.expires_at {
                    log::info!("get_stored_token: file token valid, expires_at: {}", creds.expires_at);
                    // Update in-memory state
                    if let Ok(mut creds_guard) = state.credentials.lock() {
                        *creds_guard = Some(creds.clone());
                    }
                    return Ok(Some(creds.access_token));
                } else {
                    log::info!("get_stored_token: file token expired at {}", creds.expires_at);
                }
            }
            Err(e) => {
                log::error!("get_stored_token: failed to parse file data: {}", e);
            }
        }
    }

    log::info!("get_stored_token: no valid credentials found");
    Ok(None)
}

/// Helper to read from OS keychain
fn read_from_keychain() -> Option<StoredCredentials> {
    match keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME) {
        Ok(entry) => match entry.get_password() {
            Ok(json_str) => {
                log::info!("read_from_keychain: found entry in keychain, parsing...");
                match serde_json::from_str::<StoredCredentials>(&json_str) {
                    Ok(creds) => Some(creds),
                    Err(e) => {
                        log::error!("read_from_keychain: failed to parse: {}", e);
                        None
                    }
                }
            }
            Err(keyring::Error::NoEntry) => {
                log::info!("read_from_keychain: no entry found");
                None
            }
            Err(e) => {
                log::error!("read_from_keychain: access error: {}", e);
                None
            }
        },
        Err(e) => {
            log::error!("read_from_keychain: failed to create entry: {}", e);
            None
        }
    }
}

/// Clears token (alias for clear_credentials for web UI compatibility)
#[tauri::command]
pub async fn clear_token(state: State<'_, AppState>) -> Result<(), String> {
    clear_credentials_internal(&state)
}

/// Token validation result
#[derive(Debug, Serialize)]
pub struct TokenValidationResult {
    /// Whether the token is valid
    pub valid: bool,
    /// Whether the validation was definitive (server responded) vs uncertain (network error)
    pub definitive: bool,
    /// Error message if any
    pub error: Option<String>,
}

#[tauri::command]
pub async fn validate_token(token: String) -> Result<TokenValidationResult, String> {
    log::info!("validate_token: validating token with API");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let url = format!("{}/api/sso/userinfo", API_BASE_URL);
    log::info!("validate_token: POST {}", url);

    match client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                log::info!("validate_token: token valid (status {})", status);
                Ok(TokenValidationResult {
                    valid: true,
                    definitive: true,
                    error: None,
                })
            } else if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
                // Definitive rejection - token is invalid
                log::info!("validate_token: server rejected token with status {}", status);
                Ok(TokenValidationResult {
                    valid: false,
                    definitive: true,
                    error: Some(format!("Token rejected: {}", status)),
                })
            } else {
                // Server error or other issue - not a definitive rejection
                let body = response.text().await.unwrap_or_default();
                log::warn!("validate_token: server returned status {}, treating as uncertain: {}", status, body);
                Ok(TokenValidationResult {
                    valid: false,
                    definitive: false,
                    error: Some(format!("Server error: {}", status)),
                })
            }
        }
        Err(e) => {
            // Network error - cannot determine token validity
            log::error!("validate_token: network error: {}", e);
            Ok(TokenValidationResult {
                valid: false,
                definitive: false,
                error: Some(format!("Network error: {}", e)),
            })
        }
    }
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
    let validation = validate_token(payload.token.clone()).await?;
    if !validation.valid {
        return Err(format!("Invalid token in deep link: {}", validation.error.unwrap_or_else(|| "unknown error".to_string())));
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
