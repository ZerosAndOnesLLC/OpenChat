use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::errors::{ApiError, ApiResult};

const NONCE_SIZE: usize = 12; // 96 bits for GCM

/// Encrypted payload structure
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedPayload {
    /// Base64-encoded nonce
    pub nonce: String,
    /// Base64-encoded ciphertext
    pub ciphertext: String,
}

impl EncryptedPayload {
    /// Encode as base64 URL-safe string
    pub fn encode(&self) -> ApiResult<String> {
        let json = serde_json::to_string(self)
            .map_err(|e| ApiError::Internal(format!("Failed to serialize payload: {}", e)))?;
        Ok(URL_SAFE_NO_PAD.encode(json.as_bytes()))
    }

    /// Decode from base64 URL-safe string
    #[allow(dead_code)]
    pub fn decode(encoded: &str) -> ApiResult<Self> {
        let json_bytes = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|e| ApiError::BadRequest(format!("Invalid payload encoding: {}", e)))?;
        let json_str = String::from_utf8(json_bytes)
            .map_err(|e| ApiError::BadRequest(format!("Invalid UTF-8 in payload: {}", e)))?;
        serde_json::from_str(&json_str)
            .map_err(|e| ApiError::BadRequest(format!("Invalid payload format: {}", e)))
    }
}

/// Encrypt data using AES-256-GCM
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> ApiResult<EncryptedPayload> {
    let cipher = Aes256Gcm::new(key.into());

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| ApiError::Internal(format!("Encryption failed: {}", e)))?;

    Ok(EncryptedPayload {
        nonce: URL_SAFE_NO_PAD.encode(nonce_bytes),
        ciphertext: URL_SAFE_NO_PAD.encode(ciphertext),
    })
}

/// Decrypt data using AES-256-GCM
#[allow(dead_code)]
pub fn decrypt(payload: &EncryptedPayload, key: &[u8; 32]) -> ApiResult<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());

    // Decode nonce
    let nonce_bytes = URL_SAFE_NO_PAD
        .decode(&payload.nonce)
        .map_err(|e| ApiError::BadRequest(format!("Invalid nonce encoding: {}", e)))?;
    if nonce_bytes.len() != NONCE_SIZE {
        return Err(ApiError::BadRequest(format!(
            "Invalid nonce size: expected {}, got {}",
            NONCE_SIZE,
            nonce_bytes.len()
        )));
    }
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Decode ciphertext
    let ciphertext = URL_SAFE_NO_PAD
        .decode(&payload.ciphertext)
        .map_err(|e| ApiError::BadRequest(format!("Invalid ciphertext encoding: {}", e)))?;

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| ApiError::BadRequest(format!("Decryption failed: {}", e)))?;

    Ok(plaintext)
}

/// Generate a random 256-bit key
#[allow(dead_code)]
pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

/// Derive a key from a shared secret (e.g., environment variable)
/// Uses SHA-256 to ensure correct key size
pub fn derive_key_from_secret(secret: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let result = hasher.finalize();
    result.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key();
        let plaintext = b"Hello, OpenChat!";

        // Encrypt
        let encrypted = encrypt(plaintext, &key).unwrap();

        // Decrypt
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_with_derived_key() {
        let secret = "test-secret-key-12345";
        let key = derive_key_from_secret(secret);
        let plaintext = b"Sensitive data";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_payload_encode_decode() {
        let payload = EncryptedPayload {
            nonce: "test-nonce".to_string(),
            ciphertext: "test-ciphertext".to_string(),
        };

        let encoded = payload.encode().unwrap();
        let decoded = EncryptedPayload::decode(&encoded).unwrap();

        assert_eq!(payload.nonce, decoded.nonce);
        assert_eq!(payload.ciphertext, decoded.ciphertext);
    }

    #[test]
    fn test_decrypt_with_wrong_key() {
        let key1 = generate_key();
        let key2 = generate_key();
        let plaintext = b"Secret message";

        let encrypted = encrypt(plaintext, &key1).unwrap();
        let result = decrypt(&encrypted, &key2);

        assert!(result.is_err());
    }

    #[test]
    fn test_derived_key_deterministic() {
        let secret = "my-secret";
        let key1 = derive_key_from_secret(secret);
        let key2 = derive_key_from_secret(secret);

        assert_eq!(key1, key2);
    }
}
