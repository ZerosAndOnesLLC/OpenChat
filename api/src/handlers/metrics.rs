use actix_web::{web, HttpResponse};
use redis::aio::MultiplexedConnection;
use serde_json::json;

use crate::cache::metrics;
use crate::errors::ApiResult;

/// GET /api/metrics/cache - Get cache hit/miss metrics
pub async fn get_cache_metrics(
    redis_conn: web::Data<MultiplexedConnection>,
) -> ApiResult<HttpResponse> {
    let mut redis_conn = redis_conn.as_ref().clone();
    let metrics = metrics::get_metrics(&mut redis_conn).await?;

    let response = json!({
        "total_hits": metrics.total_hits(),
        "total_misses": metrics.total_misses(),
        "total_operations": metrics.total_operations(),
        "hit_rate_percentage": format!("{:.2}%", metrics.hit_rate()),
        "by_type": {
            "channels": {
                "hits": metrics.channels_hits,
                "misses": metrics.channels_misses,
                "total": metrics.channels_hits + metrics.channels_misses,
                "hit_rate": if metrics.channels_hits + metrics.channels_misses > 0 {
                    format!("{:.2}%", (metrics.channels_hits as f64 / (metrics.channels_hits + metrics.channels_misses) as f64) * 100.0)
                } else {
                    "0.00%".to_string()
                }
            },
            "channel_members": {
                "hits": metrics.channel_members_hits,
                "misses": metrics.channel_members_misses,
                "total": metrics.channel_members_hits + metrics.channel_members_misses,
                "hit_rate": if metrics.channel_members_hits + metrics.channel_members_misses > 0 {
                    format!("{:.2}%", (metrics.channel_members_hits as f64 / (metrics.channel_members_hits + metrics.channel_members_misses) as f64) * 100.0)
                } else {
                    "0.00%".to_string()
                }
            },
            "dms": {
                "hits": metrics.dms_hits,
                "misses": metrics.dms_misses,
                "total": metrics.dms_hits + metrics.dms_misses,
                "hit_rate": if metrics.dms_hits + metrics.dms_misses > 0 {
                    format!("{:.2}%", (metrics.dms_hits as f64 / (metrics.dms_hits + metrics.dms_misses) as f64) * 100.0)
                } else {
                    "0.00%".to_string()
                }
            },
            "dm_participants": {
                "hits": metrics.dm_participants_hits,
                "misses": metrics.dm_participants_misses,
                "total": metrics.dm_participants_hits + metrics.dm_participants_misses,
                "hit_rate": if metrics.dm_participants_hits + metrics.dm_participants_misses > 0 {
                    format!("{:.2}%", (metrics.dm_participants_hits as f64 / (metrics.dm_participants_hits + metrics.dm_participants_misses) as f64) * 100.0)
                } else {
                    "0.00%".to_string()
                }
            },
            "users": {
                "hits": metrics.users_hits,
                "misses": metrics.users_misses,
                "total": metrics.users_hits + metrics.users_misses,
                "hit_rate": if metrics.users_hits + metrics.users_misses > 0 {
                    format!("{:.2}%", (metrics.users_hits as f64 / (metrics.users_hits + metrics.users_misses) as f64) * 100.0)
                } else {
                    "0.00%".to_string()
                }
            },
            "messages": {
                "hits": metrics.messages_hits,
                "misses": metrics.messages_misses,
                "total": metrics.messages_hits + metrics.messages_misses,
                "hit_rate": if metrics.messages_hits + metrics.messages_misses > 0 {
                    format!("{:.2}%", (metrics.messages_hits as f64 / (metrics.messages_hits + metrics.messages_misses) as f64) * 100.0)
                } else {
                    "0.00%".to_string()
                }
            },
            "read_status": {
                "hits": metrics.read_status_hits,
                "misses": metrics.read_status_misses,
                "total": metrics.read_status_hits + metrics.read_status_misses,
                "hit_rate": if metrics.read_status_hits + metrics.read_status_misses > 0 {
                    format!("{:.2}%", (metrics.read_status_hits as f64 / (metrics.read_status_hits + metrics.read_status_misses) as f64) * 100.0)
                } else {
                    "0.00%".to_string()
                }
            },
            "notifications": {
                "hits": metrics.notifications_hits,
                "misses": metrics.notifications_misses,
                "total": metrics.notifications_hits + metrics.notifications_misses,
                "hit_rate": if metrics.notifications_hits + metrics.notifications_misses > 0 {
                    format!("{:.2}%", (metrics.notifications_hits as f64 / (metrics.notifications_hits + metrics.notifications_misses) as f64) * 100.0)
                } else {
                    "0.00%".to_string()
                }
            }
        }
    });

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/metrics/cache/reset - Reset cache metrics (admin only)
pub async fn reset_cache_metrics(
    redis_conn: web::Data<MultiplexedConnection>,
) -> ApiResult<HttpResponse> {
    let mut redis_conn = redis_conn.as_ref().clone();
    metrics::reset_metrics(&mut redis_conn).await?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Cache metrics reset successfully"
    })))
}
