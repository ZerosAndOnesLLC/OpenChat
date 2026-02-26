use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::{
        call::{Call, CallParticipant},
        channel::ChannelMember,
        direct_message::{DmParticipant, DirectMessage as DM},
        user::User,
    },
    services::{livekit::LiveKitService, tv_api::TokenClaims},
    websocket::{
        messages::ServerMessage,
        server::{BroadcastMessage, BroadcastToUser, WsServer},
    },
};

#[derive(Debug, Deserialize)]
pub struct StartCallRequest {
    pub channel_id: Option<Uuid>,
    pub dm_id: Option<Uuid>,
    pub call_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StartCallResponse {
    pub call_id: Uuid,
    pub token: String,
    pub livekit_url: String,
    pub livekit_room_name: String,
}

#[derive(Debug, Serialize)]
pub struct JoinCallResponse {
    pub call_id: Uuid,
    pub token: String,
    pub livekit_url: String,
    pub livekit_room_name: String,
}

fn get_livekit(
    lk: &web::Data<Option<Arc<LiveKitService>>>,
) -> ApiResult<&Arc<LiveKitService>> {
    lk.as_ref()
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Voice/video calling is not configured".to_string()))
}

fn get_livekit_url(config: &web::Data<crate::config::Config>) -> String {
    config
        .livekit
        .as_ref()
        .map(|c| c.url.clone())
        .unwrap_or_default()
}

/// POST /api/calls/start
pub async fn start_call(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    lk: web::Data<Option<Arc<LiveKitService>>>,
    config: web::Data<crate::config::Config>,
    body: web::Json<StartCallRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;
    let lk_service = get_livekit(&lk)?;

    let call_type = body.call_type.as_deref().unwrap_or("audio");
    if !["audio", "video"].contains(&call_type) {
        return Err(ApiError::BadRequest("call_type must be 'audio' or 'video'".to_string()));
    }

    if body.channel_id.is_none() && body.dm_id.is_none() {
        return Err(ApiError::BadRequest("channel_id or dm_id is required".to_string()));
    }

    // Validate membership
    if let Some(ch_id) = body.channel_id {
        if !ChannelMember::is_member(pool.get_ref(), ch_id, current_user.id).await? {
            return Err(ApiError::Authorization("Not a channel member".to_string()));
        }
        // Check no existing active call
        if Call::get_active_for_channel(pool.get_ref(), ch_id).await?.is_some() {
            return Err(ApiError::BadRequest("An active call already exists in this channel".to_string()));
        }
    }
    if let Some(d_id) = body.dm_id {
        if !DM::is_participant(pool.get_ref(), d_id, current_user.id).await? {
            return Err(ApiError::Authorization("Not a DM participant".to_string()));
        }
        if Call::get_active_for_dm(pool.get_ref(), d_id).await?.is_some() {
            return Err(ApiError::BadRequest("An active call already exists in this DM".to_string()));
        }
    }

    // Create call record
    let call = Call::create(
        pool.get_ref(),
        current_user.org_id,
        body.channel_id,
        body.dm_id,
        call_type,
        current_user.id,
        "pending", // placeholder, will update
        false,
    )
    .await?;

    // Generate room name and update
    let room_name = LiveKitService::room_name(current_user.org_id, call.id);
    sqlx::query("UPDATE calls SET livekit_room_name = $1 WHERE id = $2")
        .bind(&room_name)
        .bind(call.id)
        .execute(pool.get_ref())
        .await?;

    // Create LiveKit room
    lk_service.create_room(&room_name).await?;

    // Add caller as first participant
    CallParticipant::join(pool.get_ref(), call.id, current_user.id).await?;

    // Generate token for caller
    let token = lk_service
        .generate_token(
            &room_name,
            &current_user.id.to_string(),
            &current_user.display_name,
            true,
            true,
        )
        .await?;

    // Broadcast CallStarted
    let call_started = ServerMessage::CallStarted {
        call_id: call.id,
        channel_id: body.channel_id,
        dm_id: body.dm_id,
        call_type: call_type.to_string(),
        started_by: current_user.id,
        started_by_name: current_user.display_name.clone(),
        is_huddle: false,
    };

    if let Some(ch_id) = body.channel_id {
        ws_server.do_send(BroadcastMessage {
            org_id: current_user.org_id,
            channel_id: Some(ch_id),
            message: call_started,
        });

        // Send CallRinging to channel members (excluding caller)
        let ringing = ServerMessage::CallRinging {
            call_id: call.id,
            channel_id: Some(ch_id),
            dm_id: None,
            call_type: call_type.to_string(),
            started_by: current_user.id,
            started_by_name: current_user.display_name.clone(),
        };
        ws_server.do_send(BroadcastMessage {
            org_id: current_user.org_id,
            channel_id: Some(ch_id),
            message: ringing,
        });
    } else if let Some(d_id) = body.dm_id {
        // Broadcast to DM participants
        if let Ok(participants) = DmParticipant::list_by_dm(pool.get_ref(), d_id).await {
            for participant in &participants {
                ws_server.do_send(BroadcastToUser {
                    org_id: current_user.org_id,
                    user_id: participant.user_id,
                    message: call_started.clone(),
                });
            }
            // Send ringing to other participants
            let ringing = ServerMessage::CallRinging {
                call_id: call.id,
                channel_id: None,
                dm_id: Some(d_id),
                call_type: call_type.to_string(),
                started_by: current_user.id,
                started_by_name: current_user.display_name.clone(),
            };
            for participant in &participants {
                if participant.user_id != current_user.id {
                    ws_server.do_send(BroadcastToUser {
                        org_id: current_user.org_id,
                        user_id: participant.user_id,
                        message: ringing.clone(),
                    });
                }
            }
        }
    }

    Ok(HttpResponse::Created().json(StartCallResponse {
        call_id: call.id,
        token,
        livekit_url: get_livekit_url(&config),
        livekit_room_name: room_name,
    }))
}

/// POST /api/calls/{id}/join
pub async fn join_call(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    lk: web::Data<Option<Arc<LiveKitService>>>,
    config: web::Data<crate::config::Config>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;
    let lk_service = get_livekit(&lk)?;
    let call_id = path.into_inner();

    let call = Call::get_by_id(pool.get_ref(), call_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Call not found".to_string()))?;

    if call.status == "ended" {
        return Err(ApiError::BadRequest("Call has ended".to_string()));
    }

    // Verify membership
    if let Some(ch_id) = call.channel_id {
        if !ChannelMember::is_member(pool.get_ref(), ch_id, current_user.id).await? {
            return Err(ApiError::Authorization("Not a channel member".to_string()));
        }
    }
    if let Some(d_id) = call.dm_id {
        if !DM::is_participant(pool.get_ref(), d_id, current_user.id).await? {
            return Err(ApiError::Authorization("Not a DM participant".to_string()));
        }
    }

    // Transition ringing → active on first join (by someone other than starter)
    if call.status == "ringing" && current_user.id != call.started_by {
        Call::set_active(pool.get_ref(), call.id).await?;
    }

    // Upsert participant
    CallParticipant::join(pool.get_ref(), call.id, current_user.id).await?;

    // Generate token
    let token = lk_service
        .generate_token(
            &call.livekit_room_name,
            &current_user.id.to_string(),
            &current_user.display_name,
            true,
            true,
        )
        .await?;

    // Broadcast participant joined
    let joined_msg = ServerMessage::CallParticipantJoined {
        call_id: call.id,
        channel_id: call.channel_id,
        dm_id: call.dm_id,
        user_id: current_user.id,
        user_name: current_user.display_name.clone(),
    };
    broadcast_call_event(&ws_server, &pool, &call, joined_msg).await;

    Ok(HttpResponse::Ok().json(JoinCallResponse {
        call_id: call.id,
        token,
        livekit_url: get_livekit_url(&config),
        livekit_room_name: call.livekit_room_name,
    }))
}

/// POST /api/calls/{id}/leave
pub async fn leave_call(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    lk: web::Data<Option<Arc<LiveKitService>>>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;
    let call_id = path.into_inner();

    let call = Call::get_by_id(pool.get_ref(), call_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Call not found".to_string()))?;

    if call.status == "ended" {
        return Err(ApiError::BadRequest("Call has already ended".to_string()));
    }

    // Mark participant as left
    CallParticipant::leave(pool.get_ref(), call.id, current_user.id).await?;

    // Broadcast participant left
    let left_msg = ServerMessage::CallParticipantLeft {
        call_id: call.id,
        channel_id: call.channel_id,
        dm_id: call.dm_id,
        user_id: current_user.id,
        user_name: current_user.display_name.clone(),
    };
    broadcast_call_event(&ws_server, &pool, &call, left_msg).await;

    // Auto-end if no active participants
    let remaining = CallParticipant::count_active(pool.get_ref(), call.id).await?;
    if remaining == 0 {
        Call::end_call(pool.get_ref(), call.id).await?;
        if let Some(lk_service) = lk.as_ref().as_ref() {
            let _ = lk_service.delete_room(&call.livekit_room_name).await;
        }
        let ended_msg = ServerMessage::CallEnded {
            call_id: call.id,
            channel_id: call.channel_id,
            dm_id: call.dm_id,
        };
        broadcast_call_event(&ws_server, &pool, &call, ended_msg).await;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "left"})))
}

/// POST /api/calls/{id}/end
pub async fn end_call(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    lk: web::Data<Option<Arc<LiveKitService>>>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;
    let call_id = path.into_inner();

    let call = Call::get_by_id(pool.get_ref(), call_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Call not found".to_string()))?;

    if call.status == "ended" {
        return Err(ApiError::BadRequest("Call has already ended".to_string()));
    }

    // Only starter can end the call (unless they're an admin—simplified here)
    if call.started_by != current_user.id {
        return Err(ApiError::Authorization("Only the call starter can end the call".to_string()));
    }

    Call::end_call(pool.get_ref(), call.id).await?;
    if let Some(lk_service) = lk.as_ref().as_ref() {
        let _ = lk_service.delete_room(&call.livekit_room_name).await;
    }

    let ended_msg = ServerMessage::CallEnded {
        call_id: call.id,
        channel_id: call.channel_id,
        dm_id: call.dm_id,
    };
    broadcast_call_event(&ws_server, &pool, &call, ended_msg).await;

    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "ended"})))
}

/// GET /api/calls/active
pub async fn list_active_calls(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let calls = Call::list_active_for_user(pool.get_ref(), current_user.id).await?;
    Ok(HttpResponse::Ok().json(calls))
}

/// POST /api/channels/{id}/huddle/join
pub async fn join_huddle(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    lk: web::Data<Option<Arc<LiveKitService>>>,
    config: web::Data<crate::config::Config>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;
    let lk_service = get_livekit(&lk)?;
    let channel_id = path.into_inner();

    if !ChannelMember::is_member(pool.get_ref(), channel_id, current_user.id).await? {
        return Err(ApiError::Authorization("Not a channel member".to_string()));
    }

    // Find or create huddle
    let call = match Call::get_active_huddle_for_channel(pool.get_ref(), channel_id).await? {
        Some(existing) => existing,
        None => {
            let call = Call::create(
                pool.get_ref(),
                current_user.org_id,
                Some(channel_id),
                None,
                "audio",
                current_user.id,
                "pending",
                true,
            )
            .await?;

            let room_name = LiveKitService::room_name(current_user.org_id, call.id);
            sqlx::query("UPDATE calls SET livekit_room_name = $1 WHERE id = $2")
                .bind(&room_name)
                .bind(call.id)
                .execute(pool.get_ref())
                .await?;

            lk_service.create_room(&room_name).await?;

            // Broadcast huddle started
            let started_msg = ServerMessage::CallStarted {
                call_id: call.id,
                channel_id: Some(channel_id),
                dm_id: None,
                call_type: "audio".to_string(),
                started_by: current_user.id,
                started_by_name: current_user.display_name.clone(),
                is_huddle: true,
            };
            ws_server.do_send(BroadcastMessage {
                org_id: current_user.org_id,
                channel_id: Some(channel_id),
                message: started_msg,
            });

            // Re-fetch to get updated room name
            Call::get_by_id(pool.get_ref(), call.id)
                .await?
                .ok_or_else(|| ApiError::Internal("Failed to fetch created huddle".to_string()))?
        }
    };

    // Join
    CallParticipant::join(pool.get_ref(), call.id, current_user.id).await?;

    let token = lk_service
        .generate_token(
            &call.livekit_room_name,
            &current_user.id.to_string(),
            &current_user.display_name,
            true,
            true,
        )
        .await?;

    // Broadcast participant joined
    let joined_msg = ServerMessage::CallParticipantJoined {
        call_id: call.id,
        channel_id: Some(channel_id),
        dm_id: None,
        user_id: current_user.id,
        user_name: current_user.display_name.clone(),
    };
    ws_server.do_send(BroadcastMessage {
        org_id: current_user.org_id,
        channel_id: Some(channel_id),
        message: joined_msg,
    });

    Ok(HttpResponse::Ok().json(JoinCallResponse {
        call_id: call.id,
        token,
        livekit_url: get_livekit_url(&config),
        livekit_room_name: call.livekit_room_name,
    }))
}

/// POST /api/channels/{id}/huddle/leave
pub async fn leave_huddle(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    lk: web::Data<Option<Arc<LiveKitService>>>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req.extensions().get::<TokenClaims>().cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;
    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;
    let channel_id = path.into_inner();

    let call = Call::get_active_huddle_for_channel(pool.get_ref(), channel_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("No active huddle in this channel".to_string()))?;

    CallParticipant::leave(pool.get_ref(), call.id, current_user.id).await?;

    // Broadcast participant left
    let left_msg = ServerMessage::CallParticipantLeft {
        call_id: call.id,
        channel_id: Some(channel_id),
        dm_id: None,
        user_id: current_user.id,
        user_name: current_user.display_name.clone(),
    };
    ws_server.do_send(BroadcastMessage {
        org_id: current_user.org_id,
        channel_id: Some(channel_id),
        message: left_msg,
    });

    // Auto-end if empty
    let remaining = CallParticipant::count_active(pool.get_ref(), call.id).await?;
    if remaining == 0 {
        Call::end_call(pool.get_ref(), call.id).await?;
        if let Some(lk_service) = lk.as_ref().as_ref() {
            let _ = lk_service.delete_room(&call.livekit_room_name).await;
        }
        let ended_msg = ServerMessage::CallEnded {
            call_id: call.id,
            channel_id: Some(channel_id),
            dm_id: None,
        };
        ws_server.do_send(BroadcastMessage {
            org_id: current_user.org_id,
            channel_id: Some(channel_id),
            message: ended_msg,
        });
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "left"})))
}

/// Helper to broadcast call events to the appropriate channel or DM participants
async fn broadcast_call_event(
    ws_server: &web::Data<actix::Addr<WsServer>>,
    pool: &web::Data<PgPool>,
    call: &Call,
    message: ServerMessage,
) {
    if let Some(ch_id) = call.channel_id {
        ws_server.do_send(BroadcastMessage {
            org_id: call.org_id,
            channel_id: Some(ch_id),
            message,
        });
    } else if let Some(d_id) = call.dm_id {
        if let Ok(participants) = DmParticipant::list_by_dm(pool.get_ref(), d_id).await {
            for participant in participants {
                ws_server.do_send(BroadcastToUser {
                    org_id: call.org_id,
                    user_id: participant.user_id,
                    message: message.clone(),
                });
            }
        }
    }
}
