/// Cache warming module - preloads frequently accessed data into Redis on startup
use sqlx::PgPool;
use redis::aio::MultiplexedConnection;
use tracing::{info, warn};

use crate::errors::ApiResult;
use super::{channels, dms, users};

/// Warm the cache with frequently accessed data on application startup
/// This improves initial performance by pre-loading commonly used data
pub async fn warm_cache(
    db_pool: &PgPool,
    redis_conn: &mut MultiplexedConnection,
) -> ApiResult<()> {
    info!("Starting cache warming...");

    let start = std::time::Instant::now();
    let mut warmed_items = 0;

    // Warm active channels (channels with recent activity)
    match warm_active_channels(db_pool, redis_conn).await {
        Ok(count) => {
            warmed_items += count;
            info!("Warmed {} active channels", count);
        }
        Err(e) => warn!("Failed to warm active channels: {}", e),
    }

    // Warm active DMs (DMs with recent activity)
    match warm_active_dms(db_pool, redis_conn).await {
        Ok(count) => {
            warmed_items += count;
            info!("Warmed {} active DMs", count);
        }
        Err(e) => warn!("Failed to warm active DMs: {}", e),
    }

    // Warm active users (users seen in last 24 hours)
    match warm_active_users(db_pool, redis_conn).await {
        Ok(count) => {
            warmed_items += count;
            info!("Warmed {} active users", count);
        }
        Err(e) => warn!("Failed to warm active users: {}", e),
    }

    let duration = start.elapsed();
    info!("Cache warming completed: {} items in {:?}", warmed_items, duration);

    Ok(())
}

/// Warm cache with active channels (channels with messages in last 7 days)
async fn warm_active_channels(
    db_pool: &PgPool,
    redis_conn: &mut MultiplexedConnection,
) -> ApiResult<usize> {
    // Query for channels with recent activity (last 7 days)
    let active_channels = sqlx::query!(
        r#"
        SELECT DISTINCT c.id, c.org_id, c.name, c.description, c.channel_type,
               c.created_by, c.created_at, c.updated_at
        FROM channels c
        INNER JOIN messages m ON m.channel_id = c.id
        WHERE m.created_at > NOW() - INTERVAL '7 days'
        ORDER BY c.created_at DESC
        LIMIT 100
        "#
    )
    .fetch_all(db_pool)
    .await?;

    let mut count = 0;

    for channel in active_channels {
        let channel_model = crate::models::channel::Channel {
            id: channel.id,
            org_id: channel.org_id,
            name: channel.name,
            description: channel.description,
            channel_type: channel.channel_type,
            created_by: channel.created_by,
            created_at: channel.created_at,
            updated_at: channel.updated_at,
        };

        // Cache the channel
        if let Err(e) = channels::set_channel_in_cache(redis_conn, &channel_model).await {
            warn!("Failed to cache channel {}: {}", channel.id, e);
            continue;
        }

        // Cache the channel members
        let members = sqlx::query_as!(
            crate::models::channel::ChannelMember,
            r#"
            SELECT id, channel_id, user_id, role, joined_at
            FROM channel_members
            WHERE channel_id = $1
            "#,
            channel.id
        )
        .fetch_all(db_pool)
        .await?;

        if let Err(e) = channels::set_channel_members_in_cache(redis_conn, channel.id, &members).await {
            warn!("Failed to cache members for channel {}: {}", channel.id, e);
        } else {
            count += 1;
        }
    }

    Ok(count)
}

/// Warm cache with active DMs (DMs with messages in last 7 days)
async fn warm_active_dms(
    db_pool: &PgPool,
    redis_conn: &mut MultiplexedConnection,
) -> ApiResult<usize> {
    // Query for DMs with recent activity (last 7 days)
    let active_dms = sqlx::query!(
        r#"
        SELECT DISTINCT d.id, d.org_id, d.created_by, d.created_at
        FROM direct_messages d
        INNER JOIN messages m ON m.dm_id = d.id
        WHERE m.created_at > NOW() - INTERVAL '7 days'
        ORDER BY d.created_at DESC
        LIMIT 100
        "#
    )
    .fetch_all(db_pool)
    .await?;

    let mut count = 0;

    for dm in active_dms {
        let dm_model = crate::models::direct_message::DirectMessage {
            id: dm.id,
            org_id: dm.org_id,
            created_by: dm.created_by,
            created_at: dm.created_at,
        };

        // Cache the DM
        if let Err(e) = dms::set_dm_in_cache(redis_conn, &dm_model).await {
            warn!("Failed to cache DM {}: {}", dm.id, e);
            continue;
        }

        // Cache the DM participants
        let participants = sqlx::query_as!(
            crate::models::direct_message::DmParticipant,
            r#"
            SELECT id, dm_id, user_id, joined_at
            FROM dm_participants
            WHERE dm_id = $1
            "#,
            dm.id
        )
        .fetch_all(db_pool)
        .await?;

        if let Err(e) = dms::set_dm_participants_in_cache(redis_conn, dm.id, &participants).await {
            warn!("Failed to cache participants for DM {}: {}", dm.id, e);
        } else {
            count += 1;
        }
    }

    Ok(count)
}

/// Warm cache with active users (users with status updates or messages in last 24 hours)
async fn warm_active_users(
    db_pool: &PgPool,
    redis_conn: &mut MultiplexedConnection,
) -> ApiResult<usize> {
    // Query for users with recent activity (last 24 hours)
    let active_users = sqlx::query!(
        r#"
        SELECT DISTINCT u.id, u.org_id, u.tv_user_id, u.email, u.display_name,
               u.avatar_url, u.status, u.disable_read_receipts, u.created_at, u.updated_at
        FROM users u
        LEFT JOIN messages m ON m.user_id = u.id
        LEFT JOIN user_status us ON us.user_id = u.id
        WHERE m.created_at > NOW() - INTERVAL '24 hours'
           OR us.updated_at > NOW() - INTERVAL '24 hours'
           OR u.updated_at > NOW() - INTERVAL '24 hours'
        ORDER BY u.updated_at DESC
        LIMIT 200
        "#
    )
    .fetch_all(db_pool)
    .await?;

    let mut count = 0;

    for user in active_users {
        let user_model = crate::models::user::User {
            id: user.id,
            org_id: user.org_id,
            tv_user_id: user.tv_user_id,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            status: user.status,
            disable_read_receipts: user.disable_read_receipts,
            created_at: user.created_at,
            updated_at: user.updated_at,
        };

        if let Err(e) = users::set_user_in_cache(redis_conn, &user_model).await {
            warn!("Failed to cache user {}: {}", user.id, e);
        } else {
            count += 1;
        }
    }

    Ok(count)
}
