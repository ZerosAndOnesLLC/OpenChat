use std::sync::Arc;
use std::time::Duration;
use sqlx::PgPool;
use tracing::{error, info};

use crate::models::call::Call;
use crate::services::livekit::LiveKitService;
use crate::websocket::messages::ServerMessage;
use crate::websocket::server::{BroadcastMessage, WsServer};

pub async fn run_call_cleanup_task(
    pool: PgPool,
    ws_server: actix::Addr<WsServer>,
    livekit: Option<Arc<LiveKitService>>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        // End stale ringing calls (>60s)
        match Call::find_stale_ringing(&pool).await {
            Ok(calls) => {
                for call in calls {
                    if let Err(e) = Call::end_call(&pool, call.id).await {
                        error!("Failed to end stale ringing call {}: {}", call.id, e);
                        continue;
                    }
                    if let Some(lk) = &livekit {
                        let _ = lk.delete_room(&call.livekit_room_name).await;
                    }
                    let msg = ServerMessage::CallEnded {
                        call_id: call.id,
                        channel_id: call.channel_id,
                        dm_id: call.dm_id,
                    };
                    if let Some(ch_id) = call.channel_id {
                        ws_server.do_send(BroadcastMessage {
                            org_id: call.org_id,
                            channel_id: Some(ch_id),
                            message: msg,
                        });
                    } else {
                        ws_server.do_send(BroadcastMessage {
                            org_id: call.org_id,
                            channel_id: None,
                            message: msg,
                        });
                    }
                    info!("Cleaned up stale ringing call {}", call.id);
                }
            }
            Err(e) => error!("Failed to find stale ringing calls: {}", e),
        }

        // End active calls with 0 participants (>30s)
        match Call::find_empty_active(&pool).await {
            Ok(calls) => {
                for call in calls {
                    if let Err(e) = Call::end_call(&pool, call.id).await {
                        error!("Failed to end empty active call {}: {}", call.id, e);
                        continue;
                    }
                    if let Some(lk) = &livekit {
                        let _ = lk.delete_room(&call.livekit_room_name).await;
                    }
                    let msg = ServerMessage::CallEnded {
                        call_id: call.id,
                        channel_id: call.channel_id,
                        dm_id: call.dm_id,
                    };
                    if let Some(ch_id) = call.channel_id {
                        ws_server.do_send(BroadcastMessage {
                            org_id: call.org_id,
                            channel_id: Some(ch_id),
                            message: msg,
                        });
                    } else {
                        ws_server.do_send(BroadcastMessage {
                            org_id: call.org_id,
                            channel_id: None,
                            message: msg,
                        });
                    }
                    info!("Cleaned up empty active call {}", call.id);
                }
            }
            Err(e) => error!("Failed to find empty active calls: {}", e),
        }
    }
}
