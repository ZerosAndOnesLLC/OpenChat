use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    cache::notification_prefs as prefs_cache,
    db::RedisPool,
    errors::{ApiError, ApiResult},
    models::notification_pref::{NotificationPref, UpsertNotificationPref},
    models::user::User,
    services::tv_api::TokenClaims,
};

/// PUT /api/channels/{id}/notifications
pub async fn set_channel_notification_pref(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    channel_id: web::Path<Uuid>,
    body: web::Json<UpsertNotificationPref>,
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

    let data = body.into_inner();

    // Validate preference value
    if !["all", "mentions", "nothing"].contains(&data.preference.as_str()) {
        return Err(ApiError::BadRequest(
            "preference must be 'all', 'mentions', or 'nothing'".to_string(),
        ));
    }

    let pref = NotificationPref::upsert_channel(
        pool.get_ref(),
        current_user.id,
        *channel_id,
        data,
    )
    .await?;

    // Invalidate cache
    if let Err(e) = prefs_cache::invalidate_user_prefs_cache(
        redis_pool.get_ref(),
        current_user.org_id,
        current_user.id,
    )
    .await
    {
        tracing::warn!("Failed to invalidate notification prefs cache: {}", e);
    }

    Ok(HttpResponse::Ok().json(pref))
}

/// GET /api/channels/{id}/notifications
pub async fn get_channel_notification_pref(
    pool: web::Data<PgPool>,
    channel_id: web::Path<Uuid>,
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

    let pref = NotificationPref::get_for_channel(
        pool.get_ref(),
        current_user.id,
        *channel_id,
    )
    .await?;

    match pref {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        None => Ok(HttpResponse::Ok().json(serde_json::json!({
            "preference": "all",
            "mute_until": null,
        }))),
    }
}

/// PUT /api/dms/{id}/notifications
pub async fn set_dm_notification_pref(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    dm_id: web::Path<Uuid>,
    body: web::Json<UpsertNotificationPref>,
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

    let data = body.into_inner();

    if !["all", "mentions", "nothing"].contains(&data.preference.as_str()) {
        return Err(ApiError::BadRequest(
            "preference must be 'all', 'mentions', or 'nothing'".to_string(),
        ));
    }

    let pref = NotificationPref::upsert_dm(
        pool.get_ref(),
        current_user.id,
        *dm_id,
        data,
    )
    .await?;

    // Invalidate cache
    if let Err(e) = prefs_cache::invalidate_user_prefs_cache(
        redis_pool.get_ref(),
        current_user.org_id,
        current_user.id,
    )
    .await
    {
        tracing::warn!("Failed to invalidate notification prefs cache: {}", e);
    }

    Ok(HttpResponse::Ok().json(pref))
}

/// GET /api/dms/{id}/notifications
pub async fn get_dm_notification_pref(
    pool: web::Data<PgPool>,
    dm_id: web::Path<Uuid>,
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

    let pref = NotificationPref::get_for_dm(
        pool.get_ref(),
        current_user.id,
        *dm_id,
    )
    .await?;

    match pref {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        None => Ok(HttpResponse::Ok().json(serde_json::json!({
            "preference": "all",
            "mute_until": null,
        }))),
    }
}
