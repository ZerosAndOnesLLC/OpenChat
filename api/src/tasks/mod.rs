pub mod job_queue;
pub mod retention_worker;
pub mod status;
pub mod webhook_worker;
pub mod websocket;

use sqlx::PgPool;

/// Start all background tasks
pub fn start_background_tasks(pool: PgPool, ws_server: actix::Addr<crate::websocket::server::WsServer>) {
    // Start auto-away task
    tokio::spawn(status::run_auto_away_task(pool.clone(), ws_server.clone()));

    // Start clear expired statuses task
    tokio::spawn(status::run_clear_expired_statuses_task(pool));

    // Start WebSocket batch flushing task
    tokio::spawn(websocket::run_batch_flushing_task(ws_server.clone()));
}
