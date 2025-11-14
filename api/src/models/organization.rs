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
    /// Get an organization by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<Organization>> {
        let org = sqlx::query_as::<_, Organization>(
            r#"
            SELECT * FROM organizations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(org)
    }

    /// Create or update an organization
    pub async fn upsert(pool: &PgPool, id: Uuid, name: &str) -> ApiResult<Organization> {
        let org = sqlx::query_as::<_, Organization>(
            r#"
            INSERT INTO organizations (id, name)
            VALUES ($1, $2)
            ON CONFLICT (id)
            DO UPDATE SET
                name = EXCLUDED.name,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .fetch_one(pool)
        .await?;

        Ok(org)
    }
}
