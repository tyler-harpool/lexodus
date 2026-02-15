use axum::http::StatusCode;
use crate::common;

#[tokio::test]
async fn test_update_attorney_success() {
    let (app, _pool, _guard) = common::test_app().await;
    let id = common::create_test_attorney(&app, "district9", "UPD001").await;
    let update = r#"{"first_name": "Updated"}"#;
    let (status, response) = common::put_json(&app, &format!("/api/attorneys/{}", id), update, "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["first_name"], "Updated");
    assert_eq!(response["bar_number"], "UPD001");
}

#[tokio::test]
async fn test_update_attorney_add_optional() {
    let (app, _pool, _guard) = common::test_app().await;
    let id = common::create_test_attorney(&app, "district9", "UPD002").await;
    let update = r#"{"middle_name": "Added", "firm_name": "New Firm"}"#;
    let (status, response) = common::put_json(&app, &format!("/api/attorneys/{}", id), update, "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["middle_name"], "Added");
    assert_eq!(response["firm_name"], "New Firm");
}

#[tokio::test]
async fn test_update_attorney_not_found() {
    let (app, _pool, _guard) = common::test_app().await;
    let update = r#"{"first_name": "Ghost"}"#;
    let (status, _) = common::put_json(&app, "/api/attorneys/00000000-0000-0000-0000-000000000000", update, "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_attorney_invalid_email() {
    let (app, _pool, _guard) = common::test_app().await;
    let id = common::create_test_attorney(&app, "district9", "UPD003").await;
    let update = r#"{"email": "bad-email"}"#;
    let (status, _) = common::put_json(&app, &format!("/api/attorneys/{}", id), update, "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_attorney_missing_header() {
    let (app, _pool, _guard) = common::test_app().await;
    let req = axum::http::Request::builder()
        .method("PUT")
        .uri("/api/attorneys/00000000-0000-0000-0000-000000000000")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"first_name":"X"}"#))
        .unwrap();
    let response = tower::ServiceExt::oneshot(app.clone(), req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_attorney_duplicate_bar_number() {
    let (app, _pool, _guard) = common::test_app().await;
    common::create_test_attorney(&app, "district9", "UPDDUP1").await;
    let id2 = common::create_test_attorney(&app, "district9", "UPDDUP2").await;
    let update = r#"{"bar_number": "UPDDUP1"}"#;
    let (status, _) = common::put_json(&app, &format!("/api/attorneys/{}", id2), update, "district9").await;
    assert!(status == StatusCode::CONFLICT || status == StatusCode::BAD_REQUEST,
        "Should return 409 or 400 for duplicate bar number, got {}", status);
}
