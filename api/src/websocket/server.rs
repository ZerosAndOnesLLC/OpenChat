use actix::{Actor, Addr, Context, Handler, Message as ActixMessage};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::messages::ServerMessage;
use super::session::{WsSessionHandle, WsSessionMessage};

/// WebSocket server that manages all connections
pub struct WsServer {
    /// Map of session_id -> session address
    sessions: HashMap<Uuid, Addr<WsSessionHandle>>,
    /// Map of user_id -> set of session_ids (for multi-device support)
    user_sessions: HashMap<Uuid, HashSet<Uuid>>,
    /// Map of org_id -> set of session_ids
    org_sessions: HashMap<Uuid, HashSet<Uuid>>,
    /// Map of channel_id -> set of session_ids (subscriptions)
    channel_subscriptions: HashMap<Uuid, HashSet<Uuid>>,
}

impl WsServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            user_sessions: HashMap::new(),
            org_sessions: HashMap::new(),
            channel_subscriptions: HashMap::new(),
        }
    }

    /// Send message to a specific session
    fn send_message(&self, session_id: &Uuid, message: ServerMessage) {
        if let Some(addr) = self.sessions.get(session_id) {
            addr.do_send(WsSessionMessage(message));
        }
    }

    /// Send message to all users in an organization (except excluded sessions)
    fn send_to_org(&self, org_id: &Uuid, message: ServerMessage, exclude: Option<Uuid>) {
        if let Some(session_ids) = self.org_sessions.get(org_id) {
            for session_id in session_ids {
                if let Some(exclude_id) = exclude {
                    if session_id == &exclude_id {
                        continue;
                    }
                }
                self.send_message(session_id, message.clone());
            }
        }
    }

    /// Send message to all subscribers of a channel
    fn send_to_channel(&self, channel_id: &Uuid, message: ServerMessage) {
        if let Some(session_ids) = self.channel_subscriptions.get(channel_id) {
            for session_id in session_ids {
                self.send_message(session_id, message.clone());
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
        println!(
            "WebSocket: User {} connected (session: {})",
            msg.user_id, msg.session_id
        );

        // Store session
        self.sessions.insert(msg.session_id, msg.addr);

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
        println!(
            "WebSocket: User {} disconnected (session: {})",
            msg.user_id, msg.session_id
        );

        // Remove session
        self.sessions.remove(&msg.session_id);

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
    }
}

/// Message: Send a chat message
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct SendMessage {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub content: String,
    pub parent_message_id: Option<Uuid>,
}

impl Handler<SendMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, _: &mut Context<Self>) {
        // In Phase 9, we just broadcast the message immediately
        // In Phase 10+, we'll save to DB first, then broadcast

        let new_message = ServerMessage::NewMessage {
            id: Uuid::new_v4(),
            channel_id: msg.channel_id,
            dm_id: msg.dm_id,
            user_id: msg.user_id,
            content: msg.content,
            parent_message_id: msg.parent_message_id,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        if let Some(channel_id) = msg.channel_id {
            // Broadcast to channel subscribers
            self.send_to_channel(&channel_id, new_message);
        } else if let Some(_dm_id) = msg.dm_id {
            // For DMs, broadcast to all participants (simplified for Phase 9)
            // In production, we'd need to check DM participants
            self.send_to_org(&msg.org_id, new_message, None);
        }
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
            if let Some(session_ids) = self.channel_subscriptions.get(&channel_id) {
                for session_id in session_ids {
                    if session_id != &msg.exclude_session {
                        self.send_message(session_id, typing_msg.clone());
                    }
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
    pub channel_id: Uuid,
}

impl Handler<SubscribeChannel> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: SubscribeChannel, _: &mut Context<Self>) {
        println!(
            "WebSocket: Session {} subscribed to channel {}",
            msg.session_id, msg.channel_id
        );

        self.channel_subscriptions
            .entry(msg.channel_id)
            .or_insert_with(HashSet::new)
            .insert(msg.session_id);
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
            "WebSocket: Session {} unsubscribed from channel {}",
            msg.session_id, msg.channel_id
        );

        if let Some(subscribers) = self.channel_subscriptions.get_mut(&msg.channel_id) {
            subscribers.remove(&msg.session_id);
            if subscribers.is_empty() {
                self.channel_subscriptions.remove(&msg.channel_id);
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
