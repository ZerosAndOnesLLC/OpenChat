use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::scheduled_message::ScheduledMessage,
    models::user::User,
    services::tv_api::TokenClaims,
};

#[derive(Debug, Deserialize)]
pub struct CreateScheduledMessageRequest {
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub content: String,
    pub parent_message_id: Option<Uuid>,
    pub scheduled_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduledMessageRequest {
    pub content: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

/// POST /api/messages/scheduled
pub async fn create_scheduled_message(
    pool: web::Data<PgPool>,
    body: web::Json<CreateScheduledMessageRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Validate exactly one of channel_id or dm_id
    match (&body.channel_id, &body.dm_id) {
        (Some(_), None) | (None, Some(_)) => {}
        _ => {
            return Err(ApiError::BadRequest(
                "Exactly one of channel_id or dm_id must be provided".to_string(),
            ));
        }
    }

    if body.content.trim().is_empty() {
        return Err(ApiError::BadRequest("Content must not be empty".to_string()));
    }

    if body.scheduled_at <= Utc::now() {
        return Err(ApiError::BadRequest("scheduled_at must be in the future".to_string()));
    }

    let sm = ScheduledMessage::create(
        pool.get_ref(),
        user.org_id,
        user.id,
        body.channel_id,
        body.dm_id,
        &body.content,
        body.parent_message_id,
        body.scheduled_at,
    )
    .await?;

    Ok(HttpResponse::Created().json(sm))
}

/// GET /api/messages/scheduled
pub async fn list_scheduled_messages(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let messages = ScheduledMessage::list_pending_by_user(pool.get_ref(), user.id).await?;

    Ok(HttpResponse::Ok().json(messages))
}

/// PUT /api/messages/scheduled/{id}
pub async fn update_scheduled_message(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateScheduledMessageRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let id = path.into_inner();

    let existing = ScheduledMessage::get_by_id(pool.get_ref(), id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Scheduled message not found".to_string()))?;

    if existing.user_id != user.id {
        return Err(ApiError::Authorization("Not authorized to update this scheduled message".to_string()));
    }

    if existing.sent {
        return Err(ApiError::BadRequest("Cannot update a sent scheduled message".to_string()));
    }

    if let Some(ref scheduled_at) = body.scheduled_at {
        if *scheduled_at <= Utc::now() {
            return Err(ApiError::BadRequest("scheduled_at must be in the future".to_string()));
        }
    }

    let updated = ScheduledMessage::update(
        pool.get_ref(),
        id,
        body.content.as_deref(),
        body.scheduled_at,
    )
    .await?;

    Ok(HttpResponse::Ok().json(updated))
}

/// DELETE /api/messages/scheduled/{id}
pub async fn delete_scheduled_message(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let id = path.into_inner();

    let existing = ScheduledMessage::get_by_id(pool.get_ref(), id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Scheduled message not found".to_string()))?;

    if existing.user_id != user.id {
        return Err(ApiError::Authorization("Not authorized to delete this scheduled message".to_string()));
    }

    if existing.sent {
        return Err(ApiError::BadRequest("Cannot delete a sent scheduled message".to_string()));
    }

    ScheduledMessage::delete(pool.get_ref(), id).await?;

    Ok(HttpResponse::NoContent().finish())
}
