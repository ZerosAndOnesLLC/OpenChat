use actix::{Actor, Addr, AsyncContext, Context, Handler, Message as ActixMessage, StreamHandler};
use futures_util::stream::StreamExt;
use redis::aio::PubSub;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use super::messages::ServerMessage;
use super::server::{WsServer, SendMessage, TypingIndicator, UpdateUserStatus};

/// Redis Pub/Sub channels for cross-instance communication
pub struct RedisPubSubChannels;

impl RedisPubSubChannels {
    /// Channel for organization-wide messages
    pub fn org_channel(org_id: Uuid) -> String {
        format!("openchat:org:{}:events", org_id)
    }

    /// Channel for specific channel messages
    pub fn channel_channel(channel_id: Uuid) -> String {
        format!("openchat:channel:{}:events", channel_id)
    }

    /// Channel for typing indicators
    pub fn typing_channel(org_id: Uuid) -> String {
        format!("openchat:org:{}:typing", org_id)
    }

    /// Channel for status updates
    pub fn status_channel(org_id: Uuid) -> String {
        format!("openchat:org:{}:status", org_id)
    }
}

/// Events published to Redis for cross-instance communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PubSubEvent {
    /// New message event
    NewMessage {
        id: Uuid,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        user_id: Uuid,
        org_id: Uuid,
        content: String,
        parent_message_id: Option<Uuid>,
        created_at: String,
    },
    /// Message edited event
    MessageEdited {
        message_id: Uuid,
        org_id: Uuid,
        content: String,
        edited_at: String,
    },
    /// Message deleted event
    MessageDeleted {
        message_id: Uuid,
        org_id: Uuid,
    },
    /// Typing indicator
    Typing {
        user_id: Uuid,
        user_name: String,
        org_id: Uuid,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
    },
    /// User status update
    StatusUpdate {
        user_id: Uuid,
        org_id: Uuid,
        status: String,
    },
    /// Reaction added
    ReactionAdded {
        message_id: Uuid,
        user_id: Uuid,
        org_id: Uuid,
        emoji: String,
    },
    /// Reaction removed
    ReactionRemoved {
        message_id: Uuid,
        user_id: Uuid,
        org_id: Uuid,
        emoji: String,
    },
}

/// Redis Pub/Sub manager
pub struct RedisPubSub {
    redis_client: redis::Client,
    ws_server: Addr<WsServer>,
}

impl RedisPubSub {
    pub fn new(redis_url: &str, ws_server: Addr<WsServer>) -> Result<Self, redis::RedisError> {
        let redis_client = redis::Client::open(redis_url)?;
        Ok(Self {
            redis_client,
            ws_server,
        })
    }

    /// Start subscribing to Redis channels
    async fn subscribe_to_channels(&self, ctx: &mut Context<Self>) {
        let redis_client = self.redis_client.clone();
        let ws_server = self.ws_server.clone();

        // Spawn a task to handle Redis subscriptions
        ctx.spawn(
            async move {
                loop {
                    match redis_client.get_multiplexed_async_connection().await {
                        Ok(con) => {
                            let mut pubsub = con.into_pubsub();

                            // Subscribe to a pattern to catch all our channels
                            if let Err(e) = pubsub.psubscribe("openchat:*").await {
                                tracing::error!("Failed to subscribe to Redis channels: {}", e);
                                tokio::time::sleep(Duration::from_secs(5)).await;
                                continue;
                            }

                            tracing::info!("Redis Pub/Sub: Subscribed to openchat:* channels");

                            // Process messages
                            let mut stream = pubsub.on_message();
                            while let Some(msg) = stream.next().await {
                                let payload: String = match msg.get_payload() {
                                    Ok(p) => p,
                                    Err(e) => {
                                        tracing::error!("Failed to get Redis message payload: {}", e);
                                        continue;
                                    }
                                };

                                // Parse the event
                                match serde_json::from_str::<PubSubEvent>(&payload) {
                                    Ok(event) => {
                                        Self::handle_pubsub_event(event, &ws_server);
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to parse PubSubEvent: {}", e);
                                    }
                                }
                            }

                            tracing::warn!("Redis Pub/Sub stream ended, reconnecting...");
                        }
                        Err(e) => {
                            tracing::error!("Failed to connect to Redis: {}", e);
                        }
                    }

                    // Wait before reconnecting
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
            .into_actor(self),
        );
    }

    /// Handle a pub/sub event and forward to WebSocket server
    fn handle_pubsub_event(event: PubSubEvent, ws_server: &Addr<WsServer>) {
        match event {
            PubSubEvent::NewMessage {
                id,
                channel_id,
                dm_id,
                user_id,
                org_id,
                content,
                parent_message_id,
                created_at,
            } => {
                ws_server.do_send(BroadcastMessage {
                    org_id,
                    channel_id,
                    message: ServerMessage::NewMessage {
                        id,
                        channel_id,
                        dm_id,
                        user_id,
                        content,
                        parent_message_id,
                        created_at,
                    },
                });
            }
            PubSubEvent::MessageEdited {
                message_id,
                org_id,
                content,
                edited_at,
            } => {
                ws_server.do_send(BroadcastMessage {
                    org_id,
                    channel_id: None,
                    message: ServerMessage::MessageEdited {
                        message_id,
                        content,
                        edited_at,
                    },
                });
            }
            PubSubEvent::MessageDeleted { message_id, org_id } => {
                ws_server.do_send(BroadcastMessage {
                    org_id,
                    channel_id: None,
                    message: ServerMessage::MessageDeleted { message_id },
                });
            }
            PubSubEvent::Typing {
                user_id,
                user_name,
                org_id,
                channel_id,
                dm_id,
            } => {
                ws_server.do_send(BroadcastTyping {
                    org_id,
                    channel_id,
                    message: ServerMessage::UserTyping {
                        user_id,
                        channel_id,
                        dm_id,
                        user_name,
                    },
                });
            }
            PubSubEvent::StatusUpdate {
                user_id,
                org_id,
                status,
            } => {
                ws_server.do_send(BroadcastStatus {
                    org_id,
                    message: ServerMessage::UserStatus { user_id, status },
                });
            }
            PubSubEvent::ReactionAdded {
                message_id,
                user_id,
                org_id,
                emoji,
            } => {
                ws_server.do_send(BroadcastMessage {
                    org_id,
                    channel_id: None,
                    message: ServerMessage::ReactionAdded {
                        message_id,
                        user_id,
                        emoji,
                    },
                });
            }
            PubSubEvent::ReactionRemoved {
                message_id,
                user_id,
                org_id,
                emoji,
            } => {
                ws_server.do_send(BroadcastMessage {
                    org_id,
                    channel_id: None,
                    message: ServerMessage::ReactionRemoved {
                        message_id,
                        user_id,
                        emoji,
                    },
                });
            }
        }
    }
}

impl Actor for RedisPubSub {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        tracing::info!("Redis Pub/Sub actor started");
        self.subscribe_to_channels(ctx);
    }
}

/// Message to broadcast to WebSocket clients via org
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct BroadcastMessage {
    pub org_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub message: ServerMessage,
}

/// Message to broadcast typing indicator
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct BroadcastTyping {
    pub org_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub message: ServerMessage,
}

/// Message to broadcast status update
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct BroadcastStatus {
    pub org_id: Uuid,
    pub message: ServerMessage,
}

/// Publish an event to Redis
#[derive(ActixMessage)]
#[rtype(result = "Result<(), redis::RedisError>")]
pub struct PublishEvent {
    pub channel: String,
    pub event: PubSubEvent,
}

impl Handler<PublishEvent> for RedisPubSub {
    type Result = Result<(), redis::RedisError>;

    fn handle(&mut self, msg: PublishEvent, _ctx: &mut Self::Context) -> Self::Result {
        let redis_client = self.redis_client.clone();
        let channel = msg.channel;
        let event = msg.event;

        // Spawn async task to publish
        actix::spawn(async move {
            match redis_client.get_multiplexed_async_connection().await {
                Ok(mut con) => {
                    let payload = match serde_json::to_string(&event) {
                        Ok(p) => p,
                        Err(e) => {
                            tracing::error!("Failed to serialize PubSubEvent: {}", e);
                            return;
                        }
                    };

                    if let Err(e) = con.publish::<_, _, ()>(&channel, payload).await {
                        tracing::error!("Failed to publish to Redis: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get Redis connection: {}", e);
                }
            }
        });

        Ok(())
    }
}
