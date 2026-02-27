use actix::{Actor, Addr, AsyncContext, Context, Handler, Message as ActixMessage, WrapFuture};
use futures_util::stream::StreamExt;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use super::messages::ServerMessage;
use super::server::WsServer;

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
        user_name: String,
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
    /// Unread count updated
    UnreadCountUpdated {
        user_id: Uuid,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        org_id: Uuid,
        unread_count: i32,
        last_read_message_id: Option<Uuid>,
    },
    /// New notification created
    NewNotification {
        user_id: Uuid,
        notification_id: Uuid,
        notification_type: String,
        message_id: Option<Uuid>,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        org_id: Uuid,
        created_at: String,
    },
    /// Notification count updated
    NotificationCountUpdated {
        user_id: Uuid,
        org_id: Uuid,
        unread_count: i32,
    },
    /// Reminder triggered
    ReminderTriggered {
        user_id: Uuid,
        org_id: Uuid,
        reminder_id: Uuid,
        message_id: Uuid,
        channel_id: Option<Uuid>,
        dm_id: Option<Uuid>,
        message_preview: String,
        created_at: String,
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
                    match redis_client.get_async_pubsub().await {
                        Ok(mut pubsub) => {
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
                user_name,
                org_id,
                content,
                parent_message_id,
                created_at,
            } => {
                ws_server.do_send(super::server::BroadcastMessage {
                    org_id,
                    channel_id,
                    message: ServerMessage::NewMessage {
                        id,
                        channel_id,
                        dm_id,
                        user_id,
                        user_name,
                        user_avatar: None,
                        content,
                        parent_message_id,
                        created_at,
                        is_webhook: None,
                        forwarded_from_message_id: None,
                        forwarded_from_channel_id: None,
                        forwarded_from_channel_name: None,
                        encrypted_content: None,
                        encryption_metadata: None,
                    },
                });
            }
            PubSubEvent::MessageEdited {
                message_id,
                org_id,
                content,
                edited_at,
            } => {
                ws_server.do_send(super::server::BroadcastMessage {
                    org_id,
                    channel_id: None,
                    message: ServerMessage::MessageEdited {
                        message_id,
                        content,
                        edited_at,
                        encrypted_content: None,
                        encryption_metadata: None,
                    },
                });
            }
            PubSubEvent::MessageDeleted { message_id, org_id } => {
                ws_server.do_send(super::server::BroadcastMessage {
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
                ws_server.do_send(super::server::BroadcastTyping {
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
                ws_server.do_send(super::server::BroadcastStatus {
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
                ws_server.do_send(super::server::BroadcastMessage {
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
                ws_server.do_send(super::server::BroadcastMessage {
                    org_id,
                    channel_id: None,
                    message: ServerMessage::ReactionRemoved {
                        message_id,
                        user_id,
                        emoji,
                    },
                });
            }
            PubSubEvent::UnreadCountUpdated {
                user_id: _,
                channel_id,
                dm_id,
                org_id,
                unread_count,
                last_read_message_id,
            } => {
                ws_server.do_send(super::server::BroadcastMessage {
                    org_id,
                    channel_id,
                    message: ServerMessage::UnreadCountUpdated {
                        channel_id,
                        dm_id,
                        unread_count,
                        last_read_message_id,
                    },
                });
            }
            PubSubEvent::NewNotification {
                user_id,
                notification_id,
                notification_type,
                message_id,
                channel_id,
                dm_id,
                org_id,
                created_at,
            } => {
                ws_server.do_send(super::server::BroadcastToUser {
                    org_id,
                    user_id,
                    message: ServerMessage::NewNotification {
                        notification_id,
                        notification_type,
                        message_id,
                        channel_id,
                        dm_id,
                        created_at,
                    },
                });
            }
            PubSubEvent::NotificationCountUpdated {
                user_id,
                org_id,
                unread_count,
            } => {
                ws_server.do_send(super::server::BroadcastToUser {
                    org_id,
                    user_id,
                    message: ServerMessage::NotificationCountUpdated { unread_count },
                });
            }
            PubSubEvent::ReminderTriggered {
                user_id,
                org_id,
                reminder_id,
                message_id,
                channel_id,
                dm_id,
                message_preview,
                created_at,
            } => {
                ws_server.do_send(super::server::BroadcastToUser {
                    org_id,
                    user_id,
                    message: ServerMessage::ReminderTriggered {
                        reminder_id,
                        message_id,
                        channel_id,
                        dm_id,
                        message_preview,
                        created_at,
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
        let _ = self.subscribe_to_channels(ctx);
    }
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
