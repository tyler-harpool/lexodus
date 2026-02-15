use axum::http::StatusCode;

use crate::common::{
    create_test_case, create_test_party, get_with_court, post_json, test_app,
};

#[tokio::test]
async fn submit_filing_creates_nef_row() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-NF-00001").await;

    // Add a party so the NEF has recipients
    let _party_id = create_test_party(&pool, "district9", &case_id).await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "Motion for Summary Judgment",
        "filed_by": "Attorney Adams",
    });

    let (status, response) =
        post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);

    let filing_id = response["filing_id"].as_str().unwrap();

    // GET the NEF via the REST endpoint
    let uri = format!("/api/filings/{}/nef", filing_id);
    let (nef_status, nef_response) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(nef_status, StatusCode::OK, "NEF response: {:?}", nef_response);

    // Verify NEF fields
    assert!(nef_response["id"].is_string());
    assert_eq!(nef_response["filing_id"], filing_id);
    assert_eq!(nef_response["case_id"], case_id);
    assert!(nef_response["document_id"].is_string());
    assert!(nef_response["docket_entry_id"].is_string());
    assert!(nef_response["created_at"].is_string());

    // Verify recipients
    let recipients = nef_response["recipients"].as_array().unwrap();
    assert_eq!(recipients.len(), 1, "Should have 1 recipient (1 party)");
    assert_eq!(recipients[0]["name"], "Test Party");
    assert_eq!(recipients[0]["electronic"], true);
}

#[tokio::test]
async fn submit_filing_creates_service_records_for_all_parties() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-NF-00002").await;

    // Add 3 parties
    let _p1 = create_test_party(&pool, "district9", &case_id).await;
    let _p2 = create_test_party(&pool, "district9", &case_id).await;
    let _p3 = create_test_party(&pool, "district9", &case_id).await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Notice",
        "title": "Notice of Appearance",
        "filed_by": "Attorney Jones",
    });

    let (status, response) =
        post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);

    let document_id = response["document_id"].as_str().unwrap();

    // Get service records for the created document
    let uri = format!("/api/service-records/document/{}", document_id);
    let (sr_status, sr_response) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(sr_status, StatusCode::OK);

    let records = sr_response.as_array().unwrap();
    assert_eq!(
        records.len(),
        3,
        "Should have 3 service records (one per party)"
    );

    // All should be served_by the filer
    for rec in records {
        assert_eq!(rec["served_by"], "Attorney Jones");
        assert_eq!(rec["service_method"], "Electronic");
    }
}

#[tokio::test]
async fn submit_filing_with_no_parties_creates_empty_nef_recipients() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-NF-00003").await;

    // No parties added

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "Motion to Dismiss",
        "filed_by": "Attorney Smith",
    });

    let (status, response) =
        post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);

    let filing_id = response["filing_id"].as_str().unwrap();

    // NEF should still exist with empty recipients
    let uri = format!("/api/filings/{}/nef", filing_id);
    let (nef_status, nef_response) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(nef_status, StatusCode::OK);

    let recipients = nef_response["recipients"].as_array().unwrap();
    assert_eq!(recipients.len(), 0, "Should have 0 recipients (no parties)");
}

#[tokio::test]
async fn get_nef_nonexistent_filing_returns_404() {
    let (app, _pool, _guard) = test_app().await;

    let fake_uuid = "00000000-0000-0000-0000-000000000099";
    let uri = format!("/api/filings/{}/nef", fake_uuid);
    let (status, _) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
