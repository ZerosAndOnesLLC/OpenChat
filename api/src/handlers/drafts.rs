use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::{message_draft::MessageDraft, user::User},
    services::tv_api::TokenClaims,
};

#[derive(Debug, Serialize)]
pub struct DraftResponse {
    pub id: String,
    pub channel_id: Option<String>,
    pub dm_id: Option<String>,
    pub content: String,
    pub updated_at: String,
}

impl From<MessageDraft> for DraftResponse {
    fn from(draft: MessageDraft) -> Self {
        Self {
            id: draft.id.to_string(),
            channel_id: draft.channel_id.map(|id| id.to_string()),
            dm_id: draft.dm_id.map(|id| id.to_string()),
            content: draft.content,
            updated_at: draft.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveDraftRequest {
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub content: String,
}

/// POST /api/drafts - Save or update a draft
pub async fn save_draft(
    pool: web::Data<PgPool>,
    body: web::Json<SaveDraftRequest>,
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

    // Validate request
    if body.channel_id.is_none() && body.dm_id.is_none() {
        return Err(ApiError::BadRequest(
            "Either channel_id or dm_id must be provided".to_string(),
        ));
    }

    if body.channel_id.is_some() && body.dm_id.is_some() {
        return Err(ApiError::BadRequest(
            "Only one of channel_id or dm_id can be provided".to_string(),
        ));
    }

    // Validate content is not empty
    if body.content.trim().is_empty() {
        return Err(ApiError::BadRequest("Content cannot be empty".to_string()));
    }

    // Save draft
    let draft = if let Some(channel_id) = body.channel_id {
        MessageDraft::save_for_channel(
            pool.get_ref(),
            current_user.id,
            channel_id,
            body.content.clone(),
        )
        .await?
    } else if let Some(dm_id) = body.dm_id {
        MessageDraft::save_for_dm(pool.get_ref(), current_user.id, dm_id, body.content.clone())
            .await?
    } else {
        return Err(ApiError::BadRequest("Invalid request".to_string()));
    };

    Ok(HttpResponse::Ok().json(DraftResponse::from(draft)))
}

/// GET /api/drafts - Get all drafts for the current user
pub async fn get_all_drafts(
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

    // Get all drafts
    let drafts = MessageDraft::get_all_for_user(pool.get_ref(), current_user.id).await?;
    let response: Vec<DraftResponse> = drafts.into_iter().map(DraftResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/drafts/channel/{channel_id} - Get draft for a specific channel
pub async fn get_channel_draft(
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

    // Get draft
    let draft =
        MessageDraft::get_for_channel(pool.get_ref(), current_user.id, *channel_id).await?;

    match draft {
        Some(draft) => Ok(HttpResponse::Ok().json(DraftResponse::from(draft))),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

/// GET /api/drafts/dm/{dm_id} - Get draft for a specific DM
pub async fn get_dm_draft(
    pool: web::Data<PgPool>,
    dm_id: web::Path<Uuid>,
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

    // Get draft
    let draft = MessageDraft::get_for_dm(pool.get_ref(), current_user.id, *dm_id).await?;

    match draft {
        Some(draft) => Ok(HttpResponse::Ok().json(DraftResponse::from(draft))),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

/// DELETE /api/drafts/channel/{channel_id} - Delete draft for a channel
pub async fn delete_channel_draft(
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

    // Delete draft
    let deleted =
        MessageDraft::delete_for_channel(pool.get_ref(), current_user.id, *channel_id).await?;

    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

/// DELETE /api/drafts/dm/{dm_id} - Delete draft for a DM
pub async fn delete_dm_draft(
    pool: web::Data<PgPool>,
    dm_id: web::Path<Uuid>,
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

    // Delete draft
    let deleted = MessageDraft::delete_for_dm(pool.get_ref(), current_user.id, *dm_id).await?;

    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

/// DELETE /api/drafts - Delete all drafts for the current user
pub async fn delete_all_drafts(
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

    // Delete all drafts
    let count = MessageDraft::delete_all_for_user(pool.get_ref(), current_user.id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "deleted_count": count
    })))
}
