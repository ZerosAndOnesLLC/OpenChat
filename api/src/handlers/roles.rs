use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::ApiResult,
    models::{audit_log::actions, role::{Permission, Role}, user::User},
    services::{audit_logger::AuditLogger, tv_api::TokenClaims},
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
    body: web::Json<CreateRoleRequest>,
    http_req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Get current user for audit logging
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| crate::errors::ApiError::NotFound("Current user not found".to_string()))?;

    let role = Role::create(
        pool.get_ref(),
        Some(claims.org_id),
        &body.role_name,
        body.description.as_deref(),
    )
    .await?;

    // Log role creation in audit log
    if let Err(e) = AuditLogger::log(
        pool.get_ref(),
        Some(current_user.id),
        actions::ROLE_CREATED,
        "role",
        Some(role.id),
        json!({
            "role_name": &role.role_name,
            "description": &role.description,
        }),
        Some(&http_req),
    )
    .await
    {
        tracing::warn!("Failed to create audit log for role creation: {}", e);
    }

    Ok(HttpResponse::Created().json(role))
}

/// PUT /api/roles/{id} - Update a role (admin only)
/// Note: This endpoint is protected by PermissionMiddleware with "org.manage_roles" permission
pub async fn update_role(
    pool: web::Data<PgPool>,
    claims: web::ReqData<TokenClaims>,
    role_id: web::Path<Uuid>,
    body: web::Json<UpdateRoleRequest>,
    http_req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Get current user for audit logging
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| crate::errors::ApiError::NotFound("Current user not found".to_string()))?;

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
        &body.role_name,
        body.description.as_deref(),
    )
    .await?;

    // Log role update in audit log
    if let Err(e) = AuditLogger::log(
        pool.get_ref(),
        Some(current_user.id),
        actions::ROLE_UPDATED,
        "role",
        Some(role.id),
        json!({
            "old_name": &existing_role.role_name,
            "new_name": &role.role_name,
            "old_description": &existing_role.description,
            "new_description": &role.description,
        }),
        Some(&http_req),
    )
    .await
    {
        tracing::warn!("Failed to create audit log for role update: {}", e);
    }

    Ok(HttpResponse::Ok().json(role))
}

/// DELETE /api/roles/{id} - Delete a role (admin only)
/// Note: This endpoint is protected by PermissionMiddleware with "org.manage_roles" permission
pub async fn delete_role(
    pool: web::Data<PgPool>,
    claims: web::ReqData<TokenClaims>,
    role_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Get current user for audit logging
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| crate::errors::ApiError::NotFound("Current user not found".to_string()))?;

    // Check if role exists and is not a system role
    let existing_role = Role::get_by_id(pool.get_ref(), *role_id)
        .await?
        .ok_or_else(|| crate::errors::ApiError::NotFound("Role not found".to_string()))?;

    if existing_role.is_system_role {
        return Err(crate::errors::ApiError::BadRequest(
            "Cannot delete system roles".to_string(),
        ));
    }

    // Store role name for audit log before deletion
    let role_name = existing_role.role_name.clone();

    Role::delete(pool.get_ref(), *role_id).await?;

    // Log role deletion in audit log
    if let Err(e) = AuditLogger::log(
        pool.get_ref(),
        Some(current_user.id),
        actions::ROLE_DELETED,
        "role",
        Some(*role_id),
        json!({
            "role_name": &role_name,
        }),
        Some(&http_req),
    )
    .await
    {
        tracing::warn!("Failed to create audit log for role deletion: {}", e);
    }

    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/roles/{id}/permissions - Assign permissions to a role (admin only)
/// Note: This endpoint is protected by PermissionMiddleware with "org.manage_roles" permission
pub async fn assign_permissions(
    pool: web::Data<PgPool>,
    claims: web::ReqData<TokenClaims>,
    role_id: web::Path<Uuid>,
    body: web::Json<AssignPermissionsRequest>,
    http_req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Get current user for audit logging
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| crate::errors::ApiError::NotFound("Current user not found".to_string()))?;

    // Check if role exists and is not a system role
    let existing_role = Role::get_by_id(pool.get_ref(), *role_id)
        .await?
        .ok_or_else(|| crate::errors::ApiError::NotFound("Role not found".to_string()))?;

    if existing_role.is_system_role {
        return Err(crate::errors::ApiError::BadRequest(
            "Cannot modify permissions for system roles".to_string(),
        ));
    }

    // Get old permissions for audit log
    let old_permissions = Role::get_permissions(pool.get_ref(), *role_id).await?;

    Role::assign_permissions(pool.get_ref(), *role_id, body.permission_ids.clone()).await?;

    // Get new permissions for audit log
    let new_permissions = Role::get_permissions(pool.get_ref(), *role_id).await?;

    // Log permission assignment in audit log
    if let Err(e) = AuditLogger::log(
        pool.get_ref(),
        Some(current_user.id),
        actions::PERMISSION_GRANTED,
        "role",
        Some(*role_id),
        json!({
            "role_name": &existing_role.role_name,
            "old_permissions": old_permissions.iter().map(|p| &p.permission_name).collect::<Vec<_>>(),
            "new_permissions": new_permissions.iter().map(|p| &p.permission_name).collect::<Vec<_>>(),
        }),
        Some(&http_req),
    )
    .await
    {
        tracing::warn!("Failed to create audit log for permission assignment: {}", e);
    }

    // Note: Permission cache will automatically expire after TTL (5 minutes)
    // For immediate effect, admins can restart the service or wait for cache to expire

    Ok(HttpResponse::NoContent().finish())
}

/// GET /api/permissions - List all available permissions
pub async fn list_permissions(pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
    let permissions = Permission::list_all(pool.get_ref()).await?;

    Ok(HttpResponse::Ok().json(permissions))
}
