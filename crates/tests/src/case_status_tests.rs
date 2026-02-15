use axum::http::StatusCode;

use crate::common::{test_app, patch_json, create_test_case_via_api};

#[tokio::test]
async fn update_status_success() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_case_via_api(&app, "district9", "Status Test Case").await;
    let id = created["id"].as_str().unwrap();
    assert_eq!(created["status"], "filed");

    let body = serde_json::json!({ "status": "arraigned" });
    let (status, resp) = patch_json(
        &app,
        &format!("/api/cases/{}/status", id),
        &body.to_string(),
        "district9",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["status"], "arraigned");
    assert_eq!(resp["id"], id);
}

#[tokio::test]
async fn update_status_all_valid_values() {
    let (app, _pool, _guard) = test_app().await;

    let valid_statuses = [
        "filed", "arraigned", "discovery", "pretrial_motions", "plea_negotiations",
        "trial_ready", "in_trial", "awaiting_sentencing", "sentenced", "dismissed", "on_appeal",
    ];

    for target_status in valid_statuses {
        let created = create_test_case_via_api(
            &app,
            "district9",
            &format!("Status {}", target_status),
        )
        .await;
        let id = created["id"].as_str().unwrap();

        let body = serde_json::json!({ "status": target_status });
        let (status, resp) = patch_json(
            &app,
            &format!("/api/cases/{}/status", id),
            &body.to_string(),
            "district9",
        )
        .await;

        assert_eq!(status, StatusCode::OK, "Failed for status: {}", target_status);
        assert_eq!(resp["status"], target_status);
    }
}

#[tokio::test]
async fn update_status_invalid_value_400() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_case_via_api(&app, "district9", "Invalid Status Case").await;
    let id = created["id"].as_str().unwrap();

    let body = serde_json::json!({ "status": "bogus_status" });
    let (status, resp) = patch_json(
        &app,
        &format!("/api/cases/{}/status", id),
        &body.to_string(),
        "district9",
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(resp["message"].as_str().unwrap().contains("Invalid status"));
}

#[tokio::test]
async fn update_status_not_found_404() {
    let (app, _pool, _guard) = test_app().await;

    let fake_id = uuid::Uuid::new_v4();
    let body = serde_json::json!({ "status": "arraigned" });
    let (status, _) = patch_json(
        &app,
        &format!("/api/cases/{}/status", fake_id),
        &body.to_string(),
        "district9",
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_status_invalid_uuid_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({ "status": "arraigned" });
    let (status, _) = patch_json(
        &app,
        "/api/cases/not-a-uuid/status",
        &body.to_string(),
        "district9",
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_status_updates_timestamp() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_case_via_api(&app, "district9", "Timestamp Test Case").await;
    let id = created["id"].as_str().unwrap();
    let original_updated = created["updated_at"].as_str().unwrap().to_string();

    // Small delay to ensure timestamp changes
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let body = serde_json::json!({ "status": "discovery" });
    let (status, resp) = patch_json(
        &app,
        &format!("/api/cases/{}/status", id),
        &body.to_string(),
        "district9",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let new_updated = resp["updated_at"].as_str().unwrap();
    assert_ne!(new_updated, original_updated);
}
