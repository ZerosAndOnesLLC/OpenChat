use chrono::{DateTime, Utc};
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

/// Channel metadata for initial state
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelMetadata {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub channel_type: String,
    pub unread_count: i32,
    pub last_message_preview: Option<String>,
    pub last_message_at: Option<DateTime<Utc>>,
}

/// Direct message metadata for initial state
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DmMetadata {
    pub id: Uuid,
    pub other_user_id: Uuid,
    pub other_user_name: String,
    pub unread_count: i32,
    pub last_message_preview: Option<String>,
    pub last_message_at: Option<DateTime<Utc>>,
}

/// Unread information for a channel or DM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreadInfo {
    pub count: i32,
    pub last_read_message_id: Option<Uuid>,
    pub mentions: i32,
}

/// Message with additional details for channel subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageWithDetails {
    pub id: Uuid,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub user_id: Uuid,
    pub user_name: String,
    pub content: String,
    pub parent_message_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub reply_count: i64,
}

/// Pinned message information for channel subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedMessageInfo {
    pub id: Uuid,
    pub message_id: Uuid,
    pub pinned_by: Uuid,
    pub pinned_at: DateTime<Utc>,
}

/// Channel member information for channel subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMemberInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Initial state sent after connection
    InitialState {
        user_id: Uuid,
        channels: Vec<ChannelMetadata>,
        dms: Vec<DmMetadata>,
    },
    /// Complete channel data sent when subscribing to a channel
    ChannelData {
        channel_id: Uuid,
        messages: Vec<MessageWithDetails>,
        pins: Vec<PinnedMessageInfo>,
        members: Vec<ChannelMemberInfo>,
        unread_info: UnreadInfo,
    },
    /// New message received
    NewMessage {
        id: Uuid,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        user_id: Uuid,
        user_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_avatar: Option<String>,
        content: String,
        parent_message_id: Option<Uuid>,
        created_at: String,
        /// Whether this message was sent via webhook
        #[serde(skip_serializing_if = "Option::is_none")]
        is_webhook: Option<bool>,
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
    /// Advanced user status changed (with custom message and emoji)
    StatusUpdate {
        user_id: Uuid,
        status: String,
        custom_message: Option<String>,
        emoji: Option<String>,
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
        last_read_message_id: Option<Uuid>,
    },
    /// Read receipt recorded for a message
    ReadReceipt {
        message_id: Uuid,
        user_id: Uuid,
        read_at: String,
    },
    /// New notification received
    NewNotification {
        notification_id: Uuid,
        notification_type: String, // "mention", "dm", "thread_reply", "channel_invite"
        message_id: Option<Uuid>,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        created_at: String,
    },
    /// Notification count updated
    NotificationCountUpdated {
        unread_count: i32,
    },
    /// Message was pinned
    MessagePinned {
        channel_id: Uuid,
        message_id: Uuid,
        pinned_by: Uuid,
        pinned_by_name: String,
        pinned_at: String,
    },
    /// Message was unpinned
    MessageUnpinned {
        channel_id: Uuid,
        message_id: Uuid,
        unpinned_by: Uuid,
        unpinned_by_name: String,
    },
    /// Bookmark was added (user-specific)
    BookmarkAdded {
        message_id: Uuid,
        bookmarked_at: String,
    },
    /// Bookmark was removed (user-specific)
    BookmarkRemoved {
        message_id: Uuid,
    },
    /// Channel was updated
    ChannelUpdated {
        channel_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        updated_by: Uuid,
        updated_by_name: String,
    },
    /// Member joined channel
    MemberJoined {
        channel_id: Uuid,
        user_id: Uuid,
        user_name: String,
        role: String,
        joined_at: String,
    },
    /// Member left channel
    MemberLeft {
        channel_id: Uuid,
        user_id: Uuid,
        user_name: String,
    },
}
