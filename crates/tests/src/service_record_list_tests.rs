use axum::http::StatusCode;

use crate::common::*;

#[tokio::test]
async fn list_service_records_empty() {
    let (app, _pool, _guard) = test_app().await;

    let (status, resp) = get_with_court(&app, "/api/service-records", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["records"].as_array().unwrap().len(), 0);
    assert_eq!(resp["total"], 0);
}

#[tokio::test]
async fn list_service_records_returns_created() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "SR-LIST-001").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;
    let party_id = create_test_party(&pool, "district9", &case_id).await;

    let body = serde_json::json!({
        "document_id": doc_id,
        "party_id": party_id,
        "service_method": "Mail",
        "served_by": "Mail Room",
    });

    let (s, _) = post_json(&app, "/api/service-records", &body.to_string(), "district9").await;
    assert_eq!(s, StatusCode::CREATED);

    let (status, resp) = get_with_court(&app, "/api/service-records", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["records"].as_array().unwrap().len(), 1);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["records"][0]["service_method"], "Mail");
}

#[tokio::test]
async fn list_service_records_with_document_filter() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "SR-LIST-002").await;
    let doc1 = create_test_document(&pool, "district9", &case_id).await;
    let doc2 = create_test_document(&pool, "district9", &case_id).await;
    let party_id = create_test_party(&pool, "district9", &case_id).await;

    // Record on doc1
    let body1 = serde_json::json!({
        "document_id": doc1,
        "party_id": party_id,
        "service_method": "Electronic",
        "served_by": "Clerk A",
    });
    post_json(&app, "/api/service-records", &body1.to_string(), "district9").await;

    // Record on doc2
    let body2 = serde_json::json!({
        "document_id": doc2,
        "party_id": party_id,
        "service_method": "Mail",
        "served_by": "Clerk B",
    });
    post_json(&app, "/api/service-records", &body2.to_string(), "district9").await;

    // Filter by doc1
    let uri = format!("/api/service-records?document_id={}", doc1);
    let (status, resp) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["records"].as_array().unwrap().len(), 1);
    assert_eq!(resp["records"][0]["service_method"], "Electronic");
}

#[tokio::test]
async fn list_by_document_endpoint() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "SR-LIST-003").await;
    let doc_id = create_test_document(&pool, "district9", &case_id).await;
    let party_id = create_test_party(&pool, "district9", &case_id).await;

    let body = serde_json::json!({
        "document_id": doc_id,
        "party_id": party_id,
        "service_method": "Certified Mail",
        "served_by": "Certified Carrier",
    });
    post_json(&app, "/api/service-records", &body.to_string(), "district9").await;

    let uri = format!("/api/service-records/document/{}", doc_id);
    let (status, resp) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(status, StatusCode::OK);
    let records = resp.as_array().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["service_method"], "Certified Mail");
}

#[tokio::test]
async fn list_service_records_missing_court_returns_400() {
    let (app, _pool, _guard) = test_app().await;

    let (status, _) = get_no_court(&app, "/api/service-records").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
