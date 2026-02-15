use axum::http::StatusCode;

use crate::common::{
    create_test_case, create_test_party, create_test_party_with_method, get_with_court, post_json,
    post_no_court, test_app,
};

#[tokio::test]
async fn submit_missing_court_header_returns_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "case_id": "00000000-0000-0000-0000-000000000000",
        "document_type": "Motion",
        "title": "Test Motion",
        "filed_by": "Attorney Smith",
    });

    let (status, _body) = post_no_court(&app, "/api/filings", &body.to_string()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn submit_valid_filing_returns_201_with_nef() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-FS-00001").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "Motion to Compel Discovery",
        "filed_by": "Attorney Jones",
    });

    let (status, response) = post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);

    // Verify response structure
    assert!(response["filing_id"].is_string());
    assert!(response["document_id"].is_string());
    assert!(response["docket_entry_id"].is_string());
    assert_eq!(response["case_id"], case_id);
    assert_eq!(response["status"], "Filed");
    assert!(response["filed_date"].is_string());

    // Verify NEF summary
    let nef = &response["nef"];
    assert_eq!(nef["case_number"], "2026-FS-00001");
    assert_eq!(nef["document_title"], "Motion to Compel Discovery");
    assert_eq!(nef["filed_by"], "Attorney Jones");
    assert!(nef["docket_number"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn submit_creates_linked_docket_entry() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-FS-00002").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Brief",
        "title": "Opening Brief",
        "filed_by": "Attorney Adams",
    });

    let (status, response) = post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);

    let docket_entry_id = response["docket_entry_id"].as_str().unwrap();
    let document_id = response["document_id"].as_str().unwrap();

    // GET the docket entry and verify it has the document linked
    let uri = format!("/api/docket/entries/{}", docket_entry_id);
    let (get_status, entry) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(entry["document_id"], document_id);
    assert_eq!(entry["entry_type"], "motion"); // Brief maps to motion
    assert!(entry["description"].as_str().unwrap().contains("Opening Brief"));
}

#[tokio::test]
async fn submit_missing_required_fields_returns_400() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-FS-00003").await;

    // Missing title and filed_by
    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "",
        "filed_by": "",
    });

    let (status, _response) = post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn submit_cross_tenant_case_returns_400() {
    let (app, pool, _guard) = test_app().await;
    // Case in district12
    let case_id = create_test_case(&pool, "district12", "2026-FS-00004").await;

    // Submit from district9
    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Notice",
        "title": "Cross-tenant Notice",
        "filed_by": "Attorney Evil",
    });

    let (status, _response) = post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn submit_auto_seeds_service_records_for_all_parties() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-FS-00005").await;

    // Add 2 parties to the case
    let _p1 = create_test_party(&pool, "district9", &case_id).await;
    let _p2 = create_test_party(&pool, "district9", &case_id).await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "Motion with Parties",
        "filed_by": "Attorney Park",
    });

    let (status, response) =
        post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);

    let document_id = response["document_id"].as_str().unwrap();

    // Service records should have been auto-created for both parties
    let uri = format!("/api/service-records/document/{}", document_id);
    let (sr_status, sr_response) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(sr_status, StatusCode::OK);

    let records = sr_response.as_array().unwrap();
    assert_eq!(
        records.len(),
        2,
        "Should have 2 auto-seeded service records (one per party)"
    );

    // All should be served_by the filer
    for rec in records {
        assert_eq!(rec["served_by"], "Attorney Park");
        assert_eq!(rec["service_method"], "Electronic");
    }
}

#[tokio::test]
async fn submit_electronic_service_auto_completes_with_proof() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-FS-00006").await;

    // Electronic party — should auto-complete
    let _p1 =
        create_test_party_with_method(&pool, "district9", &case_id, "Electronic Party", "Electronic")
            .await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "Motion to Test Electronic Service",
        "filed_by": "Attorney Electric",
    });

    let (status, response) =
        post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);

    let document_id = response["document_id"].as_str().unwrap();

    let uri = format!("/api/service-records/document/{}", document_id);
    let (sr_status, sr_response) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(sr_status, StatusCode::OK);

    let records = sr_response.as_array().unwrap();
    assert_eq!(records.len(), 1);

    let rec = &records[0];
    assert_eq!(rec["service_method"], "Electronic");
    assert_eq!(rec["successful"], true, "Electronic service should auto-complete");
    assert_eq!(
        rec["proof_of_service_filed"], true,
        "Electronic service should have proof filed"
    );
}

#[tokio::test]
async fn submit_non_electronic_service_creates_pending_record() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-FS-00007").await;

    // Non-electronic party — should create pending record
    let _p1 =
        create_test_party_with_method(&pool, "district9", &case_id, "Mail Party", "Mail").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Notice",
        "title": "Notice to Test Mail Service",
        "filed_by": "Attorney Postal",
    });

    let (status, response) =
        post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);

    let document_id = response["document_id"].as_str().unwrap();

    let uri = format!("/api/service-records/document/{}", document_id);
    let (sr_status, sr_response) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(sr_status, StatusCode::OK);

    let records = sr_response.as_array().unwrap();
    assert_eq!(records.len(), 1);

    let rec = &records[0];
    assert_eq!(rec["service_method"], "Mail");
    assert_eq!(rec["successful"], false, "Non-electronic service should be pending");
    assert_eq!(
        rec["proof_of_service_filed"], false,
        "Non-electronic service should not have proof filed"
    );
}

#[tokio::test]
async fn submit_mixed_service_methods_handles_each_correctly() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-FS-00008").await;

    // One electronic, one mail
    let _p1 =
        create_test_party_with_method(&pool, "district9", &case_id, "E-Party", "Electronic").await;
    let _p2 =
        create_test_party_with_method(&pool, "district9", &case_id, "M-Party", "Mail").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "Motion with Mixed Service",
        "filed_by": "Attorney Mixed",
    });

    let (status, response) =
        post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);

    let document_id = response["document_id"].as_str().unwrap();

    let uri = format!("/api/service-records/document/{}", document_id);
    let (sr_status, sr_response) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(sr_status, StatusCode::OK);

    let records = sr_response.as_array().unwrap();
    assert_eq!(records.len(), 2);

    // Find the electronic and mail records
    let e_rec = records
        .iter()
        .find(|r| r["service_method"] == "Electronic")
        .expect("Should have electronic record");
    let m_rec = records
        .iter()
        .find(|r| r["service_method"] == "Mail")
        .expect("Should have mail record");

    assert_eq!(e_rec["successful"], true);
    assert_eq!(e_rec["proof_of_service_filed"], true);

    assert_eq!(m_rec["successful"], false);
    assert_eq!(m_rec["proof_of_service_filed"], false);
}

#[tokio::test]
async fn submit_sealed_filing_with_reason_code() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-FS-00009").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "Sealed Motion with Reason",
        "filed_by": "Attorney Seal",
        "is_sealed": true,
        "sealing_level": "SealedCourtOnly",
        "reason_code": "JuvenileRecord",
    });

    let (status, response) =
        post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);

    // Verify the document has the sealing fields set
    let document_id = response["document_id"].as_str().unwrap();
    let doc: (bool, String, Option<String>) = sqlx::query_as(
        "SELECT is_sealed, sealing_level, seal_reason_code FROM documents WHERE id = $1::uuid AND court_id = 'district9'",
    )
    .bind(document_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch document");

    assert!(doc.0, "Document should be sealed");
    assert_eq!(doc.1, "SealedCourtOnly");
    assert_eq!(doc.2.as_deref(), Some("JuvenileRecord"));
}
