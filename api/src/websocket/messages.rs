use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::models::call::ActiveCallInfo;

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
    /// Subscribe to a DM for real-time updates and get DM data
    SubscribeDm {
        dm_id: Uuid,
    },
    /// Unsubscribe from a DM
    UnsubscribeDm {
        dm_id: Uuid,
    },
    /// Update user status
    UpdateStatus {
        status: String, // "online", "offline", "away"
    },
    /// Ping to keep connection alive
    Ping,
    /// Mark a channel or DM as read
    MarkAsRead {
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        last_message_id: Option<Uuid>,
    },
    /// Add a reaction to a message
    AddReaction {
        message_id: Uuid,
        emoji: String,
    },
    /// Remove a reaction from a message
    RemoveReaction {
        message_id: Uuid,
        emoji: String,
    },
    /// Pin a message in a channel
    PinMessage {
        message_id: Uuid,
    },
    /// Unpin a message from a channel
    UnpinMessage {
        message_id: Uuid,
    },
    /// Add a bookmark to a message
    AddBookmark {
        message_id: Uuid,
    },
    /// Remove a bookmark from a message
    RemoveBookmark {
        message_id: Uuid,
    },
    /// Edit a message
    EditMessage {
        message_id: Uuid,
        content: String,
    },
    /// Delete a message
    DeleteMessage {
        message_id: Uuid,
    },
    /// Subscribe to a thread for real-time updates
    SubscribeThread {
        message_id: Uuid,
    },
    /// Unsubscribe from a thread
    UnsubscribeThread {
        message_id: Uuid,
    },
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forwarded_from_message_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forwarded_from_channel_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forwarded_from_channel_name: Option<String>,
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

/// Per-channel/DM notification preference info sent in InitialState
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPrefInfo {
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub preference: String,
    pub mute_until: Option<DateTime<Utc>>,
}

/// Poll option result for real-time vote updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOptionResult {
    pub index: i32,
    pub text: String,
    pub votes: i64,
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
        notification_preferences: Vec<NotificationPrefInfo>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        active_calls: Vec<ActiveCallInfo>,
    },
    /// Complete channel data sent when subscribing to a channel
    ChannelData {
        channel_id: Uuid,
        messages: Vec<MessageWithDetails>,
        pins: Vec<PinnedMessageInfo>,
        members: Vec<ChannelMemberInfo>,
        unread_info: UnreadInfo,
    },
    /// Complete DM data sent when subscribing to a DM
    DmData {
        dm_id: Uuid,
        messages: Vec<MessageWithDetails>,
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
        #[serde(skip_serializing_if = "Option::is_none")]
        forwarded_from_message_id: Option<Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        forwarded_from_channel_id: Option<Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        forwarded_from_channel_name: Option<String>,
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
    /// Reminder triggered for user
    ReminderTriggered {
        reminder_id: Uuid,
        message_id: Uuid,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        message_preview: String,
        created_at: String,
    },
    /// Ephemeral message (only visible to one user, not persisted)
    EphemeralMessage {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_id: Option<Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dm_id: Option<Uuid>,
    },
    /// Poll vote was updated (broadcast to channel/DM subscribers)
    PollVoteUpdated {
        poll_id: Uuid,
        message_id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_id: Option<Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dm_id: Option<Uuid>,
        options: Vec<PollOptionResult>,
        total_votes: i64,
        user_votes: Vec<i32>,
    },
    /// A call was started in a channel or DM
    CallStarted {
        call_id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_id: Option<Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dm_id: Option<Uuid>,
        call_type: String,
        started_by: Uuid,
        started_by_name: String,
        is_huddle: bool,
    },
    /// A call ended
    CallEnded {
        call_id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_id: Option<Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dm_id: Option<Uuid>,
    },
    /// A participant joined a call
    CallParticipantJoined {
        call_id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_id: Option<Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dm_id: Option<Uuid>,
        user_id: Uuid,
        user_name: String,
    },
    /// A participant left a call
    CallParticipantLeft {
        call_id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_id: Option<Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dm_id: Option<Uuid>,
        user_id: Uuid,
        user_name: String,
    },
    /// Incoming call ringing notification (sent to target users)
    CallRinging {
        call_id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_id: Option<Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dm_id: Option<Uuid>,
        call_type: String,
        started_by: Uuid,
        started_by_name: String,
    },
    /// A workflow execution started
    WorkflowExecutionStarted {
        workflow_id: Uuid,
        execution_id: Uuid,
        workflow_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_id: Option<Uuid>,
    },
    /// A workflow form was requested from a user
    FormRequested {
        form_id: Uuid,
        workflow_name: String,
        title: String,
        fields: serde_json::Value,
    },
    /// A workflow execution completed or failed
    WorkflowExecutionCompleted {
        workflow_id: Uuid,
        execution_id: Uuid,
        workflow_name: String,
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_message: Option<String>,
    },
}
