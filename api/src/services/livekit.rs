use std::time::Duration;

use livekit_api::access_token::{AccessToken, VideoGrants};
use livekit_api::services::room::{CreateRoomOptions, RoomClient};
use tracing::{error, info};
use uuid::Uuid;

use crate::config::LiveKitConfig;
use crate::errors::{ApiError, ApiResult};

pub struct LiveKitService {
    config: LiveKitConfig,
    room_client: RoomClient,
}

impl LiveKitService {
    pub fn new(config: LiveKitConfig) -> Self {
        let room_client = RoomClient::with_api_key(
            &config.url,
            &config.api_key,
            &config.api_secret,
        );
        Self { config, room_client }
    }

    /// Generate a LiveKit room name from org_id and call_id
    pub fn room_name(org_id: Uuid, call_id: Uuid) -> String {
        let org_prefix = &org_id.to_string()[..8];
        format!("oc_{}_{}", org_prefix, call_id)
    }

    /// Create a LiveKit room
    pub async fn create_room(&self, room_name: &str) -> ApiResult<()> {
        let options = CreateRoomOptions {
            empty_timeout: 300,
            max_participants: 50,
            ..Default::default()
        };

        self.room_client
            .create_room(room_name, options)
            .await
            .map_err(|e| {
                error!("Failed to create LiveKit room '{}': {}", room_name, e);
                ApiError::Internal(format!("Failed to create call room: {}", e))
            })?;

        info!("Created LiveKit room: {}", room_name);
        Ok(())
    }

    /// Generate a participant token for joining a room
    pub async fn generate_token(
        &self,
        room_name: &str,
        identity: &str,
        name: &str,
        can_publish: bool,
        can_subscribe: bool,
    ) -> ApiResult<String> {
        let grants = VideoGrants {
            room_join: true,
            room: room_name.to_string(),
            can_publish,
            can_subscribe,
            can_publish_data: true,
            ..Default::default()
        };

        let token = AccessToken::with_api_key(&self.config.api_key, &self.config.api_secret)
            .with_identity(identity)
            .with_name(name)
            .with_grants(grants)
            .with_ttl(Duration::from_secs(24 * 3600))
            .to_jwt()
            .map_err(|e| {
                error!("Failed to generate LiveKit token: {}", e);
                ApiError::Internal(format!("Failed to generate call token: {}", e))
            })?;

        Ok(token)
    }

    /// Delete a LiveKit room (cleanup on call end)
    pub async fn delete_room(&self, room_name: &str) -> ApiResult<()> {
        self.room_client
            .delete_room(room_name)
            .await
            .map_err(|e| {
                error!("Failed to delete LiveKit room '{}': {}", room_name, e);
                ApiError::Internal(format!("Failed to delete call room: {}", e))
            })?;

        info!("Deleted LiveKit room: {}", room_name);
        Ok(())
    }
}
