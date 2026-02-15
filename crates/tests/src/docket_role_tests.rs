use axum::http::StatusCode;
use std::collections::HashMap;

use crate::common::{
    create_test_case, create_test_token, create_test_token_with_courts, post_json, post_json_authed,
    test_app,
};

const COURT: &str = "district9";

#[tokio::test]
async fn docket_entry_creation_requires_clerk_role() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "ROLE-DOCK-001").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": "motion",
        "description": "Test motion entry",
    });

    // Attorney role in this court should be rejected
    let court_roles = HashMap::from([(COURT.to_string(), "attorney".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, _) =
        post_json_authed(&app, "/api/docket/entries", &body.to_string(), COURT, &token).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn docket_entry_creation_succeeds_as_clerk() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "ROLE-DOCK-002").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": "motion",
        "description": "Test motion entry",
    });

    let court_roles = HashMap::from([(COURT.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, response) =
        post_json_authed(&app, "/api/docket/entries", &body.to_string(), COURT, &token).await;
    assert_eq!(status, StatusCode::CREATED, "Clerk should create entry: {:?}", response);
}

#[tokio::test]
async fn docket_entry_creation_succeeds_as_judge() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "ROLE-DOCK-003").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": "order",
        "description": "Test order entry",
    });

    let court_roles = HashMap::from([(COURT.to_string(), "judge".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, response) =
        post_json_authed(&app, "/api/docket/entries", &body.to_string(), COURT, &token).await;
    assert_eq!(status, StatusCode::CREATED, "Judge should create entry: {:?}", response);
}

#[tokio::test]
async fn docket_entry_creation_succeeds_as_admin() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "ROLE-DOCK-004").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": "notice",
        "description": "Test notice entry",
    });

    // Admin bypasses court membership — use global admin role
    let token = create_test_token("admin");
    let (status, response) =
        post_json_authed(&app, "/api/docket/entries", &body.to_string(), COURT, &token).await;
    assert_eq!(status, StatusCode::CREATED, "Admin should create entry: {:?}", response);
}

#[tokio::test]
async fn docket_entry_creation_rejects_unauthenticated() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "ROLE-DOCK-005").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": "motion",
        "description": "Test motion entry",
    });

    // No auth token
    let (status, _) = post_json(&app, "/api/docket/entries", &body.to_string(), COURT).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn docket_entry_creation_rejects_wrong_court_membership() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "ROLE-DOCK-006").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": "motion",
        "description": "Test motion entry",
    });

    // Clerk in district12 should NOT be able to create entries in district9
    let court_roles = HashMap::from([("district12".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, _) =
        post_json_authed(&app, "/api/docket/entries", &body.to_string(), COURT, &token).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
