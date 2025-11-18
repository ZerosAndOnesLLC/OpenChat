use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    cache::notifications as notif_cache,
    errors::{ApiError, ApiResult},
    models::notification::Notification,
    models::user::User,
    services::tv_api::TokenClaims,
};

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: String,
    pub message_id: Option<Uuid>,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub read: bool,
    pub created_at: String,
}

impl From<Notification> for NotificationResponse {
    fn from(notification: Notification) -> Self {
        let notification_type = match notification.notification_type {
            crate::models::notification::NotificationType::Mention => "mention",
            crate::models::notification::NotificationType::Dm => "dm",
            crate::models::notification::NotificationType::ThreadReply => "thread_reply",
            crate::models::notification::NotificationType::ChannelInvite => "channel_invite",
        };

        Self {
            id: notification.id,
            user_id: notification.user_id,
            notification_type: notification_type.to_string(),
            message_id: notification.message_id,
            channel_id: notification.channel_id,
            dm_id: notification.dm_id,
            read: notification.read,
            created_at: notification.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NotificationsListResponse {
    pub notifications: Vec<NotificationResponse>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct NotificationQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub unread_only: Option<bool>,
}

/// GET /api/notifications - List user's notifications
pub async fn list_notifications(
    pool: web::Data<PgPool>,
    query: web::Query<NotificationQueryParams>,
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

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);
    let unread_only = query.unread_only.unwrap_or(false);

    let notifications = if unread_only {
        Notification::list_unread_by_user(pool.get_ref(), current_user.id, limit, offset).await?
    } else {
        Notification::list_by_user(pool.get_ref(), current_user.id, limit, offset).await?
    };

    let response = NotificationsListResponse {
        total: notifications.len(),
        notifications: notifications.into_iter().map(NotificationResponse::from).collect(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/notifications/unread-count - Get count of unread notifications
pub async fn get_unread_count(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
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

    let user_id_str = current_user.id.to_string();

    // Try to get from cache first
    let mut redis_conn = redis.as_ref().clone();
    if let Ok(Some(cached_count)) = notif_cache::get_notification_count_from_cache(&mut redis_conn, &user_id_str).await {
        return Ok(HttpResponse::Ok().json(serde_json::json!({ "count": cached_count })));
    }

    // If not in cache, get from database
    let count = Notification::count_unread_by_user(pool.get_ref(), current_user.id).await?;

    // Cache the result
    let _ = notif_cache::set_notification_count_in_cache(&mut redis_conn, &user_id_str, count as i32).await;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "count": count })))
}

/// POST /api/notifications/:id/read - Mark a notification as read
pub async fn mark_notification_as_read(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    notification_id: web::Path<Uuid>,
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

    // Mark notification as read
    Notification::mark_as_read(pool.get_ref(), *notification_id, current_user.id).await?;

    // Decrement cache count
    let user_id_str = current_user.id.to_string();
    let mut redis_conn = redis.as_ref().clone();
    let _ = notif_cache::decrement_notification_count_in_cache(&mut redis_conn, &user_id_str).await;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "success": true })))
}

/// POST /api/notifications/read-all - Mark all notifications as read
pub async fn mark_all_notifications_as_read(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
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

    // Mark all notifications as read
    let count = Notification::mark_all_as_read(pool.get_ref(), current_user.id).await?;

    // Clear cache count (set to 0)
    let user_id_str = current_user.id.to_string();
    let mut redis_conn = redis.as_ref().clone();
    let _ = notif_cache::set_notification_count_in_cache(&mut redis_conn, &user_id_str, 0).await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "count": count
    })))
}
