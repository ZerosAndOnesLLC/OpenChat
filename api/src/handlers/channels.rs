use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    cache::channels as channel_cache,
    errors::{ApiError, ApiResult},
    models::channel::{Channel, ChannelMember},
    models::user::User,
    services::{audit_logger::AuditLogger, tv_api::TokenClaims},
    websocket::{
        messages::ServerMessage,
        server::{BroadcastMessage, WsServer},
    },
};

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub description: Option<String>,
    pub channel_type: String, // "public" or "private"
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    pub role: Option<String>, // "admin" or "member"
}

#[derive(Debug, Serialize)]
pub struct ChannelResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub channel_type: String,
    pub created_by: Uuid,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Channel> for ChannelResponse {
    fn from(channel: Channel) -> Self {
        Self {
            id: channel.id,
            org_id: channel.org_id,
            name: channel.name,
            description: channel.description,
            channel_type: channel.channel_type,
            created_by: channel.created_by,
            created_at: channel.created_at.to_rfc3339(),
            updated_at: channel.updated_at.to_rfc3339(),
        }
    }
}

/// GET /api/channels - List channels where the user is a member
pub async fn list_channels(
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

    // Only return channels where the user is a member
    let channels = Channel::list_by_user_membership(pool.get_ref(), claims.org_id, current_user.id).await?;
    let response: Vec<ChannelResponse> = channels.into_iter().map(ChannelResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/channels - Create a new channel
pub async fn create_channel(
    pool: web::Data<PgPool>,
    body: web::Json<CreateChannelRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Validate channel type
    if !["public", "private"].contains(&body.channel_type.as_str()) {
        return Err(ApiError::BadRequest(
            "Channel type must be 'public' or 'private'".to_string(),
        ));
    }

    // Get current user
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Create channel
    let channel = Channel::create(
        pool.get_ref(),
        claims.org_id,
        &body.name,
        body.description.as_deref(),
        &body.channel_type,
        current_user.id,
    )
    .await?;

    // Add creator as admin member
    ChannelMember::add(pool.get_ref(), channel.id, current_user.id, "admin").await?;

    // Log channel creation in audit log
    if let Err(e) = AuditLogger::log_channel_created(
        pool.get_ref(),
        current_user.id,
        channel.id,
        &channel.name,
        Some(&req),
    )
    .await
    {
        tracing::warn!("Failed to create audit log for channel creation: {}", e);
    }

    Ok(HttpResponse::Created().json(ChannelResponse::from(channel)))
}

/// GET /api/channels/:id - Get channel details
pub async fn get_channel(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    channel_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let _claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let mut redis_conn = redis.as_ref().clone();

    // Try to get from cache first
    if let Some(channel) = channel_cache::get_channel_from_cache(&mut redis_conn, *channel_id).await? {
        return Ok(HttpResponse::Ok().json(ChannelResponse::from(channel)));
    }

    // Cache miss - fetch from database
    let channel = Channel::get_by_id(pool.get_ref(), *channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Store in cache for next time
    if let Err(e) = channel_cache::set_channel_in_cache(&mut redis_conn, &channel).await {
        tracing::warn!("Failed to cache channel: {}", e);
    }

    Ok(HttpResponse::Ok().json(ChannelResponse::from(channel)))
}

/// PUT /api/channels/:id - Update channel
pub async fn update_channel(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    channel_id: web::Path<Uuid>,
    body: web::Json<UpdateChannelRequest>,
    req: HttpRequest,
    ws_server: web::Data<actix::Addr<WsServer>>,
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

    // Get channel
    let channel = Channel::get_by_id(pool.get_ref(), *channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Only creator or admins can update (for now, just check creator)
    if channel.created_by != current_user.id {
        return Err(ApiError::Authorization(
            "Only channel creator can update the channel".to_string(),
        ));
    }

    let updated_channel = Channel::update(
        pool.get_ref(),
        *channel_id,
        body.name.as_deref(),
        body.description.as_deref(),
    )
    .await?;

    // Invalidate cache after update
    let mut redis_conn = redis.as_ref().clone();
    if let Err(e) = channel_cache::invalidate_channel_cache(&mut redis_conn, *channel_id).await {
        tracing::warn!("Failed to invalidate channel cache: {}", e);
    }

    // Broadcast channel update via WebSocket
    ws_server.do_send(BroadcastMessage {
        org_id: channel.org_id,
        channel_id: Some(*channel_id),
        message: ServerMessage::ChannelUpdated {
            channel_id: *channel_id,
            name: body.name.clone(),
            description: body.description.clone(),
            updated_by: current_user.id,
            updated_by_name: current_user.display_name.clone(),
        },
    });

    Ok(HttpResponse::Ok().json(ChannelResponse::from(updated_channel)))
}

/// DELETE /api/channels/:id - Delete channel
pub async fn delete_channel(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    channel_id: web::Path<Uuid>,
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

    // Get channel
    let channel = Channel::get_by_id(pool.get_ref(), *channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Only creator can delete
    if channel.created_by != current_user.id {
        return Err(ApiError::Authorization(
            "Only channel creator can delete the channel".to_string(),
        ));
    }

    // Store channel name for audit log before deletion
    let channel_name = channel.name.clone();

    Channel::delete(pool.get_ref(), *channel_id).await?;

    // Log channel deletion in audit log
    if let Err(e) = AuditLogger::log_channel_deleted(
        pool.get_ref(),
        current_user.id,
        *channel_id,
        &channel_name,
        Some(&req),
    )
    .await
    {
        tracing::warn!("Failed to create audit log for channel deletion: {}", e);
    }

    // Invalidate cache after deletion
    let mut redis_conn = redis.as_ref().clone();
    if let Err(e) = channel_cache::invalidate_channel_cache(&mut redis_conn, *channel_id).await {
        tracing::warn!("Failed to invalidate channel cache: {}", e);
    }
    if let Err(e) = channel_cache::invalidate_channel_members_cache(&mut redis_conn, *channel_id).await {
        tracing::warn!("Failed to invalidate channel members cache: {}", e);
    }

    Ok(HttpResponse::NoContent().finish())
}

/// GET /api/channels/:id/members - List channel members
pub async fn list_members(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    channel_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let _claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let mut redis_conn = redis.as_ref().clone();

    // Try to get from cache first
    if let Some(members) = channel_cache::get_channel_members_from_cache(&mut redis_conn, *channel_id).await? {
        return Ok(HttpResponse::Ok().json(members));
    }

    // Verify channel exists
    Channel::get_by_id(pool.get_ref(), *channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Cache miss - fetch from database
    let members = ChannelMember::list_by_channel(pool.get_ref(), *channel_id).await?;

    // Store in cache for next time
    if let Err(e) = channel_cache::set_channel_members_in_cache(&mut redis_conn, *channel_id, &members).await {
        tracing::warn!("Failed to cache channel members: {}", e);
    }

    Ok(HttpResponse::Ok().json(members))
}

/// POST /api/channels/:id/members - Add member to channel
pub async fn add_member(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    channel_id: web::Path<Uuid>,
    body: web::Json<AddMemberRequest>,
    req: HttpRequest,
    ws_server: web::Data<actix::Addr<WsServer>>,
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

    // Get channel
    let channel = Channel::get_by_id(pool.get_ref(), *channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Only creator or admins can add members (for now, just check creator)
    if channel.created_by != current_user.id {
        return Err(ApiError::Authorization(
            "Only channel creator can add members".to_string(),
        ));
    }

    // Verify user to add exists
    let user_to_add = User::get_by_id(pool.get_ref(), body.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User to add not found".to_string()))?;

    let role = body.role.as_deref().unwrap_or("member");
    let member = ChannelMember::add(pool.get_ref(), *channel_id, body.user_id, role).await?;

    // Log member addition in audit log
    if let Err(e) = AuditLogger::log_channel_member_added(
        pool.get_ref(),
        current_user.id,
        *channel_id,
        body.user_id,
        Some(&req),
    )
    .await
    {
        tracing::warn!("Failed to create audit log for member addition: {}", e);
    }

    // Invalidate members cache after adding a new member
    let mut redis_conn = redis.as_ref().clone();
    if let Err(e) = channel_cache::invalidate_channel_members_cache(&mut redis_conn, *channel_id).await {
        tracing::warn!("Failed to invalidate channel members cache: {}", e);
    }

    // Broadcast member joined event via WebSocket
    ws_server.do_send(BroadcastMessage {
        org_id: channel.org_id,
        channel_id: Some(*channel_id),
        message: ServerMessage::MemberJoined {
            channel_id: *channel_id,
            user_id: user_to_add.id,
            user_name: user_to_add.display_name.clone(),
            role: role.to_string(),
            joined_at: member.joined_at.to_rfc3339(),
        },
    });

    Ok(HttpResponse::Created().json(member))
}

/// DELETE /api/channels/:id/members/:user_id - Remove member from channel
pub async fn remove_member(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    path: web::Path<(Uuid, Uuid)>,
    req: HttpRequest,
    ws_server: web::Data<actix::Addr<WsServer>>,
) -> ApiResult<HttpResponse> {
    let (channel_id, user_id) = path.into_inner();

    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Get current user
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Get channel
    let channel = Channel::get_by_id(pool.get_ref(), channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Only creator or admins can remove members (for now, just check creator)
    if channel.created_by != current_user.id {
        return Err(ApiError::Authorization(
            "Only channel creator can remove members".to_string(),
        ));
    }

    // Get user being removed for name
    let user_to_remove = User::get_by_id(pool.get_ref(), user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User to remove not found".to_string()))?;

    ChannelMember::remove(pool.get_ref(), channel_id, user_id).await?;

    // Log member removal in audit log
    if let Err(e) = AuditLogger::log_channel_member_removed(
        pool.get_ref(),
        current_user.id,
        channel_id,
        user_id,
        Some(&req),
    )
    .await
    {
        tracing::warn!("Failed to create audit log for member removal: {}", e);
    }

    // Invalidate members cache after removing a member
    let mut redis_conn = redis.as_ref().clone();
    if let Err(e) = channel_cache::invalidate_channel_members_cache(&mut redis_conn, channel_id).await {
        tracing::warn!("Failed to invalidate channel members cache: {}", e);
    }

    // Broadcast member left event via WebSocket
    ws_server.do_send(BroadcastMessage {
        org_id: channel.org_id,
        channel_id: Some(channel_id),
        message: ServerMessage::MemberLeft {
            channel_id,
            user_id,
            user_name: user_to_remove.display_name.clone(),
        },
    });

    Ok(HttpResponse::NoContent().finish())
}

/// GET /api/channels/public - List public channels available to join (excludes already-joined channels)
pub async fn list_public_channels(
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

    // Only return public channels the user is NOT already a member of
    let channels = Channel::list_public_channels(pool.get_ref(), claims.org_id, current_user.id).await?;
    let response: Vec<ChannelResponse> = channels.into_iter().map(ChannelResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/channels/:id/join - Join a public channel
pub async fn join_channel(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    channel_id: web::Path<Uuid>,
    req: HttpRequest,
    ws_server: web::Data<actix::Addr<WsServer>>,
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

    // Get channel
    let channel = Channel::get_by_id(pool.get_ref(), *channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Verify channel is in the same org
    if channel.org_id != claims.org_id {
        return Err(ApiError::Authorization(
            "Channel does not belong to your organization".to_string(),
        ));
    }

    // Only public channels can be joined
    if channel.channel_type != "public" {
        return Err(ApiError::BadRequest(
            "Only public channels can be joined. Private channels require an invitation.".to_string(),
        ));
    }

    // Check if already a member
    let is_member = ChannelMember::is_member(pool.get_ref(), *channel_id, current_user.id).await?;
    if is_member {
        return Err(ApiError::BadRequest(
            "You are already a member of this channel".to_string(),
        ));
    }

    // Add user as a member
    let member = ChannelMember::add(pool.get_ref(), *channel_id, current_user.id, "member").await?;

    // Log channel join in audit log
    if let Err(e) = AuditLogger::log_channel_member_added(
        pool.get_ref(),
        current_user.id,
        *channel_id,
        current_user.id,
        Some(&req),
    )
    .await
    {
        tracing::warn!("Failed to create audit log for channel join: {}", e);
    }

    // Invalidate members cache after adding a new member
    let mut redis_conn = redis.as_ref().clone();
    if let Err(e) = channel_cache::invalidate_channel_members_cache(&mut redis_conn, *channel_id).await {
        tracing::warn!("Failed to invalidate channel members cache: {}", e);
    }

    // Broadcast member joined event via WebSocket
    ws_server.do_send(BroadcastMessage {
        org_id: channel.org_id,
        channel_id: Some(*channel_id),
        message: ServerMessage::MemberJoined {
            channel_id: *channel_id,
            user_id: current_user.id,
            user_name: current_user.display_name.clone(),
            role: "member".to_string(),
            joined_at: member.joined_at.to_rfc3339(),
        },
    });

    Ok(HttpResponse::Created().json(member))
}
