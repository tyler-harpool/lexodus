use axum::extract::State;
use axum::Json;
use serde::Serialize;
use sqlx::{Pool, Postgres};
use std::sync::OnceLock;
use std::time::Instant;

static START_TIME: OnceLock<Instant> = OnceLock::new();

/// Record the application start time. Call once during startup.
pub fn record_start_time() {
    START_TIME.get_or_init(Instant::now);
}

/// Health check response.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub db: String,
    pub uptime_seconds: u64,
    pub version: String,
}

/// Health check handler.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    ),
    tag = "health"
)]
pub async fn health_check(State(pool): State<Pool<Postgres>>) -> Json<HealthResponse> {
    let db_status = match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await
    {
        Ok(_) => "connected".to_string(),
        Err(e) => format!("error: {e}"),
    };

    let uptime = START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0);

    Json(HealthResponse {
        status: "ok".to_string(),
        db: db_status,
        uptime_seconds: uptime,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
