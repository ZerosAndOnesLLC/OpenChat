use actix::Addr;
use std::time::Duration;
use tokio::time::sleep;

use crate::websocket::server::{FlushBatches, WsServer};

/// Background task to periodically flush message batches
/// This ensures messages don't get stuck in batches if traffic is low
pub async fn run_batch_flushing_task(ws_server: Addr<WsServer>) {
    tracing::info!("Starting WebSocket batch flushing task");

    loop {
        // Flush batches every 50ms (matches default batch timeout)
        sleep(Duration::from_millis(50)).await;

        // Send flush message to WsServer
        ws_server.do_send(FlushBatches);
    }
}
