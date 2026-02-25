use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use crate::db::RedisPool;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    cache::messages as message_cache,
    errors::{ApiError, ApiResult},
    models::attachment::Attachment,
    models::channel::{Channel, ChannelMember},
    models::mention::{Mention, MentionType},
    models::message::{Message, PaginatedMessages},
    models::notification::{CreateNotification, Notification, NotificationType},
    models::reaction::Reaction,
    models::user::User,
    models::user_group::UserGroup,
    services::{audit_logger::AuditLogger, mention_parser, tv_api::TokenClaims},
    websocket::{
        messages::ServerMessage,
        server::{BroadcastMessage, WsServer},
    },
};

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub content: String,
    pub parent_message_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub limit: Option<i64>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub org_id: Uuid,
    pub tv_user_id: Uuid,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            org_id: user.org_id,
            tv_user_id: user.tv_user_id,
            status: user.status,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ReactionResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: String,
}

impl From<Reaction> for ReactionResponse {
    fn from(reaction: Reaction) -> Self {
        Self {
            id: reaction.id,
            message_id: reaction.message_id,
            user_id: reaction.user_id,
            emoji: reaction.emoji,
            created_at: reaction.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct AttachmentResponse {
    pub id: Uuid,
    pub file_name: String,
    pub file_url: String,
    pub file_type: Option<String>,
    pub file_size: Option<i64>,
    pub created_at: String,
}

impl From<Attachment> for AttachmentResponse {
    fn from(attachment: Attachment) -> Self {
        Self {
            id: attachment.id,
            file_name: attachment.file_name,
            file_url: attachment.file_url,
            file_type: attachment.file_type,
            file_size: attachment.file_size,
            created_at: attachment.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub user_id: Uuid,
    pub content: String,
    pub parent_message_id: Option<Uuid>,
    pub created_at: String,
    pub edited_at: Option<String>,
    pub user: Option<UserResponse>,
    pub reactions: Vec<ReactionResponse>,
    pub reply_count: i64,
    pub first_reply: Option<Box<MessageResponse>>,
    pub attachments: Vec<AttachmentResponse>,
}

impl From<Message> for MessageResponse {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            channel_id: message.channel_id,
            dm_id: message.dm_id,
            user_id: message.user_id,
            content: message.content,
            parent_message_id: message.parent_message_id,
            created_at: message.created_at.to_rfc3339(),
            edited_at: message.edited_at.map(|dt| dt.to_rfc3339()),
            user: None,
            reactions: vec![],
            reply_count: 0,
            first_reply: None,
            attachments: vec![],
        }
    }
}

impl MessageResponse {
    pub fn with_user(mut self, user: User) -> Self {
        self.user = Some(UserResponse::from(user));
        self
    }

    pub fn with_reactions(mut self, reactions: Vec<Reaction>) -> Self {
        self.reactions = reactions.into_iter().map(ReactionResponse::from).collect();
        self
    }

    pub fn with_reply_count(mut self, reply_count: i64) -> Self {
        self.reply_count = reply_count;
        self
    }

    pub fn with_first_reply(mut self, first_reply: MessageResponse) -> Self {
        self.first_reply = Some(Box::new(first_reply));
        self
    }

    pub fn with_attachments(mut self, attachments: Vec<Attachment>) -> Self {
        self.attachments = attachments.into_iter().map(AttachmentResponse::from).collect();
        self
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedMessagesResponse {
    pub messages: Vec<MessageResponse>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

impl From<PaginatedMessages> for PaginatedMessagesResponse {
    fn from(paginated: PaginatedMessages) -> Self {
        Self {
            messages: paginated.messages.into_iter().map(MessageResponse::from).collect(),
            has_more: paginated.has_more,
            next_cursor: paginated.next_cursor,
        }
    }
}

/// POST /api/messages - Send a new message
pub async fn send_message(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    body: web::Json<SendMessageRequest>,
    req: HttpRequest,
    ws_server: web::Data<actix::Addr<WsServer>>,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Validate that exactly one of channel_id or dm_id is provided
    match (&body.channel_id, &body.dm_id) {
        (Some(_), Some(_)) => {
            return Err(ApiError::BadRequest(
                "Cannot specify both channel_id and dm_id".to_string(),
            ))
        }
        (None, None) => {
            return Err(ApiError::BadRequest(
                "Must specify either channel_id or dm_id".to_string(),
            ))
        }
        _ => {}
    }

    // Validate content is not empty
    if body.content.trim().is_empty() {
        return Err(ApiError::BadRequest("Message content cannot be empty".to_string()));
    }

    // Get current user
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let message = if let Some(channel_id) = body.channel_id {
        // Verify channel exists
        Channel::get_by_id(pool.get_ref(), channel_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

        // Verify user is a member of the channel
        let is_member = ChannelMember::is_member(pool.get_ref(), channel_id, current_user.id).await?;
        if !is_member {
            return Err(ApiError::Authorization(
                "You are not a member of this channel".to_string(),
            ));
        }

        // Create channel message
        Message::create_channel_message(
            pool.get_ref(),
            channel_id,
            current_user.id,
            &body.content,
            body.parent_message_id,
        )
        .await?
    } else if let Some(dm_id) = body.dm_id {
        // For now, we'll implement DM verification in Phase 7
        // Just create the message if dm_id is provided
        Message::create_dm_message(
            pool.get_ref(),
            dm_id,
            current_user.id,
            &body.content,
            body.parent_message_id,
        )
        .await?
    } else {
        unreachable!() // Already validated above
    };

    // Parse mentions from message content
    let parsed_mentions = mention_parser::parse_mentions(&body.content, current_user.org_id, pool.get_ref()).await?;

    // Create mention records
    if !parsed_mentions.is_empty() {
        let create_mentions = mention_parser::to_create_mentions(parsed_mentions.clone(), message.id);
        Mention::create_batch(pool.get_ref(), create_mentions).await?;

        // Create notifications for mentions
        for parsed_mention in parsed_mentions {
            match parsed_mention.mention_type {
                MentionType::User => {
                    // Create notification for the mentioned user
                    if let Some(mentioned_user_id) = parsed_mention.mentioned_user_id {
                        // Don't notify if user mentioned themselves
                        if mentioned_user_id != current_user.id {
                            let notification = CreateNotification {
                                user_id: mentioned_user_id,
                                notification_type: NotificationType::Mention,
                                message_id: Some(message.id),
                                channel_id: message.channel_id,
                                dm_id: message.dm_id,
                            };
                            let created_notif = Notification::create(pool.get_ref(), notification).await?;

                            // Broadcast notification count update
                            if let Ok(notif_count) = Notification::count_unread_by_user(pool.get_ref(), mentioned_user_id).await {
                                ws_server.do_send(crate::websocket::server::BroadcastToUser {
                                    org_id: current_user.org_id,
                                    user_id: mentioned_user_id,
                                    message: ServerMessage::NotificationCountUpdated {
                                        unread_count: notif_count as i32,
                                    },
                                });
                            }

                            // Broadcast new notification event
                            ws_server.do_send(crate::websocket::server::BroadcastToUser {
                                org_id: current_user.org_id,
                                user_id: mentioned_user_id,
                                message: ServerMessage::NewNotification {
                                    notification_id: created_notif.id,
                                    notification_type: "mention".to_string(),
                                    message_id: Some(message.id),
                                    channel_id: message.channel_id,
                                    dm_id: message.dm_id,
                                    created_at: created_notif.created_at.to_rfc3339(),
                                },
                            });
                        }
                    }
                }
                MentionType::Channel | MentionType::Here | MentionType::Everyone => {
                    // Get all channel members and create notifications for them
                    if let Some(channel_id) = message.channel_id {
                        let member_ids = mention_parser::get_channel_members(pool.get_ref(), channel_id).await?;
                        for member_id in member_ids {
                            // Don't notify the sender
                            if member_id != current_user.id {
                                let notification = CreateNotification {
                                    user_id: member_id,
                                    notification_type: NotificationType::Mention,
                                    message_id: Some(message.id),
                                    channel_id: Some(channel_id),
                                    dm_id: None,
                                };
                                let created_notif = Notification::create(pool.get_ref(), notification).await?;

                                // Broadcast notification count and new notification
                                if let Ok(notif_count) = Notification::count_unread_by_user(pool.get_ref(), member_id).await {
                                    ws_server.do_send(crate::websocket::server::BroadcastToUser {
                                        org_id: current_user.org_id,
                                        user_id: member_id,
                                        message: ServerMessage::NotificationCountUpdated {
                                            unread_count: notif_count as i32,
                                        },
                                    });
                                }

                                ws_server.do_send(crate::websocket::server::BroadcastToUser {
                                    org_id: current_user.org_id,
                                    user_id: member_id,
                                    message: ServerMessage::NewNotification {
                                        notification_id: created_notif.id,
                                        notification_type: "mention".to_string(),
                                        message_id: Some(message.id),
                                        channel_id: Some(channel_id),
                                        dm_id: None,
                                        created_at: created_notif.created_at.to_rfc3339(),
                                    },
                                });
                            }
                        }
                    }
                }
                MentionType::Group => {
                    // Get group members and create notifications for them
                    if let Some(group_id) = parsed_mention.mentioned_group_id {
                        let member_ids = UserGroup::get_member_ids(pool.get_ref(), group_id).await?;
                        for member_id in member_ids {
                            // Don't notify the sender
                            if member_id != current_user.id {
                                let notification = CreateNotification {
                                    user_id: member_id,
                                    notification_type: NotificationType::Mention,
                                    message_id: Some(message.id),
                                    channel_id: message.channel_id,
                                    dm_id: message.dm_id,
                                };
                                let created_notif = Notification::create(pool.get_ref(), notification).await?;

                                // Broadcast notification count and new notification
                                if let Ok(notif_count) = Notification::count_unread_by_user(pool.get_ref(), member_id).await {
                                    ws_server.do_send(crate::websocket::server::BroadcastToUser {
                                        org_id: current_user.org_id,
                                        user_id: member_id,
                                        message: ServerMessage::NotificationCountUpdated {
                                            unread_count: notif_count as i32,
                                        },
                                    });
                                }

                                ws_server.do_send(crate::websocket::server::BroadcastToUser {
                                    org_id: current_user.org_id,
                                    user_id: member_id,
                                    message: ServerMessage::NewNotification {
                                        notification_id: created_notif.id,
                                        notification_type: "mention".to_string(),
                                        message_id: Some(message.id),
                                        channel_id: message.channel_id,
                                        dm_id: message.dm_id,
                                        created_at: created_notif.created_at.to_rfc3339(),
                                    },
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // If this is a thread reply, create a notification for the parent message author
    if let Some(parent_message_id) = body.parent_message_id {
        if let Ok(Some(parent_message)) = Message::get_by_id(pool.get_ref(), parent_message_id).await {
            // Don't notify if replying to your own message
            if parent_message.user_id != current_user.id {
                let notification = CreateNotification {
                    user_id: parent_message.user_id,
                    notification_type: NotificationType::ThreadReply,
                    message_id: Some(message.id),
                    channel_id: message.channel_id,
                    dm_id: message.dm_id,
                };
                let created_notif = Notification::create(pool.get_ref(), notification).await?;

                // Broadcast notification count and new notification
                if let Ok(notif_count) = Notification::count_unread_by_user(pool.get_ref(), parent_message.user_id).await {
                    ws_server.do_send(crate::websocket::server::BroadcastToUser {
                        org_id: current_user.org_id,
                        user_id: parent_message.user_id,
                        message: ServerMessage::NotificationCountUpdated {
                            unread_count: notif_count as i32,
                        },
                    });
                }

                ws_server.do_send(crate::websocket::server::BroadcastToUser {
                    org_id: current_user.org_id,
                    user_id: parent_message.user_id,
                    message: ServerMessage::NewNotification {
                        notification_id: created_notif.id,
                        notification_type: "thread_reply".to_string(),
                        message_id: Some(message.id),
                        channel_id: message.channel_id,
                        dm_id: message.dm_id,
                        created_at: created_notif.created_at.to_rfc3339(),
                    },
                });
            }
        }
    }

    // Invalidate message cache after sending a new message
    
    if let Some(channel_id) = body.channel_id {
        if let Err(e) = message_cache::invalidate_channel_messages_cache(redis_pool.get_ref(), current_user.org_id, channel_id).await {
            tracing::warn!("Failed to invalidate channel messages cache: {}", e);
        }
    } else if let Some(dm_id) = body.dm_id {
        if let Err(e) = message_cache::invalidate_dm_messages_cache(redis_pool.get_ref(), current_user.org_id, dm_id).await {
            tracing::warn!("Failed to invalidate DM messages cache: {}", e);
        }
    }

    // Broadcast message to WebSocket clients
    ws_server.do_send(BroadcastMessage {
        org_id: current_user.org_id,
        channel_id: message.channel_id,
        message: ServerMessage::NewMessage {
            id: message.id,
            channel_id: message.channel_id,
            dm_id: message.dm_id,
            user_id: message.user_id,
            user_name: current_user.display_name.clone(),
            user_avatar: current_user.avatar_url.clone(),
            content: message.content.clone(),
            parent_message_id: message.parent_message_id,
            created_at: message.created_at.to_rfc3339(),
            is_webhook: None,
        },
    });

    // Broadcast unread count updates to channel members (except sender)
    if let Some(channel_id) = message.channel_id {
        use crate::websocket::server::BroadcastToUser;
        use crate::models::read_status::ChannelReadStatus;

        // Get all channel members
        let members = ChannelMember::list_by_channel(pool.get_ref(), channel_id).await?;
        for member in members {
            // Skip the sender
            if member.user_id == current_user.id {
                continue;
            }

            // Get unread count and last read message ID for this user
            if let Ok(unread_count) = ChannelReadStatus::get_unread_count(pool.get_ref(), member.user_id, channel_id).await {
                let last_read_message_id = ChannelReadStatus::get_last_read_message_id(pool.get_ref(), member.user_id, channel_id).await.unwrap_or(None);
                ws_server.do_send(BroadcastToUser {
                    org_id: current_user.org_id,
                    user_id: member.user_id,
                    message: ServerMessage::UnreadCountUpdated {
                        channel_id: Some(channel_id),
                        dm_id: None,
                        unread_count,
                        last_read_message_id,
                    },
                });
            }
        }
    } else if let Some(dm_id) = message.dm_id {
        use crate::websocket::server::BroadcastToUser;
        use crate::models::{direct_message::DmParticipant, read_status::DmReadStatus};

        // Get DM participants
        if let Ok(participants) = DmParticipant::list_by_dm(pool.get_ref(), dm_id).await {
            for participant in participants {
                // Skip the sender
                if participant.user_id == current_user.id {
                    continue;
                }

                // Get unread count and last read message ID for this participant
                if let Ok(unread_count) = DmReadStatus::get_unread_count(pool.get_ref(), participant.user_id, dm_id).await {
                    let last_read_message_id = DmReadStatus::get_last_read_message_id(pool.get_ref(), participant.user_id, dm_id).await.unwrap_or(None);
                    ws_server.do_send(BroadcastToUser {
                        org_id: current_user.org_id,
                        user_id: participant.user_id,
                        message: ServerMessage::UnreadCountUpdated {
                            channel_id: None,
                            dm_id: Some(dm_id),
                            unread_count,
                            last_read_message_id,
                        },
                    });
                }
            }
        }
    }

    Ok(HttpResponse::Created().json(MessageResponse::from(message)))
}

/// GET /api/channels/:id/messages - List messages in a channel
pub async fn list_channel_messages(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    channel_id: web::Path<Uuid>,
    query: web::Query<ListMessagesQuery>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get current user
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Verify channel exists
    Channel::get_by_id(pool.get_ref(), *channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Verify user is a member of the channel
    let is_member = ChannelMember::is_member(pool.get_ref(), *channel_id, current_user.id).await?;
    if !is_member {
        return Err(ApiError::Authorization(
            "You are not a member of this channel".to_string(),
        ));
    }

    // Get messages with pagination
    let limit = query.limit.unwrap_or(50);

    // Try to get from cache if this is the first page
    let paginated = if query.cursor.is_none() {
        

        match message_cache::get_channel_messages_from_cache(redis_pool.get_ref(), current_user.org_id, *channel_id).await? {
            Some(cached) => cached,
            None => {
                // Cache miss - fetch from database
                let paginated = Message::list_by_channel(
                    pool.get_ref(),
                    *channel_id,
                    limit,
                    query.cursor.clone(),
                )
                .await?;

                // Store in cache for next time
                if let Err(e) = message_cache::set_channel_messages_in_cache(redis_pool.get_ref(), current_user.org_id, *channel_id, &paginated).await {
                    tracing::warn!("Failed to cache channel messages: {}", e);
                }

                paginated
            }
        }
    } else {
        // Not first page - don't use cache
        Message::list_by_channel(
            pool.get_ref(),
            *channel_id,
            limit,
            query.cursor.clone(),
        )
        .await?
    };

    // Fetch users for all messages
    let user_ids: Vec<Uuid> = paginated.messages.iter().map(|m| m.user_id).collect();
    let users = if !user_ids.is_empty() {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = ANY($1)"
        )
        .bind(&user_ids)
        .fetch_all(pool.get_ref())
        .await?
    } else {
        vec![]
    };

    // Create a map of user_id -> User for quick lookup
    let user_map: std::collections::HashMap<Uuid, User> = users
        .into_iter()
        .map(|u| (u.id, u))
        .collect();

    // Fetch reactions for all messages
    let message_ids: Vec<Uuid> = paginated.messages.iter().map(|m| m.id).collect();
    let reactions = if !message_ids.is_empty() {
        sqlx::query_as::<_, Reaction>(
            "SELECT * FROM reactions WHERE message_id = ANY($1) ORDER BY created_at"
        )
        .bind(&message_ids)
        .fetch_all(pool.get_ref())
        .await?
    } else {
        vec![]
    };

    // Create a map of message_id -> Vec<Reaction> for quick lookup
    let mut reaction_map: std::collections::HashMap<Uuid, Vec<Reaction>> = std::collections::HashMap::new();
    for reaction in reactions {
        reaction_map
            .entry(reaction.message_id)
            .or_insert_with(Vec::new)
            .push(reaction);
    }

    // Fetch reply counts for all messages
    let reply_counts = Message::count_replies_batch(pool.get_ref(), &message_ids).await?;

    // Fetch first replies for messages with reply_count > 0
    let first_replies = Message::get_first_replies_batch(pool.get_ref(), &message_ids).await?;

    // Fetch users for first replies
    let first_reply_user_ids: Vec<Uuid> = first_replies.values().map(|m| m.user_id).collect();
    let first_reply_users = if !first_reply_user_ids.is_empty() {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = ANY($1)"
        )
        .bind(&first_reply_user_ids)
        .fetch_all(pool.get_ref())
        .await?
    } else {
        vec![]
    };

    // Create a map of user_id -> User for first reply users
    let first_reply_user_map: std::collections::HashMap<Uuid, User> = first_reply_users
        .into_iter()
        .map(|u| (u.id, u))
        .collect();

    // Fetch attachments for all messages
    let attachments = if !message_ids.is_empty() {
        sqlx::query_as::<_, Attachment>(
            "SELECT * FROM attachments WHERE message_id = ANY($1) ORDER BY created_at"
        )
        .bind(&message_ids)
        .fetch_all(pool.get_ref())
        .await?
    } else {
        vec![]
    };

    // Create a map of message_id -> Vec<Attachment> for quick lookup
    let mut attachment_map: std::collections::HashMap<Uuid, Vec<Attachment>> = std::collections::HashMap::new();
    for attachment in attachments {
        attachment_map
            .entry(attachment.message_id)
            .or_insert_with(Vec::new)
            .push(attachment);
    }

    // Enrich messages with user data, reactions, reply counts, first replies, and attachments
    let messages_with_users: Vec<MessageResponse> = paginated.messages
        .into_iter()
        .map(|msg| {
            let mut response = MessageResponse::from(msg.clone());
            if let Some(user) = user_map.get(&msg.user_id) {
                response = response.with_user(user.clone());
            }
            if let Some(reactions) = reaction_map.get(&msg.id) {
                response = response.with_reactions(reactions.clone());
            }
            if let Some(&reply_count) = reply_counts.get(&msg.id) {
                response = response.with_reply_count(reply_count);
                // If there's a reply count, include the first reply
                if reply_count > 0 {
                    if let Some(first_reply) = first_replies.get(&msg.id) {
                        let mut first_reply_response = MessageResponse::from(first_reply.clone());
                        if let Some(user) = first_reply_user_map.get(&first_reply.user_id) {
                            first_reply_response = first_reply_response.with_user(user.clone());
                        }
                        response = response.with_first_reply(first_reply_response);
                    }
                }
            }
            if let Some(attachments) = attachment_map.get(&msg.id) {
                response = response.with_attachments(attachments.clone());
            }
            response
        })
        .collect();

    let response = PaginatedMessagesResponse {
        messages: messages_with_users,
        has_more: paginated.has_more,
        next_cursor: paginated.next_cursor,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// PUT /api/messages/:id - Edit a message
pub async fn update_message(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    message_id: web::Path<Uuid>,
    body: web::Json<UpdateMessageRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Validate content is not empty
    if body.content.trim().is_empty() {
        return Err(ApiError::BadRequest("Message content cannot be empty".to_string()));
    }

    // Get current user
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Verify message exists and belongs to current user
    let message = Message::get_by_id(pool.get_ref(), *message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    if message.user_id != current_user.id {
        return Err(ApiError::Authorization(
            "You can only edit your own messages".to_string(),
        ));
    }

    // Update the message
    let updated_message = Message::update(pool.get_ref(), *message_id, &body.content, current_user.id).await?;

    // Invalidate message cache after updating

    if let Some(channel_id) = message.channel_id {
        if let Err(e) = message_cache::invalidate_channel_messages_cache(redis_pool.get_ref(), current_user.org_id, channel_id).await {
            tracing::warn!("Failed to invalidate channel messages cache: {}", e);
        }
    } else if let Some(dm_id) = message.dm_id {
        if let Err(e) = message_cache::invalidate_dm_messages_cache(redis_pool.get_ref(), current_user.org_id, dm_id).await {
            tracing::warn!("Failed to invalidate DM messages cache: {}", e);
        }
    }

    Ok(HttpResponse::Ok().json(MessageResponse::from(updated_message)))
}

/// DELETE /api/messages/:id - Soft delete a message
pub async fn delete_message(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    message_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get current user
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Verify message exists and belongs to current user
    let message = Message::get_by_id(pool.get_ref(), *message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    if message.user_id != current_user.id {
        return Err(ApiError::Authorization(
            "You can only delete your own messages".to_string(),
        ));
    }

    // Store message content for audit log before deletion
    let message_content = message.content.clone();

    // Soft delete the message
    Message::soft_delete(pool.get_ref(), *message_id).await?;

    // Log the deletion in audit log
    if let Err(e) = AuditLogger::log_message_deleted(
        pool.get_ref(),
        current_user.id,
        *message_id,
        &message_content,
        Some(&req),
    )
    .await
    {
        tracing::warn!("Failed to create audit log for message deletion: {}", e);
    }

    // Invalidate message cache after deleting

    if let Some(channel_id) = message.channel_id {
        if let Err(e) = message_cache::invalidate_channel_messages_cache(redis_pool.get_ref(), current_user.org_id, channel_id).await {
            tracing::warn!("Failed to invalidate channel messages cache: {}", e);
        }
    } else if let Some(dm_id) = message.dm_id {
        if let Err(e) = message_cache::invalidate_dm_messages_cache(redis_pool.get_ref(), current_user.org_id, dm_id).await {
            tracing::warn!("Failed to invalidate DM messages cache: {}", e);
        }
    }

    Ok(HttpResponse::NoContent().finish())
}

/// GET /api/messages/:id/thread - Get thread messages (replies to a message)
pub async fn get_message_thread(
    pool: web::Data<PgPool>,
    message_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get current user
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Verify parent message exists
    let parent_message = Message::get_by_id(pool.get_ref(), *message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    // Verify user has access to the message (is member of channel or DM)
    if let Some(channel_id) = parent_message.channel_id {
        let is_member = ChannelMember::is_member(pool.get_ref(), channel_id, current_user.id).await?;
        if !is_member {
            return Err(ApiError::Authorization(
                "You are not a member of this channel".to_string(),
            ));
        }
    }
    // TODO: Add DM permission check in Phase 7

    // Get thread messages (replies)
    let thread_messages = Message::list_thread_messages(pool.get_ref(), *message_id).await?;

    // Fetch users for all messages (including parent)
    let mut all_messages = vec![parent_message.clone()];
    all_messages.extend(thread_messages.clone());

    let user_ids: Vec<Uuid> = all_messages.iter().map(|m| m.user_id).collect();
    let users = if !user_ids.is_empty() {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = ANY($1)"
        )
        .bind(&user_ids)
        .fetch_all(pool.get_ref())
        .await?
    } else {
        vec![]
    };

    // Create a map of user_id -> User for quick lookup
    let user_map: std::collections::HashMap<Uuid, User> = users
        .into_iter()
        .map(|u| (u.id, u))
        .collect();

    // Fetch reactions for all messages
    let message_ids: Vec<Uuid> = all_messages.iter().map(|m| m.id).collect();
    let reactions = if !message_ids.is_empty() {
        sqlx::query_as::<_, Reaction>(
            "SELECT * FROM reactions WHERE message_id = ANY($1) ORDER BY created_at"
        )
        .bind(&message_ids)
        .fetch_all(pool.get_ref())
        .await?
    } else {
        vec![]
    };

    // Create a map of message_id -> Vec<Reaction> for quick lookup
    let mut reaction_map: std::collections::HashMap<Uuid, Vec<Reaction>> = std::collections::HashMap::new();
    for reaction in reactions {
        reaction_map
            .entry(reaction.message_id)
            .or_insert_with(Vec::new)
            .push(reaction);
    }

    // Fetch reply counts for all messages (including nested replies)
    let reply_counts = Message::count_replies_batch(pool.get_ref(), &message_ids).await?;

    // Build parent message response
    let mut parent_response = MessageResponse::from(parent_message.clone());
    if let Some(user) = user_map.get(&parent_message.user_id) {
        parent_response = parent_response.with_user(user.clone());
    }
    if let Some(reactions) = reaction_map.get(&parent_message.id) {
        parent_response = parent_response.with_reactions(reactions.clone());
    }
    if let Some(&reply_count) = reply_counts.get(&parent_message.id) {
        parent_response = parent_response.with_reply_count(reply_count);
    }

    // Build thread message responses
    let thread_responses: Vec<MessageResponse> = thread_messages
        .into_iter()
        .map(|msg| {
            let mut response = MessageResponse::from(msg.clone());
            if let Some(user) = user_map.get(&msg.user_id) {
                response = response.with_user(user.clone());
            }
            if let Some(reactions) = reaction_map.get(&msg.id) {
                response = response.with_reactions(reactions.clone());
            }
            if let Some(&reply_count) = reply_counts.get(&msg.id) {
                response = response.with_reply_count(reply_count);
            }
            response
        })
        .collect();

    #[derive(Debug, Serialize)]
    struct ThreadResponse {
        parent: MessageResponse,
        replies: Vec<MessageResponse>,
    }

    let response = ThreadResponse {
        parent: parent_response,
        replies: thread_responses,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/messages/:id/history - Get edit history for a message
pub async fn get_message_history(
    pool: web::Data<PgPool>,
    message_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get current user
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Verify message exists
    let message = Message::get_by_id(pool.get_ref(), *message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    // Verify user has access to the message (is member of channel or DM)
    if let Some(channel_id) = message.channel_id {
        let is_member = ChannelMember::is_member(pool.get_ref(), channel_id, current_user.id).await?;
        if !is_member {
            return Err(ApiError::Authorization(
                "You are not a member of this channel".to_string(),
            ));
        }
    }
    // TODO: Add DM permission check when DM functionality is fully implemented

    // Get edit history
    let edits = crate::models::message_edit::MessageEdit::list_by_message(pool.get_ref(), *message_id).await?;

    Ok(HttpResponse::Ok().json(edits))
}
