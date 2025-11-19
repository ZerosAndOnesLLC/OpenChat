use sqlx::PgPool;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info};

use crate::models::user_status::UserStatus;
use crate::models::user::User;
use crate::websocket::{
    messages::ServerMessage,
    server::{BroadcastMessage, WsServer},
};

/// Run auto-away task every 5 minutes
/// Sets users to 'away' if they've been inactive for 15+ minutes
pub async fn run_auto_away_task(pool: PgPool, ws_server: actix::Addr<WsServer>) {
    let mut interval_timer = interval(Duration::from_secs(300)); // 5 minutes

    loop {
        interval_timer.tick().await;

        match UserStatus::auto_away_inactive_users(&pool).await {
            Ok(user_ids) => {
                if !user_ids.is_empty() {
                    info!("Set {} users to 'away' due to inactivity", user_ids.len());

                    // Broadcast status changes
                    for user_id in user_ids {
                        // Get user's org_id for broadcast
                        if let Ok(Some(user)) = User::get_by_id(&pool, user_id).await {
                            ws_server.do_send(BroadcastMessage {
                                org_id: user.org_id,
                                channel_id: None,
                                message: ServerMessage::StatusUpdate {
                                    user_id,
                                    status: "away".to_string(),
                                    custom_message: None,
                                    emoji: None,
                                },
                            });
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to run auto-away task: {:?}", e);
            }
        }
    }
}

/// Run clear expired statuses task every 10 minutes
/// Clears custom status messages that have expired
pub async fn run_clear_expired_statuses_task(pool: PgPool) {
    let mut interval_timer = interval(Duration::from_secs(600)); // 10 minutes

    loop {
        interval_timer.tick().await;

        match UserStatus::clear_expired_statuses(&pool).await {
            Ok(count) => {
                if count > 0 {
                    info!("Cleared {} expired custom status messages", count);
                }
            }
            Err(e) => {
                error!("Failed to clear expired statuses: {:?}", e);
            }
        }
    }
}
