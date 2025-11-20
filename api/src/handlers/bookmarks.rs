use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::{
        bookmark::Bookmark,
        channel::ChannelMember,
        direct_message::DirectMessage,
        message::Message,
        user::User,
    },
    services::tv_api::TokenClaims,
    websocket::{
        messages::ServerMessage,
        server::{BroadcastToUser, WsServer},
    },
};

#[derive(Debug, Serialize)]
pub struct BookmarkResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub message_id: Uuid,
    pub bookmarked_at: String,
}

impl From<Bookmark> for BookmarkResponse {
    fn from(bookmark: Bookmark) -> Self {
        Self {
            id: bookmark.id,
            user_id: bookmark.user_id,
            message_id: bookmark.message_id,
            bookmarked_at: bookmark.bookmarked_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateBookmarkRequest {
    pub message_id: Uuid,
}

/// POST /api/bookmarks - Bookmark a message
pub async fn create_bookmark(
    pool: web::Data<PgPool>,
    body: web::Json<CreateBookmarkRequest>,
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
    let message = Message::get_by_id(pool.get_ref(), body.message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    // Verify user has access to the message
    // Check if it's a channel message
    if let Some(channel_id) = message.channel_id {
        let is_member =
            ChannelMember::is_member(pool.get_ref(), channel_id, current_user.id).await?;
        if !is_member {
            return Err(ApiError::Authorization(
                "You don't have access to this message".to_string(),
            ));
        }
    }
    // Check if it's a DM
    else if let Some(dm_id) = message.dm_id {
        let is_participant =
            DirectMessage::is_participant(pool.get_ref(), dm_id, current_user.id).await?;
        if !is_participant {
            return Err(ApiError::Authorization(
                "You don't have access to this message".to_string(),
            ));
        }
    }

    // Create the bookmark
    let bookmark = Bookmark::create(pool.get_ref(), current_user.id, body.message_id).await?;

    // Send WebSocket notification to the user
    ws_server.do_send(BroadcastToUser {
        org_id: current_user.org_id,
        user_id: current_user.id,
        message: ServerMessage::BookmarkAdded {
            message_id: body.message_id,
            bookmarked_at: bookmark.bookmarked_at.to_rfc3339(),
        },
    });

    Ok(HttpResponse::Ok().json(BookmarkResponse::from(bookmark)))
}

/// DELETE /api/bookmarks/{message_id} - Remove a bookmark
pub async fn delete_bookmark(
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

    // Delete the bookmark
    Bookmark::delete(pool.get_ref(), current_user.id, *message_id).await?;

    // Send WebSocket notification to the user
    ws_server.do_send(BroadcastToUser {
        org_id: current_user.org_id,
        user_id: current_user.id,
        message: ServerMessage::BookmarkRemoved {
            message_id: *message_id,
        },
    });

    Ok(HttpResponse::NoContent().finish())
}

/// GET /api/bookmarks - List all bookmarks for the current user
pub async fn list_bookmarks(
    pool: web::Data<PgPool>,
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

    // Get bookmarks from database
    let bookmarks = Bookmark::list_by_user(pool.get_ref(), current_user.id).await?;
    let response: Vec<BookmarkResponse> = bookmarks.into_iter().map(BookmarkResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}
