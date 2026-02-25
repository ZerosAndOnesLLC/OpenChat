use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserGroup {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub handle: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserGroupMember {
    pub id: Uuid,
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserGroup {
    pub name: String,
    pub handle: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserGroup {
    pub name: Option<String>,
    pub handle: Option<String>,
    pub description: Option<String>,
}

impl UserGroup {
    pub async fn list_by_org(pool: &PgPool, org_id: Uuid) -> ApiResult<Vec<UserGroup>> {
        let groups = sqlx::query_as::<_, UserGroup>(
            r#"
            SELECT * FROM user_groups
            WHERE org_id = $1
            ORDER BY name
            "#,
        )
        .bind(org_id)
        .fetch_all(pool)
        .await?;

        Ok(groups)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<UserGroup>> {
        let group = sqlx::query_as::<_, UserGroup>(
            "SELECT * FROM user_groups WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(group)
    }

    pub async fn get_by_handle(pool: &PgPool, org_id: Uuid, handle: &str) -> ApiResult<Option<UserGroup>> {
        let group = sqlx::query_as::<_, UserGroup>(
            r#"
            SELECT * FROM user_groups
            WHERE org_id = $1 AND LOWER(handle) = LOWER($2)
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(handle)
        .fetch_optional(pool)
        .await?;

        Ok(group)
    }

    pub async fn create(
        pool: &PgPool,
        org_id: Uuid,
        created_by: Uuid,
        data: CreateUserGroup,
    ) -> ApiResult<UserGroup> {
        let group = sqlx::query_as::<_, UserGroup>(
            r#"
            INSERT INTO user_groups (id, org_id, name, handle, description, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(&data.name)
        .bind(&data.handle)
        .bind(&data.description)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(group)
    }

    pub async fn update(pool: &PgPool, id: Uuid, data: UpdateUserGroup) -> ApiResult<Option<UserGroup>> {
        let group = sqlx::query_as::<_, UserGroup>(
            r#"
            UPDATE user_groups
            SET name = COALESCE($2, name),
                handle = COALESCE($3, handle),
                description = COALESCE($4, description),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(data.name)
        .bind(data.handle)
        .bind(data.description)
        .fetch_optional(pool)
        .await?;

        Ok(group)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query("DELETE FROM user_groups WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_member_ids(pool: &PgPool, group_id: Uuid) -> ApiResult<Vec<Uuid>> {
        let ids = sqlx::query_scalar::<_, Uuid>(
            "SELECT user_id FROM user_group_members WHERE group_id = $1",
        )
        .bind(group_id)
        .fetch_all(pool)
        .await?;

        Ok(ids)
    }
}

impl UserGroupMember {
    pub async fn list_by_group(pool: &PgPool, group_id: Uuid) -> ApiResult<Vec<UserGroupMember>> {
        let members = sqlx::query_as::<_, UserGroupMember>(
            r#"
            SELECT * FROM user_group_members
            WHERE group_id = $1
            ORDER BY added_at
            "#,
        )
        .bind(group_id)
        .fetch_all(pool)
        .await?;

        Ok(members)
    }

    pub async fn add(pool: &PgPool, group_id: Uuid, user_id: Uuid) -> ApiResult<UserGroupMember> {
        let member = sqlx::query_as::<_, UserGroupMember>(
            r#"
            INSERT INTO user_group_members (id, group_id, user_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (group_id, user_id) DO UPDATE SET added_at = user_group_members.added_at
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(group_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(member)
    }

    pub async fn remove(pool: &PgPool, group_id: Uuid, user_id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query(
            "DELETE FROM user_group_members WHERE group_id = $1 AND user_id = $2",
        )
        .bind(group_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
