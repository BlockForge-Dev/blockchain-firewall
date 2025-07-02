use redis::AsyncCommands;
use redis::Client;
use std::net::IpAddr;
use tracing::info;
use axum::http::StatusCode;

const RATE_LIMIT_MAX: usize = 9;
const WINDOW_SECONDS: usize = 60;

pub async fn check_rate_limit(redis_client: &Client, ip: IpAddr) -> Result<(), StatusCode> {
    let key = format!("rate_limit:{}", ip);
    info!("🔑 Using Redis key: {}", key); // ✅ shows what key is actually used

    let mut conn = match redis_client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(err) => {
            tracing::error!("❌ Redis connection error: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Increment count
    let count: usize = match conn.incr(&key, 1).await {
        Ok(c) => c,
        Err(err) => {
            tracing::error!("❌ Redis INCR error: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Set TTL if it's the first time
  if count == 1 {
    let expire_result: redis::RedisResult<bool> =
        conn.expire(&key, WINDOW_SECONDS.try_into().unwrap()).await;

    if let Err(err) = expire_result {
        tracing::error!("❌ Redis EXPIRE error: {:?}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
}


    info!("📊 IP {} has made {} requests", ip, count);

    if count > RATE_LIMIT_MAX {
        info!("🚫 Rate limit exceeded for IP: {}", ip);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(())
}
