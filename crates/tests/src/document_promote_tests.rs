use axum::http::StatusCode;

use crate::common::{
    create_test_case, create_uploaded_attachment, create_pending_attachment,
    post_json, post_no_court, test_app,
};

#[tokio::test]
async fn promote_attachment_returns_201() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-PROM-001").await;
    let entry = crate::common::create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();
    let att_id = create_uploaded_attachment(&pool, "district9", entry_id).await;

    let body = serde_json::json!({
        "docket_attachment_id": att_id,
        "title": "Filed Motion",
        "document_type": "Motion"
    });

    let (status, response) = post_json(
        &app,
        "/api/documents/from-attachment",
        &body.to_string(),
        "district9",
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "body: {:?}", response);
    assert!(response["id"].is_string());
    assert_eq!(response["title"], "Filed Motion");
    assert_eq!(response["document_type"], "Motion");
    assert_eq!(response["source_attachment_id"], att_id);
    assert_eq!(response["court_id"], "district9");
}

#[tokio::test]
async fn promote_attachment_defaults_title_and_type() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-PROM-002").await;
    let entry = crate::common::create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();
    let att_id = create_uploaded_attachment(&pool, "district9", entry_id).await;

    let body = serde_json::json!({
        "docket_attachment_id": att_id,
    });

    let (status, response) = post_json(
        &app,
        "/api/documents/from-attachment",
        &body.to_string(),
        "district9",
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "body: {:?}", response);
    // Title defaults to attachment filename
    assert_eq!(response["title"], "test-file.pdf");
    // document_type defaults to "Other"
    assert_eq!(response["document_type"], "Other");
}

#[tokio::test]
async fn promote_attachment_idempotent() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-PROM-003").await;
    let entry = crate::common::create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();
    let att_id = create_uploaded_attachment(&pool, "district9", entry_id).await;

    let body = serde_json::json!({
        "docket_attachment_id": att_id,
        "title": "Filed Motion",
        "document_type": "Motion"
    });

    // First call: 201
    let (status1, response1) = post_json(
        &app,
        "/api/documents/from-attachment",
        &body.to_string(),
        "district9",
    )
    .await;
    assert_eq!(status1, StatusCode::CREATED);

    // Second call: 200 with same document
    let (status2, response2) = post_json(
        &app,
        "/api/documents/from-attachment",
        &body.to_string(),
        "district9",
    )
    .await;
    assert_eq!(status2, StatusCode::OK);
    assert_eq!(response1["id"], response2["id"]);
}

#[tokio::test]
async fn promote_attachment_missing_court_returns_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "docket_attachment_id": "00000000-0000-0000-0000-000000000001",
    });

    let (status, _) = post_no_court(
        &app,
        "/api/documents/from-attachment",
        &body.to_string(),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn promote_attachment_not_uploaded_returns_400() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-PROM-004").await;
    let entry = crate::common::create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();
    let att_id = create_pending_attachment(&pool, "district9", entry_id).await;

    let body = serde_json::json!({
        "docket_attachment_id": att_id,
    });

    let (status, response) = post_json(
        &app,
        "/api/documents/from-attachment",
        &body.to_string(),
        "district9",
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {:?}", response);
}

#[tokio::test]
async fn promote_attachment_nonexistent_returns_404() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "docket_attachment_id": "00000000-0000-0000-0000-000000000099",
    });

    let (status, _) = post_json(
        &app,
        "/api/documents/from-attachment",
        &body.to_string(),
        "district9",
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn promote_attachment_invalid_document_type_returns_400() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-PROM-005").await;
    let entry = crate::common::create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();
    let att_id = create_uploaded_attachment(&pool, "district9", entry_id).await;

    let body = serde_json::json!({
        "docket_attachment_id": att_id,
        "document_type": "InvalidType",
    });

    let (status, response) = post_json(
        &app,
        "/api/documents/from-attachment",
        &body.to_string(),
        "district9",
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {:?}", response);
}
