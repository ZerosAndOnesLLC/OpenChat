use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    cache::search,
    errors::{ApiError, ApiResult},
    models::message::Message,
    services::tv_api::TokenClaims,
};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// Search query text
    pub q: String,
    /// Search scope: channel, dm, or all
    #[serde(default = "default_scope")]
    pub scope: String,
    /// Optional channel ID filter
    pub channel_id: Option<Uuid>,
    /// Optional DM ID filter
    pub dm_id: Option<Uuid>,
    /// Optional limit for results
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_scope() -> String {
    "all".to_string()
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub messages: Vec<Message>,
    pub total_count: i64,
}

/// Search messages using PostgreSQL full-text search
/// Supports basic filters extracted from query:
/// - from:@username - filter by user (future enhancement)
/// - in:#channel - filter by channel (future enhancement)
/// - before:date - messages before date (future enhancement)
/// - after:date - messages after date (future enhancement)
pub async fn search_messages(
    pool: web::Data<PgPool>,
    redis: web::Data<MultiplexedConnection>,
    query: web::Query<SearchQuery>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let org_id = claims.org_id;
    let limit = query.limit.min(100); // Cap at 100 results

    // Parse search query - for now we'll use simple full-text search
    // Future: parse advanced filters like from:@user, in:#channel, before:date, after:date
    let search_query = &query.q;

    if search_query.trim().is_empty() {
        return Err(ApiError::BadRequest("Search query cannot be empty".to_string()));
    }

    // Build cache key parameters
    let org_id_str = org_id.to_string();
    let channel_id_str = query.channel_id.map(|id| id.to_string());
    let dm_id_str = query.dm_id.map(|id| id.to_string());

    // Try to get cached results
    let mut redis_conn = redis.as_ref().clone();
    if let Ok(Some(cached_results)) = search::get_search_results_from_cache(
        &mut redis_conn,
        &org_id_str,
        search_query,
        &query.scope,
        channel_id_str.as_deref(),
        dm_id_str.as_deref(),
    ).await {
        return Ok(HttpResponse::Ok().json(cached_results));
    }

    // Build the search query based on scope
    let (messages, total_count) = match query.scope.as_str() {
        "channel" => {
            if let Some(channel_id) = query.channel_id {
                search_in_channel(&pool, org_id, channel_id, search_query, limit).await?
            } else {
                return Err(ApiError::BadRequest("channel_id is required when scope is 'channel'".to_string()));
            }
        }
        "dm" => {
            if let Some(dm_id) = query.dm_id {
                search_in_dm(&pool, org_id, dm_id, search_query, limit).await?
            } else {
                return Err(ApiError::BadRequest("dm_id is required when scope is 'dm'".to_string()));
            }
        }
        "all" => {
            search_all(&pool, org_id, search_query, limit).await?
        }
        _ => {
            return Err(ApiError::BadRequest("Invalid scope. Must be 'channel', 'dm', or 'all'".to_string()));
        }
    };

    let result = SearchResult {
        messages,
        total_count,
    };

    // Cache the results
    let _ = search::set_search_results_in_cache(
        &mut redis_conn,
        &org_id_str,
        search_query,
        &query.scope,
        channel_id_str.as_deref(),
        dm_id_str.as_deref(),
        &result,
    ).await; // Ignore cache errors

    Ok(HttpResponse::Ok().json(result))
}

/// Search messages in a specific channel
async fn search_in_channel(
    pool: &PgPool,
    org_id: Uuid,
    channel_id: Uuid,
    query: &str,
    limit: i64,
) -> ApiResult<(Vec<Message>, i64)> {
    // Verify channel belongs to org
    let channel_exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM channels WHERE id = $1 AND org_id = $2
        )
        "#,
    )
    .bind(channel_id)
    .bind(org_id)
    .fetch_one(pool)
    .await?;

    if !channel_exists {
        return Err(ApiError::NotFound("Channel not found".to_string()));
    }

    // Convert query to tsquery format
    let tsquery = query
        .split_whitespace()
        .map(|word| format!("{}:*", word))
        .collect::<Vec<_>>()
        .join(" & ");

    // Search messages using full-text search
    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT m.*
        FROM messages m
        WHERE m.channel_id = $1
            AND m.deleted_at IS NULL
            AND m.content_tsv @@ to_tsquery('english', $2)
        ORDER BY m.created_at DESC
        LIMIT $3
        "#,
    )
    .bind(channel_id)
    .bind(&tsquery)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    // Get total count
    let total_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM messages m
        WHERE m.channel_id = $1
            AND m.deleted_at IS NULL
            AND m.content_tsv @@ to_tsquery('english', $2)
        "#,
    )
    .bind(channel_id)
    .bind(&tsquery)
    .fetch_one(pool)
    .await?;

    Ok((messages, total_count))
}

/// Search messages in a specific DM
async fn search_in_dm(
    pool: &PgPool,
    org_id: Uuid,
    dm_id: Uuid,
    query: &str,
    limit: i64,
) -> ApiResult<(Vec<Message>, i64)> {
    // Verify DM belongs to org
    let dm_exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM direct_messages WHERE id = $1 AND org_id = $2
        )
        "#,
    )
    .bind(dm_id)
    .bind(org_id)
    .fetch_one(pool)
    .await?;

    if !dm_exists {
        return Err(ApiError::NotFound("DM not found".to_string()));
    }

    // Convert query to tsquery format
    let tsquery = query
        .split_whitespace()
        .map(|word| format!("{}:*", word))
        .collect::<Vec<_>>()
        .join(" & ");

    // Search messages using full-text search
    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT m.*
        FROM messages m
        WHERE m.dm_id = $1
            AND m.deleted_at IS NULL
            AND m.content_tsv @@ to_tsquery('english', $2)
        ORDER BY m.created_at DESC
        LIMIT $3
        "#,
    )
    .bind(dm_id)
    .bind(&tsquery)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    // Get total count
    let total_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM messages m
        WHERE m.dm_id = $1
            AND m.deleted_at IS NULL
            AND m.content_tsv @@ to_tsquery('english', $2)
        "#,
    )
    .bind(dm_id)
    .bind(&tsquery)
    .fetch_one(pool)
    .await?;

    Ok((messages, total_count))
}

/// Search all messages accessible to the user
async fn search_all(
    pool: &PgPool,
    org_id: Uuid,
    query: &str,
    limit: i64,
) -> ApiResult<(Vec<Message>, i64)> {
    // Convert query to tsquery format
    let tsquery = query
        .split_whitespace()
        .map(|word| format!("{}:*", word))
        .collect::<Vec<_>>()
        .join(" & ");

    // Search messages across all channels and DMs in the org
    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT m.*
        FROM messages m
        LEFT JOIN channels c ON m.channel_id = c.id
        LEFT JOIN direct_messages dm ON m.dm_id = dm.id
        WHERE (c.org_id = $1 OR dm.org_id = $1)
            AND m.deleted_at IS NULL
            AND m.content_tsv @@ to_tsquery('english', $2)
        ORDER BY m.created_at DESC
        LIMIT $3
        "#,
    )
    .bind(org_id)
    .bind(&tsquery)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    // Get total count
    let total_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM messages m
        LEFT JOIN channels c ON m.channel_id = c.id
        LEFT JOIN direct_messages dm ON m.dm_id = dm.id
        WHERE (c.org_id = $1 OR dm.org_id = $1)
            AND m.deleted_at IS NULL
            AND m.content_tsv @@ to_tsquery('english', $2)
        "#,
    )
    .bind(org_id)
    .bind(&tsquery)
    .fetch_one(pool)
    .await?;

    Ok((messages, total_count))
}
