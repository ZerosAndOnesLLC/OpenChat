use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    cache::channel_sections as section_cache,
    db::RedisPool,
    errors::{ApiError, ApiResult},
    models::channel_section::{
        ChannelSection, ChannelSectionItem, CreateChannelSection, ReorderSection,
        ReorderSectionItem, UpdateChannelSection,
    },
    models::user::User,
    services::tv_api::TokenClaims,
};

#[derive(Debug, Serialize)]
pub struct SectionWithItems {
    pub id: Uuid,
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub position: i32,
    pub collapsed: bool,
    pub created_at: String,
    pub channel_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AddChannelRequest {
    pub channel_id: Uuid,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderSectionsRequest {
    pub order: Vec<ReorderSection>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderItemsRequest {
    pub order: Vec<ReorderSectionItem>,
}

async fn build_section_with_items(
    pool: &PgPool,
    section: ChannelSection,
) -> ApiResult<SectionWithItems> {
    let items = ChannelSectionItem::list_by_section(pool, section.id).await?;
    let channel_ids = items.into_iter().map(|i| i.channel_id).collect();

    Ok(SectionWithItems {
        id: section.id,
        user_id: section.user_id,
        org_id: section.org_id,
        name: section.name,
        position: section.position,
        collapsed: section.collapsed,
        created_at: section.created_at.to_rfc3339(),
        channel_ids,
    })
}

/// GET /api/channel-sections - List user's channel sections
pub async fn list_sections(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    // Check cache first
    let sections = match section_cache::get_sections_from_cache(
        redis_pool.get_ref(),
        current_user.org_id,
        current_user.id,
    )
    .await?
    {
        Some(cached) => cached,
        None => {
            let mut sections =
                ChannelSection::list_by_user(pool.get_ref(), current_user.id, current_user.org_id)
                    .await?;

            // Lazy-init default sections on first call
            if sections.is_empty() {
                let starred = ChannelSection::create(
                    pool.get_ref(),
                    current_user.id,
                    current_user.org_id,
                    CreateChannelSection {
                        name: "Starred".to_string(),
                    },
                )
                .await?;
                let channels = ChannelSection::create(
                    pool.get_ref(),
                    current_user.id,
                    current_user.org_id,
                    CreateChannelSection {
                        name: "Channels".to_string(),
                    },
                )
                .await?;
                sections = vec![starred, channels];
            }

            // Cache the result
            if let Err(e) = section_cache::set_sections_in_cache(
                redis_pool.get_ref(),
                current_user.org_id,
                current_user.id,
                &sections,
            )
            .await
            {
                tracing::warn!("Failed to cache channel sections: {}", e);
            }

            sections
        }
    };

    let mut response = Vec::new();
    for section in sections {
        response.push(build_section_with_items(pool.get_ref(), section).await?);
    }

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/channel-sections - Create a new section
pub async fn create_section(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    body: web::Json<CreateChannelSection>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "Section name cannot be empty".to_string(),
        ));
    }

    let section = ChannelSection::create(
        pool.get_ref(),
        current_user.id,
        current_user.org_id,
        body.into_inner(),
    )
    .await?;

    // Invalidate cache
    if let Err(e) = section_cache::invalidate_sections_cache(
        redis_pool.get_ref(),
        current_user.org_id,
        current_user.id,
    )
    .await
    {
        tracing::warn!("Failed to invalidate channel sections cache: {}", e);
    }

    let response = build_section_with_items(pool.get_ref(), section).await?;
    Ok(HttpResponse::Created().json(response))
}

/// PUT /api/channel-sections/{id} - Update a section
pub async fn update_section(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    section_id: web::Path<Uuid>,
    body: web::Json<UpdateChannelSection>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let section =
        ChannelSection::update(pool.get_ref(), *section_id, current_user.id, body.into_inner())
            .await?
            .ok_or_else(|| ApiError::NotFound("Section not found".to_string()))?;

    // Invalidate cache
    if let Err(e) = section_cache::invalidate_sections_cache(
        redis_pool.get_ref(),
        current_user.org_id,
        current_user.id,
    )
    .await
    {
        tracing::warn!("Failed to invalidate channel sections cache: {}", e);
    }

    let response = build_section_with_items(pool.get_ref(), section).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// DELETE /api/channel-sections/{id} - Delete a section
pub async fn delete_section(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    section_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let deleted = ChannelSection::delete(pool.get_ref(), *section_id, current_user.id).await?;
    if !deleted {
        return Err(ApiError::NotFound("Section not found".to_string()));
    }

    // Invalidate cache
    if let Err(e) = section_cache::invalidate_sections_cache(
        redis_pool.get_ref(),
        current_user.org_id,
        current_user.id,
    )
    .await
    {
        tracing::warn!("Failed to invalidate channel sections cache: {}", e);
    }

    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/channel-sections/{id}/channels - Add a channel to a section
pub async fn add_channel(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    section_id: web::Path<Uuid>,
    body: web::Json<AddChannelRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let item =
        ChannelSectionItem::add(pool.get_ref(), *section_id, body.channel_id, body.position)
            .await?;

    // Invalidate cache
    if let Err(e) = section_cache::invalidate_sections_cache(
        redis_pool.get_ref(),
        current_user.org_id,
        current_user.id,
    )
    .await
    {
        tracing::warn!("Failed to invalidate channel sections cache: {}", e);
    }

    Ok(HttpResponse::Created().json(item))
}

/// DELETE /api/channel-sections/{id}/channels/{channel_id} - Remove a channel from a section
pub async fn remove_channel(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    path: web::Path<(Uuid, Uuid)>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let (section_id, channel_id) = path.into_inner();
    let removed = ChannelSectionItem::remove(pool.get_ref(), section_id, channel_id).await?;
    if !removed {
        return Err(ApiError::NotFound(
            "Channel not found in section".to_string(),
        ));
    }

    // Invalidate cache
    if let Err(e) = section_cache::invalidate_sections_cache(
        redis_pool.get_ref(),
        current_user.org_id,
        current_user.id,
    )
    .await
    {
        tracing::warn!("Failed to invalidate channel sections cache: {}", e);
    }

    Ok(HttpResponse::NoContent().finish())
}

/// PUT /api/channel-sections/reorder - Bulk reorder sections
pub async fn reorder_sections(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    body: web::Json<ReorderSectionsRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    ChannelSection::bulk_reorder(
        pool.get_ref(),
        current_user.id,
        current_user.org_id,
        body.into_inner().order,
    )
    .await?;

    // Invalidate cache
    if let Err(e) = section_cache::invalidate_sections_cache(
        redis_pool.get_ref(),
        current_user.org_id,
        current_user.id,
    )
    .await
    {
        tracing::warn!("Failed to invalidate channel sections cache: {}", e);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "ok"})))
}

/// PUT /api/channel-sections/{id}/reorder - Bulk reorder items within a section
pub async fn reorder_items(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    section_id: web::Path<Uuid>,
    body: web::Json<ReorderItemsRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    ChannelSectionItem::bulk_reorder(pool.get_ref(), *section_id, body.into_inner().order).await?;

    // Invalidate cache
    if let Err(e) = section_cache::invalidate_sections_cache(
        redis_pool.get_ref(),
        current_user.org_id,
        current_user.id,
    )
    .await
    {
        tracing::warn!("Failed to invalidate channel sections cache: {}", e);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "ok"})))
}
