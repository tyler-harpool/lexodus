use axum::http::StatusCode;
use crate::common;

#[tokio::test]
async fn test_pagination_first_page() {
    let (app, _pool, _guard) = common::test_app().await;
    for i in 0..5 { common::create_test_attorney(&app, "district9", &format!("PAGE{}", i)).await; }
    let (status, response) = common::get_with_court(&app, "/api/attorneys?page=1&limit=3", "district9").await;
    assert_eq!(status, StatusCode::OK);
    let data = response["data"].as_array().unwrap();
    assert!(data.len() <= 3);
    let meta = &response["meta"];
    assert_eq!(meta["page"], 1);
    assert_eq!(meta["limit"], 3);
    assert!(meta["total"].as_i64().unwrap() >= 5);
    assert!(meta["has_next"].as_bool().unwrap());
    assert!(!meta["has_prev"].as_bool().unwrap());
}

#[tokio::test]
async fn test_pagination_second_page() {
    let (app, _pool, _guard) = common::test_app().await;
    for i in 0..5 { common::create_test_attorney(&app, "district12", &format!("PG2_{}", i)).await; }
    let (status, response) = common::get_with_court(&app, "/api/attorneys?page=2&limit=2", "district12").await;
    assert_eq!(status, StatusCode::OK);
    let data = response["data"].as_array().unwrap();
    assert!(data.len() <= 2);
    assert_eq!(response["meta"]["page"], 2);
    assert!(response["meta"]["has_prev"].as_bool().unwrap());
}

#[tokio::test]
async fn test_pagination_empty_page() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, response) = common::get_with_court(&app, "/api/attorneys?page=1000&limit=20", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_pagination_defaults() {
    let (app, _pool, _guard) = common::test_app().await;
    for i in 0..25 { common::create_test_attorney(&app, "district12", &format!("DFLT{}", i)).await; }
    let (status, response) = common::get_with_court(&app, "/api/attorneys", "district12").await;
    assert_eq!(status, StatusCode::OK);
    let data = response["data"].as_array().unwrap();
    assert_eq!(data.len(), 20, "Default limit should be 20");
    assert_eq!(response["meta"]["page"], 1);
    assert_eq!(response["meta"]["limit"], 20);
}

#[tokio::test]
async fn test_search_pagination() {
    let (app, _pool, _guard) = common::test_app().await;
    for i in 0..10 {
        let body = serde_json::json!({
            "bar_number": format!("SPAG{}", i), "first_name": "Searchable", "last_name": format!("Atty{}", i),
            "email": format!("sp{}@l.com", i), "phone": "555",
            "address": {"street1": "1", "city": "C", "state": "S", "zip_code": "1", "country": "US"}
        });
        common::post_json(&app, "/api/attorneys", &body.to_string(), "district9").await;
    }
    let (status, response) = common::get_with_court(&app, "/api/attorneys/search?q=Searchable&page=1&limit=5", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["data"].as_array().unwrap().len(), 5);
    assert_eq!(response["meta"]["total"], 10);
    assert!(response["meta"]["has_next"].as_bool().unwrap());
}

#[tokio::test]
async fn test_pagination_limit_max() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, response) = common::get_with_court(&app, "/api/attorneys?page=1&limit=200", "district12").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["meta"]["limit"], 100, "Limit should be capped at 100");
}

#[tokio::test]
async fn test_pagination_invalid_params() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, response) = common::get_with_court(&app, "/api/attorneys?page=0&limit=10", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["meta"]["page"], 1, "Page 0 should default to 1");

    let (status2, response2) = common::get_with_court(&app, "/api/attorneys?page=1&limit=0", "district9").await;
    assert_eq!(status2, StatusCode::OK);
    assert_eq!(response2["meta"]["limit"], 1, "Limit 0 should be clamped to 1");
}

#[tokio::test]
async fn test_pagination_metadata_accuracy() {
    let (app, _pool, _guard) = common::test_app().await;
    for i in 0..7 { common::create_test_attorney(&app, "district12", &format!("META{}", i)).await; }
    let (_, response) = common::get_with_court(&app, "/api/attorneys?page=1&limit=3", "district12").await;
    let meta = &response["meta"];
    assert!(meta["total"].as_i64().unwrap() >= 7);
    assert!(meta["total_pages"].as_i64().unwrap() >= 3);
    assert!(meta["has_next"].as_bool().unwrap());
    assert!(!meta["has_prev"].as_bool().unwrap());
}
