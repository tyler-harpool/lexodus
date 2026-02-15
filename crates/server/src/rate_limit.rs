use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Sliding window rate limit state shared across requests.
#[derive(Clone)]
pub struct RateLimitState {
    inner: Arc<Mutex<RateLimitInner>>,
}

struct RateLimitInner {
    /// Map from client key -> list of request timestamps.
    requests: HashMap<String, Vec<Instant>>,
    /// Maximum requests allowed within the window.
    max_requests: u32,
    /// Sliding window duration.
    window: Duration,
}

impl RateLimitState {
    /// Create rate limiter allowing `max_requests` per `window`.
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RateLimitInner {
                requests: HashMap::new(),
                max_requests,
                window,
            })),
        }
    }

    /// Check if a request from `key` is allowed. Returns true if allowed.
    fn check(&self, key: &str) -> bool {
        let mut inner = self.inner.lock().unwrap();
        let now = Instant::now();
        let cutoff = now - inner.window;
        let max = inner.max_requests;

        let timestamps = inner.requests.entry(key.to_string()).or_default();
        timestamps.retain(|t| *t > cutoff);

        if timestamps.len() as u32 >= max {
            return false;
        }

        timestamps.push(now);
        true
    }
}

/// Axum middleware that enforces rate limits per court district.
///
/// The client key is derived from the `X-Court-District` header
/// combined with the remote address (if available), falling back
/// to just the court district.
pub async fn rate_limit_middleware(
    axum::extract::State(state): axum::extract::State<RateLimitState>,
    request: Request,
    next: Next,
) -> Response {
    let key = request
        .headers()
        .get("x-court-district")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    if !state.check(&key) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "kind": "RateLimited",
                "message": "Rate limit exceeded. Please try again later."
            })),
        )
            .into_response();
    }

    next.run(request).await
}

use axum::response::IntoResponse;
