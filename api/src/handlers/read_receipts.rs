use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::{
        channel::ChannelMember,
        direct_message::DirectMessage,
        message::Message,
        read_receipt::{MessageReadReceipt, ReadReceiptWithUser},
        user::User,
    },
    services::tv_api::TokenClaims,
    websocket::{
        messages::ServerMessage,
        server::{BroadcastMessage, WsServer},
    },
};

#[derive(Debug, Serialize)]
pub struct ReadReceiptResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub read_at: String,
}

impl From<MessageReadReceipt> for ReadReceiptResponse {
    fn from(receipt: MessageReadReceipt) -> Self {
        Self {
            id: receipt.id,
            message_id: receipt.message_id,
            user_id: receipt.user_id,
            read_at: receipt.read_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ReadReceiptWithUserResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub read_at: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

impl From<ReadReceiptWithUser> for ReadReceiptWithUserResponse {
    fn from(receipt: ReadReceiptWithUser) -> Self {
        Self {
            id: receipt.id,
            message_id: receipt.message_id,
            user_id: receipt.user_id,
            read_at: receipt.read_at.to_rfc3339(),
            display_name: receipt.display_name,
            avatar_url: receipt.avatar_url,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BatchReadReceiptsRequest {
    pub message_ids: Vec<Uuid>,
}

/// POST /api/messages/{id}/read - Record read receipt for a message
pub async fn record_read_receipt(
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

    // Check if user has disabled read receipts
    if current_user.disable_read_receipts {
        return Ok(HttpResponse::NoContent().finish());
    }

    // Get the message to verify it exists and user has access
    let message = Message::get_by_id(pool.get_ref(), *message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    // Verify user has access to the message (either in channel or DM)
    if let Some(channel_id) = message.channel_id {
        // Check if user is a member of the channel
        let is_member =
            ChannelMember::is_member(pool.get_ref(), channel_id, current_user.id).await?;
        if !is_member {
            return Err(ApiError::Authorization(
                "You are not a member of this channel".to_string(),
            ));
        }
    } else if let Some(dm_id) = message.dm_id {
        // Check if user is part of the DM
        let is_participant =
            DirectMessage::is_participant(pool.get_ref(), dm_id, current_user.id).await?;
        if !is_participant {
            return Err(ApiError::Authorization(
                "You are not part of this DM".to_string(),
            ));
        }
    } else {
        return Err(ApiError::BadRequest(
            "Message has no channel or DM".to_string(),
        ));
    }

    // Record the read receipt
    let receipt = MessageReadReceipt::record(pool.get_ref(), *message_id, current_user.id).await?;

    // Broadcast read receipt via WebSocket to the message sender
    // Get the org_id for the broadcast
    let org_id = current_user.org_id;

    ws_server.do_send(BroadcastMessage {
        org_id,
        channel_id: message.channel_id,
        message: ServerMessage::ReadReceipt {
            message_id: *message_id,
            user_id: current_user.id,
            read_at: receipt.read_at.to_rfc3339(),
        },
    });

    Ok(HttpResponse::Ok().json(ReadReceiptResponse::from(receipt)))
}

/// GET /api/messages/{id}/receipts - Get all read receipts for a message
pub async fn get_message_receipts(
    pool: web::Data<PgPool>,
    message_id: web::Path<Uuid>,
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

    // Get the message to verify it exists and user has access
    let message = Message::get_by_id(pool.get_ref(), *message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    // Verify user has access to the message
    if let Some(channel_id) = message.channel_id {
        let is_member =
            ChannelMember::is_member(pool.get_ref(), channel_id, current_user.id).await?;
        if !is_member {
            return Err(ApiError::Authorization(
                "You are not a member of this channel".to_string(),
            ));
        }
    } else if let Some(dm_id) = message.dm_id {
        let is_participant =
            DirectMessage::is_participant(pool.get_ref(), dm_id, current_user.id).await?;
        if !is_participant {
            return Err(ApiError::Authorization(
                "You are not part of this DM".to_string(),
            ));
        }
    } else {
        return Err(ApiError::BadRequest(
            "Message has no channel or DM".to_string(),
        ));
    }

    // Get receipts with user details
    let receipts = MessageReadReceipt::get_with_user_details(pool.get_ref(), *message_id).await?;
    let response: Vec<ReadReceiptWithUserResponse> = receipts
        .into_iter()
        .map(ReadReceiptWithUserResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/read-receipts/batch - Record read receipts for multiple messages at once
pub async fn record_batch_read_receipts(
    pool: web::Data<PgPool>,
    body: web::Json<BatchReadReceiptsRequest>,
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

    // Check if user has disabled read receipts
    if current_user.disable_read_receipts {
        return Ok(HttpResponse::NoContent().finish());
    }

    // Validate that message_ids is not empty
    if body.message_ids.is_empty() {
        return Err(ApiError::BadRequest(
            "message_ids cannot be empty".to_string(),
        ));
    }

    // Record batch read receipts
    let receipts =
        MessageReadReceipt::record_batch(pool.get_ref(), body.message_ids.clone(), current_user.id)
            .await?;

    // Broadcast read receipts via WebSocket
    let org_id = current_user.org_id;
    for receipt in &receipts {
        ws_server.do_send(BroadcastMessage {
            org_id,
            channel_id: None, // Batch operation may span multiple channels
            message: ServerMessage::ReadReceipt {
                message_id: receipt.message_id,
                user_id: current_user.id,
                read_at: receipt.read_at.to_rfc3339(),
            },
        });
    }

    let response: Vec<ReadReceiptResponse> = receipts
        .into_iter()
        .map(ReadReceiptResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(response))
}
