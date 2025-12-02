use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomEmoji {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub image_url: Option<String>,
    pub storage_type: String,
    pub storage_path: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmojiUploadResponse {
    pub id: Uuid,
    pub name: String,
    pub image_url: String,
    pub storage_type: String,
    pub created_at: DateTime<Utc>,
}

// Emoji validation constants
pub const MAX_EMOJI_SIZE: i64 = 512 * 1024; // 512KB
#[allow(dead_code)]
pub const EMOJI_DIMENSIONS: u32 = 128; // 128x128px
pub const ALLOWED_EMOJI_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
];

pub fn is_allowed_emoji_type(content_type: &str) -> bool {
    ALLOWED_EMOJI_TYPES.contains(&content_type)
}

pub fn validate_emoji_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Emoji name cannot be empty".to_string());
    }
    if name.len() > 100 {
        return Err("Emoji name cannot exceed 100 characters".to_string());
    }
    // Check alphanumeric, underscore, hyphen only (matches database constraint)
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err("Emoji name can only contain alphanumeric characters, underscores, and hyphens".to_string());
    }
    Ok(())
}

pub fn validate_emoji_size(size: i64) -> Result<(), String> {
    if size > MAX_EMOJI_SIZE {
        Err(format!(
            "Emoji size {} exceeds maximum allowed size of {} (512KB)",
            size, MAX_EMOJI_SIZE
        ))
    } else {
        Ok(())
    }
}
