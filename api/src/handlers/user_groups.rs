use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use regex::Regex;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    cache::user_groups as group_cache,
    db::RedisPool,
    errors::{ApiError, ApiResult},
    models::user::User,
    models::user_group::{CreateUserGroup, UpdateUserGroup, UserGroup, UserGroupMember},
    services::tv_api::TokenClaims,
};

fn validate_handle(handle: &str) -> Result<(), ApiError> {
    let re = Regex::new(r"^[a-z0-9_-]+$").unwrap();
    if !re.is_match(handle) {
        return Err(ApiError::BadRequest(
            "Handle must contain only lowercase letters, numbers, hyphens, and underscores"
                .to_string(),
        ));
    }
    Ok(())
}

/// GET /api/user-groups - List all groups in the org
pub async fn list_groups(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Check cache first
    let groups = match group_cache::get_groups_from_cache(redis_pool.get_ref(), current_user.org_id)
        .await?
    {
        Some(cached) => cached,
        None => {
            let groups = UserGroup::list_by_org(pool.get_ref(), current_user.org_id).await?;

            if let Err(e) =
                group_cache::set_groups_in_cache(redis_pool.get_ref(), current_user.org_id, &groups)
                    .await
            {
                tracing::warn!("Failed to cache user groups: {}", e);
            }

            groups
        }
    };

    Ok(HttpResponse::Ok().json(groups))
}

/// POST /api/user-groups - Create a new group
pub async fn create_group(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    body: web::Json<CreateUserGroup>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "Group name cannot be empty".to_string(),
        ));
    }

    validate_handle(&body.handle)?;

    let group = UserGroup::create(
        pool.get_ref(),
        current_user.org_id,
        current_user.id,
        body.into_inner(),
    )
    .await?;

    if let Err(e) =
        group_cache::invalidate_groups_cache(redis_pool.get_ref(), current_user.org_id).await
    {
        tracing::warn!("Failed to invalidate user groups cache: {}", e);
    }

    Ok(HttpResponse::Created().json(group))
}

/// GET /api/user-groups/{id} - Get a group by ID
pub async fn get_group(
    pool: web::Data<PgPool>,
    group_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let _current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let group = UserGroup::get_by_id(pool.get_ref(), *group_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found".to_string()))?;

    Ok(HttpResponse::Ok().json(group))
}

/// PUT /api/user-groups/{id} - Update a group
pub async fn update_group(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    group_id: web::Path<Uuid>,
    body: web::Json<UpdateUserGroup>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    if let Some(ref handle) = body.handle {
        validate_handle(handle)?;
    }

    let group = UserGroup::update(pool.get_ref(), *group_id, body.into_inner())
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found".to_string()))?;

    if let Err(e) =
        group_cache::invalidate_groups_cache(redis_pool.get_ref(), current_user.org_id).await
    {
        tracing::warn!("Failed to invalidate user groups cache: {}", e);
    }

    Ok(HttpResponse::Ok().json(group))
}

/// DELETE /api/user-groups/{id} - Delete a group
pub async fn delete_group(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    group_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let deleted = UserGroup::delete(pool.get_ref(), *group_id).await?;
    if !deleted {
        return Err(ApiError::NotFound("Group not found".to_string()));
    }

    if let Err(e) =
        group_cache::invalidate_groups_cache(redis_pool.get_ref(), current_user.org_id).await
    {
        tracing::warn!("Failed to invalidate user groups cache: {}", e);
    }

    Ok(HttpResponse::NoContent().finish())
}

/// GET /api/user-groups/{id}/members - List group members
pub async fn list_members(
    pool: web::Data<PgPool>,
    group_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let _current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let members = UserGroupMember::list_by_group(pool.get_ref(), *group_id).await?;

    Ok(HttpResponse::Ok().json(members))
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
}

/// POST /api/user-groups/{id}/members - Add a member to a group
pub async fn add_member(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    group_id: web::Path<Uuid>,
    body: web::Json<AddMemberRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let member = UserGroupMember::add(pool.get_ref(), *group_id, body.user_id).await?;

    if let Err(e) =
        group_cache::invalidate_groups_cache(redis_pool.get_ref(), current_user.org_id).await
    {
        tracing::warn!("Failed to invalidate user groups cache: {}", e);
    }

    Ok(HttpResponse::Created().json(member))
}

/// DELETE /api/user-groups/{id}/members/{user_id} - Remove a member from a group
pub async fn remove_member(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    path: web::Path<(Uuid, Uuid)>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let (group_id, user_id) = path.into_inner();
    let removed = UserGroupMember::remove(pool.get_ref(), group_id, user_id).await?;
    if !removed {
        return Err(ApiError::NotFound("Member not found in group".to_string()));
    }

    if let Err(e) =
        group_cache::invalidate_groups_cache(redis_pool.get_ref(), current_user.org_id).await
    {
        tracing::warn!("Failed to invalidate user groups cache: {}", e);
    }

    Ok(HttpResponse::NoContent().finish())
}
