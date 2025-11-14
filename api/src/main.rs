use actix::Actor;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod cache;
mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod services;
mod websocket;

use config::Config;
use handlers::{
    channels as channel_handlers,
    dms as dm_handlers,
    messages as message_handlers,
    reactions as reaction_handlers,
    users as user_handlers,
};
use services::tv_api::TvApiClient;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "openchat-api",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");

    // Initialize database pool
    info!("Connecting to database...");
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to initialize database pool");
    info!("Database connection established");

    // Start WebSocket server
    info!("Starting WebSocket server...");
    let ws_server = websocket::server::WsServer::new().start();
    info!("WebSocket server started");

    // Start Redis Pub/Sub for WebSocket scaling
    info!("Starting Redis Pub/Sub for WebSocket scaling...");
    match websocket::pubsub::RedisPubSub::new(&config.redis_url, ws_server.clone()) {
        Ok(redis_pubsub) => {
            redis_pubsub.start();
            info!("Redis Pub/Sub started - WebSocket scaling enabled");
        }
        Err(e) => {
            tracing::warn!("Failed to start Redis Pub/Sub: {}. WebSocket will work in single-instance mode only.", e);
        }
    }

    info!("Starting OpenChat API server on {}:{}", config.host, config.port);

    // Create TV API client
    let tv_api_client = Arc::new(TvApiClient::new(config.tv_api_url.clone()));

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(ws_server.clone()))
            .app_data(web::Data::new(tv_api_client.clone()))
            .route("/health", web::get().to(health_check))
            .route("/api/ws", web::get().to(websocket::ws_route))
            // User routes
            .service(
                web::scope("/api/users")
                    .route("", web::get().to(user_handlers::list_users))
                    .route("/{id}", web::get().to(user_handlers::get_user))
                    .route("/{id}", web::put().to(user_handlers::update_user))
                    .route("/{id}/status", web::put().to(user_handlers::update_user_status))
            )
            // Channel routes
            .service(
                web::scope("/api/channels")
                    .route("", web::get().to(channel_handlers::list_channels))
                    .route("", web::post().to(channel_handlers::create_channel))
                    .route("/{id}", web::get().to(channel_handlers::get_channel))
                    .route("/{id}", web::put().to(channel_handlers::update_channel))
                    .route("/{id}", web::delete().to(channel_handlers::delete_channel))
                    .route("/{id}/members", web::get().to(channel_handlers::list_members))
                    .route("/{id}/members", web::post().to(channel_handlers::add_member))
                    .route("/{id}/members/{user_id}", web::delete().to(channel_handlers::remove_member))
                    .route("/{id}/messages", web::get().to(message_handlers::list_channel_messages))
            )
            // Message routes
            .service(
                web::scope("/api/messages")
                    .route("", web::post().to(message_handlers::send_message))
                    .route("/{id}", web::put().to(message_handlers::update_message))
                    .route("/{id}", web::delete().to(message_handlers::delete_message))
                    .route("/{id}/reactions", web::post().to(reaction_handlers::add_reaction))
                    .route("/{id}/reactions", web::get().to(reaction_handlers::list_reactions))
                    .route("/{id}/reactions/counts", web::get().to(reaction_handlers::get_reaction_counts))
                    .route("/{id}/reactions/{emoji}", web::delete().to(reaction_handlers::remove_reaction))
            )
            // Direct Message routes
            .service(
                web::scope("/api/dms")
                    .route("", web::get().to(dm_handlers::list_dms))
                    .route("", web::post().to(dm_handlers::create_dm))
                    .route("/{id}", web::get().to(dm_handlers::get_dm))
                    .route("/{id}/messages", web::get().to(dm_handlers::list_dm_messages))
            )
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}
