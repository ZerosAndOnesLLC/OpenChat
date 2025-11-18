use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::ApiResult,
    models::role::{Permission, Role},
    services::tv_api::TokenClaims,
};

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub role_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignPermissionsRequest {
    pub permission_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub role_name: String,
    pub is_system_role: bool,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RoleWithPermissionsResponse {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub role_name: String,
    pub is_system_role: bool,
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
}

/// GET /api/roles - List all roles (org-specific + global system roles)
pub async fn list_roles(
    pool: web::Data<PgPool>,
    claims: web::ReqData<TokenClaims>,
) -> ApiResult<HttpResponse> {
    let roles = Role::list_for_org(pool.get_ref(), claims.org_id).await?;

    Ok(HttpResponse::Ok().json(roles))
}

/// GET /api/roles/{id} - Get a specific role with its permissions
pub async fn get_role(
    pool: web::Data<PgPool>,
    role_id: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let role = Role::get_by_id(pool.get_ref(), *role_id)
        .await?
        .ok_or_else(|| crate::errors::ApiError::NotFound("Role not found".to_string()))?;

    let permissions = Role::get_permissions(pool.get_ref(), role.id).await?;

    let response = RoleWithPermissionsResponse {
        id: role.id,
        org_id: role.org_id,
        role_name: role.role_name,
        is_system_role: role.is_system_role,
        description: role.description,
        permissions,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/roles - Create a new role (admin only)
/// Note: This endpoint is protected by PermissionMiddleware with "org.manage_roles" permission
pub async fn create_role(
    pool: web::Data<PgPool>,
    claims: web::ReqData<TokenClaims>,
    req: web::Json<CreateRoleRequest>,
) -> ApiResult<HttpResponse> {
    let role = Role::create(
        pool.get_ref(),
        Some(claims.org_id),
        &req.role_name,
        req.description.as_deref(),
    )
    .await?;

    Ok(HttpResponse::Created().json(role))
}

/// PUT /api/roles/{id} - Update a role (admin only)
/// Note: This endpoint is protected by PermissionMiddleware with "org.manage_roles" permission
pub async fn update_role(
    pool: web::Data<PgPool>,
    role_id: web::Path<Uuid>,
    req: web::Json<UpdateRoleRequest>,
) -> ApiResult<HttpResponse> {
    // Check if role exists and is not a system role
    let existing_role = Role::get_by_id(pool.get_ref(), *role_id)
        .await?
        .ok_or_else(|| crate::errors::ApiError::NotFound("Role not found".to_string()))?;

    if existing_role.is_system_role {
        return Err(crate::errors::ApiError::BadRequest(
            "Cannot update system roles".to_string(),
        ));
    }

    let role = Role::update(
        pool.get_ref(),
        *role_id,
        &req.role_name,
        req.description.as_deref(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(role))
}

/// DELETE /api/roles/{id} - Delete a role (admin only)
/// Note: This endpoint is protected by PermissionMiddleware with "org.manage_roles" permission
pub async fn delete_role(
    pool: web::Data<PgPool>,
    role_id: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    // Check if role exists and is not a system role
    let existing_role = Role::get_by_id(pool.get_ref(), *role_id)
        .await?
        .ok_or_else(|| crate::errors::ApiError::NotFound("Role not found".to_string()))?;

    if existing_role.is_system_role {
        return Err(crate::errors::ApiError::BadRequest(
            "Cannot delete system roles".to_string(),
        ));
    }

    Role::delete(pool.get_ref(), *role_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/roles/{id}/permissions - Assign permissions to a role (admin only)
/// Note: This endpoint is protected by PermissionMiddleware with "org.manage_roles" permission
pub async fn assign_permissions(
    pool: web::Data<PgPool>,
    role_id: web::Path<Uuid>,
    req: web::Json<AssignPermissionsRequest>,
) -> ApiResult<HttpResponse> {
    // Check if role exists and is not a system role
    let existing_role = Role::get_by_id(pool.get_ref(), *role_id)
        .await?
        .ok_or_else(|| crate::errors::ApiError::NotFound("Role not found".to_string()))?;

    if existing_role.is_system_role {
        return Err(crate::errors::ApiError::BadRequest(
            "Cannot modify permissions for system roles".to_string(),
        ));
    }

    Role::assign_permissions(pool.get_ref(), *role_id, req.permission_ids.clone()).await?;

    // Note: Permission cache will automatically expire after TTL (5 minutes)
    // For immediate effect, admins can restart the service or wait for cache to expire

    Ok(HttpResponse::NoContent().finish())
}

/// GET /api/permissions - List all available permissions
pub async fn list_permissions(pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
    let permissions = Permission::list_all(pool.get_ref()).await?;

    Ok(HttpResponse::Ok().json(permissions))
}
