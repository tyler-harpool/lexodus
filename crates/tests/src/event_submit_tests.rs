use axum::http::StatusCode;
use std::collections::HashMap;

use crate::common::{
    create_test_case, create_test_docket_entry, create_test_document,
    create_test_party, create_uploaded_attachment, get_authed,
    post_json_authed, test_app,
};

// ---------------------------------------------------------------------------
// POST /api/events — text_entry kind
// ---------------------------------------------------------------------------

#[tokio::test]
async fn text_entry_creates_docket_entry() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "26-EVT-TE01").await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let body = serde_json::json!({
        "event_kind": "text_entry",
        "case_id": case_id,
        "entry_type": "minute_order",
        "description": "Court convened at 10:00 AM. All parties present.",
    });

    let (status, response) = post_json_authed(
        &app,
        "/api/events",
        &body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);
    assert_eq!(response["event_kind"], "text_entry");
    assert!(response["docket_entry_id"].is_string());
    assert!(response["entry_number"].as_i64().unwrap() > 0);
    // Text entries should NOT create documents, filings, or NEFs
    assert!(response["document_id"].is_null());
    assert!(response["filing_id"].is_null());
    assert!(response["nef_id"].is_null());
}

#[tokio::test]
async fn text_entry_requires_description() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "26-EVT-TE02").await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let body = serde_json::json!({
        "event_kind": "text_entry",
        "case_id": case_id,
        "entry_type": "minute_order",
    });

    let (status, _) = post_json_authed(
        &app,
        "/api/events",
        &body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn text_entry_rejects_empty_description() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "26-EVT-TE03").await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let body = serde_json::json!({
        "event_kind": "text_entry",
        "case_id": case_id,
        "entry_type": "minute_order",
        "description": "   ",
    });

    let (status, _) = post_json_authed(
        &app,
        "/api/events",
        &body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// POST /api/events — filing kind
// ---------------------------------------------------------------------------

#[tokio::test]
async fn filing_event_creates_full_chain() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "26-EVT-FI01").await;

    let court_roles = HashMap::from([("district9".to_string(), "attorney".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let body = serde_json::json!({
        "event_kind": "filing",
        "case_id": case_id,
        "document_type": "Motion",
        "title": "Motion for Summary Judgment",
        "filed_by": "Attorney Williams",
    });

    let (status, response) = post_json_authed(
        &app,
        "/api/events",
        &body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);
    assert_eq!(response["event_kind"], "filing");
    assert!(response["docket_entry_id"].is_string());
    assert!(response["document_id"].is_string());
    assert!(response["filing_id"].is_string());
    assert!(response["nef_id"].is_string());
    assert!(response["entry_number"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn filing_event_requires_document_type() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "26-EVT-FI02").await;

    let court_roles = HashMap::from([("district9".to_string(), "attorney".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let body = serde_json::json!({
        "event_kind": "filing",
        "case_id": case_id,
        "title": "Missing Doc Type",
        "filed_by": "Attorney X",
    });

    let (status, _) = post_json_authed(
        &app,
        "/api/events",
        &body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// POST /api/events — promote_attachment kind
// ---------------------------------------------------------------------------

#[tokio::test]
async fn promote_event_creates_document() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "26-EVT-PA01").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();
    let att_id = create_uploaded_attachment(&pool, "district9", entry_id).await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let body = serde_json::json!({
        "event_kind": "promote_attachment",
        "case_id": case_id,
        "attachment_id": att_id,
        "promote_title": "Exhibit A — Contract",
        "promote_document_type": "Exhibit",
    });

    let (status, response) = post_json_authed(
        &app,
        "/api/events",
        &body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);
    assert_eq!(response["event_kind"], "promote_attachment");
    assert!(response["document_id"].is_string());
    assert!(response["docket_entry_id"].is_string());
    // Promote doesn't create a filing or NEF
    assert!(response["filing_id"].is_null());
    assert!(response["nef_id"].is_null());
}

#[tokio::test]
async fn promote_event_idempotent() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "26-EVT-PA02").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();
    let att_id = create_uploaded_attachment(&pool, "district9", entry_id).await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let body = serde_json::json!({
        "event_kind": "promote_attachment",
        "case_id": case_id,
        "attachment_id": att_id,
    });

    // First promote
    let (status1, response1) = post_json_authed(
        &app,
        "/api/events",
        &body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status1, StatusCode::CREATED);
    let doc_id_1 = response1["document_id"].as_str().unwrap().to_string();

    // Second promote — same attachment → same document (idempotent)
    let (status2, response2) = post_json_authed(
        &app,
        "/api/events",
        &body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status2, StatusCode::CREATED);
    let doc_id_2 = response2["document_id"].as_str().unwrap().to_string();

    assert_eq!(doc_id_1, doc_id_2, "Promote should be idempotent");
}

// ---------------------------------------------------------------------------
// Role enforcement
// ---------------------------------------------------------------------------

#[tokio::test]
async fn text_entry_rejects_attorney() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "26-EVT-RL01").await;

    // text_entry requires clerk role
    let court_roles = HashMap::from([("district9".to_string(), "attorney".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let body = serde_json::json!({
        "event_kind": "text_entry",
        "case_id": case_id,
        "entry_type": "minute_order",
        "description": "Should be forbidden",
    });

    let (status, _) = post_json_authed(
        &app,
        "/api/events",
        &body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn filing_allowed_for_attorney() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "26-EVT-RL02").await;

    let court_roles = HashMap::from([("district9".to_string(), "attorney".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let body = serde_json::json!({
        "event_kind": "filing",
        "case_id": case_id,
        "document_type": "Brief",
        "title": "Reply Brief",
        "filed_by": "Attorney Garcia",
    });

    let (status, _) = post_json_authed(
        &app,
        "/api/events",
        &body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn unknown_event_kind_returns_400() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "26-EVT-UK01").await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let body = serde_json::json!({
        "event_kind": "nonexistent_kind",
        "case_id": case_id,
    });

    let (status, _) = post_json_authed(
        &app,
        "/api/events",
        &body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// GET /api/cases/{case_id}/timeline
// ---------------------------------------------------------------------------

#[tokio::test]
async fn timeline_returns_docket_entries() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "26-EVT-TL01").await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    // Create a text entry event
    let body = serde_json::json!({
        "event_kind": "text_entry",
        "case_id": case_id,
        "entry_type": "minute_order",
        "description": "Hearing held on motions in limine.",
    });
    let (status, _) = post_json_authed(
        &app,
        "/api/events",
        &body.to_string(),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Fetch timeline
    let (status, response) = get_authed(
        &app,
        &format!("/api/cases/{}/timeline", case_id),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["total"].as_i64().unwrap() >= 1);

    let entries = response["entries"].as_array().expect("entries should be array");
    assert!(!entries.is_empty());
    assert_eq!(entries[0]["source"], "docket_entry");
    assert!(entries[0]["summary"]
        .as_str()
        .unwrap()
        .contains("motions in limine"));
}

#[tokio::test]
async fn timeline_includes_document_events() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "26-EVT-TL02").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    // Seal the document to create a document event
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

    // Fetch timeline — should include document_event
    let (status, response) = get_authed(
        &app,
        &format!("/api/cases/{}/timeline", case_id),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let entries = response["entries"].as_array().expect("entries should be array");
    let doc_event = entries
        .iter()
        .find(|e| e["source"] == "document_event")
        .expect("Timeline should include document events");
    assert_eq!(doc_event["entry_type"], "sealed");
    assert_eq!(doc_event["document_id"], doc_id);
}

#[tokio::test]
async fn timeline_pagination_works() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "26-EVT-TL03").await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    // Create 3 text entries
    for i in 0..3 {
        let body = serde_json::json!({
            "event_kind": "text_entry",
            "case_id": case_id,
            "entry_type": "minute_order",
            "description": format!("Entry number {}", i),
        });
        let (status, _) = post_json_authed(
            &app,
            "/api/events",
            &body.to_string(),
            "district9",
            &token,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    // Fetch with limit=2
    let (status, response) = get_authed(
        &app,
        &format!("/api/cases/{}/timeline?limit=2", case_id),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["total"].as_i64().unwrap(), 3);
    assert_eq!(response["entries"].as_array().unwrap().len(), 2);

    // Fetch with offset=2
    let (status, response) = get_authed(
        &app,
        &format!("/api/cases/{}/timeline?offset=2&limit=10", case_id),
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["entries"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn timeline_invalid_case_id_returns_400() {
    let (app, _pool, _guard) = test_app().await;

    let court_roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let (status, _) = get_authed(
        &app,
        "/api/cases/not-a-uuid/timeline",
        "district9",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// Court isolation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn event_isolation_across_courts() {
    let (app, pool, _guard) = test_app().await;
    let case_d9 = create_test_case(&pool, "district9", "26-EVT-IS01").await;
    let case_d12 = create_test_case(&pool, "district12", "26-EVT-IS02").await;

    let token_d9 = {
        let roles = HashMap::from([("district9".to_string(), "clerk".to_string())]);
        create_test_token_with_courts("user", &roles)
    };
    let token_d12 = {
        let roles = HashMap::from([("district12".to_string(), "clerk".to_string())]);
        create_test_token_with_courts("user", &roles)
    };

    // Create events in both courts
    let body_d9 = serde_json::json!({
        "event_kind": "text_entry",
        "case_id": case_d9,
        "entry_type": "minute_order",
        "description": "District 9 event",
    });
    let (status, _) = post_json_authed(
        &app,
        "/api/events",
        &body_d9.to_string(),
        "district9",
        &token_d9,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let body_d12 = serde_json::json!({
        "event_kind": "text_entry",
        "case_id": case_d12,
        "entry_type": "minute_order",
        "description": "District 12 event",
    });
    let (status, _) = post_json_authed(
        &app,
        "/api/events",
        &body_d12.to_string(),
        "district12",
        &token_d12,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // District 9 timeline should only show district 9 events
    let (status, response) = get_authed(
        &app,
        &format!("/api/cases/{}/timeline", case_d9),
        "district9",
        &token_d9,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let entries = response["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0]["summary"]
        .as_str()
        .unwrap()
        .contains("District 9"));
}

// Import the helper
use crate::common::create_test_token_with_courts;
