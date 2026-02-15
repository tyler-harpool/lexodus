use axum::http::StatusCode;

use crate::common::{
    create_test_case, create_test_party, get_with_court, post_json, test_app,
};

#[tokio::test]
async fn district9_cannot_access_district12_nef() {
    let (app, pool, _guard) = test_app().await;

    // Create case + party + filing in district12
    let case_id = create_test_case(&pool, "district12", "2026-NI-00001").await;
    let _party_id = create_test_party(&pool, "district12", &case_id).await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Motion",
        "title": "District 12 Filing",
        "filed_by": "Attorney D12",
    });

    let (status, response) =
        post_json(&app, "/api/filings", &body.to_string(), "district12").await;
    assert_eq!(status, StatusCode::CREATED);

    let filing_id = response["filing_id"].as_str().unwrap();

    // district12 can see its own NEF
    let uri = format!("/api/filings/{}/nef", filing_id);
    let (d12_status, _) = get_with_court(&app, &uri, "district12").await;
    assert_eq!(d12_status, StatusCode::OK);

    // district9 cannot see district12 NEF
    let (d9_status, _) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(
        d9_status,
        StatusCode::NOT_FOUND,
        "district9 should not see district12 NEF"
    );
}

#[tokio::test]
async fn service_records_only_visible_in_filing_tenant() {
    let (app, pool, _guard) = test_app().await;

    // Create case + party + filing in district9
    let case_id = create_test_case(&pool, "district9", "2026-NI-00002").await;
    let _party_id = create_test_party(&pool, "district9", &case_id).await;

    let body = serde_json::json!({
        "case_id": case_id,
        "document_type": "Notice",
        "title": "District 9 Filing",
        "filed_by": "Attorney D9",
    });

    let (status, response) =
        post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);

    let document_id = response["document_id"].as_str().unwrap();

    // district9 can see the service records
    let uri = format!("/api/service-records/document/{}", document_id);
    let (d9_status, d9_resp) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(d9_status, StatusCode::OK);
    assert_eq!(d9_resp.as_array().unwrap().len(), 1);

    // district12 cannot see district9's document at all
    let (d12_status, _d12_resp) = get_with_court(&app, &uri, "district12").await;
    assert_eq!(
        d12_status,
        StatusCode::NOT_FOUND,
        "district12 should get 404 for district9's document"
    );
}
