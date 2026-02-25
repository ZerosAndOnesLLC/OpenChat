use serde_json::Value as JsonValue;
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::message::Message;
use crate::models::scheduled_message::ScheduledMessage;
use crate::models::user::User;
use crate::websocket::pubsub::PubSubEvent;

/// Execute a scheduled message job: send the message and broadcast via Redis Pub/Sub.
pub async fn execute(
    pool: &PgPool,
    redis: &mut redis::aio::MultiplexedConnection,
    payload: &JsonValue,
) -> Result<(), String> {
    let scheduled_message_id = payload
        .get("scheduled_message_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("Missing or invalid scheduled_message_id in payload")?;

    let sm = match ScheduledMessage::get_by_id(pool, scheduled_message_id)
        .await
        .map_err(|e| format!("Failed to load scheduled message: {}", e))?
    {
        Some(sm) if !sm.sent => sm,
        _ => {
            info!(id = %scheduled_message_id, "Scheduled message already sent or not found, skipping");
            return Ok(());
        }
    };

    let user = User::get_by_id(pool, sm.user_id)
        .await
        .map_err(|e| format!("Failed to load user: {}", e))?
        .ok_or_else(|| format!("User {} not found", sm.user_id))?;

    let message = if let Some(channel_id) = sm.channel_id {
        Message::create_channel_message(pool, channel_id, sm.user_id, &sm.content, sm.parent_message_id)
            .await
            .map_err(|e| format!("Failed to create channel message: {}", e))?
    } else if let Some(dm_id) = sm.dm_id {
        Message::create_dm_message(pool, dm_id, sm.user_id, &sm.content, sm.parent_message_id)
            .await
            .map_err(|e| format!("Failed to create DM message: {}", e))?
    } else {
        return Err("Scheduled message has neither channel_id nor dm_id".to_string());
    };

    ScheduledMessage::mark_sent(pool, scheduled_message_id)
        .await
        .map_err(|e| format!("Failed to mark scheduled message as sent: {}", e))?;

    // Broadcast via Redis Pub/Sub
    let event = PubSubEvent::NewMessage {
        id: message.id,
        channel_id: message.channel_id,
        dm_id: message.dm_id,
        user_id: message.user_id,
        user_name: user.display_name,
        org_id: sm.org_id,
        content: message.content,
        parent_message_id: message.parent_message_id,
        created_at: message.created_at.to_rfc3339(),
    };

    let event_json = serde_json::to_string(&event)
        .map_err(|e| format!("Failed to serialize PubSubEvent: {}", e))?;

    if let Err(e) = redis::cmd("PUBLISH")
        .arg("openchat:messages")
        .arg(&event_json)
        .query_async::<()>(redis)
        .await
    {
        warn!("Failed to publish NewMessage event: {}", e);
    }

    info!(
        scheduled_message_id = %scheduled_message_id,
        message_id = %message.id,
        "Scheduled message sent successfully"
    );

    Ok(())
}
