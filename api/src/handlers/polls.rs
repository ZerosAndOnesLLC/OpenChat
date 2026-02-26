use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::message::Message,
    models::poll::{Poll, PollOptionInfo, PollResults},
    models::user::User,
    services::tv_api::TokenClaims,
    websocket::{
        messages::{PollOptionResult as WsPollOptionResult, ServerMessage},
        server::{BroadcastMessage, WsServer},
    },
};

#[derive(Debug, Deserialize)]
pub struct CreatePollRequest {
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub question: String,
    pub options: Vec<String>,
    pub poll_type: Option<String>,
    pub anonymous: Option<bool>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub option_index: i32,
}

#[derive(Debug, Serialize)]
pub struct PollResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub question: String,
    pub options: Vec<PollOptionResultResponse>,
    pub poll_type: String,
    pub anonymous: bool,
    pub total_votes: i64,
    pub user_votes: Vec<i32>,
    pub closed: bool,
    pub expires_at: Option<String>,
    pub created_by: Uuid,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PollOptionResultResponse {
    pub index: i32,
    pub text: String,
    pub votes: i64,
}

impl From<PollResults> for PollResponse {
    fn from(results: PollResults) -> Self {
        Self {
            id: results.poll_id,
            message_id: Uuid::nil(), // Will be set after
            question: results.question,
            options: results
                .options
                .into_iter()
                .map(|o| PollOptionResultResponse {
                    index: o.index,
                    text: o.text,
                    votes: o.votes,
                })
                .collect(),
            poll_type: results.poll_type,
            anonymous: results.anonymous,
            total_votes: results.total_votes,
            user_votes: results.user_votes,
            closed: results.closed,
            expires_at: None,
            created_by: Uuid::nil(),
            created_at: String::new(),
        }
    }
}

/// POST /api/polls
pub async fn create_poll(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    body: web::Json<CreatePollRequest>,
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

    if body.options.len() < 2 {
        return Err(ApiError::BadRequest(
            "Poll must have at least 2 options".to_string(),
        ));
    }
    if body.options.len() > 10 {
        return Err(ApiError::BadRequest(
            "Poll cannot have more than 10 options".to_string(),
        ));
    }
    if body.question.is_empty() {
        return Err(ApiError::BadRequest(
            "Poll question cannot be empty".to_string(),
        ));
    }

    let poll_type = body.poll_type.as_deref().unwrap_or("single");
    if !["single", "multiple"].contains(&poll_type) {
        return Err(ApiError::BadRequest(
            "poll_type must be 'single' or 'multiple'".to_string(),
        ));
    }

    let anonymous = body.anonymous.unwrap_or(false);

    // Create the message first
    let content = format!("\u{1F4CA} **Poll:** {}", body.question);
    let message = if let Some(ch_id) = body.channel_id {
        Message::create_channel_message(pool.get_ref(), ch_id, current_user.id, &content, None)
            .await?
    } else if let Some(d_id) = body.dm_id {
        Message::create_dm_message(pool.get_ref(), d_id, current_user.id, &content, None).await?
    } else {
        return Err(ApiError::BadRequest(
            "channel_id or dm_id is required".to_string(),
        ));
    };

    // Build options JSONB
    let options_json: Vec<PollOptionInfo> = body
        .options
        .iter()
        .enumerate()
        .map(|(i, text)| PollOptionInfo {
            index: i as i32,
            text: text.clone(),
        })
        .collect();
    let options_value = serde_json::to_value(&options_json)
        .map_err(|e| ApiError::Internal(format!("Failed to serialize options: {}", e)))?;

    let poll = Poll::create(
        pool.get_ref(),
        message.id,
        current_user.org_id,
        &body.question,
        &options_value,
        poll_type,
        anonymous,
        body.expires_at,
        current_user.id,
    )
    .await?;

    // Broadcast message via WebSocket
    let ws_msg = ServerMessage::NewMessage {
        id: message.id,
        channel_id: message.channel_id,
        dm_id: message.dm_id,
        user_id: current_user.id,
        user_name: current_user.display_name.clone(),
        user_avatar: None,
        content: message.content.clone(),
        parent_message_id: None,
        created_at: message.created_at.to_rfc3339(),
        is_webhook: None,
        forwarded_from_message_id: None,
        forwarded_from_channel_id: None,
        forwarded_from_channel_name: None,
    };

    ws_server.do_send(BroadcastMessage {
        org_id: current_user.org_id,
        channel_id: message.channel_id,
        message: ws_msg,
    });

    // Return poll response
    let response = PollResponse {
        id: poll.id,
        message_id: message.id,
        question: poll.question,
        options: options_json
            .into_iter()
            .map(|o| PollOptionResultResponse {
                index: o.index,
                text: o.text,
                votes: 0,
            })
            .collect(),
        poll_type: poll.poll_type,
        anonymous: poll.anonymous,
        total_votes: 0,
        user_votes: vec![],
        closed: false,
        expires_at: poll.expires_at.map(|dt| dt.to_rfc3339()),
        created_by: poll.created_by,
        created_at: poll.created_at.to_rfc3339(),
    };

    Ok(HttpResponse::Created().json(response))
}

/// GET /api/polls/{id}
pub async fn get_poll(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
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

    let poll = Poll::get_by_id(pool.get_ref(), *id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Poll not found".to_string()))?;

    let results = Poll::get_results(pool.get_ref(), poll.id, current_user.id).await?;

    let response = PollResponse {
        id: poll.id,
        message_id: poll.message_id,
        question: results.question,
        options: results
            .options
            .into_iter()
            .map(|o| PollOptionResultResponse {
                index: o.index,
                text: o.text,
                votes: o.votes,
            })
            .collect(),
        poll_type: results.poll_type,
        anonymous: results.anonymous,
        total_votes: results.total_votes,
        user_votes: results.user_votes,
        closed: results.closed,
        expires_at: poll.expires_at.map(|dt| dt.to_rfc3339()),
        created_by: poll.created_by,
        created_at: poll.created_at.to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/polls/{id}/vote
pub async fn vote(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    id: web::Path<Uuid>,
    body: web::Json<VoteRequest>,
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

    Poll::vote(pool.get_ref(), *id, current_user.id, body.option_index).await?;

    // Get updated results and broadcast
    let poll = Poll::get_by_id(pool.get_ref(), *id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Poll not found".to_string()))?;

    let results = Poll::get_results(pool.get_ref(), poll.id, current_user.id).await?;

    broadcast_poll_update(&ws_server, &poll, &results);

    let response = PollResponse {
        id: poll.id,
        message_id: poll.message_id,
        question: results.question,
        options: results
            .options
            .into_iter()
            .map(|o| PollOptionResultResponse {
                index: o.index,
                text: o.text,
                votes: o.votes,
            })
            .collect(),
        poll_type: results.poll_type,
        anonymous: results.anonymous,
        total_votes: results.total_votes,
        user_votes: results.user_votes,
        closed: results.closed,
        expires_at: poll.expires_at.map(|dt| dt.to_rfc3339()),
        created_by: poll.created_by,
        created_at: poll.created_at.to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// DELETE /api/polls/{id}/vote
pub async fn remove_vote(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    id: web::Path<Uuid>,
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

    Poll::remove_vote(pool.get_ref(), *id, current_user.id).await?;

    let poll = Poll::get_by_id(pool.get_ref(), *id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Poll not found".to_string()))?;

    let results = Poll::get_results(pool.get_ref(), poll.id, current_user.id).await?;

    broadcast_poll_update(&ws_server, &poll, &results);

    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/polls/{id}/close
pub async fn close_poll(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    id: web::Path<Uuid>,
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

    let poll = Poll::get_by_id(pool.get_ref(), *id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Poll not found".to_string()))?;

    if poll.created_by != current_user.id {
        return Err(ApiError::Authorization(
            "Only the poll creator can close it".to_string(),
        ));
    }

    if poll.closed_at.is_some() {
        return Err(ApiError::BadRequest(
            "Poll is already closed".to_string(),
        ));
    }

    let closed_poll = Poll::close(pool.get_ref(), *id).await?;

    let results = Poll::get_results(pool.get_ref(), closed_poll.id, current_user.id).await?;

    broadcast_poll_update(&ws_server, &closed_poll, &results);

    let response = PollResponse {
        id: closed_poll.id,
        message_id: closed_poll.message_id,
        question: results.question,
        options: results
            .options
            .into_iter()
            .map(|o| PollOptionResultResponse {
                index: o.index,
                text: o.text,
                votes: o.votes,
            })
            .collect(),
        poll_type: results.poll_type,
        anonymous: results.anonymous,
        total_votes: results.total_votes,
        user_votes: results.user_votes,
        closed: true,
        expires_at: closed_poll.expires_at.map(|dt| dt.to_rfc3339()),
        created_by: closed_poll.created_by,
        created_at: closed_poll.created_at.to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

fn broadcast_poll_update(
    ws_server: &web::Data<actix::Addr<WsServer>>,
    poll: &Poll,
    results: &PollResults,
) {
    // Get message channel/dm info from poll's message_id
    let ws_options: Vec<WsPollOptionResult> = results
        .options
        .iter()
        .map(|o| WsPollOptionResult {
            index: o.index,
            text: o.text.clone(),
            votes: o.votes,
        })
        .collect();

    let ws_msg = ServerMessage::PollVoteUpdated {
        poll_id: poll.id,
        message_id: poll.message_id,
        channel_id: None,   // We'll need to look this up
        dm_id: None,
        options: ws_options,
        total_votes: results.total_votes,
        user_votes: results.user_votes.clone(),
    };

    ws_server.do_send(BroadcastMessage {
        org_id: poll.org_id,
        channel_id: None, // Broadcast to org; clients filter by message_id
        message: ws_msg,
    });
}
