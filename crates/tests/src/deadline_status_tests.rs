use axum::http::StatusCode;

use crate::common::{test_app, patch_json, get_with_court, create_test_deadline};

#[tokio::test]
async fn update_deadline_status_to_met() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_deadline(&app, "district9", "Status test").await;
    let id = created["id"].as_str().unwrap();
    assert_eq!(created["status"], "open");

    let body = serde_json::json!({ "status": "met" });
    let (status, resp) = patch_json(
        &app,
        &format!("/api/deadlines/{}/status", id),
        &body.to_string(),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["status"], "met");

    // Verify persisted
    let (_, fetched) = get_with_court(&app, &format!("/api/deadlines/{}", id), "district9").await;
    assert_eq!(fetched["status"], "met");
}

#[tokio::test]
async fn update_deadline_status_all_valid_values() {
    let (app, _pool, _guard) = test_app().await;

    for status_val in &["met", "extended", "cancelled", "expired", "open"] {
        let created = create_test_deadline(&app, "district9", &format!("Status {}", status_val)).await;
        let id = created["id"].as_str().unwrap();

        let body = serde_json::json!({ "status": status_val });
        let (status, resp) = patch_json(
            &app,
            &format!("/api/deadlines/{}/status", id),
            &body.to_string(),
            "district9",
        )
        .await;
        assert_eq!(status, StatusCode::OK, "Failed for status: {}", status_val);
        assert_eq!(resp["status"], *status_val);
    }
}

#[tokio::test]
async fn update_deadline_status_invalid_rejected() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_deadline(&app, "district9", "Bad status test").await;
    let id = created["id"].as_str().unwrap();

    let body = serde_json::json!({ "status": "invalid_status" });
    let (status, _) = patch_json(
        &app,
        &format!("/api/deadlines/{}/status", id),
        &body.to_string(),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_deadline_status_not_found() {
    let (app, _pool, _guard) = test_app().await;

    let fake_id = "00000000-0000-0000-0000-000000000099";
    let body = serde_json::json!({ "status": "met" });
    let (status, _) = patch_json(
        &app,
        &format!("/api/deadlines/{}/status", fake_id),
        &body.to_string(),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_deadline_status_wrong_court() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_deadline(&app, "district9", "Court 9 status").await;
    let id = created["id"].as_str().unwrap();

    let body = serde_json::json!({ "status": "met" });
    let (status, _) = patch_json(
        &app,
        &format!("/api/deadlines/{}/status", id),
        &body.to_string(),
        "district12",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
