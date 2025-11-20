// Redis Pub/Sub for cross-server WebSocket event distribution
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Pub/Sub channels
pub const CHANNEL_EVENTS: &str = "openchat:events:channels";
pub const MESSAGE_EVENTS: &str = "openchat:events:messages";
pub const PRESENCE_EVENTS: &str = "openchat:events:presence";
pub const PIN_EVENTS: &str = "openchat:events:pins";
pub const BOOKMARK_EVENTS: &str = "openchat:events:bookmarks";

/// Events that can be published across servers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PubSubEvent {
    /// A new message was created
    MessageCreated {
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        message_id: Uuid,
        user_id: Uuid,
    },
    /// A message was pinned
    MessagePinned {
        channel_id: Uuid,
        message_id: Uuid,
        user_id: Uuid,
    },
    /// A message was unpinned
    MessageUnpinned {
        channel_id: Uuid,
        message_id: Uuid,
        user_id: Uuid,
    },
    /// A bookmark was added
    BookmarkAdded {
        user_id: Uuid,
        message_id: Uuid,
    },
    /// A bookmark was removed
    BookmarkRemoved {
        user_id: Uuid,
        message_id: Uuid,
    },
    /// Channel was updated
    ChannelUpdated {
        channel_id: Uuid,
        org_id: Uuid,
    },
    /// Member joined channel
    MemberJoined {
        channel_id: Uuid,
        user_id: Uuid,
        org_id: Uuid,
    },
    /// Member left channel
    MemberLeft {
        channel_id: Uuid,
        user_id: Uuid,
        org_id: Uuid,
    },
    /// User presence changed
    PresenceChanged {
        user_id: Uuid,
        org_id: Uuid,
        status: String,
    },
}

impl PubSubEvent {
    /// Get the channel name for this event type
    pub fn channel(&self) -> &'static str {
        match self {
            PubSubEvent::MessageCreated { .. } => MESSAGE_EVENTS,
            PubSubEvent::MessagePinned { .. } | PubSubEvent::MessageUnpinned { .. } => PIN_EVENTS,
            PubSubEvent::BookmarkAdded { .. } | PubSubEvent::BookmarkRemoved { .. } => BOOKMARK_EVENTS,
            PubSubEvent::ChannelUpdated { .. } | PubSubEvent::MemberJoined { .. } | PubSubEvent::MemberLeft { .. } => CHANNEL_EVENTS,
            PubSubEvent::PresenceChanged { .. } => PRESENCE_EVENTS,
        }
    }
}

/// Publish an event to Redis pub/sub
pub async fn publish_event(
    redis: &mut MultiplexedConnection,
    event: &PubSubEvent,
) -> Result<(), RedisError> {
    let channel = event.channel();
    let payload = serde_json::to_string(event).map_err(|e| {
        RedisError::from((
            redis::ErrorKind::IoError,
            "Serialization error",
            e.to_string(),
        ))
    })?;

    let _: () = redis.publish(channel, payload).await?;
    Ok(())
}

/// Subscribe to a specific event channel
pub async fn subscribe_to_channel(
    redis: &redis::Client,
    channel: &str,
) -> Result<redis::aio::PubSub, RedisError> {
    let mut pubsub = redis.get_async_pubsub().await?;
    pubsub.subscribe(channel).await?;
    Ok(pubsub)
}

/// Subscribe to all event channels
pub async fn subscribe_to_all(
    redis: &redis::Client,
) -> Result<redis::aio::PubSub, RedisError> {
    let mut pubsub = redis.get_async_pubsub().await?;
    pubsub.subscribe(MESSAGE_EVENTS).await?;
    pubsub.subscribe(PRESENCE_EVENTS).await?;
    pubsub.subscribe(PIN_EVENTS).await?;
    pubsub.subscribe(BOOKMARK_EVENTS).await?;
    pubsub.subscribe(CHANNEL_EVENTS).await?;
    Ok(pubsub)
}
