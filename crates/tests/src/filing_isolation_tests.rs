use axum::http::StatusCode;

use crate::common::{get_with_court, post_json, test_app, create_test_case};

#[tokio::test]
async fn filing_submit_isolated_by_tenant() {
    let (app, pool, _guard) = test_app().await;

    // district9 case
    let d9_case = create_test_case(&pool, "district9", "2026-FI-00001").await;
    // district12 case
    let d12_case = create_test_case(&pool, "district12", "2026-FI-00002").await;

    // district9 trying to file on district12 case → should fail
    let body = serde_json::json!({
        "case_id": d12_case,
        "document_type": "Motion",
        "title": "Cross-tenant Filing",
        "filed_by": "Attorney Cross",
    });

    let (status, _) = post_json(&app, "/api/filings", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "Should reject cross-tenant filing");

    // district12 trying to file on district9 case → should fail
    let body = serde_json::json!({
        "case_id": d9_case,
        "document_type": "Notice",
        "title": "Reverse Cross-tenant Filing",
        "filed_by": "Attorney Reverse",
    });

    let (status, _) = post_json(&app, "/api/filings", &body.to_string(), "district12").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "Should reject reverse cross-tenant filing");
}

#[tokio::test]
async fn jurisdictions_returns_only_requesting_court() {
    let (app, _pool, _guard) = test_app().await;

    let (status, response) = get_with_court(&app, "/api/filings/jurisdictions", "district9").await;
    assert_eq!(status, StatusCode::OK);

    let courts = response.as_array().unwrap();
    assert_eq!(courts.len(), 1);
    assert_eq!(courts[0]["court_id"], "district9");
    assert_eq!(courts[0]["name"], "District 9 (Test)");

    // district12 should only see itself
    let (status, response) = get_with_court(&app, "/api/filings/jurisdictions", "district12").await;
    assert_eq!(status, StatusCode::OK);

    let courts = response.as_array().unwrap();
    assert_eq!(courts.len(), 1);
    assert_eq!(courts[0]["court_id"], "district12");
}
