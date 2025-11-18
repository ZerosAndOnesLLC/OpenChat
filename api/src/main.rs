use actix::Actor;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use rustls::{ServerConfig, pki_types::{CertificateDer, PrivateKeyDer}};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
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
mod routes;
mod services;
mod storage;
mod websocket;

use config::Config;
use handlers::{
    attachment as attachment_handlers,
    channels as channel_handlers,
    dms as dm_handlers,
    mentions as mention_handlers,
    messages as message_handlers,
    notifications as notification_handlers,
    reactions as reaction_handlers,
    read_status as read_status_handlers,
    search as search_handlers,
    users as user_handlers,
};
use middleware::auth::AuthMiddleware;
use services::tv_api::TvApiClient;
use storage::StorageFactory;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "openchat-api",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

fn load_tls_config(cert_path: &str, key_path: &str) -> std::io::Result<ServerConfig> {
    // Load certificate chain
    let cert_file = File::open(cert_path)?;
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain: Vec<CertificateDer> = certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()?;

    // Load private key
    let key_file = File::open(key_path)?;
    let mut key_reader = BufReader::new(key_file);
    let mut keys = pkcs8_private_keys(&mut key_reader)
        .collect::<Result<Vec<_>, _>>()?;

    if keys.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No private key found in key file",
        ));
    }

    let private_key = PrivateKeyDer::Pkcs8(keys.remove(0));

    // Build TLS configuration
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    Ok(config)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Install rustls crypto provider for TLS support
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

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

    // Initialize Redis connection
    info!("Connecting to Redis...");
    let redis_conn = db::init_redis(&config.redis_url)
        .await
        .expect("Failed to initialize Redis connection");
    info!("Redis connection established");

    // Start WebSocket server
    info!("Starting WebSocket server...");
    let ws_server = websocket::server::WsServer::new(db_pool.clone()).start();
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

    // Create TV API client
    let tv_api_client = Arc::new(TvApiClient::new(config.tv_api_url.clone()));

    // Create storage factory
    let local_storage_path = std::env::var("LOCAL_STORAGE_PATH")
        .unwrap_or_else(|_| "/var/openchat/uploads".to_string());
    let storage_factory = Arc::new(StorageFactory::new(db_pool.clone(), local_storage_path));
    info!("Storage factory initialized");

    // Build HTTP server
    let server = HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin("https://openchat.zerosandones.us")
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        // Create auth middleware that requires "openchat" role
        let openchat_auth = AuthMiddleware::with_openchat_role(config.tv_api_url.clone());

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(redis_conn.clone()))
            .app_data(web::Data::new(ws_server.clone()))
            .app_data(web::Data::new(tv_api_client.clone()))
            .app_data(web::Data::new(storage_factory.clone()))
            .route("/health", web::get().to(health_check))
            .route("/api/ws", web::get().to(websocket::ws_route))
            // User routes - require "openchat" role
            .service(
                web::scope("/api/users")
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(user_handlers::list_users))
                    .route("/{id}", web::get().to(user_handlers::get_user))
                    .route("/{id}", web::put().to(user_handlers::update_user))
                    .route("/{id}/status", web::put().to(user_handlers::update_user_status))
            )
            // Channel routes - require "openchat" role
            .service(
                web::scope("/api/channels")
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(channel_handlers::list_channels))
                    .route("", web::post().to(channel_handlers::create_channel))
                    .route("/{id}", web::get().to(channel_handlers::get_channel))
                    .route("/{id}", web::put().to(channel_handlers::update_channel))
                    .route("/{id}", web::delete().to(channel_handlers::delete_channel))
                    .route("/{id}/members", web::get().to(channel_handlers::list_members))
                    .route("/{id}/members", web::post().to(channel_handlers::add_member))
                    .route("/{id}/members/{user_id}", web::delete().to(channel_handlers::remove_member))
                    .route("/{id}/messages", web::get().to(message_handlers::list_channel_messages))
                    .route("/{id}/read", web::post().to(read_status_handlers::mark_channel_as_read))
                    .route("/{id}/unread", web::get().to(read_status_handlers::get_channel_unread_count))
            )
            // Message routes - require "openchat" role
            .service(
                web::scope("/api/messages")
                    .wrap(openchat_auth.clone())
                    .route("", web::post().to(message_handlers::send_message))
                    .route("/{id}", web::put().to(message_handlers::update_message))
                    .route("/{id}", web::delete().to(message_handlers::delete_message))
                    .route("/{id}/thread", web::get().to(message_handlers::get_message_thread))
                    .route("/{id}/reactions", web::post().to(reaction_handlers::add_reaction))
                    .route("/{id}/reactions", web::get().to(reaction_handlers::list_reactions))
                    .route("/{id}/reactions/counts", web::get().to(reaction_handlers::get_reaction_counts))
                    .route("/{id}/reactions/{emoji}", web::delete().to(reaction_handlers::remove_reaction))
                    .route("/{id}/attachments", web::get().to(attachment_handlers::get_message_attachments))
            )
            // Attachment routes - require "openchat" role
            .service(
                web::scope("/api/attachments")
                    .wrap(openchat_auth.clone())
                    .route("/upload", web::post().to(attachment_handlers::upload_attachment))
                    .route("/{id}/download", web::get().to(attachment_handlers::download_attachment))
                    .route("/{id}", web::delete().to(attachment_handlers::delete_attachment))
            )
            // Direct Message routes - require "openchat" role
            .service(
                web::scope("/api/dms")
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(dm_handlers::list_dms))
                    .route("", web::post().to(dm_handlers::create_dm))
                    .route("/{id}", web::get().to(dm_handlers::get_dm))
                    .route("/{id}/messages", web::get().to(dm_handlers::list_dm_messages))
                    .route("/{id}/read", web::post().to(read_status_handlers::mark_dm_as_read))
                    .route("/{id}/unread", web::get().to(read_status_handlers::get_dm_unread_count))
            )
            // Search routes - require "openchat" role
            .service(
                web::scope("/api/search")
                    .wrap(openchat_auth.clone())
                    .route("/messages", web::get().to(search_handlers::search_messages))
            )
            // Mention routes - require "openchat" role
            .service(
                web::scope("/api/mentions")
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(mention_handlers::list_mentions))
                    .route("/unread-count", web::get().to(mention_handlers::get_unread_mention_count))
            )
            // Notification routes - require "openchat" role
            .service(
                web::scope("/api/notifications")
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(notification_handlers::list_notifications))
                    .route("/unread-count", web::get().to(notification_handlers::get_unread_count))
                    .route("/{id}/read", web::post().to(notification_handlers::mark_notification_as_read))
                    .route("/read-all", web::post().to(notification_handlers::mark_all_notifications_as_read))
            )
            // SSO routes - no auth required (they handle authentication themselves)
            .configure(routes::sso::configure)
    });

    // Bind server with or without TLS
    if config.enable_tls {
        let cert_path = config.tls_cert_path.as_ref()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "TLS_CERT_PATH must be set when ENABLE_TLS is true"
            ))?;

        let key_path = config.tls_key_path.as_ref()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "TLS_KEY_PATH must be set when ENABLE_TLS is true"
            ))?;

        info!("Loading TLS configuration from cert: {}, key: {}", cert_path, key_path);
        let tls_config = load_tls_config(cert_path, key_path)?;

        info!("Starting OpenChat API server with TLS on {}:{}", config.host, config.port);
        server
            .bind_rustls_0_23((config.host.as_str(), config.port), tls_config)?
            .run()
            .await
    } else {
        info!("Starting OpenChat API server (HTTP only) on {}:{}", config.host, config.port);
        server
            .bind((config.host.as_str(), config.port))?
            .run()
            .await
    }
}
