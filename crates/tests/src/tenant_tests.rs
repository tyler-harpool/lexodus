use axum::http::StatusCode;

use crate::common;

#[tokio::test]
async fn test_missing_court_header_returns_400() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, body) = common::get_no_court(&app, "/api/attorneys").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["message"].as_str().unwrap_or("").contains("X-Court-District"));
}

#[tokio::test]
async fn test_x_court_district_header_works() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, _body) = common::get_with_court(&app, "/api/attorneys", "district9").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_x_tenant_id_header_works() {
    let (app, _pool, _guard) = common::test_app().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/attorneys")
        .header("x-tenant-id", "district9")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app.clone(), req)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_query_param_fallback() {
    let (app, _pool, _guard) = common::test_app().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/attorneys?tenant=district9")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app.clone(), req)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_post_missing_court_header_returns_400() {
    let (app, _pool, _guard) = common::test_app().await;
    let body = r#"{"bar_number":"TEST","first_name":"A","last_name":"B","email":"a@b.com","phone":"555","address":{"street1":"1 St","city":"C","state":"S","zip_code":"1","country":"US"}}"#;
    let (status, _body) = common::post_no_court(&app, "/api/attorneys", body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_host_subdomain_parsing() {
    let (app, _pool, _guard) = common::test_app().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/attorneys")
        .header("host", "district9.lexodus.gov")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app.clone(), req)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK, "Subdomain parsing should resolve tenant");
}

#[tokio::test]
async fn test_invalid_tenant_code_sanitized() {
    let (app, _pool, _guard) = common::test_app().await;

    // Tenant with only special chars sanitizes to empty string -> 400
    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/attorneys")
        .header("x-court-district", "!!!@@@")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app.clone(), req)
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Invalid tenant code (all special chars) should be rejected"
    );
}

#[tokio::test]
async fn test_header_priority_x_tenant_id_over_x_court_district() {
    let (app, _pool, _guard) = common::test_app().await;

    // X-Tenant-ID should take priority over X-Court-District
    // Create attorney in district12 via X-Tenant-ID
    let body = serde_json::json!({
        "bar_number": "PRIO001", "first_name": "P", "last_name": "Test",
        "email": "p@l.com", "phone": "555",
        "address": {"street1": "1", "city": "C", "state": "S", "zip_code": "1", "country": "US"}
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/attorneys")
        .header("content-type", "application/json")
        .header("x-tenant-id", "district12")
        .header("x-court-district", "district9")
        .body(axum::body::Body::from(body.to_string()))
        .unwrap();

    let response = tower::ServiceExt::oneshot(app.clone(), req).await.unwrap();
    let status = response.status();
    assert!(status == StatusCode::OK || status == StatusCode::CREATED);

    // Should be visible in district12, not district9
    let (_, d12_list) = common::get_with_court(&app, "/api/attorneys", "district12").await;
    let bars: Vec<&str> = d12_list["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a["bar_number"].as_str())
        .collect();
    assert!(bars.contains(&"PRIO001"), "Attorney should be in district12 (X-Tenant-ID wins)");
}
