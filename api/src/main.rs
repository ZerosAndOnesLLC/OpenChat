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

use openchat_api::*;

use config::Config;
use handlers::{
    attachment as attachment_handlers,
    audit_logs as audit_log_handlers,
    bookmarks as bookmark_handlers,
    channel_sections as channel_section_handlers,
    channels as channel_handlers,
    device_auth as device_auth_handlers,
    dms as dm_handlers,
    drafts as draft_handlers,
    emoji as emoji_handlers,
    link_preview as link_preview_handlers,
    mentions as mention_handlers,
    messages as message_handlers,
    metrics as metrics_handlers,
    migration as migration_handlers,
    notifications as notification_handlers,
    pins as pin_handlers,
    reactions as reaction_handlers,
    read_receipts as read_receipt_handlers,
    read_status as read_status_handlers,
    retention as retention_handlers,
    roles as role_handlers,
    search as search_handlers,
    storage_settings as storage_settings_handlers,
    user_status as user_status_handlers,
    users as user_handlers,
    webhooks as webhook_handlers,
    outgoing_webhooks as outgoing_webhook_handlers,
    reminders as reminder_handlers,
    scheduled_messages as scheduled_message_handlers,
    user_groups as user_group_handlers,
};
use middleware::auth::AuthMiddleware;
use middleware::permissions::PermissionMiddleware;
use middleware::rate_limit::RateLimitMiddleware;
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
    // Disable timestamps in production (CloudWatch adds its own)
    let show_timestamps = std::env::var("LOG_TIMESTAMPS")
        .map(|v| v != "false")
        .unwrap_or(true);

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
        );

    if show_timestamps {
        subscriber.init();
    } else {
        subscriber.without_time().init();
    }

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");

    // Initialize database pool
    info!("Connecting to database...");
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to initialize database pool");
    info!("Database connection established");

    // Initialize Redis pool (with automatic reconnection via deadpool)
    info!("Connecting to Redis...");
    let redis_pool = db::init_redis_pool(&config.redis_url)
        .expect("Failed to initialize Redis pool");
    // Also keep a client for pub/sub which needs a dedicated connection
    let redis_client = db::init_redis_client(&config.redis_url)
        .expect("Failed to initialize Redis client");
    info!("Redis pool established");

    // Warm the cache with frequently accessed data
    info!("Warming cache...");
    if let Err(e) = cache::warming::warm_cache(&db_pool, &redis_pool).await {
        tracing::warn!("Cache warming failed (non-critical): {}", e);
    }

    // Start WebSocket server
    info!("Starting WebSocket server...");
    let ws_config = Arc::new(config.websocket.clone());
    let ws_server = websocket::server::WsServer::new(
        db_pool.clone(),
        redis_pool.clone(),
        ws_config.clone()
    ).start();
    info!(
        "WebSocket server started (max_connections: {}, max_per_user: {})",
        ws_config.max_connections,
        ws_config.max_connections_per_user
    );

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

    // Start background tasks
    info!("Starting background tasks...");
    tasks::start_background_tasks(db_pool.clone(), ws_server.clone());
    info!("Background tasks started");

    // Clone config values needed after the closure
    let server_host = config.host.clone();
    let server_port = config.port;
    let enable_tls = config.enable_tls;
    let tls_cert_path = config.tls_cert_path.clone();
    let tls_key_path = config.tls_key_path.clone();

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

        // Create rate limiting middleware
        let api_rate_limit = RateLimitMiddleware::api_request(config.enable_rate_limiting);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(redis_pool.clone()))
            .app_data(web::Data::new(redis_client.clone()))
            .app_data(web::Data::new(ws_server.clone()))
            .app_data(web::Data::new(tv_api_client.clone()))
            .app_data(web::Data::new(storage_factory.clone()))
            .app_data(web::Data::new(config.clone()))
            .route("/health", web::get().to(health_check))
            .route("/api/ws", web::get().to(websocket::ws_route))
            // User routes - require "openchat" role
            .service(
                web::scope("/api/users")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    // Static routes must come before parameterized routes
                    .route("/status/active", web::get().to(user_status_handlers::get_active_users))
                    .route("/me/status", web::put().to(user_status_handlers::update_my_status))
                    .route("/me/status/online", web::post().to(user_status_handlers::set_online))
                    .route("/me/status/away", web::post().to(user_status_handlers::set_away))
                    .route("/me/status/offline", web::post().to(user_status_handlers::set_offline))
                    // Parameterized routes after static routes
                    .route("", web::get().to(user_handlers::list_users))
                    .route("/{id}", web::get().to(user_handlers::get_user))
                    .route("/{id}", web::put().to(user_handlers::update_user))
                    .route("/{id}/status", web::get().to(user_status_handlers::get_user_status))
                    .route("/{id}/status", web::put().to(user_handlers::update_user_status))
            )
            // Channel routes - require "openchat" role
            .service(
                web::scope("/api/channels")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(channel_handlers::list_channels))
                    .route("", web::post().to(channel_handlers::create_channel))
                    .route("/public", web::get().to(channel_handlers::list_public_channels))
                    .route("/{id}", web::get().to(channel_handlers::get_channel))
                    .route("/{id}", web::put().to(channel_handlers::update_channel))
                    .route("/{id}", web::delete().to(channel_handlers::delete_channel))
                    .route("/{id}/join", web::post().to(channel_handlers::join_channel))
                    .route("/{id}/leave", web::post().to(channel_handlers::leave_channel))
                    .route("/{id}/members", web::get().to(channel_handlers::list_members))
                    .route("/{id}/members", web::post().to(channel_handlers::add_member))
                    .route("/{id}/members/{user_id}", web::delete().to(channel_handlers::remove_member))
                    .route("/{id}/messages", web::get().to(message_handlers::list_channel_messages))
                    .route("/{id}/read", web::post().to(read_status_handlers::mark_channel_as_read))
                    .route("/{id}/unread", web::get().to(read_status_handlers::get_channel_unread_count))
                    .route("/{id}/pins", web::get().to(pin_handlers::list_channel_pins))
                    .route("/{id}/legal-hold", web::post().to(retention_handlers::create_legal_hold))
                    .route("/{id}/legal-hold", web::get().to(retention_handlers::get_legal_hold))
                    .route("/{id}/legal-hold", web::delete().to(retention_handlers::disable_legal_hold))
            )
            // Message routes - require "openchat" role
            .service(
                web::scope("/api/messages")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("/scheduled", web::post().to(scheduled_message_handlers::create_scheduled_message))
                    .route("/scheduled", web::get().to(scheduled_message_handlers::list_scheduled_messages))
                    .route("/scheduled/{id}", web::put().to(scheduled_message_handlers::update_scheduled_message))
                    .route("/scheduled/{id}", web::delete().to(scheduled_message_handlers::delete_scheduled_message))
                    .route("", web::post().to(message_handlers::send_message))
                    .route("/{id}", web::put().to(message_handlers::update_message))
                    .route("/{id}", web::delete().to(message_handlers::delete_message))
                    .route("/{id}/thread", web::get().to(message_handlers::get_message_thread))
                    .route("/{id}/history", web::get().to(message_handlers::get_message_history))
                    .route("/{id}/reactions", web::post().to(reaction_handlers::add_reaction))
                    .route("/{id}/reactions", web::get().to(reaction_handlers::list_reactions))
                    .route("/{id}/reactions/counts", web::get().to(reaction_handlers::get_reaction_counts))
                    .route("/{id}/reactions/{emoji}", web::delete().to(reaction_handlers::remove_reaction))
                    .route("/{id}/attachments", web::get().to(attachment_handlers::get_message_attachments))
                    .route("/{id}/pin", web::post().to(pin_handlers::pin_message))
                    .route("/{id}/pin", web::delete().to(pin_handlers::unpin_message))
                    .route("/{id}/read", web::post().to(read_receipt_handlers::record_read_receipt))
                    .route("/{id}/receipts", web::get().to(read_receipt_handlers::get_message_receipts))
            )
            // Reminder routes - require "openchat" role
            .service(
                web::scope("/api/reminders")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("", web::post().to(reminder_handlers::create_reminder))
                    .route("", web::get().to(reminder_handlers::list_reminders))
                    .route("/{id}", web::delete().to(reminder_handlers::delete_reminder))
            )
            // Read Receipt routes - require "openchat" role
            .service(
                web::scope("/api/read-receipts")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("/batch", web::post().to(read_receipt_handlers::record_batch_read_receipts))
            )
            // Attachment routes - require "openchat" role
            .service(
                web::scope("/api/attachments")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("/upload", web::post().to(attachment_handlers::upload_attachment))
                    .route("/{id}/download", web::get().to(attachment_handlers::download_attachment))
                    .route("/{id}", web::delete().to(attachment_handlers::delete_attachment))
            )
            // Direct Message routes - require "openchat" role
            .service(
                web::scope("/api/dms")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(dm_handlers::list_dms))
                    .route("", web::post().to(dm_handlers::create_dm))
                    .route("/{id}", web::get().to(dm_handlers::get_dm))
                    .route("/{id}/messages", web::get().to(dm_handlers::list_dm_messages))
                    .route("/{id}/read", web::post().to(read_status_handlers::mark_dm_as_read))
                    .route("/{id}/unread", web::get().to(read_status_handlers::get_dm_unread_count))
                    .route("/{id}/hide", web::post().to(dm_handlers::hide_dm))
            )
            // Search routes - require "openchat" role
            .service(
                web::scope("/api/search")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("/messages", web::get().to(search_handlers::search_messages))
            )
            // Storage settings routes - require "openchat" role and admin permissions
            .service(
                web::scope("/api/settings/storage")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(storage_settings_handlers::get_storage_settings))
                    .route("", web::post().to(storage_settings_handlers::update_storage_settings))
            )
            // Retention policy routes - require "openchat" role and admin permissions
            .service(
                web::scope("/api/settings/retention")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(retention_handlers::get_retention_policies))
                    .route("", web::post().to(retention_handlers::update_retention_policy))
            )
            // Mention routes - require "openchat" role
            .service(
                web::scope("/api/mentions")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(mention_handlers::list_mentions))
                    .route("/unread-count", web::get().to(mention_handlers::get_unread_mention_count))
            )
            // Notification routes - require "openchat" role
            .service(
                web::scope("/api/notifications")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(notification_handlers::list_notifications))
                    .route("/unread-count", web::get().to(notification_handlers::get_unread_count))
                    .route("/{id}/read", web::post().to(notification_handlers::mark_notification_as_read))
                    .route("/read-all", web::post().to(notification_handlers::mark_all_notifications_as_read))
            )
            // Bookmark routes - require "openchat" role
            .service(
                web::scope("/api/bookmarks")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(bookmark_handlers::list_bookmarks))
                    .route("", web::post().to(bookmark_handlers::create_bookmark))
                    .route("/{message_id}", web::delete().to(bookmark_handlers::delete_bookmark))
            )
            // Channel Section routes - require "openchat" role
            .service(
                web::scope("/api/channel-sections")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("/reorder", web::put().to(channel_section_handlers::reorder_sections))
                    .route("", web::get().to(channel_section_handlers::list_sections))
                    .route("", web::post().to(channel_section_handlers::create_section))
                    .route("/{id}", web::put().to(channel_section_handlers::update_section))
                    .route("/{id}", web::delete().to(channel_section_handlers::delete_section))
                    .route("/{id}/channels", web::post().to(channel_section_handlers::add_channel))
                    .route("/{id}/channels/{channel_id}", web::delete().to(channel_section_handlers::remove_channel))
                    .route("/{id}/reorder", web::put().to(channel_section_handlers::reorder_items))
            )
            // User Group routes - require "openchat" role
            .service(
                web::scope("/api/user-groups")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(user_group_handlers::list_groups))
                    .route("", web::post().to(user_group_handlers::create_group))
                    .route("/{id}", web::get().to(user_group_handlers::get_group))
                    .route("/{id}", web::put().to(user_group_handlers::update_group))
                    .route("/{id}", web::delete().to(user_group_handlers::delete_group))
                    .route("/{id}/members", web::get().to(user_group_handlers::list_members))
                    .route("/{id}/members", web::post().to(user_group_handlers::add_member))
                    .route("/{id}/members/{user_id}", web::delete().to(user_group_handlers::remove_member))
            )
            // Draft routes - require "openchat" role
            .service(
                web::scope("/api/drafts")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("", web::post().to(draft_handlers::save_draft))
                    .route("", web::get().to(draft_handlers::get_all_drafts))
                    .route("", web::delete().to(draft_handlers::delete_all_drafts))
                    .route("/channel/{channel_id}", web::get().to(draft_handlers::get_channel_draft))
                    .route("/channel/{channel_id}", web::delete().to(draft_handlers::delete_channel_draft))
                    .route("/dm/{dm_id}", web::get().to(draft_handlers::get_dm_draft))
                    .route("/dm/{dm_id}", web::delete().to(draft_handlers::delete_dm_draft))
            )
            // Link Preview routes - require "openchat" role
            .service(
                web::scope("/api/links")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("/preview", web::get().to(link_preview_handlers::get_link_preview))
            )
            // Role and Permission routes - require "openchat" role
            .service(
                web::scope("/api/roles")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(role_handlers::list_roles))
                    .route("", web::post().to(role_handlers::create_role))
                    .route("/{id}", web::get().to(role_handlers::get_role))
                    .route("/{id}", web::put().to(role_handlers::update_role))
                    .route("/{id}", web::delete().to(role_handlers::delete_role))
                    .route("/{id}/permissions", web::post().to(role_handlers::assign_permissions))
            )
            .service(
                web::scope("/api/permissions")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(role_handlers::list_permissions))
            )
            // Custom Emoji routes - require "openchat" role
            .service(
                web::scope("/api/emojis")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("/upload", web::post().to(emoji_handlers::upload_emoji))
                    .route("", web::get().to(emoji_handlers::get_org_emojis))
                    .route("/{id}", web::delete().to(emoji_handlers::delete_emoji))
                    .route("/{id}/image", web::get().to(emoji_handlers::get_emoji_image))
            )
            // Audit Log routes - require "openchat" role and "org.view_audit_logs" permission
            .service(
                web::scope("/api/audit-logs")
                    .wrap(api_rate_limit.clone())
                    .wrap(PermissionMiddleware::require("org.view_audit_logs"))
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(audit_log_handlers::list_audit_logs))
                    .route("/export", web::get().to(audit_log_handlers::export_audit_logs))
                    .route("/actions", web::get().to(audit_log_handlers::list_actions))
                    .route("/resource-types", web::get().to(audit_log_handlers::list_resource_types))
            )
            // Metrics routes - require "openchat" role (admin should have additional checks in production)
            .service(
                web::scope("/api/metrics")
                    .wrap(api_rate_limit.clone())
                    .wrap(openchat_auth.clone())
                    .route("/cache", web::get().to(metrics_handlers::get_cache_metrics))
                    .route("/cache/reset", web::post().to(metrics_handlers::reset_cache_metrics))
                    .route("/websocket", web::get().to(metrics_handlers::get_websocket_metrics))
            )
            // Device authentication routes
            .service(
                web::scope("/api/auth/device")
                    .wrap(api_rate_limit.clone())
                    // Generate code requires authentication
                    .route("/generate-code", web::post().to(device_auth_handlers::generate_code).wrap(openchat_auth.clone()))
                    // Generate deep link requires authentication
                    .route("/generate-deep-link", web::post().to(device_auth_handlers::generate_deep_link).wrap(openchat_auth.clone()))
                    // Get and delete sessions require authentication
                    .route("/sessions", web::get().to(device_auth_handlers::get_sessions).wrap(openchat_auth.clone()))
                    .route("/sessions/{id}", web::delete().to(device_auth_handlers::revoke_session).wrap(openchat_auth.clone()))
                    // Verify code is public (no auth required)
                    .route("/verify-code", web::post().to(device_auth_handlers::verify_code))
            )
            // Incoming webhooks management - require "openchat" role and "org.manage_integrations" permission
            .service(
                web::scope("/api/webhooks/incoming")
                    .wrap(api_rate_limit.clone())
                    .wrap(PermissionMiddleware::require("org.manage_integrations"))
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(webhook_handlers::list_webhooks))
                    .route("", web::post().to(webhook_handlers::create_webhook))
                    .route("/{id}", web::get().to(webhook_handlers::get_webhook))
                    .route("/{id}", web::put().to(webhook_handlers::update_webhook))
                    .route("/{id}", web::delete().to(webhook_handlers::delete_webhook))
                    .route("/{id}/regenerate", web::post().to(webhook_handlers::regenerate_token))
            )
            // Outgoing webhooks management
            .service(
                web::scope("/api/webhooks/outgoing")
                    .wrap(api_rate_limit.clone())
                    .wrap(PermissionMiddleware::require("org.manage_integrations"))
                    .wrap(openchat_auth.clone())
                    .route("", web::get().to(outgoing_webhook_handlers::list_webhooks))
                    .route("", web::post().to(outgoing_webhook_handlers::create_webhook))
                    .route("/{id}", web::get().to(outgoing_webhook_handlers::get_webhook))
                    .route("/{id}", web::put().to(outgoing_webhook_handlers::update_webhook))
                    .route("/{id}", web::delete().to(outgoing_webhook_handlers::delete_webhook))
                    .route("/{id}/deliveries", web::get().to(outgoing_webhook_handlers::list_deliveries))
            )
            // Public webhook receiver endpoint - no auth (uses token in URL)
            .service(
                web::scope("/api/hooks")
                    .wrap(api_rate_limit.clone())
                    .route("/{token}", web::post().to(webhook_handlers::receive_webhook))
            )
            // Migration routes - require "openchat-admin" role and admin permissions
            .service(
                web::scope("/api/settings/import/mattermost")
                    .wrap(api_rate_limit.clone())
                    .wrap(PermissionMiddleware::require("org.manage_integrations"))
                    .wrap(openchat_auth.clone())
                    .route("/validate", web::post().to(migration_handlers::validate_connection))
                    .route("/preview", web::post().to(migration_handlers::get_preview))
                    .route("/start", web::post().to(migration_handlers::start_migration))
                    .route("/jobs", web::get().to(migration_handlers::list_jobs))
                    .route("/jobs/{id}", web::get().to(migration_handlers::get_job_status))
                    .route("/jobs/{id}/cancel", web::post().to(migration_handlers::cancel_job))
            )
            // SSO routes - no auth required (they handle authentication themselves)
            .configure(routes::sso::configure)
    });

    // Bind server with or without TLS
    if enable_tls {
        let cert_path = tls_cert_path.as_ref()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "TLS_CERT_PATH must be set when ENABLE_TLS is true"
            ))?;

        let key_path = tls_key_path.as_ref()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "TLS_KEY_PATH must be set when ENABLE_TLS is true"
            ))?;

        info!("Loading TLS configuration from cert: {}, key: {}", cert_path, key_path);
        let tls_config = load_tls_config(cert_path, key_path)?;

        info!("Starting OpenChat API server with TLS on {}:{}", server_host, server_port);
        server
            .bind_rustls_0_23((server_host.as_str(), server_port), tls_config)?
            .run()
            .await
    } else {
        info!("Starting OpenChat API server (HTTP only) on {}:{}", server_host, server_port);
        server
            .bind((server_host.as_str(), server_port))?
            .run()
            .await
    }
}
