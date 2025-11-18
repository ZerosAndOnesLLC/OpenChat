use actix_multipart::Multipart;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::models::attachment::{
    is_allowed_file_type, validate_file_size, Attachment, AttachmentUploadResponse,
    MAX_FILE_SIZE,
};
use crate::services::tv_api::TokenClaims;
use crate::storage::StorageFactory;

const MAX_FILE_SIZE_BYTES: usize = 25 * 1024 * 1024; // 25MB

pub async fn upload_attachment(
    req: HttpRequest,
    mut payload: Multipart,
    db: web::Data<PgPool>,
    storage_factory: web::Data<Arc<StorageFactory>>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let org_id = claims.org_id;

    // Get storage for this org
    let storage = storage_factory.get_storage(org_id).await?;

    let mut message_id: Option<Uuid> = None;
    let mut uploaded_files = Vec::new();

    // Process multipart form
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| {
            ApiError::BadRequest(format!("Failed to read multipart field: {}", e))
        })?;

        let field_name = field.name();

        if field_name == Some("message_id") {
            // Read message_id field
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                let chunk = chunk.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read chunk: {}", e))
                })?;
                data.extend_from_slice(&chunk);
            }
            let message_id_str = String::from_utf8(data).map_err(|e| {
                ApiError::BadRequest(format!("Invalid message_id: {}", e))
            })?;
            message_id = Some(Uuid::parse_str(&message_id_str).map_err(|e| {
                ApiError::BadRequest(format!("Invalid message_id UUID: {}", e))
            })?);
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
            if !is_allowed_file_type(&content_type) {
                return Err(ApiError::BadRequest(format!(
                    "File type not allowed: {}",
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
                if file_data.len() + chunk.len() > MAX_FILE_SIZE_BYTES {
                    return Err(ApiError::BadRequest(
                        "File size exceeds maximum allowed size".to_string(),
                    ));
                }

                file_data.extend_from_slice(&chunk);
            }

            let file_size = file_data.len() as i64;
            validate_file_size(file_size, MAX_FILE_SIZE).map_err(|e| {
                ApiError::BadRequest(e)
            })?;

            // Upload to storage
            let uploaded = storage
                .upload(&file_name, &content_type, file_data.into())
                .await?;

            uploaded_files.push((uploaded, content_type));
        }
    }

    let message_id = message_id
        .ok_or_else(|| ApiError::BadRequest("Missing message_id".to_string()))?;

    if uploaded_files.is_empty() {
        return Err(ApiError::BadRequest("No files uploaded".to_string()));
    }

    // Verify message exists and user has access
    let message: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM messages WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(message_id)
    .fetch_optional(db.get_ref())
    .await
    .map_err(|e| {
        ApiError::Internal(format!("Failed to verify message: {}", e))
    })?;

    if message.is_none() {
        return Err(ApiError::NotFound("Message not found".to_string()));
    }

    // Save attachments to database
    let mut attachment_responses = Vec::new();

    for (uploaded, content_type) in uploaded_files {
        let attachment_id = Uuid::new_v4();
        let file_url = format!("/api/attachments/{}/download", attachment_id);

        sqlx::query(
            "INSERT INTO attachments
             (id, message_id, file_name, file_url, file_type, file_size, storage_type, storage_path, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())",
        )
        .bind(attachment_id)
        .bind(message_id)
        .bind(&uploaded.file_name)
        .bind(&file_url)
        .bind(&content_type)
        .bind(uploaded.size)
        .bind(uploaded.storage_type.as_str())
        .bind(&uploaded.storage_path)
        .execute(db.get_ref())
        .await
        .map_err(|e| {
            ApiError::Internal(format!("Failed to save attachment: {}", e))
        })?;

        attachment_responses.push(AttachmentUploadResponse {
            id: attachment_id,
            file_name: uploaded.file_name,
            file_url,
            file_type: Some(content_type),
            file_size: uploaded.size,
            storage_type: uploaded.storage_type.as_str().to_string(),
        });
    }

    Ok(HttpResponse::Ok().json(attachment_responses))
}

pub async fn download_attachment(
    req: HttpRequest,
    attachment_id: web::Path<Uuid>,
    db: web::Data<PgPool>,
    storage_factory: web::Data<Arc<StorageFactory>>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let attachment_id = attachment_id.into_inner();

    // Fetch attachment metadata
    let attachment: Attachment = sqlx::query_as(
        "SELECT * FROM attachments WHERE id = $1",
    )
    .bind(attachment_id)
    .fetch_optional(db.get_ref())
    .await
    .map_err(|e| {
        ApiError::Internal(format!("Failed to fetch attachment: {}", e))
    })?
    .ok_or_else(|| ApiError::NotFound("Attachment not found".to_string()))?;

    // Verify user has access to the message
    let has_access: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM messages m
            LEFT JOIN channels c ON m.channel_id = c.id
            LEFT JOIN channel_members cm ON c.id = cm.channel_id
            LEFT JOIN direct_messages dm ON m.dm_id = dm.id
            LEFT JOIN dm_participants dp ON dm.id = dp.dm_id
            WHERE m.id = $1
            AND m.deleted_at IS NULL
            AND (
                (m.channel_id IS NOT NULL AND cm.user_id = $2)
                OR (m.dm_id IS NOT NULL AND dp.user_id = $2)
            )
        )",
    )
    .bind(attachment.message_id)
    .bind(claims.user_id)
    .fetch_one(db.get_ref())
    .await
    .map_err(|e| {
        ApiError::Internal(format!("Failed to verify access: {}", e))
    })?;

    if !has_access {
        return Err(ApiError::Authorization(
            "You don't have access to this attachment".to_string(),
        ));
    }

    // Get storage for this org
    let storage = storage_factory.get_storage(claims.org_id).await?;

    // Download from storage
    let data = storage.download(&attachment.storage_path).await?;

    let content_type = attachment
        .file_type
        .unwrap_or_else(|| "application/octet-stream".to_string());

    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .append_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", attachment.file_name),
        ))
        .body(data))
}

pub async fn delete_attachment(
    req: HttpRequest,
    attachment_id: web::Path<Uuid>,
    db: web::Data<PgPool>,
    storage_factory: web::Data<Arc<StorageFactory>>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let attachment_id = attachment_id.into_inner();

    // Fetch attachment metadata
    let attachment: Attachment = sqlx::query_as(
        "SELECT * FROM attachments WHERE id = $1",
    )
    .bind(attachment_id)
    .fetch_optional(db.get_ref())
    .await
    .map_err(|e| {
        ApiError::Internal(format!("Failed to fetch attachment: {}", e))
    })?
    .ok_or_else(|| ApiError::NotFound("Attachment not found".to_string()))?;

    // Verify user is the message author or has permission
    let message_user_id: Uuid = sqlx::query_scalar(
        "SELECT user_id FROM messages WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(attachment.message_id)
    .fetch_optional(db.get_ref())
    .await
    .map_err(|e| {
        ApiError::Internal(format!("Failed to fetch message: {}", e))
    })?
    .ok_or_else(|| ApiError::NotFound("Message not found".to_string()))?;

    if message_user_id != claims.user_id {
        return Err(ApiError::Authorization(
            "You can only delete your own attachments".to_string(),
        ));
    }

    // Get storage for this org
    let storage = storage_factory.get_storage(claims.org_id).await?;

    // Delete from storage
    storage.delete(&attachment.storage_path).await?;

    // Delete from database
    sqlx::query("DELETE FROM attachments WHERE id = $1")
        .bind(attachment_id)
        .execute(db.get_ref())
        .await
        .map_err(|e| {
            ApiError::Internal(format!("Failed to delete attachment: {}", e))
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Attachment deleted successfully"
    })))
}

pub async fn get_message_attachments(
    req: HttpRequest,
    message_id: web::Path<Uuid>,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let message_id = message_id.into_inner();

    // Verify user has access to the message
    let has_access: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM messages m
            LEFT JOIN channels c ON m.channel_id = c.id
            LEFT JOIN channel_members cm ON c.id = cm.channel_id
            LEFT JOIN direct_messages dm ON m.dm_id = dm.id
            LEFT JOIN dm_participants dp ON dm.id = dp.dm_id
            WHERE m.id = $1
            AND m.deleted_at IS NULL
            AND (
                (m.channel_id IS NOT NULL AND cm.user_id = $2)
                OR (m.dm_id IS NOT NULL AND dp.user_id = $2)
            )
        )",
    )
    .bind(message_id)
    .bind(claims.user_id)
    .fetch_one(db.get_ref())
    .await
    .map_err(|e| {
        ApiError::Internal(format!("Failed to verify access: {}", e))
    })?;

    if !has_access {
        return Err(ApiError::Authorization(
            "You don't have access to this message".to_string(),
        ));
    }

    // Fetch attachments
    let attachments: Vec<Attachment> = sqlx::query_as(
        "SELECT * FROM attachments WHERE message_id = $1 ORDER BY created_at ASC",
    )
    .bind(message_id)
    .fetch_all(db.get_ref())
    .await
    .map_err(|e| {
        ApiError::Internal(format!("Failed to fetch attachments: {}", e))
    })?;

    Ok(HttpResponse::Ok().json(attachments))
}
