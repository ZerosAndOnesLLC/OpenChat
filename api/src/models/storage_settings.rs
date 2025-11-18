use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageSettings {
    pub id: Uuid,
    pub org_id: Uuid,
    pub storage_type: String, // "local" or "s3"
    pub s3_bucket: Option<String>,
    pub s3_region: Option<String>,
    pub s3_access_key_id_encrypted: Option<String>,
    pub s3_secret_key_encrypted: Option<String>,
    pub s3_endpoint: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStorageSettingsRequest {
    pub storage_type: String,
    pub s3_bucket: Option<String>,
    pub s3_region: Option<String>,
    pub s3_access_key_id: Option<String>,
    pub s3_secret_key: Option<String>,
    pub s3_endpoint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StorageSettingsResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub storage_type: String,
    pub s3_bucket: Option<String>,
    pub s3_region: Option<String>,
    pub s3_endpoint: Option<String>,
    // Note: We don't return encrypted credentials in responses
}

impl From<StorageSettings> for StorageSettingsResponse {
    fn from(settings: StorageSettings) -> Self {
        Self {
            id: settings.id,
            org_id: settings.org_id,
            storage_type: settings.storage_type,
            s3_bucket: settings.s3_bucket,
            s3_region: settings.s3_region,
            s3_endpoint: settings.s3_endpoint,
        }
    }
}
