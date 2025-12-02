/// Cache metrics tracking module
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::errors::ApiResult;

const METRICS_KEY: &str = "openchat:cache:metrics";
const METRICS_TTL: u64 = 86400; // 24 hours

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheMetrics {
    pub channels_hits: i64,
    pub channels_misses: i64,
    pub channel_members_hits: i64,
    pub channel_members_misses: i64,
    pub dms_hits: i64,
    pub dms_misses: i64,
    pub dm_participants_hits: i64,
    pub dm_participants_misses: i64,
    pub users_hits: i64,
    pub users_misses: i64,
    pub messages_hits: i64,
    pub messages_misses: i64,
    pub read_status_hits: i64,
    pub read_status_misses: i64,
    pub notifications_hits: i64,
    pub notifications_misses: i64,
    pub organizations_hits: i64,
    pub organizations_misses: i64,
    pub tokens_hits: i64,
    pub tokens_misses: i64,
}

impl CacheMetrics {
    /// Calculate total hits across all cache types
    pub fn total_hits(&self) -> i64 {
        self.channels_hits
            + self.channel_members_hits
            + self.dms_hits
            + self.dm_participants_hits
            + self.users_hits
            + self.messages_hits
            + self.read_status_hits
            + self.notifications_hits
            + self.organizations_hits
            + self.tokens_hits
    }

    /// Calculate total misses across all cache types
    pub fn total_misses(&self) -> i64 {
        self.channels_misses
            + self.channel_members_misses
            + self.dms_misses
            + self.dm_participants_misses
            + self.users_misses
            + self.organizations_misses
            + self.messages_misses
            + self.read_status_misses
            + self.notifications_misses
            + self.tokens_misses
    }

    /// Calculate total operations (hits + misses)
    pub fn total_operations(&self) -> i64 {
        self.total_hits() + self.total_misses()
    }

    /// Calculate hit rate as a percentage (0-100)
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_operations();
        if total == 0 {
            0.0
        } else {
            (self.total_hits() as f64 / total as f64) * 100.0
        }
    }
}

/// Cache type for metrics tracking
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum CacheType {
    Channels,
    ChannelMembers,
    Dms,
    DmParticipants,
    Users,
    Messages,
    ReadStatus,
    Notifications,
    Organizations,
    Tokens,
}

/// Record a cache hit
pub async fn record_hit(
    redis: &mut redis::aio::MultiplexedConnection,
    cache_type: CacheType,
) {
    let field = match cache_type {
        CacheType::Channels => "channels_hits",
        CacheType::ChannelMembers => "channel_members_hits",
        CacheType::Dms => "dms_hits",
        CacheType::DmParticipants => "dm_participants_hits",
        CacheType::Users => "users_hits",
        CacheType::Messages => "messages_hits",
        CacheType::ReadStatus => "read_status_hits",
        CacheType::Notifications => "notifications_hits",
        CacheType::Organizations => "organizations_hits",
        CacheType::Tokens => "tokens_hits",
    };

    if let Err(e) = increment_metric(redis, field).await {
        warn!("Failed to record cache hit for {:?}: {}", cache_type, e);
    }
}

/// Record a cache miss
pub async fn record_miss(
    redis: &mut redis::aio::MultiplexedConnection,
    cache_type: CacheType,
) {
    let field = match cache_type {
        CacheType::Channels => "channels_misses",
        CacheType::ChannelMembers => "channel_members_misses",
        CacheType::Dms => "dms_misses",
        CacheType::DmParticipants => "dm_participants_misses",
        CacheType::Users => "users_misses",
        CacheType::Messages => "messages_misses",
        CacheType::ReadStatus => "read_status_misses",
        CacheType::Notifications => "notifications_misses",
        CacheType::Organizations => "organizations_misses",
        CacheType::Tokens => "tokens_misses",
    };

    if let Err(e) = increment_metric(redis, field).await {
        warn!("Failed to record cache miss for {:?}: {}", cache_type, e);
    }
}

/// Increment a metric field in Redis
async fn increment_metric(
    redis: &mut redis::aio::MultiplexedConnection,
    field: &str,
) -> ApiResult<()> {
    // Increment the field
    let _: i64 = redis.hincr(METRICS_KEY, field, 1).await?;

    // Set expiration on the hash (resets TTL on each update)
    let _: () = redis.expire(METRICS_KEY, METRICS_TTL as i64).await?;

    Ok(())
}

/// Get current cache metrics
pub async fn get_metrics(
    redis: &mut redis::aio::MultiplexedConnection,
) -> ApiResult<CacheMetrics> {
    let values: Vec<(String, i64)> = redis.hgetall(METRICS_KEY).await?;

    let mut metrics = CacheMetrics::default();

    for (field, value) in values {
        match field.as_str() {
            "channels_hits" => metrics.channels_hits = value,
            "channels_misses" => metrics.channels_misses = value,
            "channel_members_hits" => metrics.channel_members_hits = value,
            "channel_members_misses" => metrics.channel_members_misses = value,
            "dms_hits" => metrics.dms_hits = value,
            "dms_misses" => metrics.dms_misses = value,
            "dm_participants_hits" => metrics.dm_participants_hits = value,
            "dm_participants_misses" => metrics.dm_participants_misses = value,
            "users_hits" => metrics.users_hits = value,
            "users_misses" => metrics.users_misses = value,
            "messages_hits" => metrics.messages_hits = value,
            "messages_misses" => metrics.messages_misses = value,
            "read_status_hits" => metrics.read_status_hits = value,
            "read_status_misses" => metrics.read_status_misses = value,
            "notifications_hits" => metrics.notifications_hits = value,
            "notifications_misses" => metrics.notifications_misses = value,
            "organizations_hits" => metrics.organizations_hits = value,
            "organizations_misses" => metrics.organizations_misses = value,
            "tokens_hits" => metrics.tokens_hits = value,
            "tokens_misses" => metrics.tokens_misses = value,
            _ => {}
        }
    }

    Ok(metrics)
}

/// Reset cache metrics (useful for testing or periodic resets)
pub async fn reset_metrics(
    redis: &mut redis::aio::MultiplexedConnection,
) -> ApiResult<()> {
    let _: () = redis.del(METRICS_KEY).await?;
    Ok(())
}
