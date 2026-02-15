use axum::http::StatusCode;

use crate::common::*;

#[tokio::test]
async fn district9_cannot_list_district12_service_records() {
    let (app, pool, _guard) = test_app().await;

    // Create records in district12
    let case12 = create_test_case(&pool, "district12", "SR-ISO-001").await;
    let doc12 = create_test_document(&pool, "district12", &case12).await;
    let party12 = create_test_party(&pool, "district12", &case12).await;

    let body = serde_json::json!({
        "document_id": doc12,
        "party_id": party12,
        "service_method": "Electronic",
        "served_by": "D12 Clerk",
    });
    let (s, _) = post_json(&app, "/api/service-records", &body.to_string(), "district12").await;
    assert_eq!(s, StatusCode::CREATED);

    // district9 should see empty list
    let (status, resp) = get_with_court(&app, "/api/service-records", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["records"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn district9_cannot_complete_district12_service_record() {
    let (app, pool, _guard) = test_app().await;

    let case12 = create_test_case(&pool, "district12", "SR-ISO-002").await;
    let doc12 = create_test_document(&pool, "district12", &case12).await;
    let party12 = create_test_party(&pool, "district12", &case12).await;

    let body = serde_json::json!({
        "document_id": doc12,
        "party_id": party12,
        "service_method": "Mail",
        "served_by": "D12 Clerk",
    });
    let (_, created) = post_json(&app, "/api/service-records", &body.to_string(), "district12").await;
    let record_id = created["id"].as_str().unwrap();

    // district9 tries to complete — should get 404 (not leak existence)
    let uri = format!("/api/service-records/{}/complete", record_id);
    let (status, _) = post_json(&app, &uri, "{}", "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn district9_list_by_document_returns_404_for_district12_document() {
    let (app, pool, _guard) = test_app().await;

    let case12 = create_test_case(&pool, "district12", "SR-ISO-003").await;
    let doc12 = create_test_document(&pool, "district12", &case12).await;

    // district9 tries to list by d12's document — should get 404
    let uri = format!("/api/service-records/document/{}", doc12);
    let (status, _) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
