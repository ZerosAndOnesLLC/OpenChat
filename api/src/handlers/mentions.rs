use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::mention::Mention,
    models::user::User,
    services::tv_api::TokenClaims,
};

#[derive(Debug, Serialize)]
pub struct MentionResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub mentioned_user_id: Option<Uuid>,
    pub mention_type: String,
    pub created_at: String,
}

impl From<Mention> for MentionResponse {
    fn from(mention: Mention) -> Self {
        let mention_type = match mention.mention_type {
            crate::models::mention::MentionType::User => "user",
            crate::models::mention::MentionType::Channel => "channel",
            crate::models::mention::MentionType::Here => "here",
            crate::models::mention::MentionType::Everyone => "everyone",
        };

        Self {
            id: mention.id,
            message_id: mention.message_id,
            mentioned_user_id: mention.mentioned_user_id,
            mention_type: mention_type.to_string(),
            created_at: mention.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MentionsListResponse {
    pub mentions: Vec<MentionResponse>,
    pub total: usize,
}

/// GET /api/mentions - List user's mentions
pub async fn list_mentions(
    pool: web::Data<PgPool>,
    query: web::Query<PaginationQuery>,
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

    // Get mentions for the user
    let mentions = Mention::list_by_user(pool.get_ref(), current_user.id, limit, offset).await?;

    let response = MentionsListResponse {
        total: mentions.len(),
        mentions: mentions.into_iter().map(MentionResponse::from).collect(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/mentions/unread-count - Get count of unread mentions
pub async fn get_unread_mention_count(
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

    let count = Mention::count_unread_by_user(pool.get_ref(), current_user.id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "count": count })))
}

#[derive(Debug, serde::Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
