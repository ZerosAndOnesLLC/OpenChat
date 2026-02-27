use serde_json::Value as JsonValue;
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::models::channel::{Channel, ChannelMember};
use crate::models::message::Message;
use crate::models::reaction::Reaction;
use crate::models::user::User;
use crate::models::workflow::{Workflow, WorkflowExecution, WorkflowExecutionStep, WorkflowStep};
use crate::websocket::messages::ServerMessage;
use crate::websocket::server::{BroadcastMessage, WsServer};

/// Check if any enabled workflows match the given trigger and execute them.
pub async fn check_triggers(
    pool: &PgPool,
    ws_server: &actix::Addr<WsServer>,
    org_id: Uuid,
    trigger_type: &str,
    trigger_data: JsonValue,
) {
    let workflows = match Workflow::find_enabled_by_trigger(pool, org_id, trigger_type).await {
        Ok(w) => w,
        Err(e) => {
            error!("Failed to find workflows for trigger {}: {}", trigger_type, e);
            return;
        }
    };

    for workflow in workflows {
        if !matches_trigger_config(&workflow.trigger_config, &trigger_data) {
            continue;
        }

        let pool = pool.clone();
        let ws = ws_server.clone();
        let td = trigger_data.clone();

        tokio::spawn(async move {
            if let Err(e) = execute_workflow(&pool, &ws, &workflow, td).await {
                error!(
                    workflow_id = %workflow.id,
                    workflow_name = %workflow.name,
                    "Workflow execution failed: {}",
                    e
                );
            }
        });
    }
}

/// Check if the trigger_data matches the workflow's trigger_config filters.
fn matches_trigger_config(config: &JsonValue, data: &JsonValue) -> bool {
    let config_obj = match config.as_object() {
        Some(obj) => obj,
        None => return true, // No config = match all
    };

    // Channel filter
    if let Some(channel_id) = config_obj.get("channel_id").and_then(|v| v.as_str()) {
        let data_channel = data.get("channel_id").and_then(|v| v.as_str()).unwrap_or("");
        if !channel_id.is_empty() && channel_id != data_channel {
            return false;
        }
    }

    // Keyword filter (for message_posted)
    if let Some(keyword) = config_obj.get("keyword").and_then(|v| v.as_str()) {
        if !keyword.is_empty() {
            let content = data.get("content").and_then(|v| v.as_str()).unwrap_or("");
            if !content.to_lowercase().contains(&keyword.to_lowercase()) {
                return false;
            }
        }
    }

    // Emoji filter (for reaction_added)
    if let Some(emoji) = config_obj.get("emoji").and_then(|v| v.as_str()) {
        if !emoji.is_empty() {
            let data_emoji = data.get("emoji").and_then(|v| v.as_str()).unwrap_or("");
            if emoji != data_emoji {
                return false;
            }
        }
    }

    // Command name filter (for slash_command)
    if let Some(command_name) = config_obj.get("command_name").and_then(|v| v.as_str()) {
        if !command_name.is_empty() {
            let data_command = data.get("command_name").and_then(|v| v.as_str()).unwrap_or("");
            if command_name != data_command {
                return false;
            }
        }
    }

    true
}

/// Execute a workflow: create execution record, run steps sequentially.
async fn execute_workflow(
    pool: &PgPool,
    ws_server: &actix::Addr<WsServer>,
    workflow: &Workflow,
    trigger_data: JsonValue,
) -> Result<(), String> {
    info!(
        workflow_id = %workflow.id,
        workflow_name = %workflow.name,
        "Starting workflow execution"
    );

    let steps = WorkflowStep::list_by_workflow(pool, workflow.id)
        .await
        .map_err(|e| format!("Failed to load workflow steps: {}", e))?;

    if steps.is_empty() {
        warn!(workflow_id = %workflow.id, "Workflow has no steps, skipping");
        return Ok(());
    }

    let execution = WorkflowExecution::create(pool, workflow.id, &trigger_data)
        .await
        .map_err(|e| format!("Failed to create execution: {}", e))?;

    let exec_steps = WorkflowExecutionStep::create_batch(pool, execution.id, &steps)
        .await
        .map_err(|e| format!("Failed to create execution steps: {}", e))?;

    // Broadcast execution started
    ws_server.do_send(BroadcastMessage {
        org_id: workflow.org_id,
        channel_id: trigger_data.get("channel_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()),
        message: ServerMessage::WorkflowExecutionStarted {
            workflow_id: workflow.id,
            execution_id: execution.id,
            workflow_name: workflow.name.clone(),
            channel_id: trigger_data.get("channel_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()),
        },
    });

    // Build context that accumulates step outputs
    let mut context = serde_json::json!({
        "trigger": trigger_data,
        "steps": {},
    });

    // Execute steps sequentially
    for (i, (step, exec_step)) in steps.iter().zip(exec_steps.iter()).enumerate() {
        let input = serde_json::json!({
            "action_type": step.action_type,
            "action_config": step.action_config,
            "context": context,
        });

        WorkflowExecutionStep::set_running(pool, exec_step.id, &input)
            .await
            .map_err(|e| format!("Failed to set step running: {}", e))?;

        // Interpolate config values
        let interpolated_config = interpolate_config(&step.action_config, &context);

        match execute_step_action(pool, ws_server, workflow.org_id, &step.action_type, &interpolated_config).await {
            Ok(output) => {
                WorkflowExecutionStep::set_completed(pool, exec_step.id, &output)
                    .await
                    .map_err(|e| format!("Failed to set step completed: {}", e))?;

                // Add output to context
                context["steps"][i.to_string()] = output;
            }
            Err(err) => {
                let _ = WorkflowExecutionStep::set_failed(pool, exec_step.id, &err).await;
                let _ = WorkflowExecution::set_failed(pool, execution.id, &err).await;

                // Broadcast execution failed
                ws_server.do_send(BroadcastMessage {
                    org_id: workflow.org_id,
                    channel_id: None,
                    message: ServerMessage::WorkflowExecutionCompleted {
                        workflow_id: workflow.id,
                        execution_id: execution.id,
                        workflow_name: workflow.name.clone(),
                        status: "failed".to_string(),
                        error_message: Some(err.clone()),
                    },
                });

                return Err(err);
            }
        }
    }

    WorkflowExecution::set_completed(pool, execution.id)
        .await
        .map_err(|e| format!("Failed to set execution completed: {}", e))?;

    // Broadcast execution completed
    ws_server.do_send(BroadcastMessage {
        org_id: workflow.org_id,
        channel_id: None,
        message: ServerMessage::WorkflowExecutionCompleted {
            workflow_id: workflow.id,
            execution_id: execution.id,
            workflow_name: workflow.name.clone(),
            status: "completed".to_string(),
            error_message: None,
        },
    });

    info!(
        workflow_id = %workflow.id,
        execution_id = %execution.id,
        "Workflow execution completed successfully"
    );

    Ok(())
}

/// Interpolate `{{path.to.value}}` templates in action config using context.
fn interpolate_config(config: &JsonValue, context: &JsonValue) -> JsonValue {
    match config {
        JsonValue::String(s) => {
            let result = interpolate_string(s, context);
            JsonValue::String(result)
        }
        JsonValue::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                new_map.insert(k.clone(), interpolate_config(v, context));
            }
            JsonValue::Object(new_map)
        }
        JsonValue::Array(arr) => {
            JsonValue::Array(arr.iter().map(|v| interpolate_config(v, context)).collect())
        }
        other => other.clone(),
    }
}

/// Replace `{{path.to.value}}` in a string with values from context.
fn interpolate_string(template: &str, context: &JsonValue) -> String {
    let mut result = template.to_string();
    let mut start = 0;

    loop {
        let open = match result[start..].find("{{") {
            Some(pos) => start + pos,
            None => break,
        };
        let close = match result[open..].find("}}") {
            Some(pos) => open + pos,
            None => break,
        };

        let path = &result[open + 2..close].trim();
        let value = resolve_json_path(context, path);
        let replacement = match &value {
            JsonValue::String(s) => s.clone(),
            JsonValue::Null => String::new(),
            other => other.to_string(),
        };

        result = format!("{}{}{}", &result[..open], replacement, &result[close + 2..]);
        start = open + replacement.len();
    }

    result
}

/// Resolve a dot-separated JSON path like "trigger.user_name" from a JSON value.
fn resolve_json_path(value: &JsonValue, path: &str) -> JsonValue {
    let mut current = value;
    for part in path.split('.') {
        current = match current {
            JsonValue::Object(map) => match map.get(part) {
                Some(v) => v,
                None => return JsonValue::Null,
            },
            JsonValue::Array(arr) => match part.parse::<usize>() {
                Ok(idx) => match arr.get(idx) {
                    Some(v) => v,
                    None => return JsonValue::Null,
                },
                Err(_) => return JsonValue::Null,
            },
            _ => return JsonValue::Null,
        };
    }
    current.clone()
}

/// Execute a single step action and return its output data.
async fn execute_step_action(
    pool: &PgPool,
    ws_server: &actix::Addr<WsServer>,
    org_id: Uuid,
    action_type: &str,
    config: &JsonValue,
) -> Result<JsonValue, String> {
    match action_type {
        "send_message" => action_send_message(pool, ws_server, org_id, config).await,
        "add_reaction" => action_add_reaction(pool, config).await,
        "create_channel" => action_create_channel(pool, org_id, config).await,
        "invite_to_channel" => action_invite_to_channel(pool, config).await,
        "update_channel_topic" => action_update_channel_topic(pool, config).await,
        "call_webhook" => action_call_webhook(config).await,
        "delay" => action_delay(config).await,
        _ => Err(format!("Unknown action type: {}", action_type)),
    }
}

async fn action_send_message(
    pool: &PgPool,
    ws_server: &actix::Addr<WsServer>,
    org_id: Uuid,
    config: &JsonValue,
) -> Result<JsonValue, String> {
    let content = config
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or("send_message: missing content")?;

    let channel_id = config
        .get("channel_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    let dm_id = config
        .get("dm_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    let user_id = config
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    let message = if let Some(ch_id) = channel_id {
        // Use a bot/system user or the specified user_id
        let sender_id = user_id.unwrap_or(Uuid::nil());
        Message::create_channel_message(pool, ch_id, sender_id, content, None)
            .await
            .map_err(|e| format!("Failed to create channel message: {}", e))?
    } else if let Some(d_id) = dm_id {
        let sender_id = user_id.unwrap_or(Uuid::nil());
        Message::create_dm_message(pool, d_id, sender_id, content, None)
            .await
            .map_err(|e| format!("Failed to create DM message: {}", e))?
    } else {
        return Err("send_message: channel_id or dm_id required".to_string());
    };

    // Broadcast via WebSocket (workflow messages skip trigger checks — they're already spawned)
    let user_name = if let Some(uid) = user_id {
        User::get_by_id(pool, uid)
            .await
            .ok()
            .flatten()
            .map(|u| u.display_name)
            .unwrap_or_else(|| "Workflow Bot".to_string())
    } else {
        "Workflow Bot".to_string()
    };

    ws_server.do_send(BroadcastMessage {
        org_id,
        channel_id: message.channel_id,
        message: ServerMessage::NewMessage {
            id: message.id,
            channel_id: message.channel_id,
            dm_id: message.dm_id,
            user_id: message.user_id,
            user_name,
            user_avatar: None,
            content: message.content.clone(),
            parent_message_id: None,
            created_at: message.created_at.to_rfc3339(),
            is_webhook: Some(true),
            forwarded_from_message_id: None,
            forwarded_from_channel_id: None,
            forwarded_from_channel_name: None,
        },
    });

    Ok(serde_json::json!({
        "message_id": message.id.to_string(),
        "channel_id": message.channel_id.map(|id| id.to_string()),
        "dm_id": message.dm_id.map(|id| id.to_string()),
    }))
}

async fn action_add_reaction(pool: &PgPool, config: &JsonValue) -> Result<JsonValue, String> {
    let message_id = config
        .get("message_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("add_reaction: missing or invalid message_id")?;

    let emoji = config
        .get("emoji")
        .and_then(|v| v.as_str())
        .ok_or("add_reaction: missing emoji")?;

    let user_id = config
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());

    Reaction::add(pool, message_id, user_id, emoji)
        .await
        .map_err(|e| format!("Failed to add reaction: {}", e))?;

    Ok(serde_json::json!({
        "message_id": message_id.to_string(),
        "emoji": emoji,
    }))
}

async fn action_create_channel(
    pool: &PgPool,
    org_id: Uuid,
    config: &JsonValue,
) -> Result<JsonValue, String> {
    let name = config
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("create_channel: missing name")?;

    let description = config.get("description").and_then(|v| v.as_str());

    let channel_type = config
        .get("channel_type")
        .and_then(|v| v.as_str())
        .unwrap_or("public");

    let created_by = config
        .get("created_by")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());

    let channel = Channel::create(pool, org_id, name, description, channel_type, created_by)
        .await
        .map_err(|e| format!("Failed to create channel: {}", e))?;

    // Add creator as member
    let _ = ChannelMember::add(pool, channel.id, created_by, "admin").await;

    Ok(serde_json::json!({
        "channel_id": channel.id.to_string(),
        "name": channel.name,
    }))
}

async fn action_invite_to_channel(pool: &PgPool, config: &JsonValue) -> Result<JsonValue, String> {
    let channel_id = config
        .get("channel_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("invite_to_channel: missing or invalid channel_id")?;

    let user_id = config
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("invite_to_channel: missing or invalid user_id")?;

    let role = config
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("member");

    ChannelMember::add(pool, channel_id, user_id, role)
        .await
        .map_err(|e| format!("Failed to add member to channel: {}", e))?;

    Ok(serde_json::json!({
        "channel_id": channel_id.to_string(),
        "user_id": user_id.to_string(),
    }))
}

async fn action_update_channel_topic(pool: &PgPool, config: &JsonValue) -> Result<JsonValue, String> {
    let channel_id = config
        .get("channel_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("update_channel_topic: missing or invalid channel_id")?;

    let description = config
        .get("description")
        .and_then(|v| v.as_str())
        .ok_or("update_channel_topic: missing description")?;

    Channel::update(pool, channel_id, None, Some(description))
        .await
        .map_err(|e| format!("Failed to update channel topic: {}", e))?;

    Ok(serde_json::json!({
        "channel_id": channel_id.to_string(),
        "description": description,
    }))
}

async fn action_call_webhook(config: &JsonValue) -> Result<JsonValue, String> {
    let url = config
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or("call_webhook: missing url")?;

    let method = config
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("POST");

    let body = config.get("body").cloned().unwrap_or(JsonValue::Null);
    let headers = config.get("headers").and_then(|v| v.as_object());

    let client = reqwest::Client::new();
    let mut request = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        _ => client.post(url),
    };

    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            if let Some(v) = value.as_str() {
                request = request.header(key.as_str(), v);
            }
        }
    }

    if method.to_uppercase() != "GET" && !body.is_null() {
        request = request.json(&body);
    }

    let response = request
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Webhook request failed: {}", e))?;

    let status = response.status().as_u16();
    let response_body = response
        .text()
        .await
        .unwrap_or_default();

    // Truncate response for storage
    let truncated = if response_body.len() > 4096 {
        format!("{}...(truncated)", &response_body[..4096])
    } else {
        response_body
    };

    if !(200..300).contains(&status) {
        return Err(format!("Webhook returned HTTP {}: {}", status, truncated));
    }

    Ok(serde_json::json!({
        "status": status,
        "body": truncated,
    }))
}

async fn action_delay(config: &JsonValue) -> Result<JsonValue, String> {
    let seconds = config
        .get("seconds")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Cap delay at 5 minutes to prevent abuse
    let capped = seconds.min(300);

    if capped > 0 {
        tokio::time::sleep(std::time::Duration::from_secs(capped)).await;
    }

    Ok(serde_json::json!({
        "delayed_seconds": capped,
    }))
}
