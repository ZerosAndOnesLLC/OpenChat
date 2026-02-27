pub mod call_cleanup;
pub mod job_queue;
pub mod reminder_worker;
pub mod retention_worker;
pub mod scheduled_message_worker;
pub mod status;
pub mod webhook_worker;
pub mod workflow_worker;
pub mod websocket;

use std::sync::Arc;
use sqlx::PgPool;

use crate::services::livekit::LiveKitService;

/// Start all background tasks
pub fn start_background_tasks(
    pool: PgPool,
    ws_server: actix::Addr<crate::websocket::server::WsServer>,
    livekit: Option<Arc<LiveKitService>>,
) {
    // Start auto-away task
    tokio::spawn(status::run_auto_away_task(pool.clone(), ws_server.clone()));

    // Start clear expired statuses task
    tokio::spawn(status::run_clear_expired_statuses_task(pool.clone()));

    // Start WebSocket batch flushing task
    tokio::spawn(websocket::run_batch_flushing_task(ws_server.clone()));

    // Start call cleanup task
    tokio::spawn(call_cleanup::run_call_cleanup_task(pool, ws_server, livekit));
}
