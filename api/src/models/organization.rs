use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Organization {
    /// Create or update an organization
    /// This is called during authentication to ensure the org exists before creating users
    pub async fn upsert(
        pool: &PgPool,
        org_id: Uuid,
        org_name: &str,
    ) -> ApiResult<Organization> {
        let org = sqlx::query_as::<_, Organization>(
            r#"
            INSERT INTO organizations (id, name, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            ON CONFLICT (id)
            DO UPDATE SET
                name = EXCLUDED.name,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(org_name)
        .fetch_one(pool)
        .await?;

        Ok(org)
    }
}
