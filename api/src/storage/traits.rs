use actix_web::web::Bytes;
use async_trait::async_trait;
use std::path::PathBuf;

use crate::errors::ApiError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageType {
    Local,
    S3,
}

impl StorageType {
    pub fn as_str(&self) -> &str {
        match self {
            StorageType::Local => "local",
            StorageType::S3 => "s3",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "local" => Some(StorageType::Local),
            "s3" => Some(StorageType::S3),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UploadedFile {
    pub storage_type: StorageType,
    pub storage_path: String,
    pub file_name: String,
    pub content_type: String,
    pub size: i64,
}

#[async_trait]
pub trait FileStorage: Send + Sync {
    /// Upload a file and return storage metadata
    async fn upload(
        &self,
        file_name: &str,
        content_type: &str,
        data: Bytes,
    ) -> Result<UploadedFile, ApiError>;

    /// Download a file by its storage path
    async fn download(&self, storage_path: &str) -> Result<Bytes, ApiError>;

    /// Delete a file by its storage path
    async fn delete(&self, storage_path: &str) -> Result<(), ApiError>;

    /// Get the storage type
    fn storage_type(&self) -> StorageType;

    /// Generate a unique file path for storage
    fn generate_file_path(&self, file_name: &str) -> String {
        let uuid = uuid::Uuid::new_v4();
        let timestamp = chrono::Utc::now().format("%Y%m%d");
        let extension = PathBuf::from(file_name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();

        format!("{}/{}{}", timestamp, uuid, extension)
    }
}
