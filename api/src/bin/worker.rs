use sqlx::migrate::Migrator;
use std::path::Path;
use std::sync::Arc;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

use openchat_api::config::Config;
use openchat_api::db;
use openchat_api::models::job::JobType;
use openchat_api::models::reminder::Reminder;
use openchat_api::models::scheduled_message::ScheduledMessage;
use openchat_api::storage::StorageFactory;
use openchat_api::tasks::job_queue::{self, JobQueue};

const RETENTION_CHECK_INTERVAL_SECS: u64 = 60;
const RETENTION_LAST_RUN_KEY: &str = "openchat:retention:last_run";

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

    // Initialize storage factory
    let local_storage_path = std::env::var("LOCAL_STORAGE_PATH")
        .unwrap_or_else(|_| "/var/openchat/uploads".to_string());
    let storage_factory = Arc::new(StorageFactory::new(db_pool.clone(), local_storage_path));
    info!("Storage factory initialized");

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

    // Start retention scheduler loop
    let retention_pool = db_pool.clone();
    let mut retention_redis = redis_conn.clone();
    let retention_shutdown = shutdown_token.clone();

    let retention_handle = tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(RETENTION_CHECK_INTERVAL_SECS));
        loop {
            interval.tick().await;
            if retention_shutdown.is_cancelled() {
                break;
            }
            schedule_retention_jobs(&retention_pool, &mut retention_redis).await;
        }
    });

    // Start due-item poller for scheduled messages and reminders
    let due_pool = db_pool.clone();
    let mut due_redis = redis_conn.clone();
    let due_shutdown = shutdown_token.clone();

    let due_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if due_shutdown.is_cancelled() {
                break;
            }
            poll_due_items(&due_pool, &mut due_redis).await;
        }
    });

    // Create and run the job queue worker
    let job_queue = JobQueue::new(db_pool, redis_conn, storage_factory);
    job_queue.run(shutdown_token.clone()).await;

    // Wait for background tasks to finish
    let _ = poller_handle.await;
    let _ = retention_handle.await;
    let _ = due_handle.await;

    // Grace period for in-flight work
    info!("Waiting for in-flight jobs to complete (5s grace period)...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    info!("OpenChat Worker stopped");
    Ok(())
}

/// Poll for due scheduled messages and reminders, enqueue jobs for each.
async fn poll_due_items(
    pool: &sqlx::PgPool,
    redis: &mut redis::aio::MultiplexedConnection,
) {
    // Poll due scheduled messages
    match ScheduledMessage::list_due(pool).await {
        Ok(items) => {
            for sm in &items {
                let payload = serde_json::json!({
                    "scheduled_message_id": sm.id.to_string(),
                });
                if let Err(e) = job_queue::enqueue_job(
                    pool,
                    redis,
                    Some(sm.org_id),
                    JobType::ScheduledMessage,
                    payload,
                    None,
                )
                .await
                {
                    error!(id = %sm.id, "Failed to enqueue scheduled message job: {}", e);
                }
            }
            if !items.is_empty() {
                info!(count = items.len(), "Enqueued due scheduled messages");
            }
        }
        Err(e) => {
            error!("Failed to poll due scheduled messages: {}", e);
        }
    }

    // Poll due reminders
    match Reminder::list_due(pool).await {
        Ok(items) => {
            for r in &items {
                let payload = serde_json::json!({
                    "reminder_id": r.id.to_string(),
                });
                if let Err(e) = job_queue::enqueue_job(
                    pool,
                    redis,
                    Some(r.org_id),
                    JobType::Reminder,
                    payload,
                    None,
                )
                .await
                {
                    error!(id = %r.id, "Failed to enqueue reminder job: {}", e);
                }
            }
            if !items.is_empty() {
                info!(count = items.len(), "Enqueued due reminders");
            }
        }
        Err(e) => {
            error!("Failed to poll due reminders: {}", e);
        }
    }
}

/// Check if retention has been run today; if not, enqueue retention jobs for all orgs with policies.
async fn schedule_retention_jobs(
    pool: &sqlx::PgPool,
    redis: &mut redis::aio::MultiplexedConnection,
) {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Check if we've already run today
    let last_run: Result<Option<String>, redis::RedisError> = redis::cmd("GET")
        .arg(RETENTION_LAST_RUN_KEY)
        .query_async(redis)
        .await;

    match last_run {
        Ok(Some(date)) if date == today => return,
        Ok(_) => {}
        Err(e) => {
            error!("Failed to check retention last run: {}", e);
            return;
        }
    }

    info!("Scheduling retention enforcement jobs");

    // Find orgs with enabled retention policies
    let org_ids: Vec<(Uuid,)> = match sqlx::query_as(
        "SELECT DISTINCT org_id FROM retention_policies WHERE enabled = true",
    )
    .fetch_all(pool)
    .await
    {
        Ok(ids) => ids,
        Err(e) => {
            error!("Failed to query orgs with retention policies: {}", e);
            return;
        }
    };

    for (org_id,) in &org_ids {
        if let Err(e) = job_queue::enqueue_job(
            pool,
            redis,
            Some(*org_id),
            JobType::RetentionEnforcement,
            serde_json::json!({}),
            None,
        )
        .await
        {
            error!(org_id = %org_id, "Failed to enqueue retention job: {}", e);
        }
    }

    info!(count = org_ids.len(), "Retention jobs enqueued");

    // Set last run date with 25h TTL
    let _: Result<(), redis::RedisError> = redis::cmd("SET")
        .arg(RETENTION_LAST_RUN_KEY)
        .arg(&today)
        .arg("EX")
        .arg(25 * 3600)
        .query_async(redis)
        .await;
}
