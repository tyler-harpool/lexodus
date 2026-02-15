use axum::http::StatusCode;

use crate::common::{
    create_test_case, create_test_docket_entry, post_json, post_no_court, test_app,
};

#[tokio::test]
async fn create_attachment_returns_presign_info() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "ATT-CREATE-001").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();

    let body = serde_json::json!({
        "file_name": "motion_to_dismiss.pdf",
        "content_type": "application/pdf",
        "file_size": 54321,
    });

    let uri = format!("/api/docket/entries/{}/attachments", entry_id);
    let (status, resp) = post_json(&app, &uri, &body.to_string(), "district9").await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(resp["attachment_id"].is_string(), "should have attachment_id");
    assert!(resp["presign_url"].is_string(), "should have presign_url");
    assert!(resp["object_key"].is_string(), "should have object_key");

    // Verify object_key structure
    let key = resp["object_key"].as_str().unwrap();
    assert!(key.starts_with("district9/docket/"), "key should start with court/docket/");
    assert!(key.ends_with("motion_to_dismiss.pdf"), "key should end with filename");
}

#[tokio::test]
async fn create_attachment_requires_sse_header() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "ATT-CREATE-002").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "exhibit").await;
    let entry_id = entry["id"].as_str().unwrap();

    let body = serde_json::json!({
        "file_name": "exhibit_a.pdf",
        "content_type": "application/pdf",
        "file_size": 10000,
    });

    let uri = format!("/api/docket/entries/{}/attachments", entry_id);
    let (status, resp) = post_json(&app, &uri, &body.to_string(), "district9").await;

    assert_eq!(status, StatusCode::CREATED);

    let headers = resp["required_headers"].as_object().expect("should have required_headers");
    assert_eq!(
        headers.get("x-amz-server-side-encryption").and_then(|v| v.as_str()),
        Some("AES256"),
        "required_headers must contain x-amz-server-side-encryption = AES256"
    );
}

#[tokio::test]
async fn create_attachment_empty_filename_returns_400() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "ATT-CREATE-003").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();

    let body = serde_json::json!({
        "file_name": "",
        "content_type": "application/pdf",
        "file_size": 100,
    });

    let uri = format!("/api/docket/entries/{}/attachments", entry_id);
    let (status, _resp) = post_json(&app, &uri, &body.to_string(), "district9").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_attachment_invalid_entry_id_returns_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "file_name": "test.pdf",
        "content_type": "application/pdf",
        "file_size": 100,
    });

    let (status, resp) = post_json(
        &app,
        "/api/docket/entries/not-a-uuid/attachments",
        &body.to_string(),
        "district9",
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    let msg = resp["message"].as_str().unwrap_or_default();
    assert!(msg.contains("UUID"), "error should mention UUID: {}", msg);
}

#[tokio::test]
async fn create_attachment_missing_court_header_returns_400() {
    let (app, _pool, _guard) = test_app().await;
    let fake_uuid = uuid::Uuid::new_v4();

    let body = serde_json::json!({
        "file_name": "test.pdf",
        "content_type": "application/pdf",
        "file_size": 100,
    });

    let uri = format!("/api/docket/entries/{}/attachments", fake_uuid);
    let (status, _resp) = post_no_court(&app, &uri, &body.to_string()).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_attachment_nonexistent_entry_returns_404() {
    let (app, _pool, _guard) = test_app().await;
    let fake_entry_id = uuid::Uuid::new_v4();

    let body = serde_json::json!({
        "file_name": "test.pdf",
        "content_type": "application/pdf",
        "file_size": 100,
    });

    let uri = format!("/api/docket/entries/{}/attachments", fake_entry_id);
    let (status, _resp) = post_json(&app, &uri, &body.to_string(), "district9").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
