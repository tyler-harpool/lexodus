use axum::http::StatusCode;

use crate::common;

#[tokio::test]
async fn test_init_tenant_creates_court() {
    let (app, _pool, _guard) = common::test_app().await;

    let body = r#"{"id":"testcourt1","name":"Test Court 1","court_type":"district"}"#;
    let (status, response) = common::post_json(&app, "/api/admin/tenants/init", body, "district9").await;
    // init_tenant doesn't require court header itself, but since we route through the same router let's pass one
    // Actually init_tenant doesn't use CourtId extractor - it takes body params
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"], "testcourt1");
    assert_eq!(response["name"], "Test Court 1");
}

#[tokio::test]
async fn test_init_tenant_idempotent() {
    let (app, _pool, _guard) = common::test_app().await;

    let body = r#"{"id":"testcourt2","name":"Test Court 2","court_type":"district"}"#;

    // First call
    let (status1, _) = common::post_json(&app, "/api/admin/tenants/init", body, "district9").await;
    assert_eq!(status1, StatusCode::OK);

    // Second call (idempotent)
    let (status2, response2) = common::post_json(&app, "/api/admin/tenants/init", body, "district9").await;
    assert_eq!(status2, StatusCode::OK);
    assert_eq!(response2["id"], "testcourt2");
}

#[tokio::test]
async fn test_tenant_stats_returns_counts() {
    let (app, _pool, _guard) = common::test_app().await;

    // Create an attorney first
    common::create_test_attorney(&app, "district9", "STATS001").await;

    let (status, response) = common::get_with_court(&app, "/api/admin/tenants/stats", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["court_id"], "district9");
    assert!(response["attorney_count"].as_i64().unwrap() >= 1);
}
