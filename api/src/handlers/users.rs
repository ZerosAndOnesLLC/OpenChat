use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::user::User,
    services::tv_api::TokenClaims,
};

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            org_id: user.org_id,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            status: user.status,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}

/// GET /api/users - List all users in the organization
pub async fn list_users(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Get claims from request extensions (set by auth middleware)
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // List users in the same org (RLS handles filtering)
    let users = User::list_by_org(pool.get_ref(), claims.org_id).await?;

    let response: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/users/:id - Get user profile
pub async fn get_user(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<redis::aio::MultiplexedConnection>,
    user_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Verify authentication
    let _claims = req
        .extensions()
        .get::<TokenClaims>()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?
        .clone();

    // Try to get user from cache first
    let mut redis = redis_conn.get_ref().clone();
    let cached_user = crate::cache::users::get_user_from_cache(&mut redis, *user_id).await?;

    let user = match cached_user {
        Some(user) => user,
        None => {
            // Cache miss - get from database
            let user = User::get_by_id(pool.get_ref(), *user_id)
                .await?
                .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

            // Store in cache for next time
            crate::cache::users::set_user_in_cache(&mut redis, &user).await?;

            user
        }
    };

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}

/// PUT /api/users/:id - Update user profile
pub async fn update_user(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<redis::aio::MultiplexedConnection>,
    user_id: web::Path<Uuid>,
    body: web::Json<UpdateUserRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Only allow users to update their own profile
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    if current_user.id != *user_id {
        return Err(ApiError::Authorization(
            "You can only update your own profile".to_string(),
        ));
    }

    // Update user fields
    let mut updated_user = current_user;

    if let Some(display_name) = &body.display_name {
        updated_user.display_name = display_name.clone();
    }

    if let Some(avatar_url) = &body.avatar_url {
        updated_user.avatar_url = Some(avatar_url.clone());
    }

    // Save to database
    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET display_name = $1, avatar_url = $2, updated_at = NOW()
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(&updated_user.display_name)
    .bind(&updated_user.avatar_url)
    .bind(*user_id)
    .fetch_one(pool.get_ref())
    .await?;

    // Invalidate user cache
    let mut redis = redis_conn.get_ref().clone();
    crate::cache::users::invalidate_user_cache(&mut redis, *user_id).await?;

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}

/// PUT /api/users/:id/status - Update user status
pub async fn update_user_status(
    pool: web::Data<PgPool>,
    user_id: web::Path<Uuid>,
    body: web::Json<UpdateStatusRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Only allow users to update their own status
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    if current_user.id != *user_id {
        return Err(ApiError::Authorization(
            "You can only update your own status".to_string(),
        ));
    }

    // Validate status
    if !["online", "offline", "away"].contains(&body.status.as_str()) {
        return Err(ApiError::BadRequest(
            "Status must be 'online', 'offline', or 'away'".to_string(),
        ));
    }

    // Update status
    let user = User::update_status(pool.get_ref(), *user_id, &body.status).await?;

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}
