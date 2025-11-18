use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Send a message to a channel
    SendMessage {
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        content: String,
        parent_message_id: Option<Uuid>,
    },
    /// User is typing in a channel or DM
    Typing {
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
    },
    /// Subscribe to a channel for real-time updates
    SubscribeChannel {
        channel_id: Uuid,
    },
    /// Unsubscribe from a channel
    UnsubscribeChannel {
        channel_id: Uuid,
    },
    /// Update user status
    UpdateStatus {
        status: String, // "online", "offline", "away"
    },
    /// Ping to keep connection alive
    Ping,
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// New message received
    NewMessage {
        id: Uuid,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        user_id: Uuid,
        user_name: String,
        content: String,
        parent_message_id: Option<Uuid>,
        created_at: String,
    },
    /// Message was edited
    MessageEdited {
        message_id: Uuid,
        content: String,
        edited_at: String,
    },
    /// Message was deleted
    MessageDeleted {
        message_id: Uuid,
    },
    /// User is typing
    UserTyping {
        user_id: Uuid,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        user_name: String,
    },
    /// User status changed
    UserStatus {
        user_id: Uuid,
        status: String,
    },
    /// Reaction added to message
    ReactionAdded {
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },
    /// Reaction removed from message
    ReactionRemoved {
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },
    /// Connection established successfully
    Connected {
        user_id: Uuid,
    },
    /// Error message
    Error {
        message: String,
    },
    /// Pong response to ping
    Pong,
    /// Unread count updated for a channel
    UnreadCountUpdated {
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        unread_count: i32,
    },
}
