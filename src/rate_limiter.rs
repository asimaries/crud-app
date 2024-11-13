use std::{env, sync::Arc, time::Duration};

use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::{Client, Commands, RedisError};

#[derive(Clone)]
pub struct RateLimiter {
    client: Arc<redis::Client>,
    requests_per_minute: u32,
    window_size: Duration,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32, window_size: Duration) -> Self {
        Self {
            client: Arc::new(
                Client::open(format!(
                    "redis://{}/",
                    env::var("REDIS_HOST").unwrap_or("0.0.0.0".to_string())
                ))
                .expect("Redis not avialable"),
            ),
            requests_per_minute,
            window_size,
        }
    }

    async fn check_rate_limit(&self, key: &str) -> Result<(bool, f64), RedisError> {
        let mut con = self.client.get_connection()?;
        let count: Result<u32, RedisError> = con.get(key);

        match count {
            Err(e) => {
                println!("{}", e);
                let _: String = con.set_ex(key, 1, self.window_size.as_secs())?;
            }
            Ok(count) => {
                if count >= self.requests_per_minute {
                    let ex: f64 = con.expire_time(key)?;
                    return Ok((false, ex));
                } else {
                    let _: u32 = con.incr(key, 1)?;
                }
            }
        }
        Ok((true, 0.0))
    }
}
pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .map(|x| x.to_str())
        .unwrap_or(request.headers().get("host").map(|x| x.to_str()).unwrap())
        .unwrap_or("unknown");
    // LOGGER
    println!("{:?}", request.headers());
    println!("{:?}", request.body());
    println!("{:?}", request.method());
    println!("{:?}", ip);

    let key = format!("{}:rate_limit", ip);

    match limiter.check_rate_limit(&key).await {
        Ok((false, expiry)) => {
            println!("Rate limit exceeded for IP: {}", ip);
            Err(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!(
                    "{{\"message\": \"Too many requests, please try after {} secs\"}}",
                    expiry
                )))
                .expect("Failed to build response"))
        }
        Ok((true, _)) => Ok(next.run(request).await),
        Err(e) => {
            println!("Redis error: {}", e);
            Ok(next.run(request).await)
        }
    }
}
