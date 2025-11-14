use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::channel::{Channel, ChannelMember},
    models::user::User,
    services::tv_api::TokenClaims,
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

/// GET /api/channels - List all channels in the organization
pub async fn list_channels(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let channels = Channel::list_by_org(pool.get_ref(), claims.org_id).await?;
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

    Ok(HttpResponse::Created().json(ChannelResponse::from(channel)))
}

/// GET /api/channels/:id - Get channel details
pub async fn get_channel(
    pool: web::Data<PgPool>,
    channel_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let _claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let channel = Channel::get_by_id(pool.get_ref(), *channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ChannelResponse::from(channel)))
}

/// PUT /api/channels/:id - Update channel
pub async fn update_channel(
    pool: web::Data<PgPool>,
    channel_id: web::Path<Uuid>,
    body: web::Json<UpdateChannelRequest>,
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

    Ok(HttpResponse::Ok().json(ChannelResponse::from(updated_channel)))
}

/// DELETE /api/channels/:id - Delete channel
pub async fn delete_channel(
    pool: web::Data<PgPool>,
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

    Channel::delete(pool.get_ref(), *channel_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// GET /api/channels/:id/members - List channel members
pub async fn list_members(
    pool: web::Data<PgPool>,
    channel_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let _claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    // Verify channel exists
    Channel::get_by_id(pool.get_ref(), *channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    let members = ChannelMember::list_by_channel(pool.get_ref(), *channel_id).await?;

    Ok(HttpResponse::Ok().json(members))
}

/// POST /api/channels/:id/members - Add member to channel
pub async fn add_member(
    pool: web::Data<PgPool>,
    channel_id: web::Path<Uuid>,
    body: web::Json<AddMemberRequest>,
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

    // Only creator or admins can add members (for now, just check creator)
    if channel.created_by != current_user.id {
        return Err(ApiError::Authorization(
            "Only channel creator can add members".to_string(),
        ));
    }

    // Verify user to add exists
    User::get_by_id(pool.get_ref(), body.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User to add not found".to_string()))?;

    let role = body.role.as_deref().unwrap_or("member");
    let member = ChannelMember::add(pool.get_ref(), *channel_id, body.user_id, role).await?;

    Ok(HttpResponse::Created().json(member))
}

/// DELETE /api/channels/:id/members/:user_id - Remove member from channel
pub async fn remove_member(
    pool: web::Data<PgPool>,
    path: web::Path<(Uuid, Uuid)>,
    req: HttpRequest,
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

    ChannelMember::remove(pool.get_ref(), channel_id, user_id).await?;

    Ok(HttpResponse::NoContent().finish())
}
