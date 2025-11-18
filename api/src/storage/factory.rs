use aws_config::BehaviorVersion;
use aws_sdk_s3::Client as S3Client;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use super::local::LocalStorage;
use super::s3::S3Storage;
use super::traits::{FileStorage, StorageType};

#[derive(sqlx::FromRow)]
struct StorageSettings {
    storage_type: String,
    s3_bucket: Option<String>,
    s3_region: Option<String>,
    s3_access_key_id: Option<String>,
    s3_secret_key_encrypted: Option<String>,
    s3_endpoint: Option<String>,
}

pub struct StorageFactory {
    db: PgPool,
    local_storage_path: String,
}

impl StorageFactory {
    pub fn new(db: PgPool, local_storage_path: String) -> Self {
        Self {
            db,
            local_storage_path,
        }
    }

    pub async fn get_storage(
        &self,
        org_id: Uuid,
    ) -> Result<Arc<dyn FileStorage>, ApiError> {
        // Fetch storage settings for org
        let settings: Option<StorageSettings> = sqlx::query_as(
            "SELECT storage_type, s3_bucket, s3_region, s3_access_key_id,
                    s3_secret_key_encrypted, s3_endpoint
             FROM storage_settings
             WHERE org_id = $1",
        )
        .bind(org_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch storage settings: {}",
                e
            ))
        })?;

        // Default to local storage if no settings found
        let storage_type = settings
            .as_ref()
            .and_then(|s| StorageType::from_str(&s.storage_type))
            .unwrap_or(StorageType::Local);

        match storage_type {
            StorageType::Local => {
                let storage = LocalStorage::new(&self.local_storage_path);
                Ok(Arc::new(storage))
            }
            StorageType::S3 => {
                let settings = settings.ok_or_else(|| {
                    ApiError::Internal(
                        "S3 storage settings not configured".to_string(),
                    )
                })?;

                let bucket = settings.s3_bucket.ok_or_else(|| {
                    ApiError::Internal("S3 bucket not configured".to_string())
                })?;

                let region = settings.s3_region.ok_or_else(|| {
                    ApiError::Internal("S3 region not configured".to_string())
                })?;

                // Build S3 client
                let config = if let (Some(access_key), Some(secret_key)) = (
                    settings.s3_access_key_id,
                    settings.s3_secret_key_encrypted, // TODO: Decrypt this
                ) {
                    // Use provided credentials
                    let credentials = aws_sdk_s3::config::Credentials::new(
                        access_key,
                        secret_key,
                        None,
                        None,
                        "openchat",
                    );

                    let mut config_builder = aws_config::defaults(BehaviorVersion::latest())
                        .region(aws_config::Region::new(region))
                        .credentials_provider(credentials);

                    if let Some(endpoint) = settings.s3_endpoint {
                        config_builder = config_builder.endpoint_url(endpoint);
                    }

                    config_builder.load().await
                } else {
                    // Use default credentials (IAM role, env vars, etc.)
                    aws_config::defaults(BehaviorVersion::latest())
                        .region(aws_config::Region::new(region))
                        .load()
                        .await
                };

                let client = S3Client::new(&config);
                let storage = S3Storage::new(client, bucket);

                Ok(Arc::new(storage))
            }
        }
    }
}
