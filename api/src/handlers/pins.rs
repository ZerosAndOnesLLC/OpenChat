use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::{
        channel::{Channel, ChannelMember},
        message::Message,
        pin::PinnedMessage,
        user::User,
    },
    services::tv_api::TokenClaims,
    websocket::{
        messages::ServerMessage,
        server::{BroadcastMessage, WsServer},
    },
};

#[derive(Debug, Serialize)]
pub struct PinResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub message_id: Uuid,
    pub pinned_by: Uuid,
    pub pinned_at: String,
}

impl From<PinnedMessage> for PinResponse {
    fn from(pin: PinnedMessage) -> Self {
        Self {
            id: pin.id,
            channel_id: pin.channel_id,
            message_id: pin.message_id,
            pinned_by: pin.pinned_by,
            pinned_at: pin.pinned_at.to_rfc3339(),
        }
    }
}

/// POST /api/messages/{id}/pin - Pin a message
pub async fn pin_message(
    pool: web::Data<PgPool>,
    message_id: web::Path<Uuid>,
    req: HttpRequest,
    ws_server: web::Data<actix::Addr<WsServer>>,
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

    // Get the message
    let message = Message::get_by_id(pool.get_ref(), *message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    // Messages can only be pinned in channels, not DMs
    let channel_id = message
        .channel_id
        .ok_or_else(|| ApiError::BadRequest("Cannot pin messages in DMs".to_string()))?;

    // Verify user is a member of the channel
    let is_member =
        ChannelMember::is_member(pool.get_ref(), channel_id, current_user.id).await?;
    if !is_member {
        return Err(ApiError::Authorization(
            "You are not a member of this channel".to_string(),
        ));
    }

    // TODO: Add permission check (only admins/moderators should be able to pin)
    // For now, any channel member can pin

    // Pin the message
    let pin = PinnedMessage::pin(pool.get_ref(), channel_id, *message_id, current_user.id).await?;

    // Get the channel for org_id
    let channel = Channel::get_by_id(pool.get_ref(), channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Broadcast pin event via WebSocket
    ws_server.do_send(BroadcastMessage {
        org_id: channel.org_id,
        channel_id: Some(channel_id),
        message: ServerMessage::MessagePinned {
            channel_id,
            message_id: *message_id,
            pinned_by: current_user.id,
            pinned_by_name: current_user.display_name.clone(),
            pinned_at: pin.pinned_at.to_rfc3339(),
        },
    });

    Ok(HttpResponse::Ok().json(PinResponse::from(pin)))
}

/// DELETE /api/messages/{id}/pin - Unpin a message
pub async fn unpin_message(
    pool: web::Data<PgPool>,
    message_id: web::Path<Uuid>,
    req: HttpRequest,
    ws_server: web::Data<actix::Addr<WsServer>>,
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

    // Get the message
    let message = Message::get_by_id(pool.get_ref(), *message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    // Messages can only be pinned in channels, not DMs
    let channel_id = message
        .channel_id
        .ok_or_else(|| ApiError::BadRequest("Cannot unpin messages in DMs".to_string()))?;

    // Verify user is a member of the channel
    let is_member =
        ChannelMember::is_member(pool.get_ref(), channel_id, current_user.id).await?;
    if !is_member {
        return Err(ApiError::Authorization(
            "You are not a member of this channel".to_string(),
        ));
    }

    // TODO: Add permission check (only admins/moderators should be able to unpin)
    // For now, any channel member can unpin

    // Unpin the message
    PinnedMessage::unpin(pool.get_ref(), channel_id, *message_id).await?;

    // Get the channel for org_id
    let channel = Channel::get_by_id(pool.get_ref(), channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Broadcast unpin event via WebSocket
    ws_server.do_send(BroadcastMessage {
        org_id: channel.org_id,
        channel_id: Some(channel_id),
        message: ServerMessage::MessageUnpinned {
            channel_id,
            message_id: *message_id,
            unpinned_by: current_user.id,
            unpinned_by_name: current_user.display_name.clone(),
        },
    });

    Ok(HttpResponse::NoContent().finish())
}

/// GET /api/channels/{id}/pins - List all pinned messages in a channel
pub async fn list_channel_pins(
    pool: web::Data<PgPool>,
    channel_id: web::Path<Uuid>,
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

    // Verify channel exists
    Channel::get_by_id(pool.get_ref(), *channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Verify user is a member of the channel
    let is_member =
        ChannelMember::is_member(pool.get_ref(), *channel_id, current_user.id).await?;
    if !is_member {
        return Err(ApiError::Authorization(
            "You are not a member of this channel".to_string(),
        ));
    }

    // Get pins from database
    let pins = PinnedMessage::list_by_channel(pool.get_ref(), *channel_id).await?;
    let response: Vec<PinResponse> = pins.into_iter().map(PinResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}
