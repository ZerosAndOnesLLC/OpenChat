use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::channel::ChannelMember,
    models::direct_message::DirectMessage,
    models::message::Message,
    models::reaction::{Reaction, ReactionCount},
    models::user::User,
    services::tv_api::TokenClaims,
};

#[derive(Debug, Deserialize)]
pub struct AddReactionRequest {
    pub emoji: String,
}

#[derive(Debug, Serialize)]
pub struct ReactionResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: String,
}

impl From<Reaction> for ReactionResponse {
    fn from(reaction: Reaction) -> Self {
        Self {
            id: reaction.id,
            message_id: reaction.message_id,
            user_id: reaction.user_id,
            emoji: reaction.emoji,
            created_at: reaction.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ReactionCountResponse {
    pub emoji: String,
    pub count: i64,
    pub user_ids: Vec<Uuid>,
}

impl From<ReactionCount> for ReactionCountResponse {
    fn from(count: ReactionCount) -> Self {
        Self {
            emoji: count.emoji,
            count: count.count,
            user_ids: count.user_ids,
        }
    }
}

/// POST /api/messages/:id/reactions - Add a reaction to a message
pub async fn add_reaction(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<crate::websocket::server::WsServer>>,
    message_id: web::Path<Uuid>,
    body: web::Json<AddReactionRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Validate emoji (basic validation - not empty and reasonable length)
    if body.emoji.trim().is_empty() {
        return Err(ApiError::BadRequest("Emoji cannot be empty".to_string()));
    }
    if body.emoji.len() > 50 {
        return Err(ApiError::BadRequest("Emoji too long".to_string()));
    }

    // Get current user
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Get the message
    let message = Message::get_by_id(pool.get_ref(), *message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    // Verify user has access to the message (either member of channel or participant in DM)
    if let Some(channel_id) = message.channel_id {
        let is_member = ChannelMember::is_member(pool.get_ref(), channel_id, current_user.id).await?;
        if !is_member {
            return Err(ApiError::Authorization(
                "You are not a member of this channel".to_string(),
            ));
        }
    } else if let Some(dm_id) = message.dm_id {
        let is_participant = DirectMessage::is_participant(pool.get_ref(), dm_id, current_user.id).await?;
        if !is_participant {
            return Err(ApiError::Authorization(
                "You are not a participant in this DM".to_string(),
            ));
        }
    } else {
        return Err(ApiError::Internal(
            "Message has neither channel_id nor dm_id".to_string(),
        ));
    }

    // Add the reaction (ON CONFLICT will handle duplicates)
    let reaction = Reaction::add(pool.get_ref(), *message_id, current_user.id, &body.emoji).await?;

    // Fire workflow triggers (fire-and-forget)
    {
        let pool = pool.clone();
        let ws = ws_server.clone();
        let org_id = current_user.org_id;
        let trigger_data = serde_json::json!({
            "user_id": current_user.id.to_string(),
            "user_name": current_user.display_name.clone(),
            "message_id": message_id.to_string(),
            "emoji": body.emoji.clone(),
            "channel_id": message.channel_id.map(|id| id.to_string()),
            "dm_id": message.dm_id.map(|id| id.to_string()),
        });
        tokio::spawn(async move {
            crate::services::workflow_engine::check_triggers(
                pool.get_ref(), ws.get_ref(), org_id, "reaction_added", trigger_data,
            ).await;
        });
    }

    Ok(HttpResponse::Created().json(ReactionResponse::from(reaction)))
}

/// DELETE /api/messages/:id/reactions/:emoji - Remove a reaction from a message
pub async fn remove_reaction(
    pool: web::Data<PgPool>,
    path: web::Path<(Uuid, String)>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let (message_id, emoji) = path.into_inner();

    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get current user
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Verify message exists
    Message::get_by_id(pool.get_ref(), message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    // Remove the reaction (will only remove if user owns it)
    Reaction::remove(pool.get_ref(), message_id, current_user.id, &emoji).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// GET /api/messages/:id/reactions - List all reactions for a message
pub async fn list_reactions(
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

    // Get the message
    let message = Message::get_by_id(pool.get_ref(), *message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    // Verify user has access to the message
    if let Some(channel_id) = message.channel_id {
        let is_member = ChannelMember::is_member(pool.get_ref(), channel_id, current_user.id).await?;
        if !is_member {
            return Err(ApiError::Authorization(
                "You are not a member of this channel".to_string(),
            ));
        }
    } else if let Some(dm_id) = message.dm_id {
        let is_participant = DirectMessage::is_participant(pool.get_ref(), dm_id, current_user.id).await?;
        if !is_participant {
            return Err(ApiError::Authorization(
                "You are not a participant in this DM".to_string(),
            ));
        }
    }

    // Get reactions
    let reactions = Reaction::list_by_message(pool.get_ref(), *message_id).await?;
    let response: Vec<ReactionResponse> = reactions.into_iter().map(ReactionResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/messages/:id/reactions/counts - Get reaction counts for a message
pub async fn get_reaction_counts(
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

    // Get the message
    let message = Message::get_by_id(pool.get_ref(), *message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    // Verify user has access to the message
    if let Some(channel_id) = message.channel_id {
        let is_member = ChannelMember::is_member(pool.get_ref(), channel_id, current_user.id).await?;
        if !is_member {
            return Err(ApiError::Authorization(
                "You are not a member of this channel".to_string(),
            ));
        }
    } else if let Some(dm_id) = message.dm_id {
        let is_participant = DirectMessage::is_participant(pool.get_ref(), dm_id, current_user.id).await?;
        if !is_participant {
            return Err(ApiError::Authorization(
                "You are not a participant in this DM".to_string(),
            ));
        }
    }

    // Get reaction counts
    let counts = Reaction::count_by_message(pool.get_ref(), *message_id).await?;
    let response: Vec<ReactionCountResponse> = counts.into_iter().map(ReactionCountResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}
