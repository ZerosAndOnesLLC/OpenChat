use actix_web::{web, HttpMessage, HttpResponse, HttpRequest};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{ApiError, ApiResult};
use crate::migration::models::*;
use crate::migration::service::MigrationService;
use crate::models::user::User;
use crate::services::tv_api::TokenClaims;

/// Validate Mattermost connection
pub async fn validate_connection(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<ValidateConnectionRequest>,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let tv_api_url = std::env::var("TV_API_URL")
        .unwrap_or_else(|_| "https://api.titanium-vault.com".to_string());

    // Get access token from request header
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Authentication("Missing authorization header".to_string()))?;

    let service = MigrationService::new(
        pool.get_ref().clone(),
        tv_api_url,
        auth_header.to_string(),
        claims.org_id,
    );

    let result = service.validate_connection(&body.connection).await
        .map_err(|e| ApiError::Internal(format!("Validation failed: {}", e)))?;

    Ok(HttpResponse::Ok().json(result))
}

/// Get migration preview
pub async fn get_preview(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<PreviewRequest>,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let tv_api_url = std::env::var("TV_API_URL")
        .unwrap_or_else(|_| "https://api.titanium-vault.com".to_string());

    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Authentication("Missing authorization header".to_string()))?;

    let service = MigrationService::new(
        pool.get_ref().clone(),
        tv_api_url,
        auth_header.to_string(),
        claims.org_id,
    );

    let preview = service.get_preview(&body.connection).await
        .map_err(|e| ApiError::Internal(format!("Preview failed: {}", e)))?;

    Ok(HttpResponse::Ok().json(preview))
}

/// Start migration job
pub async fn start_migration(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<StartMigrationRequest>,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let tv_api_url = std::env::var("TV_API_URL")
        .unwrap_or_else(|_| "https://api.titanium-vault.com".to_string());

    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Authentication("Missing authorization header".to_string()))?;

    // Get current user
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let service = MigrationService::new(
        pool.get_ref().clone(),
        tv_api_url,
        auth_header.to_string(),
        claims.org_id,
    );

    let job_id = service.start_migration(
        body.connection.clone(),
        body.options.clone(),
        current_user.id,
    ).await
        .map_err(|e| ApiError::Internal(format!("Failed to start migration: {}", e)))?;

    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "job_id": job_id,
        "message": "Migration job started"
    })))
}

/// Get migration job status
pub async fn get_job_status(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let job_id = path.into_inner();

    let tv_api_url = std::env::var("TV_API_URL")
        .unwrap_or_else(|_| "https://api.titanium-vault.com".to_string());

    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Authentication("Missing authorization header".to_string()))?;

    let service = MigrationService::new(
        pool.get_ref().clone(),
        tv_api_url,
        auth_header.to_string(),
        claims.org_id,
    );

    let job = service.get_job_status(job_id).await
        .map_err(|e| ApiError::Internal(format!("Failed to get job status: {}", e)))?;

    match job {
        Some(j) => Ok(HttpResponse::Ok().json(j)),
        None => Err(ApiError::NotFound("Migration job not found".to_string())),
    }
}

/// List migration jobs for org
pub async fn list_jobs(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let jobs = sqlx::query_as::<_, MigrationJob>(
        r#"
        SELECT id, org_id, status, progress, error, started_at, completed_at, created_by
        FROM migration_jobs
        WHERE org_id = $1
        ORDER BY started_at DESC
        LIMIT 20
        "#
    )
    .bind(claims.org_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to list jobs: {}", e)))?;

    Ok(HttpResponse::Ok().json(jobs))
}

/// Cancel a running migration job
pub async fn cancel_job(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let job_id = path.into_inner();

    let result = sqlx::query(
        r#"
        UPDATE migration_jobs
        SET status = 'cancelled', completed_at = NOW()
        WHERE id = $1 AND org_id = $2 AND status IN ('pending', 'running')
        "#
    )
    .bind(job_id)
    .bind(claims.org_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to cancel job: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Job not found or already completed".to_string()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Migration job cancelled"
    })))
}
