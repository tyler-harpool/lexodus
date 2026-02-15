use crate::common::*;
use axum::http::StatusCode;
use std::collections::HashMap;

/// Seal a document and verify an audit event is created.
#[tokio::test]
async fn seal_creates_audit_event() {
    let (app, pool, _guard) = test_app().await;

    let case_id = create_test_case(&pool, "district9", "26-CR-EVT01").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    // Seal the document
    let seal_body = serde_json::json!({
        "sealing_level": "SealedCourtOnly",
        "reason_code": "TradeSecret",
    });
    let (status, _) = post_json_authed(
        &app,
        &format!("/api/documents/{}/seal", doc_id),
        &seal_body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Verify audit event was created
    let (status, events) = get_authed(
        &app,
        &format!("/api/documents/{}/events", doc_id),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let events = events.as_array().expect("events should be an array");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event_type"], "sealed");
    assert_eq!(events[0]["detail"]["sealing_level"], "SealedCourtOnly");
    assert_eq!(events[0]["detail"]["reason_code"], "TradeSecret");
}

/// Seal then unseal creates two audit events.
#[tokio::test]
async fn unseal_creates_audit_event() {
    let (app, pool, _guard) = test_app().await;

    let case_id = create_test_case(&pool, "district9", "26-CR-EVT02").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    // Seal
    let seal_body = serde_json::json!({
        "sealing_level": "SealedCourtOnly",
        "reason_code": "GrandJury",
    });
    let (status, _) = post_json_authed(
        &app,
        &format!("/api/documents/{}/seal", doc_id),
        &seal_body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Unseal
    let (status, _) = post_json_authed(
        &app,
        &format!("/api/documents/{}/unseal", doc_id),
        "{}",
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Verify 2 audit events
    let (status, events) = get_authed(
        &app,
        &format!("/api/documents/{}/events", doc_id),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let events = events.as_array().expect("events should be an array");
    assert_eq!(events.len(), 2);
    // Newest first
    assert_eq!(events[0]["event_type"], "unsealed");
    assert_eq!(events[1]["event_type"], "sealed");
}

/// Strike a document creates an audit event.
#[tokio::test]
async fn strike_creates_audit_event() {
    let (app, pool, _guard) = test_app().await;

    let case_id = create_test_case(&pool, "district9", "26-CR-EVT03").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let (status, _) = post_json_authed(
        &app,
        &format!("/api/documents/{}/strike", doc_id),
        "{}",
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, events) = get_authed(
        &app,
        &format!("/api/documents/{}/events", doc_id),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let events = events.as_array().expect("events should be an array");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event_type"], "stricken");
}

/// Events endpoint rejects attorney role with 403.
#[tokio::test]
async fn events_endpoint_rejects_attorney() {
    let (app, pool, _guard) = test_app().await;

    let case_id = create_test_case(&pool, "district9", "26-CR-EVT05").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;

    let court_roles = HashMap::from([("district9".to_string(), "attorney".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let (status, _) = get_authed(
        &app,
        &format!("/api/documents/{}/events", doc_id),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
