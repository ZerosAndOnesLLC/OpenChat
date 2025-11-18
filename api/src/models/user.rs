use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[allow(dead_code)]
pub struct User {
    pub id: Uuid,
    pub org_id: Uuid,
    pub tv_user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub status: String,
    pub disable_read_receipts: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
impl User {
    /// Create or update a user in the database
    /// This is called during authentication to ensure the user exists in openchat
    pub async fn upsert(
        pool: &PgPool,
        tv_user_id: Uuid,
        org_id: &Uuid,
        email: &str,
        display_name: &str,
    ) -> ApiResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, org_id, tv_user_id, email, display_name, status)
            VALUES ($1, $2, $3, $4, $5, 'offline')
            ON CONFLICT (tv_user_id)
            DO UPDATE SET
                email = EXCLUDED.email,
                display_name = EXCLUDED.display_name,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4()) // Generate new ID only on insert
        .bind(org_id)
        .bind(tv_user_id)
        .bind(email)
        .bind(display_name)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Get a user by their TitaniumVault user ID
    pub async fn get_by_tv_user_id(pool: &PgPool, tv_user_id: Uuid) -> ApiResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE tv_user_id = $1
            "#,
        )
        .bind(tv_user_id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Get a user by their openchat user ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Update user status (online/offline/away)
    pub async fn update_status(pool: &PgPool, id: Uuid, status: &str) -> ApiResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(status)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// List all users in an organization
    pub async fn list_by_org(pool: &PgPool, org_id: Uuid) -> ApiResult<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE org_id = $1
            ORDER BY display_name
            "#,
        )
        .bind(org_id)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}
