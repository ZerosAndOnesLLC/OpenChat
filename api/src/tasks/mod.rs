pub mod status;

use sqlx::PgPool;

/// Start all background tasks
pub fn start_background_tasks(pool: PgPool, ws_server: actix::Addr<crate::websocket::server::WsServer>) {
    // Start auto-away task
    tokio::spawn(status::run_auto_away_task(pool.clone(), ws_server.clone()));

    // Start clear expired statuses task
    tokio::spawn(status::run_clear_expired_statuses_task(pool));
}
