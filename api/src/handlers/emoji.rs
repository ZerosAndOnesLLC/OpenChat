use actix_multipart::Multipart;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::models::emoji::{
    is_allowed_emoji_type, validate_emoji_name, validate_emoji_size, CustomEmoji,
    EmojiUploadResponse,
};
use crate::services::tv_api::TokenClaims;
use crate::storage::StorageFactory;

const MAX_EMOJI_SIZE_BYTES: usize = 512 * 1024; // 512KB

pub async fn upload_emoji(
    req: HttpRequest,
    mut payload: Multipart,
    db: web::Data<PgPool>,
    storage_factory: web::Data<Arc<StorageFactory>>,
    redis: web::Data<MultiplexedConnection>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let org_id = claims.org_id;
    let user_id = claims.user_id;

    // Check if user has permission to upload custom emoji (admin only)
    // For now, we'll check if user has org.manage_settings permission
    let has_permission: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM user_roles ur
            JOIN role_permissions rp ON ur.role_id = rp.role_id
            JOIN permissions p ON rp.permission_id = p.id
            WHERE ur.user_id = $1
            AND p.permission_name = 'org.manage_settings'
        )"
    )
    .bind(user_id)
    .fetch_one(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to check permissions: {}", e)))?;

    if !has_permission {
        return Err(ApiError::Authorization(
            "Only admins can upload custom emojis".to_string(),
        ));
    }

    // Get storage for this org
    let storage = storage_factory.get_storage(org_id).await?;

    let mut emoji_name: Option<String> = None;
    let mut uploaded_file: Option<(Vec<u8>, String, String)> = None;

    // Process multipart form
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| {
            ApiError::BadRequest(format!("Failed to read multipart field: {}", e))
        })?;

        let field_name = field.name();

        if field_name == Some("name") {
            // Read emoji name field
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                let chunk = chunk.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read chunk: {}", e))
                })?;
                data.extend_from_slice(&chunk);
            }
            let name = String::from_utf8(data)
                .map_err(|e| ApiError::BadRequest(format!("Invalid emoji name: {}", e)))?;

            validate_emoji_name(&name).map_err(|e| ApiError::BadRequest(e))?;
            emoji_name = Some(name);
        } else if field_name == Some("file") {
            // Read file field
            let content_disposition = field.content_disposition();
            let file_name = content_disposition
                .and_then(|cd| cd.get_filename())
                .ok_or_else(|| ApiError::BadRequest("Missing filename".to_string()))?
                .to_string();

            let content_type = field
                .content_type()
                .map(|ct| ct.to_string())
                .unwrap_or_else(|| {
                    mime_guess::from_path(&file_name)
                        .first_or_octet_stream()
                        .to_string()
                });

            // Validate file type
            if !is_allowed_emoji_type(&content_type) {
                return Err(ApiError::BadRequest(format!(
                    "File type not allowed for emoji: {}. Allowed types: JPEG, PNG, GIF, WebP",
                    content_type
                )));
            }

            // Read file data
            let mut file_data = Vec::new();
            while let Some(chunk) = field.next().await {
                let chunk = chunk.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read file chunk: {}", e))
                })?;

                // Check size limit
                if file_data.len() + chunk.len() > MAX_EMOJI_SIZE_BYTES {
                    return Err(ApiError::BadRequest(
                        "Emoji file size exceeds maximum allowed size (512KB)".to_string(),
                    ));
                }

                file_data.extend_from_slice(&chunk);
            }

            let file_size = file_data.len() as i64;
            validate_emoji_size(file_size).map_err(|e| ApiError::BadRequest(e))?;

            uploaded_file = Some((file_data, file_name, content_type));
        }
    }

    let emoji_name = emoji_name
        .ok_or_else(|| ApiError::BadRequest("Missing emoji name".to_string()))?;

    let (file_data, file_name, content_type) = uploaded_file
        .ok_or_else(|| ApiError::BadRequest("No file uploaded".to_string()))?;

    // Check if emoji name already exists for this org
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM custom_emojis WHERE org_id = $1 AND name = $2)"
    )
    .bind(org_id)
    .bind(&emoji_name)
    .fetch_one(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to check emoji existence: {}", e)))?;

    if exists {
        return Err(ApiError::BadRequest(format!(
            "Emoji with name '{}' already exists",
            emoji_name
        )));
    }

    // TODO: Process image (resize to 128x128) - requires image crate
    // For now, we'll upload as-is
    let processed_data = file_data;

    // Upload to storage with emoji-specific path
    let storage_file_name = format!("emoji_{}_{}", emoji_name, file_name);
    let uploaded = storage
        .upload(&storage_file_name, &content_type, processed_data.into())
        .await?;

    // Save emoji to database
    let emoji_id = Uuid::new_v4();
    let image_url = format!("/api/emojis/{}/image", emoji_id);

    sqlx::query(
        "INSERT INTO custom_emojis
         (id, org_id, name, image_url, storage_type, storage_path, created_by, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())",
    )
    .bind(emoji_id)
    .bind(org_id)
    .bind(&emoji_name)
    .bind(&image_url)
    .bind(uploaded.storage_type.as_str())
    .bind(&uploaded.storage_path)
    .bind(user_id)
    .execute(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to save emoji: {}", e)))?;

    // Invalidate emoji cache for this org
    let cache_key = format!("emojis:org:{}", org_id);
    let mut redis_conn = redis.get_ref().clone();
    redis_conn.del::<_, ()>(&cache_key).await.ok();

    let emoji: CustomEmoji = sqlx::query_as(
        "SELECT * FROM custom_emojis WHERE id = $1"
    )
    .bind(emoji_id)
    .fetch_one(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch emoji: {}", e)))?;

    Ok(HttpResponse::Ok().json(EmojiUploadResponse {
        id: emoji.id,
        name: emoji.name,
        image_url: emoji.image_url.unwrap_or_default(),
        storage_type: emoji.storage_type,
        created_at: emoji.created_at,
    }))
}

pub async fn get_org_emojis(
    req: HttpRequest,
    db: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let org_id = claims.org_id;

    let mut redis_conn = redis.get_ref().clone();

    // Try to get from cache first
    let cache_key = format!("emojis:org:{}", org_id);
    if let Ok(Some(cached_json)) = redis_conn.get::<_, Option<String>>(&cache_key).await {
        if let Ok(cached) = serde_json::from_str::<Vec<CustomEmoji>>(&cached_json) {
            return Ok(HttpResponse::Ok().json(cached));
        }
    }

    // Fetch from database
    let emojis: Vec<CustomEmoji> = sqlx::query_as(
        "SELECT * FROM custom_emojis WHERE org_id = $1 ORDER BY name ASC",
    )
    .bind(org_id)
    .fetch_all(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch emojis: {}", e)))?;

    // Cache for 5 minutes
    if let Ok(emojis_json) = serde_json::to_string(&emojis) {
        redis_conn.set_ex::<_, _, ()>(&cache_key, emojis_json, 300).await.ok();
    }

    Ok(HttpResponse::Ok().json(emojis))
}

pub async fn delete_emoji(
    req: HttpRequest,
    emoji_id: web::Path<Uuid>,
    db: web::Data<PgPool>,
    storage_factory: web::Data<Arc<StorageFactory>>,
    redis: web::Data<MultiplexedConnection>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let emoji_id = emoji_id.into_inner();
    let org_id = claims.org_id;
    let user_id = claims.user_id;

    // Check if user has permission to delete custom emoji (admin only)
    let has_permission: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM user_roles ur
            JOIN role_permissions rp ON ur.role_id = rp.role_id
            JOIN permissions p ON rp.permission_id = p.id
            WHERE ur.user_id = $1
            AND p.permission_name = 'org.manage_settings'
        )"
    )
    .bind(user_id)
    .fetch_one(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to check permissions: {}", e)))?;

    if !has_permission {
        return Err(ApiError::Authorization(
            "Only admins can delete custom emojis".to_string(),
        ));
    }

    // Fetch emoji
    let emoji: CustomEmoji = sqlx::query_as(
        "SELECT * FROM custom_emojis WHERE id = $1 AND org_id = $2",
    )
    .bind(emoji_id)
    .bind(org_id)
    .fetch_optional(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch emoji: {}", e)))?
    .ok_or_else(|| ApiError::NotFound("Emoji not found".to_string()))?;

    // Get storage for this org
    let storage = storage_factory.get_storage(org_id).await?;

    // Delete from storage
    storage.delete(&emoji.storage_path).await?;

    // Delete from database
    sqlx::query("DELETE FROM custom_emojis WHERE id = $1")
        .bind(emoji_id)
        .execute(db.get_ref())
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to delete emoji: {}", e)))?;

    // Invalidate emoji cache for this org
    let cache_key = format!("emojis:org:{}", org_id);
    let mut redis_conn = redis.get_ref().clone();
    redis_conn.del::<_, ()>(&cache_key).await.ok();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Emoji deleted successfully"
    })))
}

pub async fn get_emoji_image(
    emoji_id: web::Path<Uuid>,
    db: web::Data<PgPool>,
    storage_factory: web::Data<Arc<StorageFactory>>,
) -> Result<HttpResponse, ApiError> {
    let emoji_id = emoji_id.into_inner();

    // Fetch emoji metadata (no auth required for viewing emojis)
    let emoji: CustomEmoji = sqlx::query_as(
        "SELECT * FROM custom_emojis WHERE id = $1",
    )
    .bind(emoji_id)
    .fetch_optional(db.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch emoji: {}", e)))?
    .ok_or_else(|| ApiError::NotFound("Emoji not found".to_string()))?;

    // Get storage for this org
    let storage = storage_factory.get_storage(emoji.org_id).await?;

    // Download from storage
    let data = storage.download(&emoji.storage_path).await?;

    // Determine content type
    let content_type = mime_guess::from_path(&emoji.storage_path)
        .first_or("image/png".parse().unwrap())
        .to_string();

    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .append_header(("Cache-Control", "public, max-age=31536000")) // Cache for 1 year
        .body(data))
}
