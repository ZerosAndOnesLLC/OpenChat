use anyhow::{anyhow, Result};
use reqwest::{Client, StatusCode};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::time::Duration;

use super::models::*;

const API_PAGE_SIZE: i32 = 200;
const DB_BATCH_SIZE: i32 = 1000;

/// Mattermost client supporting both API and DB access
pub struct MattermostClient {
    connection: MattermostConnection,
    http_client: Option<Client>,
    db_pool: Option<PgPool>,
}

impl MattermostClient {
    pub async fn new(connection: MattermostConnection) -> Result<Self> {
        match &connection {
            MattermostConnection::Api { server_url, access_token } => {
                let http_client = Client::builder()
                    .timeout(Duration::from_secs(30))
                    .build()?;

                // Validate connection
                let url = format!("{}/api/v4/users/me", server_url.trim_end_matches('/'));
                let resp = http_client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", access_token))
                    .send()
                    .await?;

                if resp.status() != StatusCode::OK {
                    return Err(anyhow!("Invalid API credentials: {}", resp.status()));
                }

                Ok(Self {
                    connection,
                    http_client: Some(http_client),
                    db_pool: None,
                })
            }
            MattermostConnection::Database { connection_string } => {
                let pool = PgPoolOptions::new()
                    .max_connections(5)
                    .acquire_timeout(Duration::from_secs(10))
                    .connect(connection_string)
                    .await?;

                // Validate connection
                sqlx::query("SELECT 1").execute(&pool).await?;

                Ok(Self {
                    connection,
                    http_client: None,
                    db_pool: Some(pool),
                })
            }
        }
    }

    pub async fn get_server_version(&self) -> Result<String> {
        match &self.connection {
            MattermostConnection::Api { server_url, access_token } => {
                let client = self.http_client.as_ref().unwrap();
                let url = format!("{}/api/v4/system/ping", server_url.trim_end_matches('/'));
                let resp: serde_json::Value = client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", access_token))
                    .send()
                    .await?
                    .json()
                    .await?;
                Ok(resp.get("status").and_then(|s| s.as_str()).unwrap_or("unknown").to_string())
            }
            MattermostConnection::Database { .. } => {
                let pool = self.db_pool.as_ref().unwrap();
                let row = sqlx::query("SELECT value FROM public.systems WHERE name = 'Version'")
                    .fetch_optional(pool)
                    .await?;
                Ok(row.map(|r| r.get::<String, _>("value")).unwrap_or_else(|| "unknown".to_string()))
            }
        }
    }

    /// Get all users (excluding deleted and bots)
    pub async fn get_users(&self) -> Result<Vec<MmUser>> {
        match &self.connection {
            MattermostConnection::Api { server_url, access_token } => {
                let client = self.http_client.as_ref().unwrap();
                let mut all_users = Vec::new();
                let mut page = 0;

                loop {
                    let url = format!(
                        "{}/api/v4/users?page={}&per_page={}",
                        server_url.trim_end_matches('/'),
                        page,
                        API_PAGE_SIZE
                    );
                    let users: Vec<MmUser> = client
                        .get(&url)
                        .header("Authorization", format!("Bearer {}", access_token))
                        .send()
                        .await?
                        .json()
                        .await?;

                    let count = users.len();
                    all_users.extend(users.into_iter().filter(|u| !u.is_deleted()));

                    if count < API_PAGE_SIZE as usize {
                        break;
                    }
                    page += 1;
                }

                // Filter out system/bot users
                Ok(all_users.into_iter().filter(|u| {
                    !u.email.ends_with("@localhost") &&
                    !u.username.contains("bot")
                }).collect())
            }
            MattermostConnection::Database { .. } => {
                let pool = self.db_pool.as_ref().unwrap();
                let rows = sqlx::query(
                    r#"
                    SELECT id, username, email, firstname, lastname, nickname, deleteat
                    FROM public.users
                    WHERE deleteat = 0
                    AND email NOT LIKE '%@localhost'
                    AND username NOT LIKE '%bot%'
                    ORDER BY createat
                    "#
                )
                .fetch_all(pool)
                .await?;

                Ok(rows.iter().map(|r| MmUser {
                    id: r.get("id"),
                    username: r.get("username"),
                    email: r.get("email"),
                    first_name: r.get("firstname"),
                    last_name: r.get("lastname"),
                    nickname: r.get("nickname"),
                    delete_at: r.get::<i64, _>("deleteat"),
                }).collect())
            }
        }
    }

    /// Get all channels (public and private, excluding deleted)
    pub async fn get_channels(&self) -> Result<Vec<MmChannel>> {
        match &self.connection {
            MattermostConnection::Api { server_url, access_token } => {
                let client = self.http_client.as_ref().unwrap();

                // First get all teams
                let teams_url = format!("{}/api/v4/teams", server_url.trim_end_matches('/'));
                let teams: Vec<serde_json::Value> = client
                    .get(&teams_url)
                    .header("Authorization", format!("Bearer {}", access_token))
                    .send()
                    .await?
                    .json()
                    .await?;

                let mut all_channels = Vec::new();

                for team in teams {
                    let team_id = team.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    if team_id.is_empty() {
                        continue;
                    }

                    // Get public channels
                    let mut page = 0;
                    loop {
                        let url = format!(
                            "{}/api/v4/teams/{}/channels?page={}&per_page={}",
                            server_url.trim_end_matches('/'),
                            team_id,
                            page,
                            API_PAGE_SIZE
                        );
                        let channels: Vec<MmChannel> = client
                            .get(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .send()
                            .await?
                            .json()
                            .await?;

                        let count = channels.len();
                        all_channels.extend(channels.into_iter().filter(|c| !c.is_deleted()));

                        if count < API_PAGE_SIZE as usize {
                            break;
                        }
                        page += 1;
                    }
                }

                Ok(all_channels)
            }
            MattermostConnection::Database { .. } => {
                let pool = self.db_pool.as_ref().unwrap();
                let rows = sqlx::query(
                    r#"
                    SELECT id, teamid, name, displayname, type, header, purpose,
                           creatorid, deleteat, totalmsgcount
                    FROM public.channels
                    WHERE deleteat = 0
                    AND type IN ('O', 'P', 'D', 'G')
                    ORDER BY createat
                    "#
                )
                .fetch_all(pool)
                .await?;

                Ok(rows.iter().map(|r| MmChannel {
                    id: r.get("id"),
                    team_id: r.get::<Option<String>, _>("teamid").unwrap_or_default(),
                    name: r.get("name"),
                    display_name: r.get("displayname"),
                    channel_type: r.get::<String, _>("type"),
                    header: r.get("header"),
                    purpose: r.get("purpose"),
                    creator_id: r.get::<Option<String>, _>("creatorid").unwrap_or_default(),
                    delete_at: r.get::<i64, _>("deleteat"),
                    total_msg_count: r.get::<i64, _>("totalmsgcount"),
                }).collect())
            }
        }
    }

    /// Get channel members
    pub async fn get_channel_members(&self, channel_id: &str) -> Result<Vec<MmChannelMember>> {
        match &self.connection {
            MattermostConnection::Api { server_url, access_token } => {
                let client = self.http_client.as_ref().unwrap();
                let mut all_members = Vec::new();
                let mut page = 0;

                loop {
                    let url = format!(
                        "{}/api/v4/channels/{}/members?page={}&per_page={}",
                        server_url.trim_end_matches('/'),
                        channel_id,
                        page,
                        API_PAGE_SIZE
                    );
                    let members: Vec<MmChannelMember> = client
                        .get(&url)
                        .header("Authorization", format!("Bearer {}", access_token))
                        .send()
                        .await?
                        .json()
                        .await?;

                    let count = members.len();
                    all_members.extend(members);

                    if count < API_PAGE_SIZE as usize {
                        break;
                    }
                    page += 1;
                }

                Ok(all_members)
            }
            MattermostConnection::Database { .. } => {
                let pool = self.db_pool.as_ref().unwrap();
                let rows = sqlx::query(
                    r#"
                    SELECT channelid, userid, roles
                    FROM public.channelmembers
                    WHERE channelid = $1
                    "#
                )
                .bind(channel_id)
                .fetch_all(pool)
                .await?;

                Ok(rows.iter().map(|r| MmChannelMember {
                    channel_id: r.get("channelid"),
                    user_id: r.get("userid"),
                    roles: r.get("roles"),
                }).collect())
            }
        }
    }

    /// Get posts for a channel with pagination
    pub async fn get_channel_posts(&self, channel_id: &str, page: i32, per_page: i32) -> Result<Vec<MmPost>> {
        match &self.connection {
            MattermostConnection::Api { server_url, access_token } => {
                let client = self.http_client.as_ref().unwrap();
                let url = format!(
                    "{}/api/v4/channels/{}/posts?page={}&per_page={}",
                    server_url.trim_end_matches('/'),
                    channel_id,
                    page,
                    per_page.min(API_PAGE_SIZE)
                );

                let post_list: MmPostList = client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", access_token))
                    .send()
                    .await?
                    .json()
                    .await?;

                // Return posts in order
                let posts: Vec<MmPost> = post_list.order.iter()
                    .filter_map(|id| post_list.posts.get(id).cloned())
                    .filter(|p| !p.is_deleted() && !p.is_system_message())
                    .collect();

                Ok(posts)
            }
            MattermostConnection::Database { .. } => {
                let pool = self.db_pool.as_ref().unwrap();
                let offset = page * per_page;
                let rows = sqlx::query(
                    r#"
                    SELECT id, channelid, userid, message, rootid, createat,
                           updateat, editat, deleteat, ispinned, fileids,
                           hasreactions, type
                    FROM public.posts
                    WHERE channelid = $1
                    AND deleteat = 0
                    AND (type IS NULL OR type = '')
                    ORDER BY createat ASC
                    LIMIT $2 OFFSET $3
                    "#
                )
                .bind(channel_id)
                .bind(per_page as i64)
                .bind(offset as i64)
                .fetch_all(pool)
                .await?;

                Ok(rows.iter().map(|r| {
                    let file_ids_str: Option<String> = r.get("fileids");
                    let file_ids = file_ids_str
                        .filter(|s| !s.is_empty())
                        .map(|s| {
                            s.trim_matches(|c| c == '[' || c == ']')
                                .split(',')
                                .map(|s| s.trim().trim_matches('"').to_string())
                                .filter(|s| !s.is_empty())
                                .collect()
                        });

                    MmPost {
                        id: r.get("id"),
                        channel_id: r.get("channelid"),
                        user_id: r.get("userid"),
                        message: r.get("message"),
                        root_id: r.get::<Option<String>, _>("rootid").filter(|s| !s.is_empty()),
                        create_at: r.get("createat"),
                        update_at: r.get("updateat"),
                        edit_at: r.get::<i64, _>("editat"),
                        delete_at: r.get("deleteat"),
                        is_pinned: r.get("ispinned"),
                        file_ids,
                        has_reactions: r.get("hasreactions"),
                        post_type: r.get::<Option<String>, _>("type"),
                    }
                }).collect())
            }
        }
    }

    /// Get all posts for a channel (handles pagination internally)
    pub async fn get_all_channel_posts(&self, channel_id: &str) -> Result<Vec<MmPost>> {
        let mut all_posts = Vec::new();
        let mut page = 0;
        let per_page = match &self.connection {
            MattermostConnection::Api { .. } => API_PAGE_SIZE,
            MattermostConnection::Database { .. } => DB_BATCH_SIZE,
        };

        loop {
            let posts = self.get_channel_posts(channel_id, page, per_page).await?;
            let count = posts.len();
            all_posts.extend(posts);

            if count < per_page as usize {
                break;
            }
            page += 1;
        }

        Ok(all_posts)
    }

    /// Get post count for a channel
    pub async fn get_channel_post_count(&self, channel_id: &str) -> Result<i64> {
        match &self.connection {
            MattermostConnection::Api { .. } => {
                // API doesn't have a direct count endpoint, use channel stats
                let channel = self.get_channel_by_id(channel_id).await?;
                Ok(channel.map(|c| c.total_msg_count).unwrap_or(0))
            }
            MattermostConnection::Database { .. } => {
                let pool = self.db_pool.as_ref().unwrap();
                let count: i64 = sqlx::query_scalar(
                    r#"
                    SELECT COUNT(*) FROM public.posts
                    WHERE channelid = $1 AND deleteat = 0 AND (type IS NULL OR type = '')
                    "#
                )
                .bind(channel_id)
                .fetch_one(pool)
                .await?;
                Ok(count)
            }
        }
    }

    /// Get channel by ID
    pub async fn get_channel_by_id(&self, channel_id: &str) -> Result<Option<MmChannel>> {
        match &self.connection {
            MattermostConnection::Api { server_url, access_token } => {
                let client = self.http_client.as_ref().unwrap();
                let url = format!(
                    "{}/api/v4/channels/{}",
                    server_url.trim_end_matches('/'),
                    channel_id
                );

                let resp = client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", access_token))
                    .send()
                    .await?;

                if resp.status() == StatusCode::NOT_FOUND {
                    return Ok(None);
                }

                let channel: MmChannel = resp.json().await?;
                Ok(Some(channel))
            }
            MattermostConnection::Database { .. } => {
                let pool = self.db_pool.as_ref().unwrap();
                let row = sqlx::query(
                    r#"
                    SELECT id, teamid, name, displayname, type, header, purpose,
                           creatorid, deleteat, totalmsgcount
                    FROM public.channels
                    WHERE id = $1
                    "#
                )
                .bind(channel_id)
                .fetch_optional(pool)
                .await?;

                Ok(row.map(|r| MmChannel {
                    id: r.get("id"),
                    team_id: r.get::<Option<String>, _>("teamid").unwrap_or_default(),
                    name: r.get("name"),
                    display_name: r.get("displayname"),
                    channel_type: r.get::<String, _>("type"),
                    header: r.get("header"),
                    purpose: r.get("purpose"),
                    creator_id: r.get::<Option<String>, _>("creatorid").unwrap_or_default(),
                    delete_at: r.get::<i64, _>("deleteat"),
                    total_msg_count: r.get::<i64, _>("totalmsgcount"),
                }))
            }
        }
    }

    /// Get file info
    pub async fn get_file_info(&self, file_id: &str) -> Result<Option<MmFileInfo>> {
        match &self.connection {
            MattermostConnection::Api { server_url, access_token } => {
                let client = self.http_client.as_ref().unwrap();
                let url = format!(
                    "{}/api/v4/files/{}/info",
                    server_url.trim_end_matches('/'),
                    file_id
                );

                let resp = client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", access_token))
                    .send()
                    .await?;

                if resp.status() == StatusCode::NOT_FOUND {
                    return Ok(None);
                }

                let file_info: MmFileInfo = resp.json().await?;
                Ok(Some(file_info))
            }
            MattermostConnection::Database { .. } => {
                let pool = self.db_pool.as_ref().unwrap();
                let row = sqlx::query(
                    r#"
                    SELECT id, postid, name, extension, size, mimetype, deleteat
                    FROM public.fileinfo
                    WHERE id = $1
                    "#
                )
                .bind(file_id)
                .fetch_optional(pool)
                .await?;

                Ok(row.map(|r| MmFileInfo {
                    id: r.get("id"),
                    post_id: r.get("postid"),
                    name: r.get("name"),
                    extension: r.get("extension"),
                    size: r.get("size"),
                    mime_type: r.get("mimetype"),
                    delete_at: r.get::<i64, _>("deleteat"),
                }))
            }
        }
    }

    /// Download file content
    pub async fn download_file(&self, file_id: &str) -> Result<bytes::Bytes> {
        match &self.connection {
            MattermostConnection::Api { server_url, access_token } => {
                let client = self.http_client.as_ref().unwrap();
                let url = format!(
                    "{}/api/v4/files/{}",
                    server_url.trim_end_matches('/'),
                    file_id
                );

                let resp = client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", access_token))
                    .send()
                    .await?;

                if !resp.status().is_success() {
                    return Err(anyhow!("Failed to download file: {}", resp.status()));
                }

                Ok(resp.bytes().await?)
            }
            MattermostConnection::Database { .. } => {
                // DB mode doesn't have file content - need file path
                Err(anyhow!("File download not supported in database mode. Use API mode or provide file storage path."))
            }
        }
    }

    /// Get reactions for a post
    pub async fn get_post_reactions(&self, post_id: &str) -> Result<Vec<MmReaction>> {
        match &self.connection {
            MattermostConnection::Api { server_url, access_token } => {
                let client = self.http_client.as_ref().unwrap();
                let url = format!(
                    "{}/api/v4/posts/{}/reactions",
                    server_url.trim_end_matches('/'),
                    post_id
                );

                let reactions: Vec<MmReaction> = client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", access_token))
                    .send()
                    .await?
                    .json()
                    .await?;

                Ok(reactions)
            }
            MattermostConnection::Database { .. } => {
                let pool = self.db_pool.as_ref().unwrap();
                let rows = sqlx::query(
                    r#"
                    SELECT userid, postid, emojiname, createat
                    FROM public.reactions
                    WHERE postid = $1 AND deleteat = 0
                    "#
                )
                .bind(post_id)
                .fetch_all(pool)
                .await?;

                Ok(rows.iter().map(|r| MmReaction {
                    user_id: r.get("userid"),
                    post_id: r.get("postid"),
                    emoji_name: r.get("emojiname"),
                    create_at: r.get("createat"),
                }).collect())
            }
        }
    }

    /// Get total counts for preview
    pub async fn get_stats(&self) -> Result<(i64, i64, i64, i64)> {
        match &self.connection {
            MattermostConnection::Api { .. } => {
                let users = self.get_users().await?.len() as i64;
                let channels = self.get_channels().await?;
                let channel_count = channels.iter().filter(|c| c.is_public() || c.is_private()).count() as i64;
                let dm_count = channels.iter().filter(|c| c.is_direct() || c.is_group()).count() as i64;

                // Message count is harder via API, estimate from channels
                let mut total_messages: i64 = 0;
                for channel in &channels {
                    total_messages += channel.total_msg_count;
                }

                Ok((users, channel_count, dm_count, total_messages))
            }
            MattermostConnection::Database { .. } => {
                let pool = self.db_pool.as_ref().unwrap();

                let user_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM public.users WHERE deleteat = 0 AND email NOT LIKE '%@localhost'"
                ).fetch_one(pool).await?;

                let channel_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM public.channels WHERE deleteat = 0 AND type IN ('O', 'P')"
                ).fetch_one(pool).await?;

                let dm_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM public.channels WHERE deleteat = 0 AND type IN ('D', 'G')"
                ).fetch_one(pool).await?;

                let message_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM public.posts WHERE deleteat = 0 AND (type IS NULL OR type = '')"
                ).fetch_one(pool).await?;

                Ok((user_count, channel_count, dm_count, message_count))
            }
        }
    }

    /// Get file stats
    pub async fn get_file_stats(&self) -> Result<(i64, i64)> {
        match &self.connection {
            MattermostConnection::Api { .. } => {
                // API doesn't have a good way to get this, return estimates
                Ok((0, 0))
            }
            MattermostConnection::Database { .. } => {
                let pool = self.db_pool.as_ref().unwrap();

                let row = sqlx::query(
                    "SELECT COUNT(*) as count, COALESCE(SUM(size), 0) as total_size FROM public.fileinfo WHERE deleteat = 0"
                ).fetch_one(pool).await?;

                Ok((row.get::<i64, _>("count"), row.get::<i64, _>("total_size")))
            }
        }
    }
}
