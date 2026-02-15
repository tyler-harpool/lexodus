use axum::http::StatusCode;
use std::collections::HashMap;

use crate::common::*;

#[tokio::test]
async fn create_docket_entry_success() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-D001").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": "motion",
        "description": "Motion to suppress evidence",
        "filed_by": "Defense Counsel",
    });

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, response) = post_json_authed(&app, "/api/docket/entries", &body.to_string(), "district9", &token).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response["entry_type"], "motion");
    assert_eq!(response["description"], "Motion to suppress evidence");
    assert_eq!(response["filed_by"], "Defense Counsel");
    assert_eq!(response["entry_number"], 1);
    assert_eq!(response["is_sealed"], false);
    assert_eq!(response["is_ex_parte"], false);
    assert!(response["id"].is_string());
    assert_eq!(response["case_id"], case_id);
}

#[tokio::test]
async fn create_docket_entry_returns_all_fields() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-D002").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": "order",
        "description": "Scheduling order",
        "filed_by": "Judge Smith",
        "is_sealed": true,
        "is_ex_parte": true,
        "page_count": 5,
        "related_entries": [1, 2],
        "service_list": ["plaintiff@test.com", "defendant@test.com"],
    });

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, response) = post_json_authed(&app, "/api/docket/entries", &body.to_string(), "district9", &token).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response["is_sealed"], true);
    assert_eq!(response["is_ex_parte"], true);
    assert_eq!(response["page_count"], 5);
    assert_eq!(response["related_entries"], serde_json::json!([1, 2]));
    assert_eq!(response["service_list"], serde_json::json!(["plaintiff@test.com", "defendant@test.com"]));
}

#[tokio::test]
async fn create_docket_entry_auto_increments_entry_number() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-D003").await;

    let entry1 = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry2 = create_test_docket_entry(&app, "district9", &case_id, "order").await;
    let entry3 = create_test_docket_entry(&app, "district9", &case_id, "notice").await;

    assert_eq!(entry1["entry_number"], 1);
    assert_eq!(entry2["entry_number"], 2);
    assert_eq!(entry3["entry_number"], 3);
}

#[tokio::test]
async fn create_docket_entry_empty_description_400() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-D004").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": "motion",
        "description": "",
    });

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, _) = post_json_authed(&app, "/api/docket/entries", &body.to_string(), "district9", &token).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_docket_entry_invalid_entry_type_400() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-D005").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": "invalid_type",
        "description": "Test entry",
    });

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, response) = post_json_authed(&app, "/api/docket/entries", &body.to_string(), "district9", &token).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["message"].as_str().unwrap().contains("Invalid entry_type"));
}

#[tokio::test]
async fn create_docket_entry_nonexistent_case_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "case_id": "00000000-0000-0000-0000-000000000000",
        "entry_type": "motion",
        "description": "Test entry",
    });

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, response) = post_json_authed(&app, "/api/docket/entries", &body.to_string(), "district9", &token).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["message"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn create_docket_entry_missing_court_header_400() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-D006").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": "motion",
        "description": "Test entry",
    });

    let (status, _) = post_no_court(&app, "/api/docket/entries", &body.to_string()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_docket_entry_without_filed_by() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-D007").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": "notice",
        "description": "Notice of hearing",
    });

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, response) = post_json_authed(&app, "/api/docket/entries", &body.to_string(), "district9", &token).await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(response["filed_by"].is_null());
}
