use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod config;
mod db;
mod errors;

use config::Config;

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
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}
