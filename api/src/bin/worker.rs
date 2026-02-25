use sqlx::migrate::Migrator;
use std::path::Path;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::info;
use tracing_subscriber::EnvFilter;

use openchat_api::config::Config;
use openchat_api::db;
use openchat_api::tasks::job_queue::JobQueue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize logging
    let show_timestamps = std::env::var("LOG_TIMESTAMPS")
        .map(|v| v != "false")
        .unwrap_or(true);

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        );

    if show_timestamps {
        subscriber.init();
    } else {
        subscriber.without_time().init();
    }

    info!("Starting OpenChat Worker");

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");

    // Initialize database pool (fewer connections for worker)
    info!("Connecting to database...");
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    info!("Database connection established");

    // Run migrations
    info!("Running database migrations...");
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(&db_pool).await?;
    info!("Migrations complete");

    // Initialize Redis
    info!("Connecting to Redis...");
    let redis_client = db::init_redis_client(&config.redis_url)?;
    let redis_conn = redis_client.get_multiplexed_async_connection().await?;
    info!("Redis connection established");

    // Set up graceful shutdown
    let shutdown_token = CancellationToken::new();
    let shutdown_token_signal = shutdown_token.clone();

    tokio::spawn(async move {
        let ctrl_c = signal::ctrl_c();
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to register SIGTERM handler");

        tokio::select! {
            _ = ctrl_c => info!("Received SIGINT"),
            _ = sigterm.recv() => info!("Received SIGTERM"),
        }

        info!("Initiating graceful shutdown...");
        shutdown_token_signal.cancel();
    });

    // Start scheduled job poller
    let poller_pool = db_pool.clone();
    let mut poller_redis = redis_conn.clone();
    let poller_shutdown = shutdown_token.clone();

    let poller_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if poller_shutdown.is_cancelled() {
                break;
            }
            JobQueue::poll_scheduled_jobs(&poller_pool, &mut poller_redis).await;
        }
    });

    // Create and run the job queue worker
    let job_queue = JobQueue::new(db_pool, redis_conn);
    job_queue.run(shutdown_token.clone()).await;

    // Wait for poller to finish
    let _ = poller_handle.await;

    // Grace period for in-flight work
    info!("Waiting for in-flight jobs to complete (5s grace period)...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    info!("OpenChat Worker stopped");
    Ok(())
}
