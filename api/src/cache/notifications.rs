use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;

/// Get notification count from cache
pub async fn get_notification_count_from_cache(
    con: &mut MultiplexedConnection,
    user_id: &str,
) -> Result<Option<i32>, redis::RedisError> {
    let key = format!("notification_count:{}", user_id);
    let count: Option<i32> = con.get(&key).await?;
    Ok(count)
}

/// Set notification count in cache (TTL: 5 minutes)
pub async fn set_notification_count_in_cache(
    con: &mut MultiplexedConnection,
    user_id: &str,
    count: i32,
) -> Result<(), redis::RedisError> {
    let key = format!("notification_count:{}", user_id);
    let _: () = con.set_ex(&key, count, 300).await?; // 5 minutes TTL
    Ok(())
}

/// Increment notification count in cache
pub async fn increment_notification_count_in_cache(
    con: &mut MultiplexedConnection,
    user_id: &str,
) -> Result<i32, redis::RedisError> {
    let key = format!("notification_count:{}", user_id);
    let new_count: i32 = con.incr(&key, 1).await?;
    // Reset TTL when incrementing
    let _: bool = con.expire(&key, 300).await?;
    Ok(new_count)
}

/// Decrement notification count in cache
pub async fn decrement_notification_count_in_cache(
    con: &mut MultiplexedConnection,
    user_id: &str,
) -> Result<i32, redis::RedisError> {
    let key = format!("notification_count:{}", user_id);
    let new_count: i32 = con.decr(&key, 1).await?;
    // Ensure count doesn't go negative
    if new_count < 0 {
        let _: () = con.set(&key, 0).await?;
        Ok(0)
    } else {
        // Reset TTL when decrementing
        let _: bool = con.expire(&key, 300).await?;
        Ok(new_count)
    }
}

/// Clear notification count from cache
pub async fn clear_notification_count_from_cache(
    con: &mut MultiplexedConnection,
    user_id: &str,
) -> Result<(), redis::RedisError> {
    let key = format!("notification_count:{}", user_id);
    let _: () = con.del(&key).await?;
    Ok(())
}

/// Invalidate notification count cache for a user
pub async fn invalidate_notification_count_cache(
    con: &mut MultiplexedConnection,
    user_id: &str,
) -> Result<(), redis::RedisError> {
    clear_notification_count_from_cache(con, user_id).await
}
