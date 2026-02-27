use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message as ActixMessage, WrapFuture};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::messages::ServerMessage;
use super::session::{WsSessionHandle, WsSessionMessage};
use crate::config::WebSocketConfig;
use crate::models::message::Message as DbMessage;

/// Connection statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub total_sessions: usize,
    pub unique_users: usize,
    pub unique_orgs: usize,
    pub channel_subscriptions: usize,
}

/// Message batch for efficient delivery
#[derive(Clone)]
struct MessageBatch {
    messages: VecDeque<ServerMessage>,
    created_at: Instant,
}

impl MessageBatch {
    fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            created_at: Instant::now(),
        }
    }

    fn add(&mut self, message: ServerMessage) {
        self.messages.push_back(message);
    }

    fn should_flush(&self, config: &WebSocketConfig) -> bool {
        self.messages.len() >= config.batch_size
            || self.created_at.elapsed() > Duration::from_millis(config.batch_timeout_ms)
    }

    fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    fn flush(&mut self) -> Vec<ServerMessage> {
        let messages: Vec<_> = self.messages.drain(..).collect();
        self.created_at = Instant::now();
        messages
    }
}

/// WebSocket server that manages all connections
pub struct WsServer {
    /// Database pool for persisting messages
    db_pool: PgPool,
    /// Redis pool for caching
    redis_pool: crate::db::RedisPool,
    /// WebSocket configuration
    config: Arc<WebSocketConfig>,
    /// Map of session_id -> session address
    sessions: HashMap<Uuid, Addr<WsSessionHandle>>,
    /// Map of user_id -> set of session_ids (for multi-device support)
    user_sessions: HashMap<Uuid, HashSet<Uuid>>,
    /// Map of org_id -> set of session_ids
    org_sessions: HashMap<Uuid, HashSet<Uuid>>,
    /// Map of channel_id -> set of session_ids (subscriptions)
    channel_subscriptions: HashMap<Uuid, HashSet<Uuid>>,
    /// Map of dm_id -> set of session_ids (subscriptions)
    dm_subscriptions: HashMap<Uuid, HashSet<Uuid>>,
    /// Map of thread (parent_message_id) -> set of session_ids (subscriptions)
    thread_subscriptions: HashMap<Uuid, HashSet<Uuid>>,
    /// Message batches per session (for batching optimization)
    message_batches: HashMap<Uuid, MessageBatch>,
    /// Total connection count
    total_connections: usize,
}

impl WsServer {
    pub fn new(db_pool: PgPool, redis_pool: crate::db::RedisPool, config: Arc<WebSocketConfig>) -> Self {
        Self {
            db_pool,
            redis_pool,
            config,
            sessions: HashMap::new(),
            user_sessions: HashMap::new(),
            org_sessions: HashMap::new(),
            channel_subscriptions: HashMap::new(),
            dm_subscriptions: HashMap::new(),
            thread_subscriptions: HashMap::new(),
            message_batches: HashMap::new(),
            total_connections: 0,
        }
    }

    /// Get current connection statistics
    pub fn connection_stats(&self) -> ConnectionStats {
        ConnectionStats {
            total_connections: self.total_connections,
            total_sessions: self.sessions.len(),
            unique_users: self.user_sessions.len(),
            unique_orgs: self.org_sessions.len(),
            channel_subscriptions: self.channel_subscriptions.len(),
        }
    }

    /// Send message to a specific session (with batching support)
    fn send_message(&mut self, session_id: &Uuid, message: ServerMessage) {
        if self.config.enable_batching {
            // Add to batch
            let batch = self.message_batches
                .entry(*session_id)
                .or_insert_with(MessageBatch::new);

            batch.add(message);

            // Check if we should flush
            if batch.should_flush(&self.config) {
                self.flush_batch(session_id);
            }
        } else {
            // Send immediately
            if let Some(addr) = self.sessions.get(session_id) {
                addr.do_send(WsSessionMessage(message));
            }
        }
    }

    /// Flush a message batch to a session
    fn flush_batch(&mut self, session_id: &Uuid) {
        if let Some(batch) = self.message_batches.get_mut(session_id) {
            if !batch.is_empty() {
                let messages = batch.flush();
                if let Some(addr) = self.sessions.get(session_id) {
                    // Send all messages in batch
                    for message in messages {
                        addr.do_send(WsSessionMessage(message));
                    }
                }
            }
        }
    }

    /// Flush all pending batches (called periodically)
    fn flush_all_batches(&mut self) {
        let session_ids: Vec<Uuid> = self.message_batches.keys().copied().collect();
        for session_id in session_ids {
            if let Some(batch) = self.message_batches.get(&session_id) {
                if batch.should_flush(&self.config) {
                    self.flush_batch(&session_id);
                }
            }
        }
    }

    /// Send message to all users in an organization (except excluded sessions)
    fn send_to_org(&mut self, org_id: &Uuid, message: ServerMessage, exclude: Option<Uuid>) {
        if let Some(session_ids) = self.org_sessions.get(org_id) {
            let session_ids: Vec<Uuid> = session_ids.iter().copied().collect();
            for session_id in session_ids {
                if let Some(exclude_id) = exclude {
                    if session_id == exclude_id {
                        continue;
                    }
                }
                self.send_message(&session_id, message.clone());
            }
        }
    }

    /// Send message to all subscribers of a channel
    fn send_to_channel(&mut self, channel_id: &Uuid, message: ServerMessage) {
        if let Some(session_ids) = self.channel_subscriptions.get(channel_id) {
            let session_ids: Vec<Uuid> = session_ids.iter().copied().collect();
            for session_id in session_ids {
                self.send_message(&session_id, message.clone());
            }
        }
    }

    /// Send message to all sessions of a specific user
    fn send_to_user(&mut self, user_id: &Uuid, message: ServerMessage) {
        if let Some(session_ids) = self.user_sessions.get(user_id) {
            let session_ids: Vec<Uuid> = session_ids.iter().copied().collect();
            for session_id in session_ids {
                self.send_message(&session_id, message.clone());
            }
        }
    }
}

impl Actor for WsServer {
    type Context = Context<Self>;
}

/// Message: New client connected
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct Connect {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub addr: Addr<WsSessionHandle>,
}

impl Handler<Connect> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        // Check global connection limit
        if self.total_connections >= self.config.max_connections {
            tracing::warn!(
                "WebSocket: Connection limit reached ({}/{}), rejecting user {}",
                self.total_connections,
                self.config.max_connections,
                msg.user_id
            );
            // Send error and don't connect
            msg.addr.do_send(WsSessionMessage(ServerMessage::Error {
                message: "Server connection limit reached. Please try again later.".to_string(),
            }));
            return;
        }

        // Check per-user connection limit
        if let Some(user_sessions) = self.user_sessions.get(&msg.user_id) {
            if user_sessions.len() >= self.config.max_connections_per_user {
                tracing::warn!(
                    "WebSocket: User {} has reached max connections per user ({}/{})",
                    msg.user_id,
                    user_sessions.len(),
                    self.config.max_connections_per_user
                );
                // Optionally disconnect oldest session or reject new connection
                // For now, we'll reject the new connection
                msg.addr.do_send(WsSessionMessage(ServerMessage::Error {
                    message: format!(
                        "Maximum connections per user reached ({}). Please disconnect another device.",
                        self.config.max_connections_per_user
                    ),
                }));
                return;
            }
        }

        tracing::info!(
            "WebSocket: User {} connected (session: {}), total connections: {}",
            msg.user_id,
            msg.session_id,
            self.total_connections + 1
        );

        // Store session
        self.sessions.insert(msg.session_id, msg.addr);
        self.total_connections += 1;

        // Add to user sessions
        self.user_sessions
            .entry(msg.user_id)
            .or_insert_with(HashSet::new)
            .insert(msg.session_id);

        // Add to org sessions
        self.org_sessions
            .entry(msg.org_id)
            .or_insert_with(HashSet::new)
            .insert(msg.session_id);

        // Broadcast user online status to org
        let status_msg = ServerMessage::UserStatus {
            user_id: msg.user_id,
            status: "online".to_string(),
        };
        self.send_to_org(&msg.org_id, status_msg, Some(msg.session_id));
    }
}

/// Message: Client disconnected
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub org_id: Uuid,
}

impl Handler<Disconnect> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        tracing::info!(
            "WebSocket: User {} disconnected (session: {}), total connections: {}",
            msg.user_id,
            msg.session_id,
            self.total_connections.saturating_sub(1)
        );

        // Flush any pending messages for this session before disconnecting
        self.flush_batch(&msg.session_id);

        // Remove session
        self.sessions.remove(&msg.session_id);
        self.total_connections = self.total_connections.saturating_sub(1);

        // Remove message batch for this session
        self.message_batches.remove(&msg.session_id);

        // Remove from user sessions
        if let Some(sessions) = self.user_sessions.get_mut(&msg.user_id) {
            sessions.remove(&msg.session_id);
            if sessions.is_empty() {
                self.user_sessions.remove(&msg.user_id);

                // User has no more sessions - send offline status
                let status_msg = ServerMessage::UserStatus {
                    user_id: msg.user_id,
                    status: "offline".to_string(),
                };
                self.send_to_org(&msg.org_id, status_msg, None);
            }
        }

        // Remove from org sessions
        if let Some(sessions) = self.org_sessions.get_mut(&msg.org_id) {
            sessions.remove(&msg.session_id);
            if sessions.is_empty() {
                self.org_sessions.remove(&msg.org_id);
            }
        }

        // Remove from all channel subscriptions
        for (_, subscribers) in self.channel_subscriptions.iter_mut() {
            subscribers.remove(&msg.session_id);
        }

        // Remove from all DM subscriptions
        for (_, subscribers) in self.dm_subscriptions.iter_mut() {
            subscribers.remove(&msg.session_id);
        }

        // Remove from all thread subscriptions
        for (_, subscribers) in self.thread_subscriptions.iter_mut() {
            subscribers.remove(&msg.session_id);
        }
    }
}

/// Message: Send a chat message
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct SendMessage {
    pub user_id: Uuid,
    pub user_name: String,
    pub org_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub content: String,
    pub parent_message_id: Option<Uuid>,
    pub encrypted_content: Option<String>,
    pub encryption_metadata: Option<serde_json::Value>,
}

impl Handler<SendMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, ctx: &mut Context<Self>) {
        let db_pool = self.db_pool.clone();
        let channel_id = msg.channel_id;
        let dm_id = msg.dm_id;
        let user_id = msg.user_id;
        let user_name = msg.user_name.clone();
        let content = msg.content.clone();
        let parent_message_id = msg.parent_message_id;
        let org_id = msg.org_id;
        let encrypted_content = msg.encrypted_content.clone();
        let encryption_metadata = msg.encryption_metadata.clone();

        // Save message to database, then broadcast
        let fut = async move {
            // Decode encrypted content if present
            let encrypted_bytes = if let Some(ref ec) = encrypted_content {
                use base64::Engine;
                match base64::engine::general_purpose::STANDARD.decode(ec) {
                    Ok(bytes) => Some(bytes),
                    Err(e) => {
                        tracing::error!("Failed to decode encrypted content: {}", e);
                        return None;
                    }
                }
            } else {
                None
            };

            // Save to database
            let db_message = if let Some(cid) = channel_id {
                if let (Some(enc_bytes), Some(enc_meta)) = (&encrypted_bytes, &encryption_metadata) {
                    DbMessage::create_encrypted_channel_message(&db_pool, cid, user_id, &content, enc_bytes, enc_meta.clone(), parent_message_id).await
                } else {
                    DbMessage::create_channel_message(&db_pool, cid, user_id, &content, parent_message_id).await
                }
            } else if let Some(did) = dm_id {
                if let (Some(enc_bytes), Some(enc_meta)) = (&encrypted_bytes, &encryption_metadata) {
                    DbMessage::create_encrypted_dm_message(&db_pool, did, user_id, &content, enc_bytes, enc_meta.clone(), parent_message_id).await
                } else {
                    DbMessage::create_dm_message(&db_pool, did, user_id, &content, parent_message_id).await
                }
            } else {
                tracing::error!("Message has neither channel_id nor dm_id");
                return None;
            };

            match db_message {
                Ok(message) => {
                    // Create WebSocket message with actual DB data
                    Some((
                        ServerMessage::NewMessage {
                            id: message.id,
                            channel_id: message.channel_id,
                            dm_id: message.dm_id,
                            user_id: message.user_id,
                            user_name,
                            user_avatar: None,
                            content: message.content,
                            parent_message_id: message.parent_message_id,
                            created_at: message.created_at.to_rfc3339(),
                            is_webhook: None,
                            forwarded_from_message_id: message.forwarded_from_message_id,
                            forwarded_from_channel_id: message.forwarded_from_channel_id,
                            forwarded_from_channel_name: None,
                            encrypted_content,
                            encryption_metadata,
                        },
                        channel_id,
                        org_id,
                    ))
                }
                Err(e) => {
                    tracing::error!("Failed to save message to database: {}", e);
                    None
                }
            }
        }
        .into_actor(self)
        .map(move |result, actor, _ctx| {
            if let Some((new_message, channel_id_opt, org_id)) = result {
                if let Some(cid) = channel_id_opt {
                    // Broadcast to channel subscribers
                    actor.send_to_channel(&cid, new_message);
                } else {
                    // For DMs, broadcast to org
                    actor.send_to_org(&org_id, new_message, None);
                }
            }
        });

        ctx.spawn(fut);
    }
}

/// Message: Typing indicator
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct TypingIndicator {
    pub user_id: Uuid,
    pub user_name: String,
    pub org_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub exclude_session: Uuid,
}

impl Handler<TypingIndicator> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: TypingIndicator, _: &mut Context<Self>) {
        let typing_msg = ServerMessage::UserTyping {
            user_id: msg.user_id,
            channel_id: msg.channel_id,
            dm_id: msg.dm_id,
            user_name: msg.user_name,
        };

        if let Some(channel_id) = msg.channel_id {
            // Send to channel subscribers (except sender)
            // Collect session IDs first to avoid borrow checker issues
            if let Some(session_ids) = self.channel_subscriptions.get(&channel_id) {
                let session_ids: Vec<Uuid> = session_ids.iter()
                    .filter(|&id| id != &msg.exclude_session)
                    .copied()
                    .collect();

                for session_id in session_ids {
                    self.send_message(&session_id, typing_msg.clone());
                }
            }
        } else {
            // For DMs, send to org (except sender)
            self.send_to_org(&msg.org_id, typing_msg, Some(msg.exclude_session));
        }
    }
}

/// Message: Subscribe to channel
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct SubscribeChannel {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub channel_id: Uuid,
}

impl Handler<SubscribeChannel> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: SubscribeChannel, ctx: &mut Context<Self>) {
        tracing::debug!(
            "WebSocket: Session {} subscribed to channel {}",
            msg.session_id, msg.channel_id
        );

        // Register subscription
        self.channel_subscriptions
            .entry(msg.channel_id)
            .or_insert_with(HashSet::new)
            .insert(msg.session_id);

        // Fetch and send channel data
        let pool = self.db_pool.clone();
        let redis_pool = self.redis_pool.clone();
        let session = self.sessions.get(&msg.session_id).cloned();
        let channel_id = msg.channel_id;
        let user_id = msg.user_id;
        let org_id = msg.org_id;

        let fut = async move {
            // Try to get pins from cache first
            let pins = match crate::cache::pins::get_pins_from_cache(&redis_pool, org_id, channel_id).await {
                Ok(Some(cached_pins)) => {
                    tracing::debug!("Cache hit: pins for channel {}", channel_id);
                    Ok(cached_pins)
                }
                Ok(None) => {
                    tracing::debug!("Cache miss: pins for channel {}, fetching from DB", channel_id);
                    // Fetch from DB
                    match crate::models::pin::PinnedMessage::get_pins_for_channel(&pool, channel_id).await {
                        Ok(db_pins) => {
                            // Cache for next time
                            if let Err(e) = crate::cache::pins::set_pins_in_cache(&redis_pool, org_id, channel_id, &db_pins).await {
                                tracing::warn!("Failed to cache pins for channel {}: {}", channel_id, e);
                            }
                            Ok(db_pins)
                        }
                        Err(e) => Err(e)
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read pins from cache: {}, falling back to DB", e);
                    crate::models::pin::PinnedMessage::get_pins_for_channel(&pool, channel_id).await
                }
            };

            // Try to get channel members from cache
            let members = match crate::cache::channels::get_channel_members_from_cache(&redis_pool, org_id, channel_id).await {
                Ok(Some(_cached_members)) => {
                    tracing::debug!("Cache hit: members for channel {}", channel_id);
                    // For now, still fetch from DB to get full member info with names
                    // TODO: Cache full ChannelMemberInfo objects instead
                    crate::models::channel::ChannelMember::get_members_for_channel(&pool, channel_id).await
                }
                Ok(None) | Err(_) => {
                    tracing::debug!("Cache miss: members for channel {}", channel_id);
                    crate::models::channel::ChannelMember::get_members_for_channel(&pool, channel_id).await
                }
            };

            // Fetch messages and unread info (not cached yet as they change frequently)
            let messages_fut = crate::models::message::Message::get_messages_with_details_for_channel(&pool, channel_id, 50);
            let unread_fut = crate::models::read_status::ChannelReadStatus::get_unread_info(&pool, user_id, channel_id);

            let (messages, unread_info) = tokio::join!(
                messages_fut,
                unread_fut
            );

            match (messages, pins, members, unread_info) {
                (Ok(messages), Ok(pins), Ok(members), Ok(unread_info)) => {
                    tracing::debug!(
                        "Loaded channel data for channel {}: {} messages, {} pins, {} members",
                        channel_id,
                        messages.len(),
                        pins.len(),
                        members.len()
                    );

                    Some(ServerMessage::ChannelData {
                        channel_id,
                        messages,
                        pins,
                        members,
                        unread_info,
                    })
                }
                (Err(e), _, _, _) | (_, Err(e), _, _) | (_, _, Err(e), _) | (_, _, _, Err(e)) => {
                    tracing::error!("Failed to load channel data for channel {}: {}", channel_id, e);
                    Some(ServerMessage::Error {
                        message: format!("Failed to load channel data: {}", e),
                    })
                }
            }
        }
        .into_actor(self)
        .map(move |result, _actor, _ctx| {
            if let Some(message) = result {
                if let Some(session) = session {
                    session.do_send(WsSessionMessage(message));
                }
            }
        });

        ctx.spawn(fut);
    }
}

/// Message: Unsubscribe from channel
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct UnsubscribeChannel {
    pub session_id: Uuid,
    pub channel_id: Uuid,
}

impl Handler<UnsubscribeChannel> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: UnsubscribeChannel, _: &mut Context<Self>) {
        println!(
            "WebSocket: A session unsubscribed from a channel"
        );

        if let Some(subscribers) = self.channel_subscriptions.get_mut(&msg.channel_id) {
            subscribers.remove(&msg.session_id);
            if subscribers.is_empty() {
                self.channel_subscriptions.remove(&msg.channel_id);
            }
        }
    }
}

/// Message: Subscribe to DM
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct SubscribeDm {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub dm_id: Uuid,
}

impl Handler<SubscribeDm> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: SubscribeDm, ctx: &mut Context<Self>) {
        tracing::debug!(
            "WebSocket: Session {} subscribed to DM {}",
            msg.session_id, msg.dm_id
        );

        // Register subscription
        self.dm_subscriptions
            .entry(msg.dm_id)
            .or_insert_with(HashSet::new)
            .insert(msg.session_id);

        // Fetch and send DM data
        let pool = self.db_pool.clone();
        let session = self.sessions.get(&msg.session_id).cloned();
        let dm_id = msg.dm_id;
        let user_id = msg.user_id;

        let fut = async move {
            // Fetch messages and unread info
            let messages_fut = crate::models::message::Message::get_messages_with_details_for_dm(&pool, dm_id, 50);
            let unread_fut = crate::models::read_status::DmReadStatus::get_unread_info(&pool, user_id, dm_id);

            let (messages, unread_info) = tokio::join!(
                messages_fut,
                unread_fut
            );

            match (messages, unread_info) {
                (Ok(messages), Ok(unread_info)) => {
                    tracing::debug!(
                        "Loaded DM data for dm {}: {} messages",
                        dm_id,
                        messages.len(),
                    );

                    Some(ServerMessage::DmData {
                        dm_id,
                        messages,
                        unread_info,
                    })
                }
                (Err(e), _) | (_, Err(e)) => {
                    tracing::error!("Failed to load DM data for dm {}: {}", dm_id, e);
                    Some(ServerMessage::Error {
                        message: format!("Failed to load DM data: {}", e),
                    })
                }
            }
        }
        .into_actor(self)
        .map(move |result, _actor, _ctx| {
            if let Some(message) = result {
                if let Some(session) = session {
                    session.do_send(WsSessionMessage(message));
                }
            }
        });

        ctx.spawn(fut);
    }
}

/// Message: Unsubscribe from DM
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct UnsubscribeDm {
    pub session_id: Uuid,
    pub dm_id: Uuid,
}

impl Handler<UnsubscribeDm> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: UnsubscribeDm, _: &mut Context<Self>) {
        tracing::debug!(
            "WebSocket: Session {} unsubscribed from DM {}",
            msg.session_id, msg.dm_id
        );

        if let Some(subscribers) = self.dm_subscriptions.get_mut(&msg.dm_id) {
            subscribers.remove(&msg.session_id);
            if subscribers.is_empty() {
                self.dm_subscriptions.remove(&msg.dm_id);
            }
        }
    }
}

/// Message: Update user status
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct UpdateUserStatus {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub status: String,
}

impl Handler<UpdateUserStatus> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: UpdateUserStatus, _: &mut Context<Self>) {
        let status_msg = ServerMessage::UserStatus {
            user_id: msg.user_id,
            status: msg.status,
        };

        // Broadcast to org
        self.send_to_org(&msg.org_id, status_msg, None);
    }
}
/// Message: Broadcast message from Redis Pub/Sub
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct BroadcastMessage {
    pub org_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub message: ServerMessage,
}

impl Handler<BroadcastMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _: &mut Context<Self>) {
        if let Some(channel_id) = msg.channel_id {
            self.send_to_channel(&channel_id, msg.message);
        } else {
            self.send_to_org(&msg.org_id, msg.message, None);
        }
    }
}

/// Message: Broadcast typing indicator from Redis Pub/Sub
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct BroadcastTyping {
    pub org_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub message: ServerMessage,
}

impl Handler<BroadcastTyping> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastTyping, _: &mut Context<Self>) {
        if let Some(channel_id) = msg.channel_id {
            self.send_to_channel(&channel_id, msg.message);
        } else {
            self.send_to_org(&msg.org_id, msg.message, None);
        }
    }
}

/// Message: Broadcast status update from Redis Pub/Sub
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct BroadcastStatus {
    pub org_id: Uuid,
    pub message: ServerMessage,
}

impl Handler<BroadcastStatus> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastStatus, _: &mut Context<Self>) {
        self.send_to_org(&msg.org_id, msg.message, None);
    }
}

/// Message: Broadcast message to a specific user (all their sessions)
#[derive(ActixMessage)]
#[rtype(result = "()")]
#[allow(dead_code)]
pub struct BroadcastToUser {
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub message: ServerMessage,
}

impl Handler<BroadcastToUser> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastToUser, _: &mut Context<Self>) {
        self.send_to_user(&msg.user_id, msg.message);
    }
}

/// Message: Get connection statistics
#[derive(ActixMessage)]
#[rtype(result = "ConnectionStats")]
pub struct GetConnectionStats;

impl Handler<GetConnectionStats> for WsServer {
    type Result = actix::prelude::MessageResult<GetConnectionStats>;

    fn handle(&mut self, _msg: GetConnectionStats, _: &mut Context<Self>) -> Self::Result {
        actix::prelude::MessageResult(self.connection_stats())
    }
}

/// Message: Flush all pending message batches
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct FlushBatches;

impl Handler<FlushBatches> for WsServer {
    type Result = ();

    fn handle(&mut self, _msg: FlushBatches, _: &mut Context<Self>) {
        self.flush_all_batches();
    }
}

/// Message: Mark channel or DM as read
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct MarkAsRead {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub last_message_id: Option<Uuid>,
}

impl Handler<MarkAsRead> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: MarkAsRead, ctx: &mut Context<Self>) {
        let db_pool = self.db_pool.clone();
        let redis_pool = self.redis_pool.clone();
        let channel_id = msg.channel_id;
        let dm_id = msg.dm_id;
        let user_id = msg.user_id;
        let org_id = msg.org_id;
        let last_message_id = msg.last_message_id;

        let fut = async move {
            use crate::cache::read_status::{invalidate_channel_unread_cache, invalidate_dm_unread_cache};
            use crate::models::read_status::{ChannelReadStatus, DmReadStatus};

            if let Some(cid) = channel_id {
                // Mark channel as read
                if let Err(e) = ChannelReadStatus::mark_as_read(&db_pool, user_id, cid, last_message_id).await {
                    tracing::error!("Failed to mark channel as read: {}", e);
                    return None;
                }

                // Invalidate cache
                if let Err(e) = invalidate_channel_unread_cache(&redis_pool, org_id, user_id, cid).await {
                    tracing::warn!("Failed to invalidate channel unread cache: {}", e);
                }

                // Get updated counts
                let unread_count = ChannelReadStatus::get_unread_count(&db_pool, user_id, cid).await.unwrap_or(0);
                let last_read_message_id = ChannelReadStatus::get_last_read_message_id(&db_pool, user_id, cid).await.unwrap_or(None);

                Some((
                    ServerMessage::UnreadCountUpdated {
                        channel_id: Some(cid),
                        dm_id: None,
                        unread_count,
                        last_read_message_id,
                    },
                    user_id,
                    org_id,
                ))
            } else if let Some(did) = dm_id {
                // Mark DM as read
                if let Err(e) = DmReadStatus::mark_as_read(&db_pool, user_id, did, last_message_id).await {
                    tracing::error!("Failed to mark DM as read: {}", e);
                    return None;
                }

                // Invalidate cache
                if let Err(e) = invalidate_dm_unread_cache(&redis_pool, org_id, user_id, did).await {
                    tracing::warn!("Failed to invalidate DM unread cache: {}", e);
                }

                // Get updated counts
                let unread_count = DmReadStatus::get_unread_count(&db_pool, user_id, did).await.unwrap_or(0);
                let last_read_message_id = DmReadStatus::get_last_read_message_id(&db_pool, user_id, did).await.unwrap_or(None);

                Some((
                    ServerMessage::UnreadCountUpdated {
                        channel_id: None,
                        dm_id: Some(did),
                        unread_count,
                        last_read_message_id,
                    },
                    user_id,
                    org_id,
                ))
            } else {
                tracing::error!("MarkAsRead: Neither channel_id nor dm_id provided");
                None
            }
        }
        .into_actor(self)
        .map(move |result, actor, _ctx| {
            if let Some((message, user_id, _org_id)) = result {
                // Send to all sessions of this user
                actor.send_to_user(&user_id, message);
            }
        });

        ctx.spawn(fut);
    }
}

/// Message: Add reaction to a message
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct AddReaction {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub message_id: Uuid,
    pub emoji: String,
}

impl Handler<AddReaction> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: AddReaction, ctx: &mut Context<Self>) {
        let db_pool = self.db_pool.clone();
        let message_id = msg.message_id;
        let user_id = msg.user_id;
        let org_id = msg.org_id;
        let emoji = msg.emoji.clone();

        let fut = async move {
            use crate::models::message::Message as DbMessage;
            use crate::models::reaction::Reaction;

            // Get the message to find channel_id or dm_id
            let message = match DbMessage::get_by_id(&db_pool, message_id).await {
                Ok(Some(m)) => m,
                Ok(None) => {
                    tracing::error!("Message not found for reaction: {}", message_id);
                    return None;
                }
                Err(e) => {
                    tracing::error!("Failed to get message for reaction: {}", e);
                    return None;
                }
            };

            // Add the reaction
            if let Err(e) = Reaction::add(&db_pool, message_id, user_id, &emoji).await {
                tracing::error!("Failed to add reaction: {}", e);
                return None;
            }

            Some((
                ServerMessage::ReactionAdded {
                    message_id,
                    user_id,
                    emoji,
                },
                message.channel_id,
                message.dm_id,
                org_id,
            ))
        }
        .into_actor(self)
        .map(move |result, actor, _ctx| {
            if let Some((message, channel_id, dm_id, org_id)) = result {
                if let Some(cid) = channel_id {
                    actor.send_to_channel(&cid, message);
                } else if dm_id.is_some() {
                    actor.send_to_org(&org_id, message, None);
                }
            }
        });

        ctx.spawn(fut);
    }
}

/// Message: Remove reaction from a message
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct RemoveReaction {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub message_id: Uuid,
    pub emoji: String,
}

impl Handler<RemoveReaction> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: RemoveReaction, ctx: &mut Context<Self>) {
        let db_pool = self.db_pool.clone();
        let message_id = msg.message_id;
        let user_id = msg.user_id;
        let org_id = msg.org_id;
        let emoji = msg.emoji.clone();

        let fut = async move {
            use crate::models::message::Message as DbMessage;
            use crate::models::reaction::Reaction;

            // Get the message to find channel_id or dm_id
            let message = match DbMessage::get_by_id(&db_pool, message_id).await {
                Ok(Some(m)) => m,
                Ok(None) => {
                    tracing::error!("Message not found for reaction removal: {}", message_id);
                    return None;
                }
                Err(e) => {
                    tracing::error!("Failed to get message for reaction removal: {}", e);
                    return None;
                }
            };

            // Remove the reaction
            if let Err(e) = Reaction::remove(&db_pool, message_id, user_id, &emoji).await {
                tracing::error!("Failed to remove reaction: {}", e);
                return None;
            }

            Some((
                ServerMessage::ReactionRemoved {
                    message_id,
                    user_id,
                    emoji,
                },
                message.channel_id,
                message.dm_id,
                org_id,
            ))
        }
        .into_actor(self)
        .map(move |result, actor, _ctx| {
            if let Some((message, channel_id, dm_id, org_id)) = result {
                if let Some(cid) = channel_id {
                    actor.send_to_channel(&cid, message);
                } else if dm_id.is_some() {
                    actor.send_to_org(&org_id, message, None);
                }
            }
        });

        ctx.spawn(fut);
    }
}

/// Message: Pin a message
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct PinMessage {
    pub user_id: Uuid,
    pub user_name: String,
    pub org_id: Uuid,
    pub message_id: Uuid,
}

impl Handler<PinMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: PinMessage, ctx: &mut Context<Self>) {
        let db_pool = self.db_pool.clone();
        let redis_pool = self.redis_pool.clone();
        let message_id = msg.message_id;
        let user_id = msg.user_id;
        let user_name = msg.user_name.clone();
        let org_id = msg.org_id;

        let fut = async move {
            use crate::cache::pins::invalidate_pins_cache;
            use crate::models::message::Message as DbMessage;
            use crate::models::pin::PinnedMessage;

            // Get the message to find channel_id
            let message = match DbMessage::get_by_id(&db_pool, message_id).await {
                Ok(Some(m)) => m,
                Ok(None) => {
                    tracing::error!("Message not found for pin: {}", message_id);
                    return None;
                }
                Err(e) => {
                    tracing::error!("Failed to get message for pin: {}", e);
                    return None;
                }
            };

            let channel_id = match message.channel_id {
                Some(cid) => cid,
                None => {
                    tracing::error!("Cannot pin DM messages");
                    return None;
                }
            };

            // Pin the message
            let pin = match PinnedMessage::pin(&db_pool, channel_id, message_id, user_id).await {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("Failed to pin message: {}", e);
                    return None;
                }
            };

            // Invalidate cache
            if let Err(e) = invalidate_pins_cache(&redis_pool, org_id, channel_id).await {
                tracing::warn!("Failed to invalidate pins cache: {}", e);
            }

            Some((
                ServerMessage::MessagePinned {
                    channel_id,
                    message_id,
                    pinned_by: user_id,
                    pinned_by_name: user_name,
                    pinned_at: pin.pinned_at.to_rfc3339(),
                },
                channel_id,
            ))
        }
        .into_actor(self)
        .map(move |result, actor, _ctx| {
            if let Some((message, channel_id)) = result {
                actor.send_to_channel(&channel_id, message);
            }
        });

        ctx.spawn(fut);
    }
}

/// Message: Unpin a message
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct UnpinMessage {
    pub user_id: Uuid,
    pub user_name: String,
    pub org_id: Uuid,
    pub message_id: Uuid,
}

impl Handler<UnpinMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: UnpinMessage, ctx: &mut Context<Self>) {
        let db_pool = self.db_pool.clone();
        let redis_pool = self.redis_pool.clone();
        let message_id = msg.message_id;
        let user_id = msg.user_id;
        let user_name = msg.user_name.clone();
        let org_id = msg.org_id;

        let fut = async move {
            use crate::cache::pins::invalidate_pins_cache;
            use crate::models::message::Message as DbMessage;
            use crate::models::pin::PinnedMessage;

            // Get the message to find channel_id
            let message = match DbMessage::get_by_id(&db_pool, message_id).await {
                Ok(Some(m)) => m,
                Ok(None) => {
                    tracing::error!("Message not found for unpin: {}", message_id);
                    return None;
                }
                Err(e) => {
                    tracing::error!("Failed to get message for unpin: {}", e);
                    return None;
                }
            };

            let channel_id = match message.channel_id {
                Some(cid) => cid,
                None => {
                    tracing::error!("Cannot unpin DM messages");
                    return None;
                }
            };

            // Unpin the message
            if let Err(e) = PinnedMessage::unpin(&db_pool, channel_id, message_id).await {
                tracing::error!("Failed to unpin message: {}", e);
                return None;
            }

            // Invalidate cache
            if let Err(e) = invalidate_pins_cache(&redis_pool, org_id, channel_id).await {
                tracing::warn!("Failed to invalidate pins cache: {}", e);
            }

            Some((
                ServerMessage::MessageUnpinned {
                    channel_id,
                    message_id,
                    unpinned_by: user_id,
                    unpinned_by_name: user_name,
                },
                channel_id,
            ))
        }
        .into_actor(self)
        .map(move |result, actor, _ctx| {
            if let Some((message, channel_id)) = result {
                actor.send_to_channel(&channel_id, message);
            }
        });

        ctx.spawn(fut);
    }
}

/// Message: Add bookmark to a message
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct AddBookmark {
    pub user_id: Uuid,
    pub message_id: Uuid,
}

impl Handler<AddBookmark> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: AddBookmark, ctx: &mut Context<Self>) {
        let db_pool = self.db_pool.clone();
        let message_id = msg.message_id;
        let user_id = msg.user_id;

        let fut = async move {
            use crate::models::bookmark::Bookmark;

            // Create the bookmark
            let bookmark = match Bookmark::create(&db_pool, user_id, message_id).await {
                Ok(b) => b,
                Err(e) => {
                    tracing::error!("Failed to create bookmark: {}", e);
                    return None;
                }
            };

            Some((
                ServerMessage::BookmarkAdded {
                    message_id,
                    bookmarked_at: bookmark.bookmarked_at.to_rfc3339(),
                },
                user_id,
            ))
        }
        .into_actor(self)
        .map(move |result, actor, _ctx| {
            if let Some((message, user_id)) = result {
                // Bookmarks are user-specific, only send to the user
                actor.send_to_user(&user_id, message);
            }
        });

        ctx.spawn(fut);
    }
}

/// Message: Remove bookmark from a message
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct RemoveBookmark {
    pub user_id: Uuid,
    pub message_id: Uuid,
}

impl Handler<RemoveBookmark> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: RemoveBookmark, ctx: &mut Context<Self>) {
        let db_pool = self.db_pool.clone();
        let message_id = msg.message_id;
        let user_id = msg.user_id;

        let fut = async move {
            use crate::models::bookmark::Bookmark;

            // Delete the bookmark
            if let Err(e) = Bookmark::delete(&db_pool, user_id, message_id).await {
                tracing::error!("Failed to delete bookmark: {}", e);
                return None;
            }

            Some((
                ServerMessage::BookmarkRemoved { message_id },
                user_id,
            ))
        }
        .into_actor(self)
        .map(move |result, actor, _ctx| {
            if let Some((message, user_id)) = result {
                // Bookmarks are user-specific, only send to the user
                actor.send_to_user(&user_id, message);
            }
        });

        ctx.spawn(fut);
    }
}

/// Message: Edit a message
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct EditMessage {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub message_id: Uuid,
    pub content: String,
    pub encrypted_content: Option<String>,
    pub encryption_metadata: Option<serde_json::Value>,
}

impl Handler<EditMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: EditMessage, ctx: &mut Context<Self>) {
        let db_pool = self.db_pool.clone();
        let message_id = msg.message_id;
        let user_id = msg.user_id;
        let org_id = msg.org_id;
        let content = msg.content.clone();
        let encrypted_content = msg.encrypted_content.clone();
        let encryption_metadata = msg.encryption_metadata.clone();

        let fut = async move {
            use crate::models::message::Message as DbMessage;

            // Get the message to verify ownership and get channel/dm info
            let message = match DbMessage::get_by_id(&db_pool, message_id).await {
                Ok(Some(m)) => m,
                Ok(None) => {
                    tracing::error!("Message not found for edit: {}", message_id);
                    return None;
                }
                Err(e) => {
                    tracing::error!("Failed to get message for edit: {}", e);
                    return None;
                }
            };

            // Verify the user owns this message
            if message.user_id != user_id {
                tracing::error!("User {} attempted to edit message {} owned by {}", user_id, message_id, message.user_id);
                return None;
            }

            // Update the message (encrypted or plaintext)
            let updated = if let (Some(ec), Some(em)) = (&encrypted_content, &encryption_metadata) {
                use base64::Engine;
                match base64::engine::general_purpose::STANDARD.decode(ec) {
                    Ok(enc_bytes) => {
                        match DbMessage::update_encrypted(&db_pool, message_id, &content, &enc_bytes, em.clone(), user_id).await {
                            Ok(m) => m,
                            Err(e) => {
                                tracing::error!("Failed to update encrypted message: {}", e);
                                return None;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to decode encrypted content for edit: {}", e);
                        return None;
                    }
                }
            } else {
                match DbMessage::update(&db_pool, message_id, &content, user_id).await {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::error!("Failed to update message: {}", e);
                        return None;
                    }
                }
            };

            let edited_at = updated.edited_at.map(|dt| dt.to_rfc3339()).unwrap_or_default();

            Some((
                ServerMessage::MessageEdited {
                    message_id,
                    content,
                    edited_at,
                    encrypted_content,
                    encryption_metadata,
                },
                message.channel_id,
                message.dm_id,
                org_id,
            ))
        }
        .into_actor(self)
        .map(move |result, actor, _ctx| {
            if let Some((message, channel_id, dm_id, org_id)) = result {
                if let Some(cid) = channel_id {
                    actor.send_to_channel(&cid, message);
                } else if dm_id.is_some() {
                    actor.send_to_org(&org_id, message, None);
                }
            }
        });

        ctx.spawn(fut);
    }
}

/// Message: Delete a message
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct DeleteMessage {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub message_id: Uuid,
}

impl Handler<DeleteMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: DeleteMessage, ctx: &mut Context<Self>) {
        let db_pool = self.db_pool.clone();
        let message_id = msg.message_id;
        let user_id = msg.user_id;
        let org_id = msg.org_id;

        let fut = async move {
            use crate::models::message::Message as DbMessage;

            // Get the message to verify ownership and get channel/dm info
            let message = match DbMessage::get_by_id(&db_pool, message_id).await {
                Ok(Some(m)) => m,
                Ok(None) => {
                    tracing::error!("Message not found for delete: {}", message_id);
                    return None;
                }
                Err(e) => {
                    tracing::error!("Failed to get message for delete: {}", e);
                    return None;
                }
            };

            // Verify the user owns this message
            if message.user_id != user_id {
                tracing::error!("User {} attempted to delete message {} owned by {}", user_id, message_id, message.user_id);
                return None;
            }

            // Soft delete the message
            if let Err(e) = DbMessage::soft_delete(&db_pool, message_id).await {
                tracing::error!("Failed to delete message: {}", e);
                return None;
            }

            Some((
                ServerMessage::MessageDeleted { message_id },
                message.channel_id,
                message.dm_id,
                org_id,
            ))
        }
        .into_actor(self)
        .map(move |result, actor, _ctx| {
            if let Some((message, channel_id, dm_id, org_id)) = result {
                if let Some(cid) = channel_id {
                    actor.send_to_channel(&cid, message);
                } else if dm_id.is_some() {
                    actor.send_to_org(&org_id, message, None);
                }
            }
        });

        ctx.spawn(fut);
    }
}

/// Message: Subscribe to a thread
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct SubscribeThread {
    pub session_id: Uuid,
    pub message_id: Uuid, // The parent message ID (thread root)
}

impl Handler<SubscribeThread> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: SubscribeThread, _: &mut Context<Self>) {
        tracing::debug!(
            "WebSocket: Session {} subscribed to thread {}",
            msg.session_id, msg.message_id
        );

        // Register thread subscription
        self.thread_subscriptions
            .entry(msg.message_id)
            .or_insert_with(HashSet::new)
            .insert(msg.session_id);
    }
}

/// Message: Unsubscribe from a thread
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct UnsubscribeThread {
    pub session_id: Uuid,
    pub message_id: Uuid, // The parent message ID (thread root)
}

impl Handler<UnsubscribeThread> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: UnsubscribeThread, _: &mut Context<Self>) {
        tracing::debug!(
            "WebSocket: Session {} unsubscribed from thread {}",
            msg.session_id, msg.message_id
        );

        if let Some(subscribers) = self.thread_subscriptions.get_mut(&msg.message_id) {
            subscribers.remove(&msg.session_id);
            if subscribers.is_empty() {
                self.thread_subscriptions.remove(&msg.message_id);
            }
        }
    }
}

/// Message: Broadcast to thread subscribers
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct BroadcastToThread {
    pub parent_message_id: Uuid,
    pub message: ServerMessage,
}

impl Handler<BroadcastToThread> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastToThread, _: &mut Context<Self>) {
        if let Some(session_ids) = self.thread_subscriptions.get(&msg.parent_message_id) {
            let session_ids: Vec<Uuid> = session_ids.iter().copied().collect();
            for session_id in session_ids {
                self.send_message(&session_id, msg.message.clone());
            }
        }
    }
}
