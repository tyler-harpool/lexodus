use axum::http::StatusCode;

use crate::common::*;

#[tokio::test]
async fn complete_service_record_returns_200() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "SR-COMP-001").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;
    let party_id = create_test_party(&pool, "district9", &case_id).await;

    let body = serde_json::json!({
        "document_id": doc_id,
        "party_id": party_id,
        "service_method": "Electronic",
        "served_by": "Clerk",
    });

    let (_, created) = post_json(&app, "/api/service-records", &body.to_string(), "district9").await;
    let record_id = created["id"].as_str().unwrap();

    // Mark complete
    let uri = format!("/api/service-records/{}/complete", record_id);
    let (status, resp) = post_json(&app, &uri, "{}", "district9").await;
    assert_eq!(status, StatusCode::OK, "body: {:?}", resp);
    assert_eq!(resp["successful"], true);
    assert_eq!(resp["proof_of_service_filed"], true);
}

#[tokio::test]
async fn complete_service_record_is_idempotent() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "SR-COMP-002").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;
    let party_id = create_test_party(&pool, "district9", &case_id).await;

    let body = serde_json::json!({
        "document_id": doc_id,
        "party_id": party_id,
        "service_method": "Mail",
        "served_by": "Clerk",
    });

    let (_, created) = post_json(&app, "/api/service-records", &body.to_string(), "district9").await;
    let record_id = created["id"].as_str().unwrap();

    let uri = format!("/api/service-records/{}/complete", record_id);

    // First call
    let (s1, r1) = post_json(&app, &uri, "{}", "district9").await;
    assert_eq!(s1, StatusCode::OK);
    assert_eq!(r1["successful"], true);

    // Second call — same result, no error
    let (s2, r2) = post_json(&app, &uri, "{}", "district9").await;
    assert_eq!(s2, StatusCode::OK);
    assert_eq!(r2["successful"], true);
    assert_eq!(r2["proof_of_service_filed"], true);
}

#[tokio::test]
async fn complete_nonexistent_returns_404() {
    let (app, _pool, _guard) = test_app().await;

    let fake_id = "00000000-0000-0000-0000-000000000000";
    let uri = format!("/api/service-records/{}/complete", fake_id);
    let (status, _) = post_json(&app, &uri, "{}", "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn complete_missing_court_returns_400() {
    let (app, _pool, _guard) = test_app().await;

    let fake_id = "00000000-0000-0000-0000-000000000000";
    let uri = format!("/api/service-records/{}/complete", fake_id);
    let (status, _) = post_no_court(&app, &uri, "{}").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
