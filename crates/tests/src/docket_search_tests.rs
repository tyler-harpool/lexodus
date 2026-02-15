use axum::http::StatusCode;
use std::collections::HashMap;

use crate::common::*;

#[tokio::test]
async fn search_docket_empty_results() {
    let (app, _pool, _guard) = test_app().await;

    let (status, response) = get_with_court(&app, "/api/docket/search", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["entries"].as_array().unwrap().len(), 0);
    assert_eq!(response["total"], 0);
}

#[tokio::test]
async fn search_docket_returns_entries() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-DS01").await;
    create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    create_test_docket_entry(&app, "district9", &case_id, "order").await;

    let (status, response) = get_with_court(&app, "/api/docket/search", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["entries"].as_array().unwrap().len(), 2);
    assert_eq!(response["total"], 2);
}

#[tokio::test]
async fn search_docket_filter_by_case_id() {
    let (app, pool, _guard) = test_app().await;
    let case1 = create_test_case(&pool, "district9", "2026-CR-DS02").await;
    let case2 = create_test_case(&pool, "district9", "2026-CR-DS03").await;
    create_test_docket_entry(&app, "district9", &case1, "motion").await;
    create_test_docket_entry(&app, "district9", &case2, "order").await;

    let (status, response) = get_with_court(
        &app,
        &format!("/api/docket/search?case_id={}", case1),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["entries"].as_array().unwrap().len(), 1);
    assert_eq!(response["total"], 1);
    assert_eq!(response["entries"][0]["case_id"], case1);
}

#[tokio::test]
async fn search_docket_filter_by_entry_type() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-DS04").await;
    create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    create_test_docket_entry(&app, "district9", &case_id, "order").await;
    create_test_docket_entry(&app, "district9", &case_id, "motion").await;

    let (status, response) =
        get_with_court(&app, "/api/docket/search?entry_type=motion", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["entries"].as_array().unwrap().len(), 2);
    assert_eq!(response["total"], 2);
}

#[tokio::test]
async fn search_docket_text_search() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-DS05").await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": "motion",
        "description": "Motion to suppress forensic evidence",
    });
    post_json_authed(&app, "/api/docket/entries", &body.to_string(), "district9", &token).await;

    let body2 = serde_json::json!({
        "case_id": case_id,
        "entry_type": "order",
        "description": "Scheduling order for trial",
    });
    post_json_authed(&app, "/api/docket/entries", &body2.to_string(), "district9", &token).await;

    let (status, response) =
        get_with_court(&app, "/api/docket/search?q=suppress", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["entries"].as_array().unwrap().len(), 1);
    assert!(response["entries"][0]["description"]
        .as_str()
        .unwrap()
        .contains("suppress"));
}

#[tokio::test]
async fn search_docket_pagination() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-DS06").await;

    for _ in 0..5 {
        create_test_docket_entry(&app, "district9", &case_id, "notice").await;
    }

    let (status, response) =
        get_with_court(&app, "/api/docket/search?limit=2&offset=0", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["entries"].as_array().unwrap().len(), 2);
    assert_eq!(response["total"], 5);

    let (status, response2) =
        get_with_court(&app, "/api/docket/search?limit=2&offset=4", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response2["entries"].as_array().unwrap().len(), 1);
}
