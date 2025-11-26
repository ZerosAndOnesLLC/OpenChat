use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    cache::read_status::{
        invalidate_channel_unread_cache, invalidate_dm_unread_cache,
        set_channel_unread_in_cache, set_dm_unread_in_cache,
    },
    errors::{ApiError, ApiResult},
    models::{
        channel::ChannelMember,
        direct_message::DirectMessage,
        read_status::{ChannelReadStatus, DmReadStatus},
        user::User,
    },
    services::tv_api::TokenClaims,
};

#[derive(Debug, Deserialize)]
pub struct MarkAsReadRequest {
    pub last_message_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct UnreadCountResponse {
    pub unread_count: i32,
    pub last_read_message_id: Option<Uuid>,
}

/// POST /api/channels/{id}/read - Mark a channel as read
pub async fn mark_channel_as_read(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    channel_id: web::Path<Uuid>,
    body: web::Json<MarkAsReadRequest>,
    req: HttpRequest,
    ws_server: web::Data<actix::Addr<crate::websocket::server::WsServer>>,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get the user from the database
    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Verify user is a member of the channel
    let is_member = ChannelMember::is_member(pool.get_ref(), *channel_id, user.id).await?;
    if !is_member {
        return Err(ApiError::Authorization(
            "User is not a member of this channel".to_string(),
        ));
    }

    // Mark the channel as read
    ChannelReadStatus::mark_as_read(pool.get_ref(), user.id, *channel_id, body.last_message_id)
        .await?;

    // Invalidate the cache
    let mut redis_conn = redis.get_ref().clone();
    invalidate_channel_unread_cache(&mut redis_conn, user.id, *channel_id).await?;

    // Get the new unread count and last read message ID, then broadcast via WebSocket
    let unread_count = ChannelReadStatus::get_unread_count(pool.get_ref(), user.id, *channel_id).await?;
    let last_read_message_id = ChannelReadStatus::get_last_read_message_id(pool.get_ref(), user.id, *channel_id).await?;
    ws_server.do_send(crate::websocket::server::BroadcastToUser {
        org_id: user.org_id,
        user_id: user.id,
        message: crate::websocket::messages::ServerMessage::UnreadCountUpdated {
            channel_id: Some(*channel_id),
            dm_id: None,
            unread_count,
            last_read_message_id,
        },
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Channel marked as read"
    })))
}

/// GET /api/channels/{id}/unread - Get unread count for a channel
pub async fn get_channel_unread_count(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    channel_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get the user from the database
    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Verify user is a member of the channel
    let is_member = ChannelMember::is_member(pool.get_ref(), *channel_id, user.id).await?;
    if !is_member {
        return Err(ApiError::Authorization(
            "User is not a member of this channel".to_string(),
        ));
    }

    let mut redis_conn = redis.get_ref().clone();

    // Get from database (we need both unread count and last_read_message_id)
    let unread_count = ChannelReadStatus::get_unread_count(pool.get_ref(), user.id, *channel_id).await?;
    let last_read_message_id = ChannelReadStatus::get_last_read_message_id(pool.get_ref(), user.id, *channel_id).await?;

    // Store unread count in cache
    set_channel_unread_in_cache(&mut redis_conn, user.id, *channel_id, unread_count).await?;

    Ok(HttpResponse::Ok().json(UnreadCountResponse {
        unread_count,
        last_read_message_id,
    }))
}

/// POST /api/dms/{id}/read - Mark a DM as read
pub async fn mark_dm_as_read(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    dm_id: web::Path<Uuid>,
    body: web::Json<MarkAsReadRequest>,
    req: HttpRequest,
    ws_server: web::Data<actix::Addr<crate::websocket::server::WsServer>>,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get the user from the database
    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Verify user is a participant of the DM
    let is_participant = DirectMessage::is_participant(pool.get_ref(), *dm_id, user.id).await?;
    if !is_participant {
        return Err(ApiError::Authorization(
            "User is not a participant of this DM".to_string(),
        ));
    }

    // Mark the DM as read
    DmReadStatus::mark_as_read(pool.get_ref(), user.id, *dm_id, body.last_message_id).await?;

    // Invalidate the cache
    let mut redis_conn = redis.get_ref().clone();
    invalidate_dm_unread_cache(&mut redis_conn, user.id, *dm_id).await?;

    // Get the new unread count and last read message ID, then broadcast via WebSocket
    let unread_count = DmReadStatus::get_unread_count(pool.get_ref(), user.id, *dm_id).await?;
    let last_read_message_id = DmReadStatus::get_last_read_message_id(pool.get_ref(), user.id, *dm_id).await?;
    ws_server.do_send(crate::websocket::server::BroadcastToUser {
        org_id: user.org_id,
        user_id: user.id,
        message: crate::websocket::messages::ServerMessage::UnreadCountUpdated {
            channel_id: None,
            dm_id: Some(*dm_id),
            unread_count,
            last_read_message_id,
        },
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "DM marked as read"
    })))
}

/// GET /api/dms/{id}/unread - Get unread count for a DM
pub async fn get_dm_unread_count(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    dm_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get the user from the database
    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Verify user is a participant of the DM
    let is_participant = DirectMessage::is_participant(pool.get_ref(), *dm_id, user.id).await?;
    if !is_participant {
        return Err(ApiError::Authorization(
            "User is not a participant of this DM".to_string(),
        ));
    }

    let mut redis_conn = redis.get_ref().clone();

    // Get from database (we need both unread count and last_read_message_id)
    let unread_count = DmReadStatus::get_unread_count(pool.get_ref(), user.id, *dm_id).await?;
    let last_read_message_id = DmReadStatus::get_last_read_message_id(pool.get_ref(), user.id, *dm_id).await?;

    // Store unread count in cache
    set_dm_unread_in_cache(&mut redis_conn, user.id, *dm_id, unread_count).await?;

    Ok(HttpResponse::Ok().json(UnreadCountResponse {
        unread_count,
        last_read_message_id,
    }))
}
