use actix::{Actor, Addr, AsyncContext, Handler, Message as ActixMessage, StreamHandler};
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::messages::{ClientMessage, ServerMessage};
use super::server::{self, WsServer};

/// WebSocket session for a single client connection
pub struct WsSession {
    /// Unique session ID
    pub id: Uuid,
    /// User ID
    pub user_id: Uuid,
    /// Organization ID
    pub org_id: Uuid,
    /// User display name
    pub user_name: String,
    /// Last heartbeat time
    pub heartbeat: Instant,
    /// WebSocket server address
    pub server: Addr<WsServer>,
}

impl WsSession {
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
            heartbeat: Instant::now(),
            server,
        }
    }

    /// Start heartbeat to check if client is still alive
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::from_secs(30), |act, ctx| {
            // Check if heartbeat is too old
            if Instant::now().duration_since(act.heartbeat) > Duration::from_secs(60) {
                // Heartbeat timeout - disconnect
                println!("WebSocket heartbeat timeout, disconnecting session {}", act.id);
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }

    /// Handle incoming client messages
    fn handle_client_message(&mut self, msg: ClientMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            ClientMessage::SendMessage {
                channel_id,
                dm_id,
                content,
                parent_message_id,
            } => {
                self.server.do_send(server::SendMessage {
                    session_id: self.id,
                    user_id: self.user_id,
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
                if let Ok(json) = serde_json::to_string(&pong) {
                    ctx.text(json);
                }
            }
        }
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Start heartbeat
        self.start_heartbeat(ctx);

        // Register with server
        self.server.do_send(server::Connect {
            session_id: self.id,
            user_id: self.user_id,
            org_id: self.org_id,
            addr: ctx.address(),
        });

        // Send connected message
        let msg = ServerMessage::Connected {
            user_id: self.user_id,
        };
        if let Ok(json) = serde_json::to_string(&msg) {
            ctx.text(json);
        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // Disconnect from server
        self.server.do_send(server::Disconnect {
            session_id: self.id,
            user_id: self.user_id,
            org_id: self.org_id,
        });
    }
}

/// Handle WebSocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                self.heartbeat = Instant::now();

                // Parse client message
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        self.handle_client_message(client_msg, ctx);
                    }
                    Err(e) => {
                        println!("Failed to parse client message: {}", e);
                        let error_msg = ServerMessage::Error {
                            message: format!("Invalid message format: {}", e),
                        };
                        if let Ok(json) = serde_json::to_string(&error_msg) {
                            ctx.text(json);
                        }
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                println!("Binary messages not supported");
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

/// Message wrapper for sending to client
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct WsMessage(pub ServerMessage);

impl Handler<WsMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        if let Ok(json) = serde_json::to_string(&msg.0) {
            ctx.text(json);
        }
    }
}
