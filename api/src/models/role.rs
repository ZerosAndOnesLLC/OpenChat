use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub role_name: String,
    pub is_system_role: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub permission_name: String,
    pub resource_type: String,
    pub action: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[allow(dead_code)]
pub struct RolePermission {
    pub id: Uuid,
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RoleWithPermissions {
    #[serde(flatten)]
    pub role: Role,
    pub permissions: Vec<Permission>,
}

impl Role {
    /// Get a role by name (for SSO role matching)
    #[allow(dead_code)]
    pub async fn get_by_name(pool: &PgPool, role_name: &str) -> ApiResult<Option<Role>> {
        let role = sqlx::query_as::<_, Role>(
            r#"
            SELECT * FROM roles
            WHERE role_name = $1 AND is_system_role = true
            "#,
        )
        .bind(role_name)
        .fetch_optional(pool)
        .await?;

        Ok(role)
    }

    /// Get all roles for an organization (including global system roles)
    pub async fn list_for_org(pool: &PgPool, org_id: Uuid) -> ApiResult<Vec<Role>> {
        let roles = sqlx::query_as::<_, Role>(
            r#"
            SELECT * FROM roles
            WHERE org_id = $1 OR org_id IS NULL
            ORDER BY is_system_role DESC, role_name
            "#,
        )
        .bind(org_id)
        .fetch_all(pool)
        .await?;

        Ok(roles)
    }

    /// Get role by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<Role>> {
        let role = sqlx::query_as::<_, Role>(
            r#"
            SELECT * FROM roles
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(role)
    }

    /// Create a new role
    pub async fn create(
        pool: &PgPool,
        org_id: Option<Uuid>,
        role_name: &str,
        description: Option<&str>,
    ) -> ApiResult<Role> {
        let role = sqlx::query_as::<_, Role>(
            r#"
            INSERT INTO roles (id, org_id, role_name, is_system_role, description)
            VALUES ($1, $2, $3, false, $4)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(role_name)
        .bind(description)
        .fetch_one(pool)
        .await?;

        Ok(role)
    }

    /// Update a role
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        role_name: &str,
        description: Option<&str>,
    ) -> ApiResult<Role> {
        let role = sqlx::query_as::<_, Role>(
            r#"
            UPDATE roles
            SET role_name = $1, description = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(role_name)
        .bind(description)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(role)
    }

    /// Delete a role (only if not system role)
    pub async fn delete(pool: &PgPool, id: Uuid) -> ApiResult<()> {
        sqlx::query(
            r#"
            DELETE FROM roles
            WHERE id = $1 AND is_system_role = false
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get permissions for a role
    pub async fn get_permissions(pool: &PgPool, role_id: Uuid) -> ApiResult<Vec<Permission>> {
        let permissions = sqlx::query_as::<_, Permission>(
            r#"
            SELECT p.*
            FROM permissions p
            JOIN role_permissions rp ON rp.permission_id = p.id
            WHERE rp.role_id = $1
            ORDER BY p.resource_type, p.action
            "#,
        )
        .bind(role_id)
        .fetch_all(pool)
        .await?;

        Ok(permissions)
    }

    /// Check if a role has a specific permission
    #[allow(dead_code)]
    pub async fn has_permission(
        pool: &PgPool,
        role_id: Uuid,
        permission_name: &str,
    ) -> ApiResult<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM role_permissions rp
                JOIN permissions p ON p.id = rp.permission_id
                WHERE rp.role_id = $1 AND p.permission_name = $2
            )
            "#,
        )
        .bind(role_id)
        .bind(permission_name)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Assign permissions to a role
    pub async fn assign_permissions(
        pool: &PgPool,
        role_id: Uuid,
        permission_ids: Vec<Uuid>,
    ) -> ApiResult<()> {
        // First, remove existing permissions
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(role_id)
            .execute(pool)
            .await?;

        // Then, insert new permissions
        for permission_id in permission_ids {
            sqlx::query(
                r#"
                INSERT INTO role_permissions (id, role_id, permission_id)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(role_id)
            .bind(permission_id)
            .execute(pool)
            .await?;
        }

        Ok(())
    }
}

impl Permission {
    /// Get all permissions
    pub async fn list_all(pool: &PgPool) -> ApiResult<Vec<Permission>> {
        let permissions = sqlx::query_as::<_, Permission>(
            r#"
            SELECT * FROM permissions
            ORDER BY resource_type, action
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(permissions)
    }

    /// Get permission by name
    #[allow(dead_code)]
    pub async fn get_by_name(pool: &PgPool, name: &str) -> ApiResult<Option<Permission>> {
        let permission = sqlx::query_as::<_, Permission>(
            r#"
            SELECT * FROM permissions
            WHERE permission_name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(permission)
    }
}

/// Check if user has a specific permission based on their SSO roles
pub async fn user_has_permission(
    pool: &PgPool,
    role_names: &[String],
    permission_name: &str,
) -> ApiResult<bool> {
    if role_names.is_empty() {
        return Ok(false);
    }

    let result = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM roles r
            JOIN role_permissions rp ON rp.role_id = r.id
            JOIN permissions p ON p.id = rp.permission_id
            WHERE r.role_name = ANY($1)
            AND p.permission_name = $2
            AND r.is_system_role = true
        )
        "#,
    )
    .bind(role_names)
    .bind(permission_name)
    .fetch_one(pool)
    .await?;

    Ok(result)
}
