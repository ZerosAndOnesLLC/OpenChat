use serde_json::Value as JsonValue;
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::notification::{CreateNotification, Notification, NotificationType};
use crate::models::reminder::Reminder;
use crate::websocket::pubsub::PubSubEvent;

/// Execute a reminder job: create notification and broadcast via Redis Pub/Sub.
pub async fn execute(
    pool: &PgPool,
    redis: &mut redis::aio::MultiplexedConnection,
    payload: &JsonValue,
) -> Result<(), String> {
    let reminder_id = payload
        .get("reminder_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("Missing or invalid reminder_id in payload")?;

    let reminder = match Reminder::get_by_id(pool, reminder_id)
        .await
        .map_err(|e| format!("Failed to load reminder: {}", e))?
    {
        Some(r) if !r.completed => r,
        _ => {
            info!(id = %reminder_id, "Reminder already completed or not found, skipping");
            return Ok(());
        }
    };

    Reminder::mark_completed(pool, reminder_id)
        .await
        .map_err(|e| format!("Failed to mark reminder as completed: {}", e))?;

    // Create notification
    let notification = Notification::create(
        pool,
        CreateNotification {
            user_id: reminder.user_id,
            notification_type: NotificationType::Reminder,
            message_id: Some(reminder.message_id),
            channel_id: reminder.channel_id,
            dm_id: reminder.dm_id,
        },
    )
    .await
    .map_err(|e| format!("Failed to create notification: {}", e))?;

    // Broadcast ReminderTriggered
    let event = PubSubEvent::ReminderTriggered {
        user_id: reminder.user_id,
        org_id: reminder.org_id,
        reminder_id: reminder.id,
        message_id: reminder.message_id,
        channel_id: reminder.channel_id,
        dm_id: reminder.dm_id,
        message_preview: reminder.message_preview.clone(),
        created_at: notification.created_at.to_rfc3339(),
    };

    let event_json = serde_json::to_string(&event)
        .map_err(|e| format!("Failed to serialize PubSubEvent: {}", e))?;

    if let Err(e) = redis::cmd("PUBLISH")
        .arg("openchat:reminders")
        .arg(&event_json)
        .query_async::<()>(redis)
        .await
    {
        warn!("Failed to publish ReminderTriggered event: {}", e);
    }

    // Broadcast NotificationCountUpdated
    let unread_count = Notification::count_unread_by_user(pool, reminder.user_id)
        .await
        .map_err(|e| format!("Failed to count unread notifications: {}", e))?;

    let count_event = PubSubEvent::NotificationCountUpdated {
        user_id: reminder.user_id,
        org_id: reminder.org_id,
        unread_count: unread_count as i32,
    };

    let count_json = serde_json::to_string(&count_event)
        .map_err(|e| format!("Failed to serialize NotificationCountUpdated: {}", e))?;

    if let Err(e) = redis::cmd("PUBLISH")
        .arg("openchat:notifications")
        .arg(&count_json)
        .query_async::<()>(redis)
        .await
    {
        warn!("Failed to publish NotificationCountUpdated event: {}", e);
    }

    info!(
        reminder_id = %reminder_id,
        user_id = %reminder.user_id,
        "Reminder triggered successfully"
    );

    Ok(())
}
