use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DevicePairingCode {
    pub id: Uuid,
    pub code: String,
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
    pub roles: Vec<String>,
}

impl DevicePairingCode {
    /// Create a new device pairing code
    pub async fn create(
        pool: &PgPool,
        code: String,
        user_id: Uuid,
        org_id: Uuid,
        expires_at: DateTime<Utc>,
        roles: Vec<String>,
    ) -> ApiResult<DevicePairingCode> {
        let pairing_code = sqlx::query_as::<_, DevicePairingCode>(
            r#"
            INSERT INTO device_pairing_codes (code, user_id, org_id, expires_at, roles)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(code)
        .bind(user_id)
        .bind(org_id)
        .bind(expires_at)
        .bind(&roles)
        .fetch_one(pool)
        .await?;

        Ok(pairing_code)
    }

    /// Get a pairing code by its code string
    pub async fn get_by_code(pool: &PgPool, code: &str) -> ApiResult<Option<DevicePairingCode>> {
        let pairing_code = sqlx::query_as::<_, DevicePairingCode>(
            r#"
            SELECT * FROM device_pairing_codes
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(pool)
        .await?;

        Ok(pairing_code)
    }

    /// Mark a pairing code as used
    pub async fn mark_as_used(pool: &PgPool, id: Uuid) -> ApiResult<DevicePairingCode> {
        let pairing_code = sqlx::query_as::<_, DevicePairingCode>(
            r#"
            UPDATE device_pairing_codes
            SET used = true
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(pairing_code)
    }

    /// Delete expired pairing codes
    #[allow(dead_code)]
    pub async fn delete_expired(pool: &PgPool) -> ApiResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM device_pairing_codes
            WHERE expires_at < NOW()
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Check if a code is valid (exists, not used, not expired)
    pub fn is_valid(&self) -> bool {
        !self.used && self.expires_at > Utc::now()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DeviceSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub device_type: String,
    pub device_name: Option<String>,
    pub device_fingerprint: Option<String>,
    pub last_active_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl DeviceSession {
    /// Create a new device session
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        org_id: Uuid,
        device_type: String,
        device_name: Option<String>,
        device_fingerprint: Option<String>,
    ) -> ApiResult<DeviceSession> {
        let device_session = sqlx::query_as::<_, DeviceSession>(
            r#"
            INSERT INTO device_sessions (user_id, org_id, device_type, device_name, device_fingerprint)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(org_id)
        .bind(device_type)
        .bind(device_name)
        .bind(device_fingerprint)
        .fetch_one(pool)
        .await?;

        Ok(device_session)
    }

    /// Get all device sessions for a user
    pub async fn get_by_user(pool: &PgPool, user_id: Uuid) -> ApiResult<Vec<DeviceSession>> {
        let sessions = sqlx::query_as::<_, DeviceSession>(
            r#"
            SELECT * FROM device_sessions
            WHERE user_id = $1
            ORDER BY last_active_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(sessions)
    }

    /// Get a device session by ID
    #[allow(dead_code)]
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<DeviceSession>> {
        let session = sqlx::query_as::<_, DeviceSession>(
            r#"
            SELECT * FROM device_sessions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(session)
    }

    /// Update last active timestamp
    #[allow(dead_code)]
    pub async fn update_last_active(pool: &PgPool, id: Uuid) -> ApiResult<DeviceSession> {
        let session = sqlx::query_as::<_, DeviceSession>(
            r#"
            UPDATE device_sessions
            SET last_active_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(session)
    }

    /// Delete a device session by ID
    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM device_sessions
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
