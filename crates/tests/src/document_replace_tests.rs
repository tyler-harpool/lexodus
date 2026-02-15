use axum::http::StatusCode;
use std::collections::HashMap;

use crate::common::{
    create_test_case, create_test_document, create_test_token_with_courts, post_json_authed,
    test_app,
};

const COURT: &str = "district9";

/// Create a finalized filing upload directly in the DB for testing replacement.
async fn create_test_upload(pool: &sqlx::Pool<sqlx::Postgres>, court: &str) -> String {
    let row = sqlx::query_scalar!(
        r#"
        INSERT INTO filing_uploads (court_id, filename, file_size, content_type, storage_key, uploaded_at)
        VALUES ($1, 'replacement.pdf', 2048, 'application/pdf', 'test/replace/key.pdf', NOW())
        RETURNING id
        "#,
        court,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test upload");

    row.to_string()
}

#[tokio::test]
async fn replace_document_as_clerk() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "REPL-001").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;
    let upload_id = create_test_upload(&pool, COURT).await;

    let body = serde_json::json!({
        "upload_id": upload_id,
        "title": "Corrected Motion",
    });

    let court_roles = HashMap::from([(COURT.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/documents/{}/replace", doc_id);
    let (status, response) = post_json_authed(&app, &uri, &body.to_string(), COURT, &token).await;

    assert_eq!(status, StatusCode::CREATED, "Replace failed: {:?}", response);
    assert_eq!(response["title"], "Corrected Motion");
    assert_eq!(response["is_stricken"], false); // The REPLACEMENT is not stricken
}

#[tokio::test]
async fn replace_document_marks_original_stricken() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "REPL-002").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;
    let upload_id = create_test_upload(&pool, COURT).await;

    // Backdate document past the grace period so full replace path is taken
    let doc_uuid = uuid::Uuid::parse_str(&doc_id).unwrap();
    sqlx::query!(
        "UPDATE documents SET created_at = NOW() - INTERVAL '1 hour' WHERE id = $1",
        doc_uuid,
    )
    .execute(&pool)
    .await
    .unwrap();

    let body = serde_json::json!({
        "upload_id": upload_id,
    });

    let court_roles = HashMap::from([(COURT.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/documents/{}/replace", doc_id);
    let (status, replacement) = post_json_authed(&app, &uri, &body.to_string(), COURT, &token).await;
    assert_eq!(status, StatusCode::CREATED);

    // Verify original is stricken (check DB directly)
    let original = sqlx::query!(
        "SELECT is_stricken, replaced_by_document_id FROM documents WHERE id = $1",
        doc_uuid,
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch original");

    assert!(original.is_stricken);
    let replacement_id = replacement["id"].as_str().unwrap();
    assert_eq!(
        original.replaced_by_document_id.unwrap().to_string(),
        replacement_id
    );
}

#[tokio::test]
async fn replace_already_replaced_fails() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "REPL-003").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;
    let upload1 = create_test_upload(&pool, COURT).await;
    let upload2 = create_test_upload(&pool, COURT).await;

    // Backdate document past the grace period so full replace path is taken
    let doc_uuid = uuid::Uuid::parse_str(&doc_id).unwrap();
    sqlx::query!(
        "UPDATE documents SET created_at = NOW() - INTERVAL '1 hour' WHERE id = $1",
        doc_uuid,
    )
    .execute(&pool)
    .await
    .unwrap();

    let court_roles = HashMap::from([(COURT.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    // First replacement succeeds (creates new doc + strikes original)
    let body = serde_json::json!({ "upload_id": upload1 });
    let uri = format!("/api/documents/{}/replace", doc_id);
    let (status, _) = post_json_authed(&app, &uri, &body.to_string(), COURT, &token).await;
    assert_eq!(status, StatusCode::CREATED);

    // Second replacement of the same document fails (already replaced)
    let body = serde_json::json!({ "upload_id": upload2 });
    let (status, _) = post_json_authed(&app, &uri, &body.to_string(), COURT, &token).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn replace_within_grace_period_updates_in_place() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "REPL-GRACE-001").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;
    let upload_id = create_test_upload(&pool, COURT).await;

    // Document is fresh — should be within grace period (in-place update)
    let body = serde_json::json!({
        "upload_id": upload_id,
        "title": "In-Place Correction",
    });

    let court_roles = HashMap::from([(COURT.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/documents/{}/replace", doc_id);
    let (status, response) = post_json_authed(&app, &uri, &body.to_string(), COURT, &token).await;
    assert_eq!(status, StatusCode::CREATED, "In-place replace failed: {:?}", response);

    // Verify: same document ID (updated in place, not a new document)
    assert_eq!(response["id"], doc_id, "Within grace period, document should be updated in-place");
    assert_eq!(response["title"], "In-Place Correction");
    assert_eq!(response["is_stricken"], false, "In-place update should not be stricken");

    // Verify: original is NOT stricken in DB (it was updated, not replaced)
    let doc_uuid = uuid::Uuid::parse_str(&doc_id).unwrap();
    let original = sqlx::query!(
        "SELECT is_stricken, replaced_by_document_id FROM documents WHERE id = $1",
        doc_uuid,
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch document");

    assert!(!original.is_stricken, "In-place update should not strike the document");
    assert!(
        original.replaced_by_document_id.is_none(),
        "In-place update should not set replaced_by"
    );
}

#[tokio::test]
async fn replace_rejects_attorney_role() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "REPL-ATT-001").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;
    let upload_id = create_test_upload(&pool, COURT).await;

    let body = serde_json::json!({ "upload_id": upload_id });
    let court_roles = HashMap::from([(COURT.to_string(), "attorney".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/documents/{}/replace", doc_id);
    let (status, _) = post_json_authed(&app, &uri, &body.to_string(), COURT, &token).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn strike_document_as_clerk() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "STRIKE-001").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;

    let court_roles = HashMap::from([(COURT.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/documents/{}/strike", doc_id);
    let (status, response) = post_json_authed(&app, &uri, "{}", COURT, &token).await;

    assert_eq!(status, StatusCode::OK, "Strike failed: {:?}", response);
    assert_eq!(response["is_stricken"], true);
}

#[tokio::test]
async fn strike_rejects_attorney_role() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, COURT, "STRIKE-ATT-001").await;
    let doc_id = create_test_document(&pool, COURT, &case_id).await;

    let court_roles = HashMap::from([(COURT.to_string(), "attorney".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/documents/{}/strike", doc_id);
    let (status, _) = post_json_authed(&app, &uri, "{}", COURT, &token).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}
