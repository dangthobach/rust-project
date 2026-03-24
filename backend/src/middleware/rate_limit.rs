// Note: tower-governor middleware is simpler to use with Axum
// We'll use a basic in-memory rate limiter for now

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<Vec<Instant>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(Vec::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    pub async fn check(&self) -> bool {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();

        // Remove old requests outside the window
        requests.retain(|&time| now.duration_since(time) < self.window);

        // Check if we're under the limit
        if requests.len() < self.max_requests {
            requests.push(now);
            true
        } else {
            false
        }
    }
}

/// General rate limiter: 100 requests per minute
pub fn create_general_limiter() -> RateLimiter {
    RateLimiter::new(100, 60)
}

/// Auth rate limiter: 5 requests per minute (brute force protection)
pub fn create_auth_limiter() -> RateLimiter {
    RateLimiter::new(5, 60)
}

/// Upload rate limiter: 10 requests per minute
pub fn create_upload_limiter() -> RateLimiter {
    RateLimiter::new(10, 60)
}

/// Middleware to apply rate limiting
pub async fn apply_rate_limit(
    limiter: RateLimiter,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if limiter.check().await {
        Ok(next.run(request).await)
    } else {
        tracing::warn!("Rate limit exceeded");
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}
