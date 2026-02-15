use axum::http::StatusCode;

use crate::common::*;

#[tokio::test]
async fn create_service_record_returns_201() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "SR-CREATE-001").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;
    let party_id = create_test_party(&pool, "district9", &case_id).await;

    let body = serde_json::json!({
        "document_id": doc_id,
        "party_id": party_id,
        "service_method": "Electronic",
        "served_by": "Jane Clerk",
    });

    let (status, resp) = post_json(&app, "/api/service-records", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED, "body: {:?}", resp);
    assert_eq!(resp["service_method"], "Electronic");
    assert_eq!(resp["served_by"], "Jane Clerk");
    assert_eq!(resp["successful"], true);
    assert_eq!(resp["proof_of_service_filed"], false);
    assert_eq!(resp["attempts"], 1);
    assert!(resp["id"].is_string());
}

#[tokio::test]
async fn create_service_record_missing_court_returns_400() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "SR-CREATE-002").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;
    let party_id = create_test_party(&pool, "district9", &case_id).await;

    let body = serde_json::json!({
        "document_id": doc_id,
        "party_id": party_id,
        "service_method": "Electronic",
        "served_by": "Jane Clerk",
    });

    let (status, _) = post_no_court(&app, "/api/service-records", &body.to_string()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_service_record_empty_served_by_returns_400() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "SR-CREATE-003").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;
    let party_id = create_test_party(&pool, "district9", &case_id).await;

    let body = serde_json::json!({
        "document_id": doc_id,
        "party_id": party_id,
        "service_method": "Electronic",
        "served_by": "  ",
    });

    let (status, _) = post_json(&app, "/api/service-records", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_service_record_invalid_method_returns_400() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "SR-CREATE-004").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;
    let party_id = create_test_party(&pool, "district9", &case_id).await;

    let body = serde_json::json!({
        "document_id": doc_id,
        "party_id": party_id,
        "service_method": "Carrier Pigeon",
        "served_by": "Jane Clerk",
    });

    let (status, _) = post_json(&app, "/api/service-records", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_service_record_cross_tenant_document_returns_404() {
    let (app, pool, _guard) = test_app().await;
    // Document in district12
    let case12 = create_test_case(&pool, "district12", "SR-CREATE-005").await;
    let doc12 = create_test_document(&pool, "district12", &case12).await;
    // Party in district9
    let case9 = create_test_case(&pool, "district9", "SR-CREATE-006").await;
    let party9 = create_test_party(&pool, "district9", &case9).await;

    let body = serde_json::json!({
        "document_id": doc12,
        "party_id": party9,
        "service_method": "Electronic",
        "served_by": "Jane Clerk",
    });

    let (status, _) = post_json(&app, "/api/service-records", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_service_record_cross_tenant_party_returns_404() {
    let (app, pool, _guard) = test_app().await;
    // Document in district9
    let case9 = create_test_case(&pool, "district9", "SR-CREATE-007").await;
    let doc9 = create_test_document(&pool, "district9", &case9).await;
    // Party in district12
    let case12 = create_test_case(&pool, "district12", "SR-CREATE-008").await;
    let party12 = create_test_party(&pool, "district12", &case12).await;

    let body = serde_json::json!({
        "document_id": doc9,
        "party_id": party12,
        "service_method": "Electronic",
        "served_by": "Jane Clerk",
    });

    let (status, _) = post_json(&app, "/api/service-records", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
