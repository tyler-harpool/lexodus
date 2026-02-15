//! Integration tests for REST API endpoints.
//!
//! These tests require a running PostgreSQL database with migrations applied.
//! Run with: `cargo test -p server --features server --test api_tests`

#![cfg(feature = "server")]

mod common;

use axum::http::StatusCode;
use common::{delete, get, post_json, put_json, test_app};
use shared_types::{AppError, Product, User};

#[tokio::test]
async fn health_check_returns_ok() {
    let app = test_app().await;
    let (status, body) = get(&app, "/health").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("\"status\":\"ok\""));
    assert!(body.contains("\"db\":\"connected\""));
}

#[tokio::test]
async fn create_and_get_user() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let username = format!("testuser_api_{ts}");

    let app = test_app().await;

    // Create a user
    let json = serde_json::json!({
        "username": username,
        "display_name": "Test User"
    });
    let (status, body) = post_json(&app, "/api/v1/users", &json.to_string()).await;
    assert_eq!(status, StatusCode::CREATED);

    let user: User = serde_json::from_str(&body).unwrap();
    assert_eq!(user.username, username);
    assert_eq!(user.display_name, "Test User");

    // Get the user by ID
    let (status, body) = get(&app, &format!("/api/v1/users/{}", user.id)).await;
    assert_eq!(status, StatusCode::OK);

    let fetched: User = serde_json::from_str(&body).unwrap();
    assert_eq!(fetched.id, user.id);

    // Clean up
    let (status, _) = delete(&app, &format!("/api/v1/users/{}", user.id)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn list_users() {
    let app = test_app().await;
    let (status, body) = get(&app, "/api/v1/users").await;

    assert_eq!(status, StatusCode::OK);
    let _users: Vec<User> = serde_json::from_str(&body).unwrap();
}

#[tokio::test]
async fn update_user() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let username = format!("update_test_{ts}");

    let app = test_app().await;

    // Create
    let create_json = serde_json::json!({
        "username": username,
        "display_name": "Before"
    });
    let (_, body) = post_json(&app, "/api/v1/users", &create_json.to_string()).await;
    let user: User = serde_json::from_str(&body).unwrap();

    // Update
    let update_json = serde_json::json!({
        "username": username,
        "display_name": "After"
    });
    let (status, body) = put_json(
        &app,
        &format!("/api/v1/users/{}", user.id),
        &update_json.to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let updated: User = serde_json::from_str(&body).unwrap();
    assert_eq!(updated.display_name, "After");

    // Clean up
    delete(&app, &format!("/api/v1/users/{}", user.id)).await;
}

#[tokio::test]
async fn get_nonexistent_user_returns_404() {
    let app = test_app().await;
    let (status, body) = get(&app, "/api/v1/users/999999").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    let err: AppError = serde_json::from_str(&body).unwrap();
    assert_eq!(err.kind, shared_types::AppErrorKind::NotFound);
}

#[tokio::test]
async fn delete_nonexistent_user_returns_404() {
    let app = test_app().await;
    let (status, _) = delete(&app, "/api/v1/users/999999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_and_get_product() {
    let app = test_app().await;

    let (status, body) = post_json(
        &app,
        "/api/v1/products",
        r#"{"name":"Test Widget","description":"A test product","price":29.99,"category":"Hardware","status":"active"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let product: Product = serde_json::from_str(&body).unwrap();
    assert_eq!(product.name, "Test Widget");
    assert_eq!(product.price, 29.99);

    // Clean up
    delete(&app, &format!("/api/v1/products/{}", product.id)).await;
}

#[tokio::test]
async fn list_products() {
    let app = test_app().await;
    let (status, body) = get(&app, "/api/v1/products").await;

    assert_eq!(status, StatusCode::OK);
    let _products: Vec<Product> = serde_json::from_str(&body).unwrap();
}

#[tokio::test]
async fn update_product() {
    let app = test_app().await;

    // Create
    let (_, body) = post_json(
        &app,
        "/api/v1/products",
        r#"{"name":"Update Me","description":"desc","price":10.0,"category":"Hardware","status":"active"}"#,
    )
    .await;
    let product: Product = serde_json::from_str(&body).unwrap();

    // Update
    let (status, body) = put_json(
        &app,
        &format!("/api/v1/products/{}", product.id),
        r#"{"name":"Updated Name","description":"new desc","price":20.0,"category":"Software","status":"inactive"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let updated: Product = serde_json::from_str(&body).unwrap();
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.price, 20.0);

    // Clean up
    delete(&app, &format!("/api/v1/products/{}", product.id)).await;
}

#[tokio::test]
async fn delete_nonexistent_product_returns_404() {
    let app = test_app().await;
    let (status, _) = delete(&app, "/api/v1/products/999999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn dashboard_stats() {
    let app = test_app().await;
    let (status, body) = get(&app, "/api/v1/dashboard/stats").await;

    assert_eq!(status, StatusCode::OK);
    let stats: shared_types::DashboardStats = serde_json::from_str(&body).unwrap();
    assert!(stats.total_users >= 0);
    assert!(stats.total_products >= 0);
}

#[tokio::test]
async fn validation_rejects_short_username() {
    let app = test_app().await;

    let (status, body) = post_json(
        &app,
        "/api/v1/users",
        r#"{"username":"ab","display_name":"Valid Name"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let err: AppError = serde_json::from_str(&body).unwrap();
    assert_eq!(err.kind, shared_types::AppErrorKind::ValidationError);
    assert!(err.field_errors.contains_key("username"));
}

#[tokio::test]
async fn validation_rejects_empty_display_name() {
    let app = test_app().await;

    let (status, body) = post_json(
        &app,
        "/api/v1/users",
        r#"{"username":"validuser","display_name":""}"#,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let err: AppError = serde_json::from_str(&body).unwrap();
    assert!(err.field_errors.contains_key("display_name"));
}

#[tokio::test]
async fn validation_rejects_negative_price() {
    let app = test_app().await;

    let (status, body) = post_json(
        &app,
        "/api/v1/products",
        r#"{"name":"Widget","description":"desc","price":-5.0,"category":"Hardware","status":"active"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let err: AppError = serde_json::from_str(&body).unwrap();
    assert!(err.field_errors.contains_key("price"));
}

#[tokio::test]
async fn update_nonexistent_user_returns_404() {
    let app = test_app().await;

    let (status, body) = put_json(
        &app,
        "/api/v1/users/999999",
        r#"{"username":"nobody","display_name":"Ghost"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let err: AppError = serde_json::from_str(&body).unwrap();
    assert_eq!(err.kind, shared_types::AppErrorKind::NotFound);
}

#[tokio::test]
async fn create_product_empty_name_returns_422() {
    let app = test_app().await;

    let (status, body) = post_json(
        &app,
        "/api/v1/products",
        r#"{"name":"","description":"desc","price":10.0,"category":"Hardware","status":"active"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let err: AppError = serde_json::from_str(&body).unwrap();
    assert_eq!(err.kind, shared_types::AppErrorKind::ValidationError);
    assert!(err.field_errors.contains_key("name"));
}

#[tokio::test]
async fn update_nonexistent_product_returns_404() {
    let app = test_app().await;

    let (status, body) = put_json(
        &app,
        "/api/v1/products/999999",
        r#"{"name":"Ghost","description":"nope","price":1.0,"category":"None","status":"active"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let err: AppError = serde_json::from_str(&body).unwrap();
    assert_eq!(err.kind, shared_types::AppErrorKind::NotFound);
}

#[tokio::test]
async fn delete_user_actually_removes() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let username = format!("delete_me_{ts}");

    let app = test_app().await;

    // Create
    let json = serde_json::json!({
        "username": username,
        "display_name": "Delete Me"
    });
    let (_, body) = post_json(&app, "/api/v1/users", &json.to_string()).await;
    let user: User = serde_json::from_str(&body).unwrap();

    // Delete
    let (status, _) = delete(&app, &format!("/api/v1/users/{}", user.id)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify gone
    let (status, _) = get(&app, &format!("/api/v1/users/{}", user.id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_product_actually_removes() {
    let app = test_app().await;

    // Create
    let (_, body) = post_json(
        &app,
        "/api/v1/products",
        r#"{"name":"Deletable Widget","description":"bye","price":5.0,"category":"Hardware","status":"active"}"#,
    )
    .await;
    let product: Product = serde_json::from_str(&body).unwrap();

    // Delete
    let (status, _) = delete(&app, &format!("/api/v1/products/{}", product.id)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify deleted — listing should not contain it
    let (_, body) = get(&app, "/api/v1/products").await;
    let products: Vec<Product> = serde_json::from_str(&body).unwrap();
    assert!(!products.iter().any(|p| p.id == product.id));
}
