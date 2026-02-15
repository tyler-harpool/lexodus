use axum::http::StatusCode;

use crate::common::{
    create_test_case, create_test_document, create_test_docket_entry,
    create_uploaded_attachment, post_json, post_no_court,
    get_with_court, test_app,
};

#[tokio::test]
async fn link_document_returns_200() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-LINK-001").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();
    let doc_id = create_test_document(&pool, "district9", &case_id).await;

    let body = serde_json::json!({ "document_id": doc_id });
    let uri = format!("/api/docket/entries/{}/link-document", entry_id);
    let (status, response) = post_json(&app, &uri, &body.to_string(), "district9").await;

    assert_eq!(status, StatusCode::OK, "body: {:?}", response);
    assert_eq!(response["document_id"], doc_id);
    assert_eq!(response["id"], entry_id);
}

#[tokio::test]
async fn link_document_idempotent() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-LINK-002").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();
    let doc_id = create_test_document(&pool, "district9", &case_id).await;

    let body = serde_json::json!({ "document_id": doc_id });
    let uri = format!("/api/docket/entries/{}/link-document", entry_id);

    // First call
    let (status1, _) = post_json(&app, &uri, &body.to_string(), "district9").await;
    assert_eq!(status1, StatusCode::OK);

    // Second call — same result, no error
    let (status2, response2) = post_json(&app, &uri, &body.to_string(), "district9").await;
    assert_eq!(status2, StatusCode::OK);
    assert_eq!(response2["document_id"], doc_id);
}

#[tokio::test]
async fn link_document_entry_visible_in_get() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-LINK-003").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();
    let doc_id = create_test_document(&pool, "district9", &case_id).await;

    // Link
    let body = serde_json::json!({ "document_id": doc_id });
    let link_uri = format!("/api/docket/entries/{}/link-document", entry_id);
    post_json(&app, &link_uri, &body.to_string(), "district9").await;

    // GET the entry — should have document_id
    let get_uri = format!("/api/docket/entries/{}", entry_id);
    let (status, response) = get_with_court(&app, &get_uri, "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["document_id"], doc_id);
}

#[tokio::test]
async fn link_document_missing_court_returns_400() {
    let (app, _pool, _guard) = test_app().await;
    let body = serde_json::json!({ "document_id": "00000000-0000-0000-0000-000000000001" });
    let (status, _) = post_no_court(
        &app,
        "/api/docket/entries/00000000-0000-0000-0000-000000000001/link-document",
        &body.to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn link_document_nonexistent_entry_returns_404() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-LINK-004").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;

    let body = serde_json::json!({ "document_id": doc_id });
    let uri = "/api/docket/entries/00000000-0000-0000-0000-000000000099/link-document";
    let (status, _) = post_json(&app, uri, &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn link_document_nonexistent_document_returns_404() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-LINK-005").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();

    let body = serde_json::json!({ "document_id": "00000000-0000-0000-0000-000000000099" });
    let uri = format!("/api/docket/entries/{}/link-document", entry_id);
    let (status, _) = post_json(&app, &uri, &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn link_document_cross_tenant_entry_returns_404() {
    let (app, pool, _guard) = test_app().await;

    // Entry in district12, document in district9
    let case12 = create_test_case(&pool, "district12", "CR-2026-LINK-006").await;
    let entry12 = create_test_docket_entry(&app, "district12", &case12, "motion").await;
    let entry12_id = entry12["id"].as_str().unwrap();

    let case9 = create_test_case(&pool, "district9", "CR-2026-LINK-007").await;
    let doc9 = create_test_document(&pool, "district9", &case9).await;

    // Try to link from district9 — entry belongs to district12
    let body = serde_json::json!({ "document_id": doc9 });
    let uri = format!("/api/docket/entries/{}/link-document", entry12_id);
    let (status, _) = post_json(&app, &uri, &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn link_document_cross_tenant_document_returns_404() {
    let (app, pool, _guard) = test_app().await;

    // Entry in district9, document in district12
    let case9 = create_test_case(&pool, "district9", "CR-2026-LINK-008").await;
    let entry9 = create_test_docket_entry(&app, "district9", &case9, "motion").await;
    let entry9_id = entry9["id"].as_str().unwrap();

    let case12 = create_test_case(&pool, "district12", "CR-2026-LINK-009").await;
    let doc12 = create_test_document(&pool, "district12", &case12).await;

    // Try to link — document belongs to district12
    let body = serde_json::json!({ "document_id": doc12 });
    let uri = format!("/api/docket/entries/{}/link-document", entry9_id);
    let (status, _) = post_json(&app, &uri, &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn promote_attachment_auto_links_docket_entry() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-LINK-010").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();

    // Verify entry starts with no document_id
    assert!(entry["document_id"].is_null(), "Entry should start without document_id");

    let att_id = create_uploaded_attachment(&pool, "district9", entry_id).await;

    // Promote the attachment
    let promote_body = serde_json::json!({
        "docket_attachment_id": att_id,
        "title": "Auto-linked Motion",
        "document_type": "Motion"
    });
    let (status, promote_resp) = post_json(
        &app,
        "/api/documents/from-attachment",
        &promote_body.to_string(),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {:?}", promote_resp);
    let doc_id = promote_resp["id"].as_str().unwrap();

    // Now GET the docket entry — it should have document_id set
    let get_uri = format!("/api/docket/entries/{}", entry_id);
    let (get_status, get_resp) = get_with_court(&app, &get_uri, "district9").await;
    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(
        get_resp["document_id"].as_str().unwrap(),
        doc_id,
        "Docket entry should be auto-linked to promoted document"
    );
}
