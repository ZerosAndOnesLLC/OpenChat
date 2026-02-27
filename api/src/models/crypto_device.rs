use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{ApiError, ApiResult};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CryptoDevice {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: String,
    pub display_name: Option<String>,
    pub identity_key: String,
    pub signing_key: String,
    pub one_time_keys: serde_json::Value,
    pub fallback_key: Option<serde_json::Value>,
    pub verified: bool,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl CryptoDevice {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        device_id: &str,
        display_name: Option<&str>,
        identity_key: &str,
        signing_key: &str,
    ) -> ApiResult<CryptoDevice> {
        let device = sqlx::query_as::<_, CryptoDevice>(
            r#"
            INSERT INTO user_crypto_devices (user_id, device_id, display_name, identity_key, signing_key)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id, device_id)
            DO UPDATE SET identity_key = $4, signing_key = $5, display_name = COALESCE($3, user_crypto_devices.display_name), last_seen_at = NOW()
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .bind(display_name)
        .bind(identity_key)
        .bind(signing_key)
        .fetch_one(pool)
        .await?;

        Ok(device)
    }

    pub async fn get_by_user_and_device(
        pool: &PgPool,
        user_id: Uuid,
        device_id: &str,
    ) -> ApiResult<Option<CryptoDevice>> {
        let device = sqlx::query_as::<_, CryptoDevice>(
            r#"
            SELECT * FROM user_crypto_devices
            WHERE user_id = $1 AND device_id = $2
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .fetch_optional(pool)
        .await?;

        Ok(device)
    }

    pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> ApiResult<Vec<CryptoDevice>> {
        let devices = sqlx::query_as::<_, CryptoDevice>(
            r#"
            SELECT * FROM user_crypto_devices
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(devices)
    }

    pub async fn list_by_users(pool: &PgPool, user_ids: &[Uuid]) -> ApiResult<Vec<CryptoDevice>> {
        if user_ids.is_empty() {
            return Ok(vec![]);
        }

        let devices = sqlx::query_as::<_, CryptoDevice>(
            r#"
            SELECT * FROM user_crypto_devices
            WHERE user_id = ANY($1)
            ORDER BY user_id, created_at DESC
            "#,
        )
        .bind(user_ids)
        .fetch_all(pool)
        .await?;

        Ok(devices)
    }

    pub async fn upload_one_time_keys(
        pool: &PgPool,
        user_id: Uuid,
        device_id: &str,
        keys: serde_json::Value,
    ) -> ApiResult<CryptoDevice> {
        let device = sqlx::query_as::<_, CryptoDevice>(
            r#"
            UPDATE user_crypto_devices
            SET one_time_keys = one_time_keys || $3, last_seen_at = NOW()
            WHERE user_id = $1 AND device_id = $2
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .bind(keys)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

        Ok(device)
    }

    pub async fn claim_one_time_key(
        pool: &PgPool,
        user_id: Uuid,
        device_id: &str,
    ) -> ApiResult<Option<(String, serde_json::Value)>> {
        // Get device and extract one key
        let device = sqlx::query_as::<_, CryptoDevice>(
            r#"
            SELECT * FROM user_crypto_devices
            WHERE user_id = $1 AND device_id = $2
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .fetch_optional(pool)
        .await?;

        let device = match device {
            Some(d) => d,
            None => return Ok(None),
        };

        let keys = device.one_time_keys.as_object();
        if let Some(obj) = keys {
            if let Some((key_id, key_value)) = obj.iter().next() {
                let key_id = key_id.clone();
                let key_value = key_value.clone();

                // Remove the claimed key
                sqlx::query(
                    r#"
                    UPDATE user_crypto_devices
                    SET one_time_keys = one_time_keys - $3, last_seen_at = NOW()
                    WHERE user_id = $1 AND device_id = $2
                    "#,
                )
                .bind(user_id)
                .bind(device_id)
                .bind(&key_id)
                .execute(pool)
                .await?;

                return Ok(Some((key_id, key_value)));
            }
        }

        // No one-time keys available, return fallback key if present
        if let Some(fallback) = device.fallback_key {
            return Ok(Some(("fallback".to_string(), fallback)));
        }

        Ok(None)
    }

    pub async fn set_verified(
        pool: &PgPool,
        id: Uuid,
        verified: bool,
    ) -> ApiResult<CryptoDevice> {
        let device = sqlx::query_as::<_, CryptoDevice>(
            r#"
            UPDATE user_crypto_devices
            SET verified = $2
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(verified)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

        Ok(device)
    }

    pub async fn delete(pool: &PgPool, user_id: Uuid, device_id: &str) -> ApiResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_crypto_devices
            WHERE user_id = $1 AND device_id = $2
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound("Device not found".to_string()));
        }

        Ok(())
    }

    pub async fn update_last_seen(
        pool: &PgPool,
        user_id: Uuid,
        device_id: &str,
    ) -> ApiResult<()> {
        sqlx::query(
            r#"
            UPDATE user_crypto_devices
            SET last_seen_at = NOW()
            WHERE user_id = $1 AND device_id = $2
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
