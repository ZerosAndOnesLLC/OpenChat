use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::channel::{Channel, ChannelMember},
    models::message::{Message, PaginatedMessages},
    models::reaction::Reaction,
    models::user::User,
    services::tv_api::TokenClaims,
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
    body: web::Json<SendMessageRequest>,
    req: HttpRequest,
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

    Ok(HttpResponse::Created().json(MessageResponse::from(message)))
}

/// GET /api/channels/:id/messages - List messages in a channel
pub async fn list_channel_messages(
    pool: web::Data<PgPool>,
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
    let paginated = Message::list_by_channel(
        pool.get_ref(),
        *channel_id,
        limit,
        query.cursor.clone(),
    )
    .await?;

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

    // Enrich messages with user data, reactions, reply counts, and first replies
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
    let updated_message = Message::update(pool.get_ref(), *message_id, &body.content).await?;

    Ok(HttpResponse::Ok().json(MessageResponse::from(updated_message)))
}

/// DELETE /api/messages/:id - Soft delete a message
pub async fn delete_message(
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

    // Verify message exists and belongs to current user
    let message = Message::get_by_id(pool.get_ref(), *message_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    if message.user_id != current_user.id {
        return Err(ApiError::Authorization(
            "You can only delete your own messages".to_string(),
        ));
    }

    // Soft delete the message
    Message::soft_delete(pool.get_ref(), *message_id).await?;

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
