//! Integration tests for authentication endpoints.
//!
//! These tests require a running PostgreSQL database with all migrations applied.
//! Run with: `cargo test -p server --features server --test auth_tests`

#![cfg(feature = "server")]

mod common;

use axum::http::StatusCode;
use common::{
    get, post_json, post_json_with_auth, put_json_with_auth, register_test_user, test_app_with_auth,
};
use shared_types::{AppError, AuthResponse};

/// Generate a unique username + email pair for test isolation.
fn unique_suffix(prefix: &str) -> (String, String) {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let username = format!("{prefix}_{ts}_{id}");
    let email = format!("{prefix}_{ts}_{id}@test.com");
    (username, email)
}

#[tokio::test]
async fn register_via_rest_creates_user() {
    let app = test_app_with_auth().await;
    let (username, email) = unique_suffix("reg");

    let (status, body) = register_test_user(&app, &username, &email, "StrongPass123!").await;

    assert_eq!(status, StatusCode::CREATED);
    let resp: AuthResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(resp.user.email, email);
    assert!(!resp.access_token.is_empty());
}

#[tokio::test]
async fn login_via_rest_returns_tokens() {
    let app = test_app_with_auth().await;
    let (username, email) = unique_suffix("login");

    // Register first
    register_test_user(&app, &username, &email, "MyPass99!").await;

    // Login
    let login_json = serde_json::json!({
        "email": email,
        "password": "MyPass99!"
    });
    let (status, body) = post_json(&app, "/api/v1/auth/login", &login_json.to_string()).await;

    assert_eq!(status, StatusCode::OK);
    let resp: AuthResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(resp.user.email, email);
    assert!(!resp.access_token.is_empty());
}

#[tokio::test]
async fn login_wrong_password_returns_401() {
    let app = test_app_with_auth().await;
    let (username, email) = unique_suffix("wrongpw");

    register_test_user(&app, &username, &email, "RealPass1!").await;

    let login_json = serde_json::json!({
        "email": email,
        "password": "WrongPass!"
    });
    let (status, body) = post_json(&app, "/api/v1/auth/login", &login_json.to_string()).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let err: AppError = serde_json::from_str(&body).unwrap();
    assert_eq!(err.kind, shared_types::AppErrorKind::Unauthorized);
}

#[tokio::test]
async fn login_nonexistent_email_returns_401() {
    let app = test_app_with_auth().await;

    let login_json = serde_json::json!({
        "email": "nobody_here@nonexistent.com",
        "password": "anything"
    });
    let (status, _) = post_json(&app, "/api/v1/auth/login", &login_json.to_string()).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn register_duplicate_email_returns_error() {
    let app = test_app_with_auth().await;
    let (username1, email) = unique_suffix("dupe");
    let (username2, _) = unique_suffix("dupe2");

    // First registration succeeds
    let (status, _) = register_test_user(&app, &username1, &email, "Pass1234!").await;
    assert_eq!(status, StatusCode::CREATED);

    // Second registration with same email fails (unique constraint → 409 Conflict)
    let (status, _) = register_test_user(&app, &username2, &email, "Pass5678!").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn logout_returns_ok() {
    let app = test_app_with_auth().await;
    let (username, email) = unique_suffix("logout");

    // Register to get a token
    let (_, body) = register_test_user(&app, &username, &email, "LogoutPass1!").await;
    let resp: AuthResponse = serde_json::from_str(&body).unwrap();

    // Logout with Bearer token
    let (status, _) =
        post_json_with_auth(&app, "/api/v1/auth/logout", "{}", &resp.access_token).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn update_tier_without_auth_returns_401() {
    let app = test_app_with_auth().await;

    let (status, _) = put_json_with_auth(
        &app,
        "/api/v1/users/1/tier",
        r#"{"tier":"pro"}"#,
        "not-a-valid-token",
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn update_tier_non_admin_returns_403() {
    let app = test_app_with_auth().await;
    let (username, email) = unique_suffix("tieruser");

    // Register a regular (non-admin) user
    let (_, body) = register_test_user(&app, &username, &email, "TierPass1!").await;
    let resp: AuthResponse = serde_json::from_str(&body).unwrap();

    // Try to update tier — should be forbidden since user role is "user", not "admin"
    let (status, body) = put_json_with_auth(
        &app,
        &format!("/api/v1/users/{}/tier", resp.user.id),
        r#"{"tier":"pro"}"#,
        &resp.access_token,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    let err: AppError = serde_json::from_str(&body).unwrap();
    assert_eq!(err.kind, shared_types::AppErrorKind::Forbidden);
}

#[tokio::test]
async fn health_includes_version() {
    let app = test_app_with_auth().await;
    let (status, body) = get(&app, "/health").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("\"version\""));
    assert!(body.contains("\"uptime_seconds\""));
}
