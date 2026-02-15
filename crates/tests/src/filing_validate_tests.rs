use axum::http::StatusCode;

use crate::common::{post_json, post_no_court, test_app, create_test_case};

#[tokio::test]
async fn validate_missing_court_header_returns_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "case_id": "00000000-0000-0000-0000-000000000000",
        "document_type": "Motion",
        "title": "Test Motion",
        "filed_by": "Attorney Smith",
    });

    let (status, _body) = post_no_court(&app, "/api/filings/validate", &body.to_string()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn validate_valid_request_returns_200_valid_true() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-FV-00001").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "Motion to Dismiss",
        "filed_by": "Attorney Smith",
    });

    let (status, response) = post_json(&app, "/api/filings/validate", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["valid"], true);
    assert_eq!(response["errors"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn validate_missing_title_returns_valid_false() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-FV-00002").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "",
        "filed_by": "Attorney Smith",
    });

    let (status, response) = post_json(&app, "/api/filings/validate", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["valid"], false);

    let errors = response["errors"].as_array().unwrap();
    assert!(errors.iter().any(|e| e["field"] == "title"));
}

#[tokio::test]
async fn validate_missing_filed_by_returns_valid_false() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-FV-00003").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "Some Motion",
        "filed_by": "",
    });

    let (status, response) = post_json(&app, "/api/filings/validate", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["valid"], false);

    let errors = response["errors"].as_array().unwrap();
    assert!(errors.iter().any(|e| e["field"] == "filed_by"));
}

#[tokio::test]
async fn validate_invalid_document_type_returns_valid_false() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-FV-00004").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "InvalidType",
        "title": "Some Filing",
        "filed_by": "Attorney Smith",
    });

    let (status, response) = post_json(&app, "/api/filings/validate", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["valid"], false);

    let errors = response["errors"].as_array().unwrap();
    assert!(errors.iter().any(|e| e["field"] == "document_type"));
}

#[tokio::test]
async fn validate_cross_tenant_case_returns_valid_false() {
    let (app, pool, _guard) = test_app().await;
    // Case in district12
    let case_id = create_test_case(&pool, "district12", "2026-FV-00005").await;

    // Validate from district9
    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "Cross-tenant Filing",
        "filed_by": "Attorney Smith",
    });

    let (status, response) = post_json(&app, "/api/filings/validate", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["valid"], false);

    let errors = response["errors"].as_array().unwrap();
    assert!(errors.iter().any(|e| e["field"] == "case_id" && e["message"].as_str().unwrap().contains("not found")));
}

#[tokio::test]
async fn validate_nonexistent_upload_returns_valid_false() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-FV-00006").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "Filing with bad upload",
        "filed_by": "Attorney Smith",
        "upload_id": "00000000-0000-0000-0000-000000000099",
    });

    let (status, response) = post_json(&app, "/api/filings/validate", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["valid"], false);

    let errors = response["errors"].as_array().unwrap();
    assert!(errors.iter().any(|e| e["field"] == "upload_id"));
}
