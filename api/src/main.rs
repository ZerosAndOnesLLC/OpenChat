use actix_web::{web, App, HttpResponse, HttpServer, Responder};
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

use config::Config;
use handlers::{channels as channel_handlers, users as user_handlers};

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

    info!("Starting OpenChat API server on {}:{}", config.host, config.port);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .route("/health", web::get().to(health_check))
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
            )
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}
