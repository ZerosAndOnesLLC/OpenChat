use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::errors::{ApiError, ApiResult};
use crate::services::tv_api::TokenClaims;

#[derive(Debug, Deserialize)]
pub struct LinkPreviewRequest {
    url: String,
}

#[derive(Debug, Serialize)]
pub struct LinkPreviewResponse {
    url: String,
    title: Option<String>,
    description: Option<String>,
    image: Option<String>,
    site_name: Option<String>,
}

/// GET /api/links/preview?url={url}
/// Fetches metadata for a URL (Open Graph tags)
pub async fn get_link_preview(
    query: web::Query<LinkPreviewRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // Verify authentication
    let _claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;
    let url = &query.url;

    // Validate URL format
    if !is_valid_url(url) {
        return Err(ApiError::BadRequest("Invalid URL format".to_string()));
    }

    // Fetch the URL content with timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("OpenChat-Bot/1.0")
        .build()
        .map_err(|e| ApiError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to fetch URL: {}", e)))?;

    // Check if response is successful
    if !response.status().is_success() {
        return Err(ApiError::BadRequest(format!("Failed to fetch URL: HTTP {}", response.status())));
    }

    // Get content type to ensure it's HTML
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !content_type.contains("text/html") {
        return Ok(HttpResponse::Ok().json(LinkPreviewResponse {
            url: url.clone(),
            title: None,
            description: None,
            image: None,
            site_name: None,
        }));
    }

    // Read response body
    let html = response
        .text()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to read response: {}", e)))?;

    // Extract metadata using regex
    let title = extract_meta_tag(&html, "og:title")
        .or_else(|| extract_title_tag(&html));
    let description = extract_meta_tag(&html, "og:description")
        .or_else(|| extract_meta_tag(&html, "description"));
    let image = extract_meta_tag(&html, "og:image");
    let site_name = extract_meta_tag(&html, "og:site_name");

    Ok(HttpResponse::Ok().json(LinkPreviewResponse {
        url: url.clone(),
        title,
        description,
        image,
        site_name,
    }))
}

fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn extract_meta_tag(html: &str, property: &str) -> Option<String> {
    // Try Open Graph property format: <meta property="og:title" content="..." />
    let og_pattern = format!(
        r#"<meta\s+property=["']{}["']\s+content=["']([^"']+)["']"#,
        regex::escape(property)
    );
    if let Ok(re) = Regex::new(&og_pattern) {
        if let Some(cap) = re.captures(html) {
            return Some(cap[1].to_string());
        }
    }

    // Try standard name format: <meta name="description" content="..." />
    let name_pattern = format!(
        r#"<meta\s+name=["']{}["']\s+content=["']([^"']+)["']"#,
        regex::escape(property)
    );
    if let Ok(re) = Regex::new(&name_pattern) {
        if let Some(cap) = re.captures(html) {
            return Some(cap[1].to_string());
        }
    }

    // Try alternate format: content before property/name
    let alt_og_pattern = format!(
        r#"<meta\s+content=["']([^"']+)["']\s+property=["']{}["']"#,
        regex::escape(property)
    );
    if let Ok(re) = Regex::new(&alt_og_pattern) {
        if let Some(cap) = re.captures(html) {
            return Some(cap[1].to_string());
        }
    }

    None
}

fn extract_title_tag(html: &str) -> Option<String> {
    let pattern = r#"<title>([^<]+)</title>"#;
    if let Ok(re) = Regex::new(pattern) {
        if let Some(cap) = re.captures(html) {
            return Some(cap[1].trim().to_string());
        }
    }
    None
}
