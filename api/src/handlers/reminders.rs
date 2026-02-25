use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::message::Message,
    models::reminder::Reminder,
    models::user::User,
    services::tv_api::TokenClaims,
};

#[derive(Debug, Deserialize)]
pub struct CreateReminderRequest {
    pub message_id: Uuid,
    pub remind_at: DateTime<Utc>,
}

/// POST /api/reminders
pub async fn create_reminder(
    pool: web::Data<PgPool>,
    body: web::Json<CreateReminderRequest>,
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

    if body.remind_at <= Utc::now() {
        return Err(ApiError::BadRequest("remind_at must be in the future".to_string()));
    }

    let message = Message::get_by_id(pool.get_ref(), body.message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    // Truncate content for preview (100 chars)
    let preview = if message.content.len() > 100 {
        format!("{}...", &message.content[..100])
    } else {
        message.content.clone()
    };

    let reminder = Reminder::create(
        pool.get_ref(),
        user.id,
        user.org_id,
        body.message_id,
        message.channel_id,
        message.dm_id,
        body.remind_at,
        &preview,
    )
    .await?;

    Ok(HttpResponse::Created().json(reminder))
}

/// GET /api/reminders
pub async fn list_reminders(
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

    let reminders = Reminder::list_pending_by_user(pool.get_ref(), user.id).await?;

    Ok(HttpResponse::Ok().json(reminders))
}

/// DELETE /api/reminders/{id}
pub async fn delete_reminder(
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

    let existing = Reminder::get_by_id(pool.get_ref(), id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Reminder not found".to_string()))?;

    if existing.user_id != user.id {
        return Err(ApiError::Authorization("Not authorized to delete this reminder".to_string()));
    }

    if existing.completed {
        return Err(ApiError::BadRequest("Cannot delete a completed reminder".to_string()));
    }

    Reminder::delete(pool.get_ref(), id).await?;

    Ok(HttpResponse::NoContent().finish())
}
