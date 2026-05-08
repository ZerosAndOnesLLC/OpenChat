use anyhow::{anyhow, Result};
use chrono::{TimeZone, Utc};
use rand::RngExt;
use reqwest::Client;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use super::mattermost::MattermostClient;
use super::models::*;

const DEFAULT_BATCH_SIZE: i32 = 100;

pub struct MigrationService {
    openchat_pool: PgPool,
    tv_api_url: String,
    tv_access_token: String,
    org_id: Uuid,
}

impl MigrationService {
    pub fn new(
        openchat_pool: PgPool,
        tv_api_url: String,
        tv_access_token: String,
        org_id: Uuid,
    ) -> Self {
        Self {
            openchat_pool,
            tv_api_url,
            tv_access_token,
            org_id,
        }
    }

    /// Validate connection and return server info
    pub async fn validate_connection(&self, connection: &MattermostConnection) -> Result<ValidationResponse> {
        match MattermostClient::new(connection.clone()).await {
            Ok(client) => {
                let version = client.get_server_version().await.unwrap_or_else(|_| "unknown".to_string());
                Ok(ValidationResponse {
                    valid: true,
                    server_version: Some(version),
                    message: Some("Connection successful".to_string()),
                })
            }
            Err(e) => Ok(ValidationResponse {
                valid: false,
                server_version: None,
                message: Some(format!("Connection failed: {}", e)),
            }),
        }
    }

    /// Get migration preview with user mappings
    pub async fn get_preview(&self, connection: &MattermostConnection) -> Result<MigrationPreview> {
        let client = MattermostClient::new(connection.clone()).await?;

        // Get Mattermost data
        let mm_users = client.get_users().await?;
        let mm_channels = client.get_channels().await?;
        let (user_count, _channel_count, _dm_count, message_count) = client.get_stats().await?;
        let (file_count, file_size) = client.get_file_stats().await?;

        // Get existing OpenChat users for matching
        let existing_users = self.get_existing_users_by_email().await?;

        // Build user mappings
        let mut users_will_create = 0i64;
        let mut users_will_match = 0i64;
        let user_mappings: Vec<UserMapping> = mm_users.iter().map(|mm_user| {
            let email_lower = mm_user.email.to_lowercase();
            if let Some(oc_user_id) = existing_users.get(&email_lower) {
                users_will_match += 1;
                UserMapping {
                    mattermost_id: mm_user.id.clone(),
                    email: mm_user.email.clone(),
                    username: mm_user.username.clone(),
                    display_name: Some(mm_user.display_name()),
                    openchat_user_id: Some(*oc_user_id),
                    action: UserAction::Match,
                }
            } else {
                users_will_create += 1;
                UserMapping {
                    mattermost_id: mm_user.id.clone(),
                    email: mm_user.email.clone(),
                    username: mm_user.username.clone(),
                    display_name: Some(mm_user.display_name()),
                    openchat_user_id: None,
                    action: UserAction::Create,
                }
            }
        }).collect();

        // Build channel info
        let public_channels: Vec<_> = mm_channels.iter().filter(|c| c.is_public()).collect();
        let private_channels: Vec<_> = mm_channels.iter().filter(|c| c.is_private()).collect();
        let direct_channels: Vec<_> = mm_channels.iter().filter(|c| c.is_direct()).collect();
        let group_channels: Vec<_> = mm_channels.iter().filter(|c| c.is_group()).collect();

        let mut channel_infos = Vec::new();
        for channel in public_channels.iter().chain(private_channels.iter()) {
            let member_count = client.get_channel_members(&channel.id).await
                .map(|m| m.len() as i64)
                .unwrap_or(0);

            channel_infos.push(ChannelInfo {
                mattermost_id: channel.id.clone(),
                name: channel.name.clone(),
                display_name: channel.display_name.clone(),
                channel_type: if channel.is_public() { "public".to_string() } else { "private".to_string() },
                member_count,
                message_count: channel.total_msg_count,
                selected: true,
            });
        }

        // Check for message limit (API mode only)
        let has_message_limit = matches!(connection, MattermostConnection::Api { .. });
        let message_limit_warning = if has_message_limit && message_count >= 10000 {
            Some("Mattermost free tier may limit message history to 10,000 messages. Consider using database connection for full history.".to_string())
        } else {
            None
        };

        Ok(MigrationPreview {
            users: UserPreview {
                total: user_count,
                will_create: users_will_create,
                will_match: users_will_match,
                users: user_mappings,
            },
            channels: ChannelPreview {
                public_count: public_channels.len() as i64,
                private_count: private_channels.len() as i64,
                channels: channel_infos,
            },
            direct_messages: DmPreview {
                direct_count: direct_channels.len() as i64,
                group_count: group_channels.len() as i64,
            },
            messages: MessagePreview {
                total: message_count,
                with_attachments: file_count,
                with_reactions: 0, // Would need to count
            },
            attachments: AttachmentPreview {
                total: file_count,
                total_size_bytes: file_size,
            },
            has_message_limit,
            message_limit_warning,
        })
    }

    /// Start migration job
    pub async fn start_migration(
        &self,
        connection: MattermostConnection,
        options: MigrationOptions,
        user_id: Uuid,
    ) -> Result<Uuid> {
        let job_id = Uuid::new_v4();

        // Create job record
        sqlx::query(
            r#"
            INSERT INTO migration_jobs (id, org_id, status, progress, started_at, created_by)
            VALUES ($1, $2, 'pending', $3, NOW(), $4)
            "#
        )
        .bind(job_id)
        .bind(self.org_id)
        .bind(serde_json::to_value(MigrationProgress::default())?)
        .bind(user_id)
        .execute(&self.openchat_pool)
        .await?;

        // Spawn background task
        let service = MigrationService {
            openchat_pool: self.openchat_pool.clone(),
            tv_api_url: self.tv_api_url.clone(),
            tv_access_token: self.tv_access_token.clone(),
            org_id: self.org_id,
        };

        tokio::spawn(async move {
            if let Err(e) = service.run_migration(job_id, connection, options).await {
                tracing::error!("Migration job {} failed: {}", job_id, e);
                let _ = service.update_job_status(job_id, MigrationStatus::Failed, Some(e.to_string())).await;
            }
        });

        Ok(job_id)
    }

    /// Run the actual migration
    async fn run_migration(
        &self,
        job_id: Uuid,
        connection: MattermostConnection,
        options: MigrationOptions,
    ) -> Result<()> {
        self.update_job_status(job_id, MigrationStatus::Running, None).await?;

        let client = MattermostClient::new(connection.clone()).await?;
        let _batch_size = options.batch_size.unwrap_or(DEFAULT_BATCH_SIZE);

        let mut progress = MigrationProgress::default();
        let mut id_mapping = IdMapping::new();

        // Phase 1: Users
        progress.phase = "users".to_string();
        self.update_job_progress(job_id, &progress).await?;

        let mm_users = client.get_users().await?;
        progress.users_total = mm_users.len() as i64;

        for mm_user in &mm_users {
            progress.current_item = Some(format!("User: {}", mm_user.email));
            self.update_job_progress(job_id, &progress).await?;

            match self.migrate_user(mm_user, &options.user_mappings).await {
                Ok(oc_user_id) => {
                    id_mapping.mattermost_user_ids.insert(mm_user.id.clone(), oc_user_id);
                }
                Err(e) => {
                    progress.errors.push(format!("User {}: {}", mm_user.email, e));
                }
            }

            progress.users_processed += 1;
        }

        // Phase 2: Channels
        progress.phase = "channels".to_string();
        self.update_job_progress(job_id, &progress).await?;

        let mm_channels = client.get_channels().await?;
        let channels_to_migrate: Vec<_> = mm_channels.iter()
            .filter(|c| {
                (c.is_public() || c.is_private()) &&
                options.include_channels.contains(&c.id)
            })
            .collect();

        progress.channels_total = channels_to_migrate.len() as i64;

        for channel in &channels_to_migrate {
            progress.current_item = Some(format!("Channel: {}", channel.display_name));
            self.update_job_progress(job_id, &progress).await?;

            match self.migrate_channel(&client, channel, &id_mapping).await {
                Ok(oc_channel_id) => {
                    id_mapping.mattermost_channel_ids.insert(channel.id.clone(), oc_channel_id);
                }
                Err(e) => {
                    progress.errors.push(format!("Channel {}: {}", channel.name, e));
                }
            }

            progress.channels_processed += 1;
        }

        // Phase 3: DMs
        if options.include_dms || options.include_group_dms {
            progress.phase = "direct_messages".to_string();
            self.update_job_progress(job_id, &progress).await?;

            let dms_to_migrate: Vec<_> = mm_channels.iter()
                .filter(|c| {
                    (c.is_direct() && options.include_dms) ||
                    (c.is_group() && options.include_group_dms)
                })
                .collect();

            progress.dms_total = dms_to_migrate.len() as i64;

            for dm in &dms_to_migrate {
                progress.current_item = Some(format!("DM: {}", dm.name));
                self.update_job_progress(job_id, &progress).await?;

                match self.migrate_dm(&client, dm, &id_mapping).await {
                    Ok(oc_dm_id) => {
                        id_mapping.mattermost_dm_ids.insert(dm.id.clone(), oc_dm_id);
                    }
                    Err(e) => {
                        progress.errors.push(format!("DM {}: {}", dm.name, e));
                    }
                }

                progress.dms_processed += 1;
            }
        }

        // Phase 4: Messages
        progress.phase = "messages".to_string();
        self.update_job_progress(job_id, &progress).await?;

        // Get total message count
        let mut total_messages = 0i64;
        for channel_id in id_mapping.mattermost_channel_ids.keys() {
            total_messages += client.get_channel_post_count(channel_id).await.unwrap_or(0);
        }
        for dm_id in id_mapping.mattermost_dm_ids.keys() {
            total_messages += client.get_channel_post_count(dm_id).await.unwrap_or(0);
        }
        progress.messages_total = total_messages;

        // Migrate channel messages
        for (mm_channel_id, oc_channel_id) in &id_mapping.mattermost_channel_ids {
            let posts = client.get_all_channel_posts(mm_channel_id).await?;

            for post in &posts {
                progress.current_item = Some(format!("Message in channel"));

                match self.migrate_message(&client, post, Some(*oc_channel_id), None, &id_mapping, options.include_attachments).await {
                    Ok(oc_msg_id) => {
                        id_mapping.mattermost_message_ids.insert(post.id.clone(), oc_msg_id);
                    }
                    Err(e) => {
                        progress.errors.push(format!("Message {}: {}", post.id, e));
                    }
                }

                progress.messages_processed += 1;

                if progress.messages_processed % 100 == 0 {
                    self.update_job_progress(job_id, &progress).await?;
                }
            }
        }

        // Migrate DM messages
        for (mm_dm_id, oc_dm_id) in &id_mapping.mattermost_dm_ids {
            let posts = client.get_all_channel_posts(mm_dm_id).await?;

            for post in &posts {
                progress.current_item = Some(format!("Message in DM"));

                match self.migrate_message(&client, post, None, Some(*oc_dm_id), &id_mapping, options.include_attachments).await {
                    Ok(oc_msg_id) => {
                        id_mapping.mattermost_message_ids.insert(post.id.clone(), oc_msg_id);
                    }
                    Err(e) => {
                        progress.errors.push(format!("Message {}: {}", post.id, e));
                    }
                }

                progress.messages_processed += 1;

                if progress.messages_processed % 100 == 0 {
                    self.update_job_progress(job_id, &progress).await?;
                }
            }
        }

        // Complete
        progress.phase = "completed".to_string();
        progress.current_item = None;
        self.update_job_progress(job_id, &progress).await?;
        self.update_job_status(job_id, MigrationStatus::Completed, None).await?;

        Ok(())
    }

    /// Get existing OpenChat users by email
    async fn get_existing_users_by_email(&self) -> Result<HashMap<String, Uuid>> {
        let rows = sqlx::query(
            "SELECT id, LOWER(email) as email FROM users WHERE org_id = $1"
        )
        .bind(self.org_id)
        .fetch_all(&self.openchat_pool)
        .await?;

        let mut map = HashMap::new();
        for row in rows {
            use sqlx::Row;
            let id: Uuid = row.get("id");
            let email: String = row.get("email");
            map.insert(email, id);
        }

        Ok(map)
    }

    /// Migrate a single user
    async fn migrate_user(&self, mm_user: &MmUser, overrides: &[UserMappingOverride]) -> Result<Uuid> {
        let email_lower = mm_user.email.to_lowercase();

        // Check for override
        if let Some(override_mapping) = overrides.iter().find(|o| o.mattermost_id == mm_user.id) {
            match override_mapping.action {
                UserAction::Skip => return Err(anyhow!("User skipped by override")),
                UserAction::Match => {
                    if let Some(oc_id) = override_mapping.openchat_user_id {
                        return Ok(oc_id);
                    }
                }
                UserAction::Create => {}
            }
        }

        // Check if user exists in OpenChat
        let existing: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM users WHERE org_id = $1 AND LOWER(email) = $2"
        )
        .bind(self.org_id)
        .bind(&email_lower)
        .fetch_optional(&self.openchat_pool)
        .await?;

        if let Some(id) = existing {
            return Ok(id);
        }

        // Create user in TitaniumVault first
        let tv_user_id = self.create_tv_user(mm_user).await?;

        // Create user in OpenChat
        let oc_user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (id, org_id, tv_user_id, email, display_name, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, 'offline', NOW(), NOW())
            "#
        )
        .bind(oc_user_id)
        .bind(self.org_id)
        .bind(tv_user_id)
        .bind(&mm_user.email)
        .bind(mm_user.display_name())
        .execute(&self.openchat_pool)
        .await?;

        Ok(oc_user_id)
    }

    /// Create user in TitaniumVault
    async fn create_tv_user(&self, mm_user: &MmUser) -> Result<Uuid> {
        let client = Client::new();

        // Generate a random password
        let password: String = {
            let mut rng = rand::rng();
            (0..16)
                .map(|_| {
                    let idx = rng.random_range(0..62usize);
                    if idx < 10 {
                        (b'0' + idx as u8) as char
                    } else if idx < 36 {
                        (b'A' + (idx - 10) as u8) as char
                    } else {
                        (b'a' + (idx - 36) as u8) as char
                    }
                })
                .collect()
        };

        let password = format!("{}!@#", password); // Add special chars for complexity

        let body = serde_json::json!({
            "email": mm_user.email,
            "password": password,
            "roles": ["User"],
            "user_type": "CUSTOMER"
        });

        let url = format!(
            "{}/organizations/{}/admin/users",
            self.tv_api_url.trim_end_matches('/'),
            self.org_id
        );

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.tv_access_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to create TV user: {} - {}", status, text));
        }

        let result: serde_json::Value = resp.json().await?;
        let id_str = result.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No user ID in TV response"))?;

        Uuid::parse_str(id_str).map_err(|e| anyhow!("Invalid UUID from TV: {}", e))
    }

    /// Migrate a channel
    async fn migrate_channel(
        &self,
        client: &MattermostClient,
        channel: &MmChannel,
        id_mapping: &IdMapping,
    ) -> Result<Uuid> {
        let channel_id = Uuid::new_v4();

        // Find creator
        let created_by = id_mapping.mattermost_user_ids.get(&channel.creator_id)
            .copied()
            .or_else(|| {
                // Use first available user
                id_mapping.mattermost_user_ids.values().next().copied()
            })
            .ok_or_else(|| anyhow!("No users available to create channel"))?;

        let channel_type = if channel.is_public() { "public" } else { "private" };

        // Create channel
        sqlx::query(
            r#"
            INSERT INTO channels (id, org_id, name, description, channel_type, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            "#
        )
        .bind(channel_id)
        .bind(self.org_id)
        .bind(&channel.display_name)
        .bind(channel.purpose.as_deref().or(channel.header.as_deref()))
        .bind(channel_type)
        .bind(created_by)
        .execute(&self.openchat_pool)
        .await?;

        // Add members
        let members = client.get_channel_members(&channel.id).await?;
        for member in &members {
            if let Some(oc_user_id) = id_mapping.mattermost_user_ids.get(&member.user_id) {
                let role = if member.roles.as_ref().map_or(false, |r| r.contains("admin")) {
                    "admin"
                } else {
                    "member"
                };

                sqlx::query(
                    r#"
                    INSERT INTO channel_members (id, channel_id, user_id, role, joined_at)
                    VALUES ($1, $2, $3, $4, NOW())
                    ON CONFLICT (channel_id, user_id) DO NOTHING
                    "#
                )
                .bind(Uuid::new_v4())
                .bind(channel_id)
                .bind(oc_user_id)
                .bind(role)
                .execute(&self.openchat_pool)
                .await?;
            }
        }

        Ok(channel_id)
    }

    /// Migrate a DM
    async fn migrate_dm(
        &self,
        client: &MattermostClient,
        dm: &MmChannel,
        id_mapping: &IdMapping,
    ) -> Result<Uuid> {
        let dm_id = Uuid::new_v4();

        // Get participants
        let members = client.get_channel_members(&dm.id).await?;
        let oc_members: Vec<Uuid> = members.iter()
            .filter_map(|m| id_mapping.mattermost_user_ids.get(&m.user_id).copied())
            .collect();

        if oc_members.is_empty() {
            return Err(anyhow!("No valid participants for DM"));
        }

        let created_by = oc_members[0];

        // Create DM
        sqlx::query(
            r#"
            INSERT INTO direct_messages (id, org_id, created_by, created_at)
            VALUES ($1, $2, $3, NOW())
            "#
        )
        .bind(dm_id)
        .bind(self.org_id)
        .bind(created_by)
        .execute(&self.openchat_pool)
        .await?;

        // Add participants
        for user_id in &oc_members {
            sqlx::query(
                r#"
                INSERT INTO dm_participants (id, dm_id, user_id, joined_at)
                VALUES ($1, $2, $3, NOW())
                ON CONFLICT (dm_id, user_id) DO NOTHING
                "#
            )
            .bind(Uuid::new_v4())
            .bind(dm_id)
            .bind(user_id)
            .execute(&self.openchat_pool)
            .await?;
        }

        Ok(dm_id)
    }

    /// Migrate a message
    async fn migrate_message(
        &self,
        client: &MattermostClient,
        post: &MmPost,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        id_mapping: &IdMapping,
        include_attachments: bool,
    ) -> Result<Uuid> {
        let message_id = Uuid::new_v4();

        let user_id = id_mapping.mattermost_user_ids.get(&post.user_id)
            .copied()
            .ok_or_else(|| anyhow!("User not found for message"))?;

        // Handle threading
        let parent_message_id = post.root_id.as_ref()
            .and_then(|root_id| id_mapping.mattermost_message_ids.get(root_id).copied());

        // Convert timestamp
        let created_at = Utc.timestamp_millis_opt(post.create_at).single()
            .unwrap_or_else(Utc::now);

        let edited_at = if post.edit_at > 0 {
            Utc.timestamp_millis_opt(post.edit_at).single()
        } else {
            None
        };

        // Insert message
        sqlx::query(
            r#"
            INSERT INTO messages (id, channel_id, dm_id, user_id, content, parent_message_id, created_at, edited_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(message_id)
        .bind(channel_id)
        .bind(dm_id)
        .bind(user_id)
        .bind(&post.message)
        .bind(parent_message_id)
        .bind(created_at)
        .bind(edited_at)
        .execute(&self.openchat_pool)
        .await?;

        // Handle pinned messages
        if post.is_pinned {
            if let Some(ch_id) = channel_id {
                sqlx::query(
                    r#"
                    INSERT INTO pinned_messages (id, channel_id, message_id, pinned_by, pinned_at)
                    VALUES ($1, $2, $3, $4, NOW())
                    ON CONFLICT (channel_id, message_id) DO NOTHING
                    "#
                )
                .bind(Uuid::new_v4())
                .bind(ch_id)
                .bind(message_id)
                .bind(user_id)
                .execute(&self.openchat_pool)
                .await?;
            }
        }

        // Handle reactions
        if post.has_reactions {
            let reactions = client.get_post_reactions(&post.id).await?;
            for reaction in &reactions {
                if let Some(reactor_id) = id_mapping.mattermost_user_ids.get(&reaction.user_id) {
                    sqlx::query(
                        r#"
                        INSERT INTO reactions (id, message_id, user_id, emoji, created_at)
                        VALUES ($1, $2, $3, $4, NOW())
                        ON CONFLICT (message_id, user_id, emoji) DO NOTHING
                        "#
                    )
                    .bind(Uuid::new_v4())
                    .bind(message_id)
                    .bind(reactor_id)
                    .bind(&reaction.emoji_name)
                    .execute(&self.openchat_pool)
                    .await?;
                }
            }
        }

        // Handle attachments
        if include_attachments {
            if let Some(file_ids) = &post.file_ids {
                for file_id in file_ids {
                    if let Err(e) = self.migrate_attachment(client, file_id, message_id).await {
                        tracing::warn!("Failed to migrate attachment {}: {}", file_id, e);
                    }
                }
            }
        }

        Ok(message_id)
    }

    /// Migrate an attachment
    async fn migrate_attachment(
        &self,
        client: &MattermostClient,
        file_id: &str,
        message_id: Uuid,
    ) -> Result<Uuid> {
        let file_info = client.get_file_info(file_id).await?
            .ok_or_else(|| anyhow!("File info not found"))?;

        // Download file content
        let _content = client.download_file(file_id).await?;

        // For now, store as local file (in production, upload to S3)
        let attachment_id = Uuid::new_v4();
        let storage_path = format!("migrations/{}/{}", message_id, file_info.name);

        // Create attachment record
        sqlx::query(
            r#"
            INSERT INTO attachments (id, message_id, file_name, file_type, file_size, storage_type, storage_path, created_at)
            VALUES ($1, $2, $3, $4, $5, 'local', $6, NOW())
            "#
        )
        .bind(attachment_id)
        .bind(message_id)
        .bind(&file_info.name)
        .bind(&file_info.mime_type)
        .bind(file_info.size)
        .bind(&storage_path)
        .execute(&self.openchat_pool)
        .await?;

        // TODO: Actually save the file to storage
        // For now we just record the metadata

        Ok(attachment_id)
    }

    /// Update job status
    async fn update_job_status(&self, job_id: Uuid, status: MigrationStatus, error: Option<String>) -> Result<()> {
        let status_str = match status {
            MigrationStatus::Pending => "pending",
            MigrationStatus::Running => "running",
            MigrationStatus::Completed => "completed",
            MigrationStatus::Failed => "failed",
            MigrationStatus::Cancelled => "cancelled",
        };

        if matches!(status, MigrationStatus::Completed | MigrationStatus::Failed | MigrationStatus::Cancelled) {
            sqlx::query(
                "UPDATE migration_jobs SET status = $1, error = $2, completed_at = NOW() WHERE id = $3"
            )
            .bind(status_str)
            .bind(error)
            .bind(job_id)
            .execute(&self.openchat_pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE migration_jobs SET status = $1, error = $2 WHERE id = $3"
            )
            .bind(status_str)
            .bind(error)
            .bind(job_id)
            .execute(&self.openchat_pool)
            .await?;
        }

        Ok(())
    }

    /// Update job progress
    async fn update_job_progress(&self, job_id: Uuid, progress: &MigrationProgress) -> Result<()> {
        sqlx::query(
            "UPDATE migration_jobs SET progress = $1 WHERE id = $2"
        )
        .bind(serde_json::to_value(progress)?)
        .bind(job_id)
        .execute(&self.openchat_pool)
        .await?;

        Ok(())
    }

    /// Get job status
    pub async fn get_job_status(&self, job_id: Uuid) -> Result<Option<MigrationJob>> {
        let row = sqlx::query_as::<_, MigrationJob>(
            r#"
            SELECT id, org_id, status, progress, error, started_at, completed_at, created_by
            FROM migration_jobs
            WHERE id = $1 AND org_id = $2
            "#
        )
        .bind(job_id)
        .bind(self.org_id)
        .fetch_optional(&self.openchat_pool)
        .await?;

        Ok(row)
    }
}
