use axum::http::StatusCode;

use crate::common::{test_app, put_json, create_test_deadline, create_test_case};

#[tokio::test]
async fn update_deadline_title() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_deadline(&app, "district9", "Original Title").await;
    let id = created["id"].as_str().unwrap();

    let body = serde_json::json!({ "title": "Updated Title" });
    let (status, resp) = put_json(&app, &format!("/api/deadlines/{}", id), &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["title"], "Updated Title");
}

#[tokio::test]
async fn update_deadline_multiple_fields() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "DL-UPD-001").await;

    let created = create_test_deadline(&app, "district9", "File Brief").await;
    let id = created["id"].as_str().unwrap();

    let body = serde_json::json!({
        "title": "Amended Brief",
        "case_id": case_id,
        "rule_code": "FRCP 15(a)",
        "notes": "Updated notes"
    });
    let (status, resp) = put_json(&app, &format!("/api/deadlines/{}", id), &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["title"], "Amended Brief");
    assert_eq!(resp["case_id"], case_id);
    assert_eq!(resp["rule_code"], "FRCP 15(a)");
    assert_eq!(resp["notes"], "Updated notes");
}

#[tokio::test]
async fn update_deadline_not_found() {
    let (app, _pool, _guard) = test_app().await;

    let fake_id = "00000000-0000-0000-0000-000000000099";
    let body = serde_json::json!({ "title": "Doesn't matter" });
    let (status, _) = put_json(&app, &format!("/api/deadlines/{}", fake_id), &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_deadline_preserves_unchanged_fields() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_deadline(&app, "district9", "Original").await;
    let id = created["id"].as_str().unwrap();

    // Only update notes, title should stay
    let body = serde_json::json!({ "notes": "Just a note" });
    let (status, resp) = put_json(&app, &format!("/api/deadlines/{}", id), &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["title"], "Original");
    assert_eq!(resp["notes"], "Just a note");
}

#[tokio::test]
async fn update_deadline_wrong_court_returns_not_found() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_deadline(&app, "district9", "Court 9 DL").await;
    let id = created["id"].as_str().unwrap();

    let body = serde_json::json!({ "title": "Hijack" });
    let (status, _) = put_json(&app, &format!("/api/deadlines/{}", id), &body.to_string(), "district12").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
