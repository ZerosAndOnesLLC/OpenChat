use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, NaiveDate, Utc};
use crate::db::RedisPool;
use regex::Regex;
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

#[derive(Debug, Clone)]
struct SearchFilters {
    /// Plain text query after filters are extracted
    text_query: String,
    /// Filter by username (from:@username)
    from_user: Option<String>,
    /// Filter by channel name (in:#channel)
    in_channel: Option<String>,
    /// Messages before this date (before:YYYY-MM-DD)
    before_date: Option<DateTime<Utc>>,
    /// Messages after this date (after:YYYY-MM-DD)
    after_date: Option<DateTime<Utc>>,
}

impl SearchFilters {
    fn parse(query: &str) -> Self {
        let mut filters = SearchFilters {
            text_query: query.to_string(),
            from_user: None,
            in_channel: None,
            before_date: None,
            after_date: None,
        };

        // Parse from:@username
        let from_re = Regex::new(r"from:@?(\w+)").unwrap();
        if let Some(cap) = from_re.captures(query) {
            filters.from_user = cap.get(1).map(|m| m.as_str().to_string());
            filters.text_query = from_re.replace(&filters.text_query, "").to_string();
        }

        // Parse in:#channel
        let in_re = Regex::new(r"in:#?(\w+)").unwrap();
        if let Some(cap) = in_re.captures(query) {
            filters.in_channel = cap.get(1).map(|m| m.as_str().to_string());
            filters.text_query = in_re.replace(&filters.text_query, "").to_string();
        }

        // Parse before:YYYY-MM-DD
        let before_re = Regex::new(r"before:(\d{4}-\d{2}-\d{2})").unwrap();
        if let Some(cap) = before_re.captures(query) {
            if let Some(date_str) = cap.get(1) {
                if let Ok(date) = NaiveDate::parse_from_str(date_str.as_str(), "%Y-%m-%d") {
                    // Set to end of day (23:59:59)
                    filters.before_date = Some(date.and_hms_opt(23, 59, 59)
                        .unwrap()
                        .and_local_timezone(Utc)
                        .single()
                        .unwrap());
                }
            }
            filters.text_query = before_re.replace(&filters.text_query, "").to_string();
        }

        // Parse after:YYYY-MM-DD
        let after_re = Regex::new(r"after:(\d{4}-\d{2}-\d{2})").unwrap();
        if let Some(cap) = after_re.captures(query) {
            if let Some(date_str) = cap.get(1) {
                if let Ok(date) = NaiveDate::parse_from_str(date_str.as_str(), "%Y-%m-%d") {
                    // Set to start of day (00:00:00)
                    filters.after_date = Some(date.and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_local_timezone(Utc)
                        .single()
                        .unwrap());
                }
            }
            filters.text_query = after_re.replace(&filters.text_query, "").to_string();
        }

        // Clean up the text query (remove extra whitespace)
        filters.text_query = filters.text_query
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        filters
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub messages: Vec<Message>,
    pub total_count: i64,
}

/// Search messages using PostgreSQL full-text search
/// Supports advanced filters:
/// - from:@username - filter by user
/// - in:#channel - filter by channel
/// - before:YYYY-MM-DD - messages before date
/// - after:YYYY-MM-DD - messages after date
pub async fn search_messages(
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    query: web::Query<SearchQuery>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let org_id = claims.org_id;
    let user_id = claims.user_id;
    let limit = query.limit.min(100); // Cap at 100 results

    // Parse search query and extract filters
    let search_query = &query.q;

    if search_query.trim().is_empty() {
        return Err(ApiError::BadRequest("Search query cannot be empty".to_string()));
    }

    // Parse filters from query
    let filters = SearchFilters::parse(search_query);

    // Build cache key parameters
    let org_id_str = org_id.to_string();
    let channel_id_str = query.channel_id.map(|id| id.to_string());
    let dm_id_str = query.dm_id.map(|id| id.to_string());

    // Try to get cached results
    
    if let Ok(Some(cached_results)) = search::get_search_results_from_cache(
        redis_pool.get_ref(),
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
                search_in_channel(&pool, org_id, user_id, channel_id, &filters, limit).await?
            } else {
                return Err(ApiError::BadRequest("channel_id is required when scope is 'channel'".to_string()));
            }
        }
        "dm" => {
            if let Some(dm_id) = query.dm_id {
                search_in_dm(&pool, org_id, user_id, dm_id, &filters, limit).await?
            } else {
                return Err(ApiError::BadRequest("dm_id is required when scope is 'dm'".to_string()));
            }
        }
        "all" => {
            search_all(&pool, org_id, user_id, &filters, limit).await?
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
        redis_pool.get_ref(),
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
    _user_id: Uuid,
    channel_id: Uuid,
    filters: &SearchFilters,
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

    // Build dynamic query with filters
    let mut query_builder = String::from(
        r#"
        SELECT m.* FROM messages m
        LEFT JOIN users u ON m.user_id = u.id
        WHERE m.channel_id = $1 AND m.deleted_at IS NULL AND m.encrypted_content IS NULL
        "#,
    );

    let mut count_builder = String::from(
        r#"
        SELECT COUNT(*) FROM messages m
        LEFT JOIN users u ON m.user_id = u.id
        WHERE m.channel_id = $1 AND m.deleted_at IS NULL AND m.encrypted_content IS NULL
        "#,
    );

    let mut bind_idx = 2;
    let mut bindings: Vec<String> = vec![];

    // Add text search filter
    if !filters.text_query.trim().is_empty() {
        let tsquery = filters
            .text_query
            .split_whitespace()
            .map(|word| format!("{}:*", word))
            .collect::<Vec<_>>()
            .join(" & ");
        query_builder.push_str(&format!(" AND m.content_tsv @@ to_tsquery('english', ${})", bind_idx));
        count_builder.push_str(&format!(" AND m.content_tsv @@ to_tsquery('english', ${})", bind_idx));
        bindings.push(tsquery);
        bind_idx += 1;
    }

    // Add from_user filter
    if let Some(ref username) = filters.from_user {
        query_builder.push_str(&format!(" AND u.username ILIKE ${}", bind_idx));
        count_builder.push_str(&format!(" AND u.username ILIKE ${}", bind_idx));
        bindings.push(username.clone());
        bind_idx += 1;
    }

    // Add before_date filter
    if let Some(before_date) = filters.before_date {
        query_builder.push_str(&format!(" AND m.created_at < ${}", bind_idx));
        count_builder.push_str(&format!(" AND m.created_at < ${}", bind_idx));
        bindings.push(before_date.to_rfc3339());
        bind_idx += 1;
    }

    // Add after_date filter
    if let Some(after_date) = filters.after_date {
        query_builder.push_str(&format!(" AND m.created_at > ${}", bind_idx));
        count_builder.push_str(&format!(" AND m.created_at > ${}", bind_idx));
        bindings.push(after_date.to_rfc3339());
        bind_idx += 1;
    }

    query_builder.push_str(&format!(" ORDER BY m.created_at DESC LIMIT ${}", bind_idx));

    // Execute search query
    let mut query = sqlx::query_as::<_, Message>(sqlx::AssertSqlSafe(query_builder.clone())).bind(channel_id);
    for binding in &bindings {
        query = query.bind(binding);
    }
    query = query.bind(limit);
    let messages = query.fetch_all(pool).await?;

    // Execute count query
    let mut count_query = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(count_builder)).bind(channel_id);
    for binding in &bindings {
        count_query = count_query.bind(binding);
    }
    let total_count = count_query.fetch_one(pool).await?;

    Ok((messages, total_count))
}

/// Search messages in a specific DM
async fn search_in_dm(
    pool: &PgPool,
    org_id: Uuid,
    _user_id: Uuid,
    dm_id: Uuid,
    filters: &SearchFilters,
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

    // Build dynamic query with filters
    let mut query_builder = String::from(
        r#"
        SELECT m.* FROM messages m
        LEFT JOIN users u ON m.user_id = u.id
        WHERE m.dm_id = $1 AND m.deleted_at IS NULL AND m.encrypted_content IS NULL
        "#,
    );

    let mut count_builder = String::from(
        r#"
        SELECT COUNT(*) FROM messages m
        LEFT JOIN users u ON m.user_id = u.id
        WHERE m.dm_id = $1 AND m.deleted_at IS NULL AND m.encrypted_content IS NULL
        "#,
    );

    let mut bind_idx = 2;
    let mut bindings: Vec<String> = vec![];

    // Add text search filter
    if !filters.text_query.trim().is_empty() {
        let tsquery = filters
            .text_query
            .split_whitespace()
            .map(|word| format!("{}:*", word))
            .collect::<Vec<_>>()
            .join(" & ");
        query_builder.push_str(&format!(" AND m.content_tsv @@ to_tsquery('english', ${})", bind_idx));
        count_builder.push_str(&format!(" AND m.content_tsv @@ to_tsquery('english', ${})", bind_idx));
        bindings.push(tsquery);
        bind_idx += 1;
    }

    // Add from_user filter
    if let Some(ref username) = filters.from_user {
        query_builder.push_str(&format!(" AND u.username ILIKE ${}", bind_idx));
        count_builder.push_str(&format!(" AND u.username ILIKE ${}", bind_idx));
        bindings.push(username.clone());
        bind_idx += 1;
    }

    // Add before_date filter
    if let Some(before_date) = filters.before_date {
        query_builder.push_str(&format!(" AND m.created_at < ${}", bind_idx));
        count_builder.push_str(&format!(" AND m.created_at < ${}", bind_idx));
        bindings.push(before_date.to_rfc3339());
        bind_idx += 1;
    }

    // Add after_date filter
    if let Some(after_date) = filters.after_date {
        query_builder.push_str(&format!(" AND m.created_at > ${}", bind_idx));
        count_builder.push_str(&format!(" AND m.created_at > ${}", bind_idx));
        bindings.push(after_date.to_rfc3339());
        bind_idx += 1;
    }

    query_builder.push_str(&format!(" ORDER BY m.created_at DESC LIMIT ${}", bind_idx));

    // Execute search query
    let mut query = sqlx::query_as::<_, Message>(sqlx::AssertSqlSafe(query_builder.clone())).bind(dm_id);
    for binding in &bindings {
        query = query.bind(binding);
    }
    query = query.bind(limit);
    let messages = query.fetch_all(pool).await?;

    // Execute count query
    let mut count_query = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(count_builder)).bind(dm_id);
    for binding in &bindings {
        count_query = count_query.bind(binding);
    }
    let total_count = count_query.fetch_one(pool).await?;

    Ok((messages, total_count))
}

/// Search all messages accessible to the user
async fn search_all(
    pool: &PgPool,
    org_id: Uuid,
    _user_id: Uuid,
    filters: &SearchFilters,
    limit: i64,
) -> ApiResult<(Vec<Message>, i64)> {
    // Build dynamic query with filters
    let mut query_builder = String::from(
        r#"
        SELECT m.* FROM messages m
        LEFT JOIN channels c ON m.channel_id = c.id
        LEFT JOIN direct_messages dm ON m.dm_id = dm.id
        LEFT JOIN users u ON m.user_id = u.id
        WHERE (c.org_id = $1 OR dm.org_id = $1)
            AND m.deleted_at IS NULL
            AND m.encrypted_content IS NULL
        "#,
    );

    let mut count_builder = String::from(
        r#"
        SELECT COUNT(*) FROM messages m
        LEFT JOIN channels c ON m.channel_id = c.id
        LEFT JOIN direct_messages dm ON m.dm_id = dm.id
        LEFT JOIN users u ON m.user_id = u.id
        WHERE (c.org_id = $1 OR dm.org_id = $1)
            AND m.deleted_at IS NULL
            AND m.encrypted_content IS NULL
        "#,
    );

    let mut bind_idx = 2;
    let mut bindings: Vec<String> = vec![];

    // Add text search filter
    if !filters.text_query.trim().is_empty() {
        let tsquery = filters
            .text_query
            .split_whitespace()
            .map(|word| format!("{}:*", word))
            .collect::<Vec<_>>()
            .join(" & ");
        query_builder.push_str(&format!(" AND m.content_tsv @@ to_tsquery('english', ${})", bind_idx));
        count_builder.push_str(&format!(" AND m.content_tsv @@ to_tsquery('english', ${})", bind_idx));
        bindings.push(tsquery);
        bind_idx += 1;
    }

    // Add from_user filter
    if let Some(ref username) = filters.from_user {
        query_builder.push_str(&format!(" AND u.username ILIKE ${}", bind_idx));
        count_builder.push_str(&format!(" AND u.username ILIKE ${}", bind_idx));
        bindings.push(username.clone());
        bind_idx += 1;
    }

    // Add in_channel filter
    if let Some(ref channel_name) = filters.in_channel {
        query_builder.push_str(&format!(" AND c.name ILIKE ${}", bind_idx));
        count_builder.push_str(&format!(" AND c.name ILIKE ${}", bind_idx));
        bindings.push(channel_name.clone());
        bind_idx += 1;
    }

    // Add before_date filter
    if let Some(before_date) = filters.before_date {
        query_builder.push_str(&format!(" AND m.created_at < ${}", bind_idx));
        count_builder.push_str(&format!(" AND m.created_at < ${}", bind_idx));
        bindings.push(before_date.to_rfc3339());
        bind_idx += 1;
    }

    // Add after_date filter
    if let Some(after_date) = filters.after_date {
        query_builder.push_str(&format!(" AND m.created_at > ${}", bind_idx));
        count_builder.push_str(&format!(" AND m.created_at > ${}", bind_idx));
        bindings.push(after_date.to_rfc3339());
        bind_idx += 1;
    }

    query_builder.push_str(&format!(" ORDER BY m.created_at DESC LIMIT ${}", bind_idx));

    // Execute search query
    let mut query = sqlx::query_as::<_, Message>(sqlx::AssertSqlSafe(query_builder.clone())).bind(org_id);
    for binding in &bindings {
        query = query.bind(binding);
    }
    query = query.bind(limit);
    let messages = query.fetch_all(pool).await?;

    // Execute count query
    let mut count_query = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(count_builder)).bind(org_id);
    for binding in &bindings {
        count_query = count_query.bind(binding);
    }
    let total_count = count_query.fetch_one(pool).await?;

    Ok((messages, total_count))
}
