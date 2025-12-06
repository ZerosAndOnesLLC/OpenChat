use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    cache::dms as dm_cache,
    cache::messages as message_cache,
    errors::{ApiError, ApiResult},
    models::attachment::Attachment,
    models::direct_message::{DirectMessage, DmParticipant},
    models::message::{Message, PaginatedMessages},
    models::reaction::Reaction,
    models::user::User,
    services::tv_api::TokenClaims,
};

#[derive(Debug, Deserialize)]
pub struct CreateDmRequest {
    pub participant_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub limit: Option<i64>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DmResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub created_by: Uuid,
    pub created_at: String,
    pub participants: Vec<Uuid>,
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

#[derive(Debug, Serialize)]
pub struct PaginatedMessagesResponse {
    pub messages: Vec<MessageResponse>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
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
    pub attachments: Vec<AttachmentResponse>,
}

impl From<crate::models::message::Message> for MessageResponse {
    fn from(message: crate::models::message::Message) -> Self {
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

    pub fn with_attachments(mut self, attachments: Vec<Attachment>) -> Self {
        self.attachments = attachments.into_iter().map(AttachmentResponse::from).collect();
        self
    }
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

/// GET /api/dms - List all DMs for the current user
pub async fn list_dms(
    pool: web::Data<PgPool>,
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

    // Get all DMs for this user
    let dms = DirectMessage::list_by_user(pool.get_ref(), current_user.id).await?;

    // Build response with participants
    let mut dm_responses = Vec::new();
    for dm in dms {
        let participants = DmParticipant::list_by_dm(pool.get_ref(), dm.id).await?;
        let participant_ids: Vec<Uuid> = participants.into_iter().map(|p| p.user_id).collect();

        dm_responses.push(DmResponse {
            id: dm.id,
            org_id: dm.org_id,
            created_by: dm.created_by,
            created_at: dm.created_at.to_rfc3339(),
            participants: participant_ids,
        });
    }

    Ok(HttpResponse::Ok().json(dm_responses))
}

/// POST /api/dms - Create a new DM
pub async fn create_dm(
    pool: web::Data<PgPool>,
    body: web::Json<CreateDmRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Validate participant list
    if body.participant_ids.is_empty() {
        return Err(ApiError::BadRequest(
            "At least one participant is required".to_string(),
        ));
    }

    // Get current user
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Build full participant list (including current user if not already included)
    let mut all_participants = body.participant_ids.clone();
    if !all_participants.contains(&current_user.id) {
        all_participants.push(current_user.id);
    }

    // Remove duplicates
    all_participants.sort();
    all_participants.dedup();

    // Validate that all participants exist and are in the same org
    for participant_id in &all_participants {
        let user = User::get_by_id(pool.get_ref(), *participant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("User {} not found", participant_id)))?;

        if user.org_id != claims.org_id {
            return Err(ApiError::Authorization(
                "Cannot create DM with users from different organizations".to_string(),
            ));
        }
    }

    // Check if a DM with these exact participants already exists
    if let Some(existing_dm) = DirectMessage::find_by_participants(pool.get_ref(), &all_participants).await? {
        // Unhide the DM for the current user (in case it was previously hidden)
        DmParticipant::unhide_dm(pool.get_ref(), existing_dm.id, current_user.id).await?;

        // Return the existing DM
        let participants = DmParticipant::list_by_dm(pool.get_ref(), existing_dm.id).await?;
        let participant_ids: Vec<Uuid> = participants.into_iter().map(|p| p.user_id).collect();

        return Ok(HttpResponse::Ok().json(DmResponse {
            id: existing_dm.id,
            org_id: existing_dm.org_id,
            created_by: existing_dm.created_by,
            created_at: existing_dm.created_at.to_rfc3339(),
            participants: participant_ids,
        }));
    }

    // Create new DM
    let dm = DirectMessage::create(
        pool.get_ref(),
        claims.org_id,
        current_user.id,
        &all_participants,
    )
    .await?;

    let participants = DmParticipant::list_by_dm(pool.get_ref(), dm.id).await?;
    let participant_ids: Vec<Uuid> = participants.into_iter().map(|p| p.user_id).collect();

    Ok(HttpResponse::Created().json(DmResponse {
        id: dm.id,
        org_id: dm.org_id,
        created_by: dm.created_by,
        created_at: dm.created_at.to_rfc3339(),
        participants: participant_ids,
    }))
}

/// GET /api/dms/:id - Get DM details
pub async fn get_dm(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    dm_id: web::Path<Uuid>,
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

    let mut redis_conn = redis.as_ref().clone();

    // Try to get DM from cache first
    let dm = match dm_cache::get_dm_from_cache(&mut redis_conn, *dm_id).await? {
        Some(cached_dm) => cached_dm,
        None => {
            // Cache miss - fetch from database
            let dm = DirectMessage::get_by_id(pool.get_ref(), *dm_id)
                .await?
                .ok_or_else(|| ApiError::NotFound("DM not found".to_string()))?;

            // Store in cache for next time
            if let Err(e) = dm_cache::set_dm_in_cache(&mut redis_conn, &dm).await {
                tracing::warn!("Failed to cache DM: {}", e);
            }

            dm
        }
    };

    // Verify user is a participant
    let is_participant = DirectMessage::is_participant(pool.get_ref(), *dm_id, current_user.id).await?;
    if !is_participant {
        return Err(ApiError::Authorization(
            "You are not a participant in this DM".to_string(),
        ));
    }

    // Get participants (could also be cached, but it's fast enough for now)
    let participants = DmParticipant::list_by_dm(pool.get_ref(), dm.id).await?;
    let participant_ids: Vec<Uuid> = participants.into_iter().map(|p| p.user_id).collect();

    Ok(HttpResponse::Ok().json(DmResponse {
        id: dm.id,
        org_id: dm.org_id,
        created_by: dm.created_by,
        created_at: dm.created_at.to_rfc3339(),
        participants: participant_ids,
    }))
}

/// GET /api/dms/:id/messages - List messages in a DM
pub async fn list_dm_messages(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    dm_id: web::Path<Uuid>,
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

    // Verify DM exists
    DirectMessage::get_by_id(pool.get_ref(), *dm_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("DM not found".to_string()))?;

    // Verify user is a participant
    let is_participant = DirectMessage::is_participant(pool.get_ref(), *dm_id, current_user.id).await?;
    if !is_participant {
        return Err(ApiError::Authorization(
            "You are not a participant in this DM".to_string(),
        ));
    }

    // Get messages with pagination
    let limit = query.limit.unwrap_or(50);

    // Try to get from cache if this is the first page
    let paginated = if query.cursor.is_none() {
        let mut redis_conn = redis.as_ref().clone();

        match message_cache::get_dm_messages_from_cache(&mut redis_conn, *dm_id).await? {
            Some(cached) => cached,
            None => {
                // Cache miss - fetch from database
                let paginated = Message::list_by_dm(
                    pool.get_ref(),
                    *dm_id,
                    limit,
                    query.cursor.clone(),
                )
                .await?;

                // Store in cache for next time
                if let Err(e) = message_cache::set_dm_messages_in_cache(&mut redis_conn, *dm_id, &paginated).await {
                    tracing::warn!("Failed to cache DM messages: {}", e);
                }

                paginated
            }
        }
    } else {
        // Not first page - don't use cache
        Message::list_by_dm(
            pool.get_ref(),
            *dm_id,
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

    // Enrich messages with user data, reactions, and attachments
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

/// POST /api/dms/:id/hide - Hide a DM from the user's list
pub async fn hide_dm(
    pool: web::Data<PgPool>,
    dm_id: web::Path<Uuid>,
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

    // Verify DM exists
    DirectMessage::get_by_id(pool.get_ref(), *dm_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("DM not found".to_string()))?;

    // Verify user is a participant
    let is_participant = DirectMessage::is_participant(pool.get_ref(), *dm_id, current_user.id).await?;
    if !is_participant {
        return Err(ApiError::Authorization(
            "You are not a participant in this DM".to_string(),
        ));
    }

    // Hide the DM for this user
    DmParticipant::hide_dm(pool.get_ref(), *dm_id, current_user.id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "DM hidden successfully"
    })))
}
