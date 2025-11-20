pub mod compression;
pub mod messages;
pub mod pubsub;
pub mod server;
pub mod session;

use actix::Addr;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use sqlx::PgPool;
use std::sync::Arc;

use crate::{errors::ApiError, models::user::User, services::tv_api::TvApiClient};
use server::WsServer;
use session::{handle_ws_session, WsSessionData};

/// WebSocket connection handler
pub async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<WsServer>>,
    pool: web::Data<PgPool>,
    tv_api_client: web::Data<Arc<TvApiClient>>,
) -> Result<HttpResponse, Error> {
    // Extract token from query parameter
    let query = req.query_string();
    let token = query
        .split('&')
        .find_map(|param| {
            let mut parts = param.split('=');
            if parts.next() == Some("token") {
                parts.next()
            } else {
                None
            }
        })
        .ok_or_else(|| ApiError::Authentication("Missing token parameter".to_string()))?;

    // Verify token with tv-api
    let claims = tv_api_client.verify_token(token)
        .await
        .map_err(|_| ApiError::Authentication("Invalid token".to_string()))?;

    // Get user from database
    let user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await
        .map_err(|_| ApiError::Internal("Database error".to_string()))?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    println!(
        "WebSocket: Initializing connection for user {} ({})",
        user.id, user.display_name
    );

    // Create WebSocket session data
    let session_data = WsSessionData::new(
        user.id,
        claims.org_id,
        user.display_name,
        server.get_ref().clone(),
        pool.get_ref().clone(),
    );

    // Upgrade connection to WebSocket
    let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // Spawn task to handle the WebSocket session (use actix runtime since it's !Send)
    actix_web::rt::spawn(handle_ws_session(session, msg_stream, session_data));

    Ok(response)
}
