use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RetentionPolicy {
    pub id: Uuid,
    pub org_id: Uuid,
    pub policy_type: String, // "messages" or "files"
    pub retention_days: i32,
    pub enabled: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LegalHold {
    pub id: Uuid,
    pub org_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub reason: String,
    pub enabled: bool,
    pub created_by: Uuid,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub disabled_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRetentionPolicyRequest {
    pub policy_type: String, // "messages" or "files"
    pub retention_days: i32,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateLegalHoldRequest {
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct RetentionPolicyResponse {
    pub policies: Vec<RetentionPolicy>,
}
