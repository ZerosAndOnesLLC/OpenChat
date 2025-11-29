use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Connection method for Mattermost migration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MattermostConnection {
    Api {
        server_url: String,
        access_token: String,
    },
    Database {
        connection_string: String,
    },
}

/// Request to validate Mattermost connection
#[derive(Debug, Deserialize)]
pub struct ValidateConnectionRequest {
    pub connection: MattermostConnection,
}

/// Response from connection validation
#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    pub valid: bool,
    pub server_version: Option<String>,
    pub message: Option<String>,
}

/// Request to get migration preview
#[derive(Debug, Deserialize)]
pub struct PreviewRequest {
    pub connection: MattermostConnection,
}

/// Migration preview statistics
#[derive(Debug, Serialize)]
pub struct MigrationPreview {
    pub users: UserPreview,
    pub channels: ChannelPreview,
    pub direct_messages: DmPreview,
    pub messages: MessagePreview,
    pub attachments: AttachmentPreview,
    pub has_message_limit: bool,
    pub message_limit_warning: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserPreview {
    pub total: i64,
    pub will_create: i64,
    pub will_match: i64,
    pub users: Vec<UserMapping>,
}

#[derive(Debug, Serialize)]
pub struct UserMapping {
    pub mattermost_id: String,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub openchat_user_id: Option<Uuid>,
    pub action: UserAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserAction {
    Match,
    Create,
    Skip,
}

#[derive(Debug, Serialize)]
pub struct ChannelPreview {
    pub public_count: i64,
    pub private_count: i64,
    pub channels: Vec<ChannelInfo>,
}

#[derive(Debug, Serialize)]
pub struct ChannelInfo {
    pub mattermost_id: String,
    pub name: String,
    pub display_name: String,
    pub channel_type: String,
    pub member_count: i64,
    pub message_count: i64,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub struct DmPreview {
    pub direct_count: i64,
    pub group_count: i64,
}

#[derive(Debug, Serialize)]
pub struct MessagePreview {
    pub total: i64,
    pub with_attachments: i64,
    pub with_reactions: i64,
}

#[derive(Debug, Serialize)]
pub struct AttachmentPreview {
    pub total: i64,
    pub total_size_bytes: i64,
}

/// Request to start migration
#[derive(Debug, Deserialize)]
pub struct StartMigrationRequest {
    pub connection: MattermostConnection,
    pub options: MigrationOptions,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MigrationOptions {
    pub include_channels: Vec<String>,
    pub include_dms: bool,
    pub include_group_dms: bool,
    pub include_attachments: bool,
    pub user_mappings: Vec<UserMappingOverride>,
    pub batch_size: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserMappingOverride {
    pub mattermost_id: String,
    pub action: UserAction,
    pub openchat_user_id: Option<Uuid>,
}

/// Migration job status
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MigrationJob {
    pub id: Uuid,
    pub org_id: Uuid,
    pub status: MigrationStatus,
    pub progress: serde_json::Value,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MigrationProgress {
    pub phase: String,
    pub users_processed: i64,
    pub users_total: i64,
    pub channels_processed: i64,
    pub channels_total: i64,
    pub dms_processed: i64,
    pub dms_total: i64,
    pub messages_processed: i64,
    pub messages_total: i64,
    pub attachments_processed: i64,
    pub attachments_total: i64,
    pub current_item: Option<String>,
    pub errors: Vec<String>,
}

/// Mattermost data structures (from API/DB)
#[derive(Debug, Clone, Deserialize)]
pub struct MmUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub nickname: Option<String>,
    #[serde(default)]
    pub delete_at: i64,
}

impl MmUser {
    pub fn display_name(&self) -> String {
        if let Some(nick) = &self.nickname {
            if !nick.is_empty() {
                return nick.clone();
            }
        }
        match (&self.first_name, &self.last_name) {
            (Some(f), Some(l)) if !f.is_empty() && !l.is_empty() => format!("{} {}", f, l),
            (Some(f), _) if !f.is_empty() => f.clone(),
            (_, Some(l)) if !l.is_empty() => l.clone(),
            _ => self.username.clone(),
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.delete_at > 0
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MmChannel {
    pub id: String,
    pub team_id: String,
    pub name: String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub channel_type: String,
    pub header: Option<String>,
    pub purpose: Option<String>,
    pub creator_id: String,
    #[serde(default)]
    pub delete_at: i64,
    #[serde(default)]
    pub total_msg_count: i64,
}

impl MmChannel {
    pub fn is_deleted(&self) -> bool {
        self.delete_at > 0
    }

    pub fn is_public(&self) -> bool {
        self.channel_type == "O"
    }

    pub fn is_private(&self) -> bool {
        self.channel_type == "P"
    }

    pub fn is_direct(&self) -> bool {
        self.channel_type == "D"
    }

    pub fn is_group(&self) -> bool {
        self.channel_type == "G"
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MmChannelMember {
    pub channel_id: String,
    pub user_id: String,
    pub roles: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MmPost {
    pub id: String,
    pub channel_id: String,
    pub user_id: String,
    pub message: String,
    pub root_id: Option<String>,
    #[serde(default)]
    pub create_at: i64,
    #[serde(default)]
    pub update_at: i64,
    #[serde(default)]
    pub edit_at: i64,
    #[serde(default)]
    pub delete_at: i64,
    #[serde(default)]
    pub is_pinned: bool,
    pub file_ids: Option<Vec<String>>,
    #[serde(default)]
    pub has_reactions: bool,
    #[serde(rename = "type")]
    pub post_type: Option<String>,
}

impl MmPost {
    pub fn is_deleted(&self) -> bool {
        self.delete_at > 0
    }

    pub fn is_system_message(&self) -> bool {
        self.post_type.as_ref().map_or(false, |t| !t.is_empty())
    }

    pub fn is_reply(&self) -> bool {
        self.root_id.as_ref().map_or(false, |r| !r.is_empty())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MmFileInfo {
    pub id: String,
    pub post_id: String,
    pub name: String,
    pub extension: String,
    pub size: i64,
    pub mime_type: String,
    #[serde(default)]
    pub delete_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MmReaction {
    pub user_id: String,
    pub post_id: String,
    pub emoji_name: String,
    #[serde(default)]
    pub create_at: i64,
}

/// API response wrappers
#[derive(Debug, Deserialize)]
pub struct MmPostList {
    pub order: Vec<String>,
    pub posts: std::collections::HashMap<String, MmPost>,
}

#[derive(Debug, Deserialize)]
pub struct MmChannelMembers(pub Vec<MmChannelMember>);

/// ID mapping for migration tracking
#[derive(Debug, Clone)]
pub struct IdMapping {
    pub mattermost_user_ids: std::collections::HashMap<String, Uuid>,
    pub mattermost_channel_ids: std::collections::HashMap<String, Uuid>,
    pub mattermost_dm_ids: std::collections::HashMap<String, Uuid>,
    pub mattermost_message_ids: std::collections::HashMap<String, Uuid>,
}

impl IdMapping {
    pub fn new() -> Self {
        Self {
            mattermost_user_ids: std::collections::HashMap::new(),
            mattermost_channel_ids: std::collections::HashMap::new(),
            mattermost_dm_ids: std::collections::HashMap::new(),
            mattermost_message_ids: std::collections::HashMap::new(),
        }
    }
}

impl Default for IdMapping {
    fn default() -> Self {
        Self::new()
    }
}
