use actix::{Addr, Handler, Message as ActixMessage};
use actix_ws::Message as WsMessage;
use futures_util::StreamExt;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use uuid::Uuid;

use super::messages::{ClientMessage, ServerMessage};
use super::server::{self, WsServer};

/// WebSocket session data
pub struct WsSessionData {
    pub id: Uuid,
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub user_name: String,
    pub server: Addr<WsServer>,
}

impl WsSessionData {
    pub fn new(
        user_id: Uuid,
        org_id: Uuid,
        user_name: String,
        server: Addr<WsServer>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            org_id,
            user_name,
            server,
        }
    }

    /// Handle incoming client messages
    fn handle_client_message(&self, msg: ClientMessage, tx: &mpsc::UnboundedSender<ServerMessage>) {
        match msg {
            ClientMessage::SendMessage {
                channel_id,
                dm_id,
                content,
                parent_message_id,
            } => {
                self.server.do_send(server::SendMessage {
                    user_id: self.user_id,
                    user_name: self.user_name.clone(),
                    org_id: self.org_id,
                    channel_id,
                    dm_id,
                    content,
                    parent_message_id,
                });
            }
            ClientMessage::Typing { channel_id, dm_id } => {
                self.server.do_send(server::TypingIndicator {
                    user_id: self.user_id,
                    user_name: self.user_name.clone(),
                    org_id: self.org_id,
                    channel_id,
                    dm_id,
                    exclude_session: self.id,
                });
            }
            ClientMessage::SubscribeChannel { channel_id } => {
                self.server.do_send(server::SubscribeChannel {
                    session_id: self.id,
                    channel_id,
                });
            }
            ClientMessage::UnsubscribeChannel { channel_id } => {
                self.server.do_send(server::UnsubscribeChannel {
                    session_id: self.id,
                    channel_id,
                });
            }
            ClientMessage::UpdateStatus { status } => {
                self.server.do_send(server::UpdateUserStatus {
                    user_id: self.user_id,
                    org_id: self.org_id,
                    status,
                });
            }
            ClientMessage::Ping => {
                let pong = ServerMessage::Pong;
                let _ = tx.send(pong);
            }
        }
    }
}

/// Handle WebSocket connection
pub async fn handle_ws_session(
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    session_data: WsSessionData,
) {
    let session_id = session_data.id;
    let user_id = session_data.user_id;
    let org_id = session_data.org_id;
    let server = session_data.server.clone();

    // Create channel for sending messages to the WebSocket client
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Create session address for receiving broadcasts from WsServer
    let session_addr = WsSessionHandle {
        tx: tx.clone(),
    };
    let session_addr = actix::Actor::start(session_addr);

    // Register with server
    server.do_send(server::Connect {
        session_id,
        user_id,
        org_id,
        addr: session_addr,
    });

    // Send connected message
    let connected_msg = ServerMessage::Connected { user_id };
    let _ = tx.send(connected_msg);

    // Clone session for the write task
    let mut session_clone = session.clone();

    // Spawn heartbeat task
    let mut session_heartbeat = session.clone();
    let mut heartbeat_interval = interval(Duration::from_secs(30));
    let heartbeat_task = tokio::spawn(async move {
        loop {
            heartbeat_interval.tick().await;
            if session_heartbeat.ping(b"").await.is_err() {
                break;
            }
        }
    });

    // Spawn task to write messages to WebSocket
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if session_clone.text(json).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming WebSocket messages
    let mut last_heartbeat = Instant::now();
    while let Some(Ok(msg)) = msg_stream.next().await {
        match msg {
            WsMessage::Ping(bytes) => {
                last_heartbeat = Instant::now();
                if session.pong(&bytes).await.is_err() {
                    break;
                }
            }
            WsMessage::Pong(_) => {
                last_heartbeat = Instant::now();
            }
            WsMessage::Text(text) => {
                last_heartbeat = Instant::now();

                // Parse client message
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        session_data.handle_client_message(client_msg, &tx);
                    }
                    Err(e) => {
                        println!("Failed to parse client message: {}", e);
                        let error_msg = ServerMessage::Error {
                            message: format!("Invalid message format: {}", e),
                        };
                        let _ = tx.send(error_msg);
                    }
                }
            }
            WsMessage::Close(_) => break,
            _ => {}
        }

        // Check heartbeat timeout
        if Instant::now().duration_since(last_heartbeat) > Duration::from_secs(60) {
            println!("WebSocket heartbeat timeout, disconnecting session {}", session_id);
            break;
        }
    }

    // Cleanup
    heartbeat_task.abort();
    write_task.abort();

    // Disconnect from server
    server.do_send(server::Disconnect {
        session_id,
        user_id,
        org_id,
    });
}

/// Actor wrapper for receiving messages from WsServer
pub struct WsSessionHandle {
    tx: mpsc::UnboundedSender<ServerMessage>,
}

impl actix::Actor for WsSessionHandle {
    type Context = actix::Context<Self>;
}

/// Message wrapper for sending to client
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct WsSessionMessage(pub ServerMessage);

impl Handler<WsSessionMessage> for WsSessionHandle {
    type Result = ();

    fn handle(&mut self, msg: WsSessionMessage, _: &mut Self::Context) {
        let _ = self.tx.send(msg.0);
    }
}
