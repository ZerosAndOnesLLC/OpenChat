use actix_web::web::Bytes;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::errors::ApiError;
use super::traits::{FileStorage, StorageType, UploadedFile};

#[derive(Clone)]
pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    pub async fn ensure_directory_exists(&self) -> Result<(), ApiError> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path)
                .await
                .map_err(|e| {
                    ApiError::Internal(format!(
                        "Failed to create storage directory: {}",
                        e
                    ))
                })?;
        }
        Ok(())
    }

    fn resolve_path(&self, storage_path: &str) -> PathBuf {
        self.base_path.join(storage_path)
    }
}

#[async_trait]
impl FileStorage for LocalStorage {
    async fn upload(
        &self,
        file_name: &str,
        content_type: &str,
        data: Bytes,
    ) -> Result<UploadedFile, ApiError> {
        // Ensure base directory exists
        self.ensure_directory_exists().await?;

        // Generate unique file path
        let storage_path = self.generate_file_path(file_name);
        let full_path = self.resolve_path(&storage_path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                ApiError::Internal(format!(
                    "Failed to create directory: {}",
                    e
                ))
            })?;
        }

        // Write file to disk
        let mut file = fs::File::create(&full_path).await.map_err(|e| {
            ApiError::Internal(format!("Failed to create file: {}", e))
        })?;

        file.write_all(&data).await.map_err(|e| {
            ApiError::Internal(format!("Failed to write file: {}", e))
        })?;

        file.flush().await.map_err(|e| {
            ApiError::Internal(format!("Failed to flush file: {}", e))
        })?;

        Ok(UploadedFile {
            storage_type: StorageType::Local,
            storage_path,
            file_name: file_name.to_string(),
            content_type: content_type.to_string(),
            size: data.len() as i64,
        })
    }

    async fn download(&self, storage_path: &str) -> Result<Bytes, ApiError> {
        let full_path = self.resolve_path(storage_path);

        if !full_path.exists() {
            return Err(ApiError::NotFound(format!(
                "File not found: {}",
                storage_path
            )));
        }

        let data = fs::read(&full_path).await.map_err(|e| {
            ApiError::Internal(format!("Failed to read file: {}", e))
        })?;

        Ok(Bytes::from(data))
    }

    async fn delete(&self, storage_path: &str) -> Result<(), ApiError> {
        let full_path = self.resolve_path(storage_path);

        if !full_path.exists() {
            return Ok(()); // Already deleted, consider success
        }

        fs::remove_file(&full_path).await.map_err(|e| {
            ApiError::Internal(format!("Failed to delete file: {}", e))
        })?;

        Ok(())
    }

    fn storage_type(&self) -> StorageType {
        StorageType::Local
    }
}
