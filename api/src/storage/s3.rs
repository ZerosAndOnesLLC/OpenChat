use actix_web::web::Bytes;
use async_trait::async_trait;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;

use crate::errors::ApiError;
use super::traits::{FileStorage, StorageType, UploadedFile};

#[derive(Clone)]
pub struct S3Storage {
    client: S3Client,
    bucket: String,
}

impl S3Storage {
    pub fn new(client: S3Client, bucket: String) -> Self {
        Self { client, bucket }
    }
}

#[async_trait]
impl FileStorage for S3Storage {
    async fn upload(
        &self,
        file_name: &str,
        content_type: &str,
        data: Bytes,
    ) -> Result<UploadedFile, ApiError> {
        // Generate unique S3 key
        let s3_key = self.generate_file_path(file_name);
        let size = data.len() as i64;

        // Upload to S3
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&s3_key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| {
                ApiError::Internal(format!("Failed to upload to S3: {}", e))
            })?;

        Ok(UploadedFile {
            storage_type: StorageType::S3,
            storage_path: s3_key,
            file_name: file_name.to_string(),
            content_type: content_type.to_string(),
            size,
        })
    }

    async fn download(&self, storage_path: &str) -> Result<Bytes, ApiError> {
        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(storage_path)
            .send()
            .await
            .map_err(|e| {
                ApiError::NotFound(format!("Failed to download from S3: {}", e))
            })?;

        let data = result
            .body
            .collect()
            .await
            .map_err(|e| {
                ApiError::Internal(format!(
                    "Failed to read S3 object body: {}",
                    e
                ))
            })?;

        Ok(data.into_bytes())
    }

    async fn delete(&self, storage_path: &str) -> Result<(), ApiError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(storage_path)
            .send()
            .await
            .map_err(|e| {
                ApiError::Internal(format!(
                    "Failed to delete from S3: {}",
                    e
                ))
            })?;

        Ok(())
    }

    fn storage_type(&self) -> StorageType {
        StorageType::S3
    }
}
