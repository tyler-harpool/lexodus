use axum::http::StatusCode;

use crate::common::{test_app, post_json, create_test_case, create_test_deadline};

#[tokio::test]
async fn create_deadline_minimal() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "File Motion Response",
        "due_at": "2026-07-01T17:00:00Z"
    });

    let (status, resp) = post_json(&app, "/api/deadlines", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["title"], "File Motion Response");
    assert_eq!(resp["status"], "open");
    assert!(resp["id"].as_str().is_some());
    assert!(resp["case_id"].is_null());
    assert!(resp["rule_code"].is_null());
}

#[tokio::test]
async fn create_deadline_with_all_fields() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "DL-2026-001").await;

    let body = serde_json::json!({
        "title": "Response to Government Motion",
        "case_id": case_id,
        "rule_code": "FRCP 12(b)",
        "due_at": "2026-08-15T12:00:00Z",
        "notes": "Must include supporting brief"
    });

    let (status, resp) = post_json(&app, "/api/deadlines", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["title"], "Response to Government Motion");
    assert_eq!(resp["case_id"], case_id);
    assert_eq!(resp["rule_code"], "FRCP 12(b)");
    assert_eq!(resp["notes"], "Must include supporting brief");
    assert_eq!(resp["status"], "open");
}

#[tokio::test]
async fn create_deadline_empty_title_rejected() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "   ",
        "due_at": "2026-07-01T17:00:00Z"
    });

    let (status, _) = post_json(&app, "/api/deadlines", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_deadline_missing_fields_rejected() {
    let (app, _pool, _guard) = test_app().await;

    // Missing due_at
    let body = serde_json::json!({
        "title": "Some deadline"
    });

    let (status, _) = post_json(&app, "/api/deadlines", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_deadline_no_court_header() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "File Motion Response",
        "due_at": "2026-07-01T17:00:00Z"
    });

    let (status, _) = crate::common::post_no_court(&app, "/api/deadlines", &body.to_string()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_multiple_deadlines_returns_different_ids() {
    let (app, _pool, _guard) = test_app().await;

    let dl1 = create_test_deadline(&app, "district9", "Deadline A").await;
    let dl2 = create_test_deadline(&app, "district9", "Deadline B").await;

    assert_ne!(dl1["id"], dl2["id"]);
}
