use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Attachment {
    pub id: Uuid,
    pub message_id: Uuid,
    pub file_name: String,
    pub file_url: String,
    pub file_type: Option<String>,
    pub file_size: Option<i64>,
    pub storage_type: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAttachment {
    pub message_id: Uuid,
    pub file_name: String,
    pub file_type: Option<String>,
    pub storage_type: String,
    pub storage_path: String,
    pub file_size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttachmentUploadResponse {
    pub id: Uuid,
    pub file_name: String,
    pub file_url: String,
    pub file_type: Option<String>,
    pub file_size: i64,
    pub storage_type: String,
}

// File validation constants
pub const MAX_FILE_SIZE: i64 = 25 * 1024 * 1024; // 25MB default
pub const ALLOWED_IMAGE_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "image/svg+xml",
];
pub const ALLOWED_DOCUMENT_TYPES: &[&str] = &[
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/vnd.ms-powerpoint",
    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    "text/plain",
    "text/csv",
];
pub const ALLOWED_VIDEO_TYPES: &[&str] = &[
    "video/mp4",
    "video/mpeg",
    "video/quicktime",
    "video/x-msvideo",
    "video/webm",
];
pub const ALLOWED_AUDIO_TYPES: &[&str] = &[
    "audio/mpeg",
    "audio/wav",
    "audio/ogg",
    "audio/webm",
];

pub fn is_allowed_file_type(content_type: &str) -> bool {
    ALLOWED_IMAGE_TYPES.contains(&content_type)
        || ALLOWED_DOCUMENT_TYPES.contains(&content_type)
        || ALLOWED_VIDEO_TYPES.contains(&content_type)
        || ALLOWED_AUDIO_TYPES.contains(&content_type)
}

pub fn validate_file_size(size: i64, max_size: i64) -> Result<(), String> {
    if size > max_size {
        Err(format!(
            "File size {} exceeds maximum allowed size of {}",
            size, max_size
        ))
    } else {
        Ok(())
    }
}
