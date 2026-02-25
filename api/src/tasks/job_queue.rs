use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::models::job::{Job, JobType};
use crate::storage::StorageFactory;
use super::{reminder_worker, retention_worker, scheduled_message_worker, webhook_worker};

const STREAM_KEY: &str = "openchat:jobs:stream";
const GROUP_NAME: &str = "openchat-workers";
const STALE_JOB_MINUTES: i64 = 5;

/// Enqueue a job into the job queue.
/// Inserts a row into Postgres and pushes the job_id to Redis Streams for instant pickup.
pub async fn enqueue_job(
    pool: &PgPool,
    redis: &mut redis::aio::MultiplexedConnection,
    org_id: Option<Uuid>,
    job_type: JobType,
    payload: serde_json::Value,
    scheduled_at: Option<chrono::DateTime<Utc>>,
) -> Result<Uuid, crate::errors::ApiError> {
    let scheduled = scheduled_at.unwrap_or_else(Utc::now);
    let job_type_str = job_type.to_string();

    let row: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO job_queue (org_id, job_type, payload, scheduled_at)
           VALUES ($1, $2, $3, $4)
           RETURNING id"#,
    )
    .bind(org_id)
    .bind(&job_type_str)
    .bind(&payload)
    .bind(scheduled)
    .fetch_one(pool)
    .await?;

    let job_id = row.0;

    // Only push to stream for immediate jobs; scheduled jobs are picked up by the poller
    if scheduled <= Utc::now() {
        if let Err(e) = redis::cmd("XADD")
            .arg(STREAM_KEY)
            .arg("*")
            .arg("job_id")
            .arg(job_id.to_string())
            .query_async::<()>(redis)
            .await
        {
            warn!("Failed to XADD job {} to stream (will be picked up by poller): {}", job_id, e);
        }
    }

    info!(job_id = %job_id, job_type = %job_type_str, "Job enqueued");
    Ok(job_id)
}

pub struct JobQueue {
    pool: PgPool,
    redis: redis::aio::MultiplexedConnection,
    consumer_id: String,
    storage_factory: Arc<StorageFactory>,
}

impl JobQueue {
    pub fn new(pool: PgPool, redis: redis::aio::MultiplexedConnection, storage_factory: Arc<StorageFactory>) -> Self {
        let consumer_id = format!("worker-{}", Uuid::new_v4());
        Self {
            pool,
            redis,
            consumer_id,
            storage_factory,
        }
    }

    /// Ensure the consumer group exists, creating it if needed.
    async fn ensure_consumer_group(&mut self) {
        // Create the stream + consumer group if they don't exist
        let result: Result<(), redis::RedisError> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(STREAM_KEY)
            .arg(GROUP_NAME)
            .arg("0")
            .arg("MKSTREAM")
            .query_async(&mut self.redis)
            .await;

        match result {
            Ok(()) => info!("Created consumer group '{}'", GROUP_NAME),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("BUSYGROUP") {
                    info!("Consumer group '{}' already exists", GROUP_NAME);
                } else {
                    error!("Failed to create consumer group: {}", e);
                }
            }
        }
    }

    /// Recover stale jobs that were left in 'running' state by a crashed worker.
    async fn recover_stale_jobs(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::minutes(STALE_JOB_MINUTES);

        let stale_jobs: Vec<Job> = match sqlx::query_as(
            r#"UPDATE job_queue
               SET status = 'pending', started_at = NULL
               WHERE status = 'running' AND started_at < $1
               RETURNING *"#,
        )
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await
        {
            Ok(jobs) => jobs,
            Err(e) => {
                error!("Failed to recover stale jobs: {}", e);
                return;
            }
        };

        for job in &stale_jobs {
            if let Err(e) = redis::cmd("XADD")
                .arg(STREAM_KEY)
                .arg("*")
                .arg("job_id")
                .arg(job.id.to_string())
                .query_async::<()>(&mut self.redis)
                .await
            {
                warn!("Failed to re-enqueue stale job {}: {}", job.id, e);
            }
        }

        if !stale_jobs.is_empty() {
            info!("Recovered {} stale jobs", stale_jobs.len());
        }
    }

    /// Main worker loop — reads from Redis Stream and processes jobs.
    pub async fn run(mut self, shutdown: CancellationToken) {
        self.ensure_consumer_group().await;
        self.recover_stale_jobs().await;

        info!(consumer_id = %self.consumer_id, "Job queue worker started");

        loop {
            if shutdown.is_cancelled() {
                info!("Shutdown signal received, stopping job queue worker");
                break;
            }

            // XREADGROUP with 5s block timeout
            let result: Result<redis::streams::StreamReadReply, redis::RedisError> =
                redis::cmd("XREADGROUP")
                    .arg("GROUP")
                    .arg(GROUP_NAME)
                    .arg(&self.consumer_id)
                    .arg("COUNT")
                    .arg(5)
                    .arg("BLOCK")
                    .arg(5000)
                    .arg("STREAMS")
                    .arg(STREAM_KEY)
                    .arg(">")
                    .query_async(&mut self.redis)
                    .await;

            let reply = match result {
                Ok(reply) => reply,
                Err(e) => {
                    // Timeout returns nil which redis-rs treats as an error
                    let msg = e.to_string();
                    if msg.contains("response was nil") {
                        continue;
                    }
                    error!("XREADGROUP error: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            };

            for stream_key in &reply.keys {
                for entry in &stream_key.ids {
                    let job_id_str = match entry.map.get("job_id") {
                        Some(redis::Value::BulkString(bytes)) => {
                            String::from_utf8_lossy(bytes).to_string()
                        }
                        _ => {
                            warn!(stream_id = %entry.id, "Missing job_id in stream entry");
                            // ACK anyway to avoid reprocessing
                            let _ = redis::cmd("XACK")
                                .arg(STREAM_KEY)
                                .arg(GROUP_NAME)
                                .arg(&entry.id)
                                .query_async::<()>(&mut self.redis)
                                .await;
                            continue;
                        }
                    };

                    let stream_id = entry.id.clone();
                    self.process_stream_entry(&job_id_str, &stream_id).await;
                }
            }
        }

        info!("Job queue worker stopped");
    }

    async fn process_stream_entry(&mut self, job_id_str: &str, stream_id: &str) {
        let job_id = match Uuid::parse_str(job_id_str) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid job_id '{}': {}", job_id_str, e);
                let _ = self.ack(stream_id).await;
                return;
            }
        };

        // Load job from Postgres
        let job: Job = match sqlx::query_as(
            "SELECT * FROM job_queue WHERE id = $1",
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await
        {
            Ok(Some(job)) => job,
            Ok(None) => {
                warn!(job_id = %job_id, "Job not found in database");
                let _ = self.ack(stream_id).await;
                return;
            }
            Err(e) => {
                error!(job_id = %job_id, "Failed to load job: {}", e);
                return; // Don't ACK — retry on next read
            }
        };

        // Skip if not in processable state
        let status = job.status.as_str();
        if status != "pending" && status != "retry" {
            let _ = self.ack(stream_id).await;
            return;
        }

        // Mark as running
        if let Err(e) = sqlx::query(
            "UPDATE job_queue SET status = 'running', started_at = NOW(), attempts = attempts + 1 WHERE id = $1",
        )
        .bind(job_id)
        .execute(&self.pool)
        .await
        {
            error!(job_id = %job_id, "Failed to mark job as running: {}", e);
            return;
        }

        // Execute the job
        let result = self.execute_job(&job).await;

        match result {
            Ok(()) => {
                info!(job_id = %job_id, job_type = %job.job_type, "Job completed successfully");
                let _ = sqlx::query(
                    "UPDATE job_queue SET status = 'completed', completed_at = NOW() WHERE id = $1",
                )
                .bind(job_id)
                .execute(&self.pool)
                .await;
            }
            Err(err_msg) => {
                let new_attempts = job.attempts + 1;
                if new_attempts < job.max_attempts {
                    let backoff = retry_backoff(new_attempts);
                    let retry_at = Utc::now() + backoff;
                    warn!(
                        job_id = %job_id,
                        job_type = %job.job_type,
                        attempt = new_attempts,
                        retry_in = ?backoff,
                        "Job failed, scheduling retry: {}",
                        err_msg
                    );
                    let _ = sqlx::query(
                        "UPDATE job_queue SET status = 'retry', scheduled_at = $1, error_message = $2 WHERE id = $3",
                    )
                    .bind(retry_at)
                    .bind(&err_msg)
                    .bind(job_id)
                    .execute(&self.pool)
                    .await;
                } else {
                    error!(
                        job_id = %job_id,
                        job_type = %job.job_type,
                        "Job failed permanently after {} attempts: {}",
                        new_attempts,
                        err_msg
                    );
                    let _ = sqlx::query(
                        "UPDATE job_queue SET status = 'failed', completed_at = NOW(), error_message = $1 WHERE id = $2",
                    )
                    .bind(&err_msg)
                    .bind(job_id)
                    .execute(&self.pool)
                    .await;
                }
            }
        }

        let _ = self.ack(stream_id).await;
    }

    /// Dispatch job execution based on job_type.
    /// Returns Ok(()) on success, Err(error_message) on failure.
    async fn execute_job(&self, job: &Job) -> Result<(), String> {
        let job_type = job.job_type_enum()?;

        match job_type {
            JobType::RetentionEnforcement => {
                info!(job_id = %job.id, "Executing retention enforcement job");
                let org_id = job.org_id.ok_or("Missing org_id for retention job")?;
                retention_worker::execute(&self.pool, &self.storage_factory, org_id).await
            }
            JobType::WebhookDelivery => {
                info!(job_id = %job.id, "Executing webhook delivery job");
                webhook_worker::execute(&self.pool, &job.payload).await
            }
            JobType::ScheduledMessage => {
                info!(job_id = %job.id, "Executing scheduled message job");
                let mut redis = self.redis.clone();
                scheduled_message_worker::execute(&self.pool, &mut redis, &job.payload).await
            }
            JobType::Reminder => {
                info!(job_id = %job.id, "Executing reminder job");
                let mut redis = self.redis.clone();
                reminder_worker::execute(&self.pool, &mut redis, &job.payload).await
            }
            JobType::EmailNotification => {
                info!(job_id = %job.id, "Executing email notification job");
                // TODO: Implement in future phase
                Ok(())
            }
        }
    }

    async fn ack(&mut self, stream_id: &str) -> Result<(), redis::RedisError> {
        redis::cmd("XACK")
            .arg(STREAM_KEY)
            .arg(GROUP_NAME)
            .arg(stream_id)
            .query_async::<()>(&mut self.redis)
            .await
    }

    /// Poll for scheduled/retry jobs that are due and push them to the stream.
    pub async fn poll_scheduled_jobs(
        pool: &PgPool,
        redis: &mut redis::aio::MultiplexedConnection,
    ) {
        let due_jobs: Vec<(Uuid,)> = match sqlx::query_as(
            r#"UPDATE job_queue
               SET status = 'pending'
               WHERE status IN ('pending', 'retry')
                 AND scheduled_at <= NOW()
                 AND id NOT IN (
                     SELECT id FROM job_queue
                     WHERE status = 'running'
                 )
               RETURNING id"#,
        )
        .fetch_all(pool)
        .await
        {
            Ok(jobs) => jobs,
            Err(e) => {
                error!("Failed to poll scheduled jobs: {}", e);
                return;
            }
        };

        for (job_id,) in &due_jobs {
            if let Err(e) = redis::cmd("XADD")
                .arg(STREAM_KEY)
                .arg("*")
                .arg("job_id")
                .arg(job_id.to_string())
                .query_async::<()>(redis)
                .await
            {
                warn!("Failed to push scheduled job {} to stream: {}", job_id, e);
            }
        }

        if !due_jobs.is_empty() {
            info!("Pushed {} scheduled jobs to stream", due_jobs.len());
        }
    }
}

/// Calculate retry backoff duration based on attempt number.
fn retry_backoff(attempt: i32) -> chrono::Duration {
    match attempt {
        1 => chrono::Duration::seconds(1),
        2 => chrono::Duration::seconds(5),
        3 => chrono::Duration::seconds(30),
        _ => chrono::Duration::minutes(5),
    }
}
