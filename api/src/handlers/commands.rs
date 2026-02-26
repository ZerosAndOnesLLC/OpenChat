use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::message::Message,
    models::notification_pref::{NotificationPref, UpsertNotificationPref},
    models::slash_command::SlashCommand,
    models::user::User,
    services::tv_api::TokenClaims,
    websocket::{
        messages::ServerMessage,
        server::{BroadcastMessage, WsServer},
    },
};

#[derive(Debug, Deserialize)]
pub struct ExecuteCommandRequest {
    pub command: String,
    pub text: String,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ExecuteCommandResponse {
    pub response_type: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_hint: Option<String>,
    pub handler_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommandRequest {
    pub command_name: String,
    pub description: String,
    pub usage_hint: Option<String>,
    pub webhook_url: String,
    pub response_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommandRequest {
    pub description: Option<String>,
    pub usage_hint: Option<String>,
    pub webhook_url: Option<String>,
    pub response_type: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub command_name: String,
    pub description: String,
    pub usage_hint: Option<String>,
    pub handler_type: String,
    pub webhook_url: Option<String>,
    pub response_type: String,
    pub created_by: Uuid,
    pub enabled: bool,
    pub created_at: String,
}

impl From<SlashCommand> for CommandResponse {
    fn from(cmd: SlashCommand) -> Self {
        Self {
            id: cmd.id,
            org_id: cmd.org_id,
            command_name: cmd.command_name,
            description: cmd.description,
            usage_hint: cmd.usage_hint,
            handler_type: cmd.handler_type,
            webhook_url: cmd.webhook_url,
            response_type: cmd.response_type,
            created_by: cmd.created_by,
            enabled: cmd.enabled,
            created_at: cmd.created_at.to_rfc3339(),
        }
    }
}

fn builtin_commands() -> Vec<CommandInfo> {
    vec![
        CommandInfo {
            name: "shrug".to_string(),
            description: "Appends ¯\\_(ツ)_/¯ to your message".to_string(),
            usage_hint: Some("[message]".to_string()),
            handler_type: "builtin".to_string(),
            id: None,
        },
        CommandInfo {
            name: "tableflip".to_string(),
            description: "Appends (╯°□°)╯︵ ┻━┻ to your message".to_string(),
            usage_hint: Some("[message]".to_string()),
            handler_type: "builtin".to_string(),
            id: None,
        },
        CommandInfo {
            name: "me".to_string(),
            description: "Displays action text in italics".to_string(),
            usage_hint: Some("<action>".to_string()),
            handler_type: "builtin".to_string(),
            id: None,
        },
        CommandInfo {
            name: "mute".to_string(),
            description: "Mute the current channel or DM".to_string(),
            usage_hint: None,
            handler_type: "builtin".to_string(),
            id: None,
        },
        CommandInfo {
            name: "unmute".to_string(),
            description: "Unmute the current channel or DM".to_string(),
            usage_hint: None,
            handler_type: "builtin".to_string(),
            id: None,
        },
    ]
}

/// POST /api/commands/execute
pub async fn execute_command(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    body: web::Json<ExecuteCommandRequest>,
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

    let command_name = body.command.trim_start_matches('/').to_lowercase();
    let text = body.text.clone();

    // Try builtin commands first
    match command_name.as_str() {
        "shrug" => {
            let content = if text.is_empty() {
                "¯\\_(ツ)_/¯".to_string()
            } else {
                format!("{} ¯\\_(ツ)_/¯", text)
            };
            return send_in_channel_message(
                pool.get_ref(),
                &ws_server,
                &current_user,
                body.channel_id,
                body.dm_id,
                &content,
            )
            .await;
        }
        "tableflip" => {
            let content = if text.is_empty() {
                "(╯°□°)╯︵ ┻━┻".to_string()
            } else {
                format!("{} (╯°□°)╯︵ ┻━┻", text)
            };
            return send_in_channel_message(
                pool.get_ref(),
                &ws_server,
                &current_user,
                body.channel_id,
                body.dm_id,
                &content,
            )
            .await;
        }
        "me" => {
            if text.is_empty() {
                return Ok(HttpResponse::Ok().json(ExecuteCommandResponse {
                    response_type: "ephemeral".to_string(),
                    content: "Usage: /me <action>".to_string(),
                    message_id: None,
                }));
            }
            let content = format!("*{}*", text);
            return send_in_channel_message(
                pool.get_ref(),
                &ws_server,
                &current_user,
                body.channel_id,
                body.dm_id,
                &content,
            )
            .await;
        }
        "mute" => {
            return handle_mute(pool.get_ref(), &current_user, body.channel_id, body.dm_id, true).await;
        }
        "unmute" => {
            return handle_mute(pool.get_ref(), &current_user, body.channel_id, body.dm_id, false).await;
        }
        _ => {}
    }

    // Try custom webhook command
    let custom_cmd =
        SlashCommand::get_by_org_and_name(pool.get_ref(), current_user.org_id, &command_name)
            .await?;

    match custom_cmd {
        Some(cmd) if cmd.enabled && cmd.handler_type == "webhook" => {
            let webhook_url = cmd.webhook_url.as_deref().ok_or_else(|| {
                ApiError::Internal("Webhook command has no URL configured".to_string())
            })?;

            let payload = serde_json::json!({
                "command": format!("/{}", command_name),
                "text": text,
                "user_id": current_user.id,
                "user_name": current_user.display_name,
                "channel_id": body.channel_id,
                "dm_id": body.dm_id,
            });

            let client = reqwest::Client::new();
            let resp = client
                .post(webhook_url)
                .json(&payload)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
                .map_err(|e| ApiError::Internal(format!("Webhook request failed: {}", e)))?;

            let response_text = resp
                .text()
                .await
                .unwrap_or_else(|_| "No response from webhook".to_string());

            if cmd.response_type == "in_channel" {
                return send_in_channel_message(
                    pool.get_ref(),
                    &ws_server,
                    &current_user,
                    body.channel_id,
                    body.dm_id,
                    &response_text,
                )
                .await;
            }

            Ok(HttpResponse::Ok().json(ExecuteCommandResponse {
                response_type: "ephemeral".to_string(),
                content: response_text,
                message_id: None,
            }))
        }
        Some(cmd) if !cmd.enabled => Err(ApiError::BadRequest(format!(
            "Command /{} is currently disabled",
            command_name
        ))),
        _ => Err(ApiError::NotFound(format!(
            "Unknown command: /{}",
            command_name
        ))),
    }
}

/// GET /api/commands
pub async fn list_commands(
    pool: web::Data<PgPool>,
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

    let mut commands: Vec<CommandInfo> = builtin_commands();

    let custom = SlashCommand::list_by_org(pool.get_ref(), current_user.org_id).await?;
    for cmd in custom {
        if cmd.enabled {
            commands.push(CommandInfo {
                name: cmd.command_name,
                description: cmd.description,
                usage_hint: cmd.usage_hint,
                handler_type: cmd.handler_type,
                id: Some(cmd.id),
            });
        }
    }

    Ok(HttpResponse::Ok().json(commands))
}

/// POST /api/commands
pub async fn create_command(
    pool: web::Data<PgPool>,
    body: web::Json<CreateCommandRequest>,
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

    let command_name = body.command_name.trim_start_matches('/').to_lowercase();

    // Prevent overriding builtins
    let builtin_names: Vec<&str> = vec!["shrug", "tableflip", "me", "mute", "unmute"];
    if builtin_names.contains(&command_name.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Cannot override builtin command: /{}",
            command_name
        )));
    }

    if command_name.is_empty() || command_name.len() > 50 {
        return Err(ApiError::BadRequest(
            "Command name must be 1-50 characters".to_string(),
        ));
    }

    let response_type = body
        .response_type
        .as_deref()
        .unwrap_or("in_channel");

    if !["ephemeral", "in_channel"].contains(&response_type) {
        return Err(ApiError::BadRequest(
            "response_type must be 'ephemeral' or 'in_channel'".to_string(),
        ));
    }

    let command = SlashCommand::create(
        pool.get_ref(),
        current_user.org_id,
        &command_name,
        &body.description,
        body.usage_hint.as_deref(),
        "webhook",
        Some(&body.webhook_url),
        response_type,
        current_user.id,
    )
    .await?;

    Ok(HttpResponse::Created().json(CommandResponse::from(command)))
}

/// PUT /api/commands/{id}
pub async fn update_command(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    body: web::Json<UpdateCommandRequest>,
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

    let cmd = SlashCommand::get_by_id(pool.get_ref(), *id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Command not found".to_string()))?;

    if cmd.org_id != current_user.org_id {
        return Err(ApiError::Authorization(
            "Cannot modify commands from another organization".to_string(),
        ));
    }

    if cmd.handler_type == "builtin" {
        return Err(ApiError::BadRequest(
            "Cannot modify builtin commands".to_string(),
        ));
    }

    if let Some(ref rt) = body.response_type {
        if !["ephemeral", "in_channel"].contains(&rt.as_str()) {
            return Err(ApiError::BadRequest(
                "response_type must be 'ephemeral' or 'in_channel'".to_string(),
            ));
        }
    }

    let updated = SlashCommand::update(
        pool.get_ref(),
        *id,
        body.description.as_deref(),
        body.usage_hint.as_deref(),
        body.webhook_url.as_deref(),
        body.response_type.as_deref(),
        body.enabled,
    )
    .await?;

    Ok(HttpResponse::Ok().json(CommandResponse::from(updated)))
}

/// DELETE /api/commands/{id}
pub async fn delete_command(
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

    let cmd = SlashCommand::get_by_id(pool.get_ref(), *id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Command not found".to_string()))?;

    if cmd.org_id != current_user.org_id {
        return Err(ApiError::Authorization(
            "Cannot delete commands from another organization".to_string(),
        ));
    }

    if cmd.handler_type == "builtin" {
        return Err(ApiError::BadRequest(
            "Cannot delete builtin commands".to_string(),
        ));
    }

    SlashCommand::delete(pool.get_ref(), *id).await?;

    Ok(HttpResponse::NoContent().finish())
}

async fn send_in_channel_message(
    pool: &PgPool,
    ws_server: &web::Data<actix::Addr<WsServer>>,
    user: &User,
    channel_id: Option<Uuid>,
    dm_id: Option<Uuid>,
    content: &str,
) -> ApiResult<HttpResponse> {
    let message = if let Some(ch_id) = channel_id {
        Message::create_channel_message(pool, ch_id, user.id, content, None).await?
    } else if let Some(d_id) = dm_id {
        Message::create_dm_message(pool, d_id, user.id, content, None).await?
    } else {
        return Err(ApiError::BadRequest(
            "channel_id or dm_id is required".to_string(),
        ));
    };

    // Broadcast via WebSocket
    let ws_msg = ServerMessage::NewMessage {
        id: message.id,
        channel_id: message.channel_id,
        dm_id: message.dm_id,
        user_id: user.id,
        user_name: user.display_name.clone(),
        user_avatar: None,
        content: message.content.clone(),
        parent_message_id: None,
        created_at: message.created_at.to_rfc3339(),
        is_webhook: None,
    };

    ws_server.do_send(BroadcastMessage {
        org_id: user.org_id,
        channel_id: message.channel_id,
        message: ws_msg,
    });

    Ok(HttpResponse::Ok().json(ExecuteCommandResponse {
        response_type: "in_channel".to_string(),
        content: message.content,
        message_id: Some(message.id),
    }))
}

async fn handle_mute(
    pool: &PgPool,
    user: &User,
    channel_id: Option<Uuid>,
    dm_id: Option<Uuid>,
    mute: bool,
) -> ApiResult<HttpResponse> {
    let data = if mute {
        UpsertNotificationPref {
            preference: "nothing".to_string(),
            mute_until: None,
        }
    } else {
        UpsertNotificationPref {
            preference: "all".to_string(),
            mute_until: None,
        }
    };

    if let Some(ch_id) = channel_id {
        NotificationPref::upsert_channel(pool, user.id, ch_id, data).await?;
    } else if let Some(d_id) = dm_id {
        NotificationPref::upsert_dm(pool, user.id, d_id, data).await?;
    } else {
        return Err(ApiError::BadRequest(
            "channel_id or dm_id is required for mute/unmute".to_string(),
        ));
    }

    let action = if mute { "muted" } else { "unmuted" };
    Ok(HttpResponse::Ok().json(ExecuteCommandResponse {
        response_type: "ephemeral".to_string(),
        content: format!("Channel {}", action),
        message_id: None,
    }))
}
