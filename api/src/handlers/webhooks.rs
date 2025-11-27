use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::Config,
    errors::{ApiError, ApiResult},
    models::channel::Channel,
    models::incoming_webhook::IncomingWebhook,
    models::message::Message,
    models::user::User,
    services::tv_api::TokenClaims,
    websocket::{
        messages::ServerMessage,
        server::{BroadcastMessage, WsServer},
    },
};

// ============================================================================
// Request/Response types for webhook management
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub channel_id: Uuid,
    pub display_name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhookRequest {
    pub channel_id: Option<Uuid>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub username: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub channel_id: Uuid,
    pub display_name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub username: Option<String>,
    pub enabled: bool,
    pub created_by: Uuid,
    pub created_at: String,
    pub updated_at: String,
    /// The webhook URL (only included on create/regenerate)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl WebhookResponse {
    fn from_webhook(webhook: IncomingWebhook, include_url: bool, base_url: &str) -> Self {
        Self {
            id: webhook.id,
            org_id: webhook.org_id,
            channel_id: webhook.channel_id,
            display_name: webhook.display_name,
            description: webhook.description,
            icon_url: webhook.icon_url,
            username: webhook.username,
            enabled: webhook.enabled,
            created_by: webhook.created_by,
            created_at: webhook.created_at.to_rfc3339(),
            updated_at: webhook.updated_at.to_rfc3339(),
            url: if include_url {
                Some(format!("{}/api/hooks/{}", base_url, webhook.token))
            } else {
                None
            },
        }
    }
}

// ============================================================================
// Incoming webhook payload (from external services)
// ============================================================================

/// Mattermost-compatible webhook payload
#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    /// Message text (required)
    pub text: String,
    /// Override the webhook's default username
    pub username: Option<String>,
    /// Override the webhook's default icon
    pub icon_url: Option<String>,
    /// Channel to post to (can override default, but must be in same org)
    pub channel: Option<String>,
}

// ============================================================================
// Webhook Management Endpoints (authenticated)
// ============================================================================

/// GET /api/webhooks/incoming - List all incoming webhooks for the org
pub async fn list_webhooks(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let webhooks = IncomingWebhook::list_by_org(pool.get_ref(), claims.org_id).await?;

    let response: Vec<WebhookResponse> = webhooks
        .into_iter()
        .map(|w| WebhookResponse::from_webhook(w, false, &config.api_base_url))
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/webhooks/incoming - Create a new incoming webhook
pub async fn create_webhook(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    body: web::Json<CreateWebhookRequest>,
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

    // Verify channel exists and belongs to the org
    let channel = Channel::get_by_id(pool.get_ref(), body.channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    if channel.org_id != claims.org_id {
        return Err(ApiError::Authorization(
            "Channel does not belong to your organization".to_string(),
        ));
    }

    let webhook = IncomingWebhook::create(
        pool.get_ref(),
        claims.org_id,
        body.channel_id,
        &body.display_name,
        body.description.as_deref(),
        body.icon_url.as_deref(),
        body.username.as_deref(),
        current_user.id,
    )
    .await?;

    // Return with URL since this is creation
    let response = WebhookResponse::from_webhook(webhook, true, &config.api_base_url);

    Ok(HttpResponse::Created().json(response))
}

/// GET /api/webhooks/incoming/{id} - Get a specific webhook
pub async fn get_webhook(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    webhook_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let webhook = IncomingWebhook::get_by_id(pool.get_ref(), *webhook_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    // Verify webhook belongs to the user's org
    if webhook.org_id != claims.org_id {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    let response = WebhookResponse::from_webhook(webhook, false, &config.api_base_url);

    Ok(HttpResponse::Ok().json(response))
}

/// PUT /api/webhooks/incoming/{id} - Update a webhook
pub async fn update_webhook(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    webhook_id: web::Path<Uuid>,
    body: web::Json<UpdateWebhookRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get existing webhook
    let existing = IncomingWebhook::get_by_id(pool.get_ref(), *webhook_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    // Verify webhook belongs to the user's org
    if existing.org_id != claims.org_id {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    // If changing channel, verify new channel belongs to org
    if let Some(new_channel_id) = body.channel_id {
        let channel = Channel::get_by_id(pool.get_ref(), new_channel_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

        if channel.org_id != claims.org_id {
            return Err(ApiError::Authorization(
                "Channel does not belong to your organization".to_string(),
            ));
        }
    }

    let webhook = IncomingWebhook::update(
        pool.get_ref(),
        *webhook_id,
        body.display_name.as_deref(),
        body.description.as_deref(),
        body.icon_url.as_deref(),
        body.username.as_deref(),
        body.channel_id,
        body.enabled,
    )
    .await?;

    let response = WebhookResponse::from_webhook(webhook, false, &config.api_base_url);

    Ok(HttpResponse::Ok().json(response))
}

/// DELETE /api/webhooks/incoming/{id} - Delete a webhook
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

    // Get existing webhook
    let existing = IncomingWebhook::get_by_id(pool.get_ref(), *webhook_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    // Verify webhook belongs to the user's org
    if existing.org_id != claims.org_id {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    IncomingWebhook::delete(pool.get_ref(), *webhook_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/webhooks/incoming/{id}/regenerate - Regenerate webhook token
pub async fn regenerate_token(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    webhook_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get existing webhook
    let existing = IncomingWebhook::get_by_id(pool.get_ref(), *webhook_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    // Verify webhook belongs to the user's org
    if existing.org_id != claims.org_id {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    let webhook = IncomingWebhook::regenerate_token(pool.get_ref(), *webhook_id).await?;

    // Return with URL since token was regenerated
    let response = WebhookResponse::from_webhook(webhook, true, &config.api_base_url);

    Ok(HttpResponse::Ok().json(response))
}

// ============================================================================
// Public Webhook Endpoint (receives messages from external services)
// ============================================================================

/// POST /api/hooks/{token} - Receive webhook message (public endpoint)
pub async fn receive_webhook(
    pool: web::Data<PgPool>,
    token: web::Path<String>,
    body: web::Json<WebhookPayload>,
    ws_server: web::Data<actix::Addr<WsServer>>,
) -> ApiResult<HttpResponse> {
    // Look up webhook by token
    let webhook = IncomingWebhook::get_by_token(pool.get_ref(), &token)
        .await?
        .ok_or_else(|| ApiError::NotFound("Invalid webhook token".to_string()))?;

    // Get the channel to verify it still exists
    let channel = Channel::get_by_id(pool.get_ref(), webhook.channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel no longer exists".to_string()))?;

    // Create message in the channel
    // Use the webhook creator's user_id as the sender
    let message = Message::create_channel_message(
        pool.get_ref(),
        webhook.channel_id,
        webhook.created_by,
        &body.text,
        None, // No parent message for webhook posts
    )
    .await?;

    // Use override username/icon if provided, otherwise use webhook defaults
    let display_name = body.username
        .as_ref()
        .or(webhook.username.as_ref())
        .unwrap_or(&webhook.display_name)
        .clone();

    let avatar_url = body.icon_url
        .as_ref()
        .or(webhook.icon_url.as_ref())
        .cloned();

    // Broadcast message via WebSocket
    ws_server.do_send(BroadcastMessage {
        org_id: channel.org_id,
        channel_id: Some(webhook.channel_id),
        message: ServerMessage::NewMessage {
            id: message.id,
            channel_id: Some(webhook.channel_id),
            dm_id: None,
            user_id: webhook.created_by,
            user_name: display_name,
            user_avatar: avatar_url,
            content: body.text.clone(),
            parent_message_id: None,
            created_at: message.created_at.to_rfc3339(),
            is_webhook: Some(true),
        },
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "ok": true,
        "message_id": message.id
    })))
}
