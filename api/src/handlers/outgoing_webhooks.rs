use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::outgoing_webhook::{OutgoingWebhook, WebhookDelivery},
    models::user::User,
    services::tv_api::TokenClaims,
};

#[derive(Debug, Deserialize)]
pub struct CreateOutgoingWebhookRequest {
    pub name: String,
    pub url: String,
    pub event_types: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOutgoingWebhookRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub event_types: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// GET /api/webhooks/outgoing
pub async fn list_webhooks(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let webhooks = OutgoingWebhook::list_by_org(pool.get_ref(), claims.org_id).await?;

    Ok(HttpResponse::Ok().json(webhooks))
}

/// POST /api/webhooks/outgoing
pub async fn create_webhook(
    pool: web::Data<PgPool>,
    body: web::Json<CreateOutgoingWebhookRequest>,
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

    let webhook = OutgoingWebhook::create(
        pool.get_ref(),
        claims.org_id,
        &body.name,
        &body.url,
        &body.event_types,
        current_user.id,
    )
    .await?;

    Ok(HttpResponse::Created().json(webhook))
}

/// GET /api/webhooks/outgoing/{id}
pub async fn get_webhook(
    pool: web::Data<PgPool>,
    webhook_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let webhook = OutgoingWebhook::get_by_id(pool.get_ref(), *webhook_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    if webhook.org_id != claims.org_id {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(webhook))
}

/// PUT /api/webhooks/outgoing/{id}
pub async fn update_webhook(
    pool: web::Data<PgPool>,
    webhook_id: web::Path<Uuid>,
    body: web::Json<UpdateOutgoingWebhookRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let existing = OutgoingWebhook::get_by_id(pool.get_ref(), *webhook_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    if existing.org_id != claims.org_id {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    let webhook = OutgoingWebhook::update(
        pool.get_ref(),
        *webhook_id,
        body.name.as_deref(),
        body.url.as_deref(),
        body.event_types.as_deref(),
        body.enabled,
    )
    .await?;

    Ok(HttpResponse::Ok().json(webhook))
}

/// DELETE /api/webhooks/outgoing/{id}
pub async fn delete_webhook(
    pool: web::Data<PgPool>,
    webhook_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let existing = OutgoingWebhook::get_by_id(pool.get_ref(), *webhook_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    if existing.org_id != claims.org_id {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    OutgoingWebhook::delete(pool.get_ref(), *webhook_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// GET /api/webhooks/outgoing/{id}/deliveries
pub async fn list_deliveries(
    pool: web::Data<PgPool>,
    webhook_id: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let webhook = OutgoingWebhook::get_by_id(pool.get_ref(), *webhook_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    if webhook.org_id != claims.org_id {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let deliveries =
        WebhookDelivery::list_by_webhook(pool.get_ref(), *webhook_id, limit, offset).await?;

    Ok(HttpResponse::Ok().json(deliveries))
}
