use axum::http::StatusCode;
use std::collections::HashMap;

use crate::common::{
    create_test_case, create_test_document, create_test_token_with_courts,
    post_json_authed, test_app,
};

const COURT: &str = "district9";

#[tokio::test]
async fn seal_document_as_clerk() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "SEAL-001").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;

    let body = serde_json::json!({
        "sealing_level": "SealedCourtOnly",
        "reason_code": "JuvenileRecord",
    });

    let court_roles = HashMap::from([(COURT.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/documents/{}/seal", doc_id);
    let (status, response) = post_json_authed(&app, &uri, &body.to_string(), COURT, &token).await;

    assert_eq!(status, StatusCode::OK, "Seal failed: {:?}", response);
    assert_eq!(response["sealing_level"], "SealedCourtOnly");
    assert_eq!(response["seal_reason_code"], "JuvenileRecord");
    assert_eq!(response["is_sealed"], true);
}

#[tokio::test]
async fn seal_document_with_motion_id() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "SEAL-002").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;
    let motion_doc_id = create_test_document(&pool, COURT, &case_id).await;

    let body = serde_json::json!({
        "sealing_level": "SealedAttorneysOnly",
        "reason_code": "TradeSecret",
        "motion_id": motion_doc_id,
    });

    let court_roles = HashMap::from([(COURT.to_string(), "judge".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/documents/{}/seal", doc_id);
    let (status, response) = post_json_authed(&app, &uri, &body.to_string(), COURT, &token).await;

    assert_eq!(status, StatusCode::OK, "Seal failed: {:?}", response);
    assert_eq!(response["sealing_level"], "SealedAttorneysOnly");
    assert_eq!(response["seal_motion_id"], motion_doc_id);
}

#[tokio::test]
async fn seal_document_rejects_public_level() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "SEAL-003").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;

    let body = serde_json::json!({
        "sealing_level": "Public",
        "reason_code": "None",
    });

    let court_roles = HashMap::from([(COURT.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/documents/{}/seal", doc_id);
    let (status, _) = post_json_authed(&app, &uri, &body.to_string(), COURT, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn unseal_document_as_clerk() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "UNSEAL-001").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;

    let court_roles = HashMap::from([(COURT.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    // Seal first
    let seal_body = serde_json::json!({
        "sealing_level": "SealedCourtOnly",
        "reason_code": "JuvenileRecord",
    });
    let uri = format!("/api/documents/{}/seal", doc_id);
    post_json_authed(&app, &uri, &seal_body.to_string(), COURT, &token).await;

    // Unseal
    let uri = format!("/api/documents/{}/unseal", doc_id);
    let (status, response) = post_json_authed(&app, &uri, "{}", COURT, &token).await;

    assert_eq!(status, StatusCode::OK, "Unseal failed: {:?}", response);
    assert_eq!(response["sealing_level"], "Public");
    assert_eq!(response["is_sealed"], false);
    assert!(response["seal_reason_code"].is_null());
}

#[tokio::test]
async fn seal_rejects_attorney_role() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "SEAL-ATT-001").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;

    let body = serde_json::json!({
        "sealing_level": "SealedCourtOnly",
        "reason_code": "JuvenileRecord",
    });

    let court_roles = HashMap::from([(COURT.to_string(), "attorney".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/documents/{}/seal", doc_id);
    let (status, _) = post_json_authed(&app, &uri, &body.to_string(), COURT, &token).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn seal_rejects_unauthenticated() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "SEAL-NOAUTH-001").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;

    let body = serde_json::json!({
        "sealing_level": "SealedCourtOnly",
        "reason_code": "JuvenileRecord",
    });

    // No auth token — use regular post_json
    let uri = format!("/api/documents/{}/seal", doc_id);
    let (status, _) =
        crate::common::post_json(&app, &uri, &body.to_string(), COURT).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn seal_cross_tenant_returns_not_found() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "SEAL-CROSS-001").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;

    let body = serde_json::json!({
        "sealing_level": "SealedCourtOnly",
        "reason_code": "JuvenileRecord",
    });

    // Clerk in district12 — doc is in district9
    let court_roles = HashMap::from([("district12".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/documents/{}/seal", doc_id);
    let (status, _) = post_json_authed(&app, &uri, &body.to_string(), "district12", &token).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
