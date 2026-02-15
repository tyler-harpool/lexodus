use axum::extract::FromRef;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

/// Shared application state passed to Axum handlers via `State`.
/// Derives `FromRef` so handlers can extract `State<PgPool>` directly.
#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: Pool<Postgres>,
}

/// Pool created lazily — no connections are opened until the first query.
/// This avoids binding to a specific tokio runtime at init time, which is
/// critical for tests where each `#[tokio::test]` creates its own runtime.
static POOL: OnceLock<Pool<Postgres>> = OnceLock::new();
static MIGRATED: AtomicBool = AtomicBool::new(false);

/// Create a new database connection pool from environment variables.
/// Uses `connect_lazy` so no connections open until the first query.
pub fn create_pool() -> Pool<Postgres> {
    // Load .env file if present (ignored in production where env vars are set directly).
    let _ = dotenvy::dotenv();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let max_connections: u32 = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);

    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_lazy(&database_url)
        .expect("Failed to create database pool")
}

/// Run database migrations against the given pool.
pub async fn run_migrations(pool: &Pool<Postgres>) {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .expect("Failed to run database migrations");
}

/// Get or initialize the database connection pool.
/// Migrations run once on the first call; subsequent calls return immediately.
///
/// Used by Dioxus server functions (`api.rs`) which share a single long-lived runtime.
/// REST handlers use `State<PgPool>` from `AppState` instead.
pub async fn get_db() -> &'static Pool<Postgres> {
    let pool = POOL.get_or_init(create_pool);

    // Run migrations at most once per process. `swap` is atomic so only
    // the first caller executes; migrations are idempotent regardless.
    if !MIGRATED.swap(true, Ordering::SeqCst) {
        run_migrations(pool).await;
    }

    pool
}
