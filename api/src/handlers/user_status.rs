use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    cache,
    errors::{ApiError, ApiResult},
    models::user::User,
    models::user_status::{UpdateStatusRequest, UserStatus},
    services::tv_api::TokenClaims,
    websocket::{
        messages::ServerMessage,
        server::{BroadcastMessage, WsServer},
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStatusResponse {
    pub user_id: Uuid,
    pub status: String,
    pub custom_message: Option<String>,
    pub emoji: Option<String>,
    pub clear_at: Option<String>,
    pub updated_at: String,
}

impl From<UserStatus> for UserStatusResponse {
    fn from(status: UserStatus) -> Self {
        Self {
            user_id: status.user_id,
            status: status.status,
            custom_message: status.custom_message,
            emoji: status.emoji,
            clear_at: status.clear_at.map(|dt| dt.to_rfc3339()),
            updated_at: status.updated_at.to_rfc3339(),
        }
    }
}

/// PUT /api/users/me/status - Update current user's status
pub async fn update_my_status(
    pool: web::Data<PgPool>,
    redis: web::Data<redis::Client>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    body: web::Json<UpdateStatusRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get current user
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Validate status
    if !["online", "offline", "away", "dnd"].contains(&body.status.as_str()) {
        return Err(ApiError::BadRequest(
            "Status must be 'online', 'offline', 'away', or 'dnd'".to_string(),
        ));
    }

    // Calculate clear_at if clear_after_minutes is provided
    let clear_at = body
        .clear_after_minutes
        .and_then(|minutes| {
            if minutes > 0 {
                Some(Utc::now() + Duration::minutes(minutes as i64))
            } else {
                None
            }
        });

    // Update status in database
    let user_status = UserStatus::upsert(
        pool.get_ref(),
        current_user.id,
        &body.status,
        body.custom_message.as_deref(),
        body.emoji.as_deref(),
        clear_at,
    )
    .await?;

    // Invalidate cache
    cache::user_status::invalidate_status(&redis, current_user.id).await;

    // Broadcast status change via WebSocket
    ws_server.do_send(BroadcastMessage {
        org_id: current_user.org_id,
        channel_id: None, // Status updates are org-wide
        message: ServerMessage::StatusUpdate {
            user_id: current_user.id,
            status: user_status.status.clone(),
            custom_message: user_status.custom_message.clone(),
            emoji: user_status.emoji.clone(),
        },
    });

    Ok(HttpResponse::Ok().json(UserStatusResponse::from(user_status)))
}

/// GET /api/users/:id/status - Get user's status
pub async fn get_user_status(
    pool: web::Data<PgPool>,
    redis: web::Data<redis::Client>,
    user_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Verify authentication
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Check cache first
    if let Some(cached_status) = cache::user_status::get_status(&redis, *user_id).await {
        return Ok(HttpResponse::Ok().json(cached_status));
    }

    // Verify user is in same org (RLS will also enforce this)
    let target_user = User::get_by_id(pool.get_ref(), *user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    if target_user.org_id != current_user.org_id {
        return Err(ApiError::Authorization(
            "Cannot view status of users in other organizations".to_string(),
        ));
    }

    // Get status from database
    let user_status = UserStatus::get_by_user_id(pool.get_ref(), *user_id)
        .await?
        .unwrap_or_else(|| UserStatus {
            user_id: *user_id,
            status: "offline".to_string(),
            custom_message: None,
            emoji: None,
            clear_at: None,
            updated_at: Utc::now(),
        });

    // Cache the result
    let response = UserStatusResponse::from(user_status.clone());
    cache::user_status::set_status(&redis, *user_id, &response).await;

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/users/status/active - Get all active users in org
pub async fn get_active_users(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get all active users in the org
    let statuses = UserStatus::get_active_users(pool.get_ref(), claims.org_id).await?;

    let response: Vec<UserStatusResponse> = statuses
        .into_iter()
        .map(UserStatusResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/users/me/status/online - Quick set status to online
pub async fn set_online(
    pool: web::Data<PgPool>,
    redis: web::Data<redis::Client>,
    ws_server: web::Data<actix::Addr<WsServer>>,
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

    let user_status = UserStatus::set_online(pool.get_ref(), current_user.id).await?;

    // Invalidate cache
    cache::user_status::invalidate_status(&redis, current_user.id).await;

    // Broadcast status change
    ws_server.do_send(BroadcastMessage {
        org_id: current_user.org_id,
        channel_id: None, // Status updates are org-wide
        message: ServerMessage::StatusUpdate {
            user_id: current_user.id,
            status: "online".to_string(),
            custom_message: None,
            emoji: None,
        },
    });

    Ok(HttpResponse::Ok().json(UserStatusResponse::from(user_status)))
}

/// POST /api/users/me/status/away - Quick set status to away
pub async fn set_away(
    pool: web::Data<PgPool>,
    redis: web::Data<redis::Client>,
    ws_server: web::Data<actix::Addr<WsServer>>,
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

    let user_status = UserStatus::set_away(pool.get_ref(), current_user.id).await?;

    // Invalidate cache
    cache::user_status::invalidate_status(&redis, current_user.id).await;

    // Broadcast status change
    ws_server.do_send(BroadcastMessage {
        org_id: current_user.org_id,
        channel_id: None, // Status updates are org-wide
        message: ServerMessage::StatusUpdate {
            user_id: current_user.id,
            status: "away".to_string(),
            custom_message: None,
            emoji: None,
        },
    });

    Ok(HttpResponse::Ok().json(UserStatusResponse::from(user_status)))
}

/// POST /api/users/me/status/offline - Quick set status to offline
pub async fn set_offline(
    pool: web::Data<PgPool>,
    redis: web::Data<redis::Client>,
    ws_server: web::Data<actix::Addr<WsServer>>,
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

    let user_status = UserStatus::set_offline(pool.get_ref(), current_user.id).await?;

    // Invalidate cache
    cache::user_status::invalidate_status(&redis, current_user.id).await;

    // Broadcast status change
    ws_server.do_send(BroadcastMessage {
        org_id: current_user.org_id,
        channel_id: None, // Status updates are org-wide
        message: ServerMessage::StatusUpdate {
            user_id: current_user.id,
            status: "offline".to_string(),
            custom_message: None,
            emoji: None,
        },
    });

    Ok(HttpResponse::Ok().json(UserStatusResponse::from(user_status)))
}
