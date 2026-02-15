use axum::http::StatusCode;
use crate::common;

#[tokio::test]
async fn test_same_bar_number_different_tenant_ok() {
    let (app, _pool, _guard) = common::test_app().await;
    let body = serde_json::json!({
        "bar_number": "CROSS001", "first_name": "Cross", "last_name": "Tenant",
        "email": "cross@l.com", "phone": "555",
        "address": {"street1": "1", "city": "C", "state": "S", "zip_code": "1", "country": "US"}
    });
    let (s1, _) = common::post_json(&app, "/api/attorneys", &body.to_string(), "district9").await;
    assert!(s1 == StatusCode::OK || s1 == StatusCode::CREATED);

    let body2 = serde_json::json!({
        "bar_number": "CROSS001", "first_name": "Cross", "last_name": "Other",
        "email": "cross2@l.com", "phone": "555",
        "address": {"street1": "2", "city": "C", "state": "S", "zip_code": "1", "country": "US"}
    });
    let (s2, _) = common::post_json(&app, "/api/attorneys", &body2.to_string(), "district12").await;
    assert!(s2 == StatusCode::OK || s2 == StatusCode::CREATED, "Same bar_number in different tenant should succeed");
}

#[tokio::test]
async fn test_get_by_id_wrong_tenant_returns_404() {
    let (app, _pool, _guard) = common::test_app().await;
    let id = common::create_test_attorney(&app, "district9", "ISO001").await;
    let (status, _) = common::get_with_court(&app, &format!("/api/attorneys/{}", id), "district12").await;
    assert_eq!(status, StatusCode::NOT_FOUND, "Should not find attorney from different tenant");
}

#[tokio::test]
async fn test_list_only_shows_own_tenant() {
    let (app, _pool, _guard) = common::test_app().await;
    common::create_test_attorney(&app, "district9", "TENISO1").await;
    common::create_test_attorney(&app, "district12", "TENISO2").await;

    let (_, d9) = common::get_with_court(&app, "/api/attorneys", "district9").await;
    let d9_bars: Vec<&str> = d9["data"].as_array().unwrap().iter()
        .filter_map(|a| a["bar_number"].as_str())
        .collect();
    assert!(d9_bars.contains(&"TENISO1"));
    assert!(!d9_bars.contains(&"TENISO2"), "district9 should not see district12's attorneys");

    let (_, d12) = common::get_with_court(&app, "/api/attorneys", "district12").await;
    let d12_bars: Vec<&str> = d12["data"].as_array().unwrap().iter()
        .filter_map(|a| a["bar_number"].as_str())
        .collect();
    assert!(d12_bars.contains(&"TENISO2"));
    assert!(!d12_bars.contains(&"TENISO1"), "district12 should not see district9's attorneys");
}
