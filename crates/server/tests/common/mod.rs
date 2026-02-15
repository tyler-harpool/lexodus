use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware, Router,
};
use server::db::AppState;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::sync::OnceLock;
use tower::ServiceExt;

/// Tables to truncate before each test run (child tables before parents).
const ALL_TABLES: &str = "device_authorizations, sms_verifications, password_resets, \
    email_verifications, stripe_webhook_events, payments, subscriptions, \
    refresh_tokens, products, users";

/// One-time flag to ensure we only set up the test database once per process.
static INITIALIZED: OnceLock<()> = OnceLock::new();

/// Set up the test database and override DATABASE_URL so all subsequent pool
/// creation uses the `_test` database instead of the main one.
async fn ensure_test_db() {
    let _ = dotenvy::dotenv();
    let original_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Derive test database name
    let (base_url, db_name) = original_url
        .rsplit_once('/')
        .expect("DATABASE_URL must contain a database name");
    let test_db_name = format!("{}_test", db_name);
    let test_url = format!("{}/{}", base_url, test_db_name);

    // Connect to `postgres` to create the test database if needed
    let admin_url = format!("{}/postgres", base_url);
    let admin_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&admin_url)
        .await
        .expect("Failed to connect to postgres admin database");

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(&test_db_name)
            .fetch_one(&admin_pool)
            .await
            .expect("Failed to check for test database");

    if !exists {
        sqlx::query(&format!("CREATE DATABASE \"{}\"", test_db_name))
            .execute(&admin_pool)
            .await
            .expect("Failed to create test database");
    }

    admin_pool.close().await;

    // Point DATABASE_URL to the test database for all future pool creation
    unsafe { std::env::set_var("DATABASE_URL", &test_url) };
}

/// Build a pool connected to the test database.
/// On the first call, creates the database, runs migrations, and truncates all tables.
async fn test_pool() -> Pool<Postgres> {
    if INITIALIZED.get().is_none() {
        ensure_test_db().await;
    }

    // Use the same pool creation as production (connect_lazy)
    let pool = server::db::create_pool();

    // First call: run migrations + truncate
    if INITIALIZED.set(()).is_ok() {
        server::db::run_migrations(&pool).await;

        sqlx::query(&format!("TRUNCATE {} CASCADE", ALL_TABLES))
            .execute(&pool)
            .await
            .expect("Failed to truncate test tables");
    }

    pool
}

#[allow(dead_code)]
/// Build a test router with the REST API routes (no auth middleware).
/// Connects to the dedicated test database.
pub async fn test_app() -> Router {
    let pool = test_pool().await;
    let state = AppState { pool };

    server::rest::rest_router()
        .route("/health", axum::routing::get(server::health::health_check))
        .with_state(state)
}

#[allow(dead_code)]
/// Build a test router with auth middleware enabled.
/// Required for endpoints that use AuthRequired/TierRequired extractors.
pub async fn test_app_with_auth() -> Router {
    let pool = test_pool().await;
    let state = AppState { pool };

    server::rest::rest_router()
        .route("/health", axum::routing::get(server::health::health_check))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            server::auth::middleware::auth_middleware,
        ))
        .with_state(state)
}

#[allow(dead_code)]
/// Register a test user via the REST API and return (status, auth_response_body).
pub async fn register_test_user(
    app: &Router,
    username: &str,
    email: &str,
    password: &str,
) -> (StatusCode, String) {
    let json = serde_json::json!({
        "username": username,
        "email": email,
        "password": password,
        "display_name": format!("Test {}", username)
    });
    post_json(app, "/api/v1/auth/register", &json.to_string()).await
}

#[allow(dead_code)]
/// Helper to make a GET request and return (status, body).
pub async fn get(app: &Router, uri: &str) -> (StatusCode, String) {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

#[allow(dead_code)]
/// Helper to make a POST request with JSON body.
pub async fn post_json(app: &Router, uri: &str, json: &str) -> (StatusCode, String) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(json.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

#[allow(dead_code)]
/// Helper to make a PUT request with JSON body.
pub async fn put_json(app: &Router, uri: &str, json: &str) -> (StatusCode, String) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(json.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

#[allow(dead_code)]
/// Helper to make a PUT request with JSON body and Bearer auth.
pub async fn put_json_with_auth(
    app: &Router,
    uri: &str,
    json: &str,
    token: &str,
) -> (StatusCode, String) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(uri)
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(json.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

#[allow(dead_code)]
/// Helper to make a POST request with JSON body and Bearer auth.
pub async fn post_json_with_auth(
    app: &Router,
    uri: &str,
    json: &str,
    token: &str,
) -> (StatusCode, String) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(json.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

#[allow(dead_code)]
/// Helper to make a DELETE request.
pub async fn delete(app: &Router, uri: &str) -> (StatusCode, String) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}
