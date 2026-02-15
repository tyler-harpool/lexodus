use axum::http::StatusCode;
use crate::common;

async fn create_searchable(app: &axum::Router, court: &str, bar: &str, first: &str, last: &str, email: &str, firm: Option<&str>) -> String {
    let mut body = serde_json::json!({
        "bar_number": bar, "first_name": first, "last_name": last,
        "email": email, "phone": "555-0100",
        "address": {"street1": "1 St", "city": "C", "state": "S", "zip_code": "1", "country": "US"}
    });
    if let Some(f) = firm { body["firm_name"] = serde_json::json!(f); }
    let (_, r) = common::post_json(app, "/api/attorneys", &body.to_string(), court).await;
    r["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_search_by_first_name() {
    let (app, _pool, _guard) = common::test_app().await;
    create_searchable(&app, "district9", "SRCH1", "Alexander", "Hamilton", "ah@l.com", None).await;
    create_searchable(&app, "district9", "SRCH2", "Alexandra", "Madison", "am@l.com", None).await;
    create_searchable(&app, "district9", "SRCH3", "Benjamin", "Franklin", "bf@l.com", None).await;
    let (status, results) = common::get_with_court(&app, "/api/attorneys/search?q=Alex", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(results["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_search_by_last_name() {
    let (app, _pool, _guard) = common::test_app().await;
    create_searchable(&app, "district9", "SRCH4", "John", "Jefferson", "jj@l.com", None).await;
    create_searchable(&app, "district9", "SRCH5", "Jane", "Jefferson", "jnj@l.com", None).await;
    let (status, results) = common::get_with_court(&app, "/api/attorneys/search?q=Jefferson", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(results["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_search_by_bar_number() {
    let (app, _pool, _guard) = common::test_app().await;
    create_searchable(&app, "district12", "NYBAR123", "Test", "Attorney", "t@l.com", None).await;
    let (status, results) = common::get_with_court(&app, "/api/attorneys/search?q=NYBAR", "district12").await;
    assert_eq!(status, StatusCode::OK);
    assert!(results["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_search_by_email() {
    let (app, _pool, _guard) = common::test_app().await;
    create_searchable(&app, "district12", "EMAIL1", "Email", "Test", "unique@example.com", None).await;
    let (status, results) = common::get_with_court(&app, "/api/attorneys/search?q=unique@example", "district12").await;
    assert_eq!(status, StatusCode::OK);
    assert!(results["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_search_by_firm_name() {
    let (app, _pool, _guard) = common::test_app().await;
    create_searchable(&app, "district9", "FIRM1", "P", "One", "p1@l.com", Some("UniqueTestFirm")).await;
    create_searchable(&app, "district9", "FIRM2", "P", "Two", "p2@l.com", Some("UniqueTestFirm")).await;
    let (status, results) = common::get_with_court(&app, "/api/attorneys/search?q=UniqueTestFirm", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(results["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_search_case_insensitive() {
    let (app, _pool, _guard) = common::test_app().await;
    create_searchable(&app, "district12", "CASE1", "CaseSensitive", "TestName", "cs@l.com", None).await;
    for q in &["casesensitive", "CASESENSITIVE", "CaSeSeNsItIvE"] {
        let (status, results) = common::get_with_court(&app, &format!("/api/attorneys/search?q={}", q), "district12").await;
        assert_eq!(status, StatusCode::OK);
        assert!(results["data"].as_array().unwrap().len() >= 1, "Case insensitive failed for: {}", q);
    }
}

#[tokio::test]
async fn test_search_empty_query_returns_all() {
    let (app, _pool, _guard) = common::test_app().await;
    create_searchable(&app, "district9", "EMPTY1", "Test", "Empty", "empty@l.com", None).await;
    let (status, results) = common::get_with_court(&app, "/api/attorneys/search?q=", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert!(results["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_search_no_matches() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, results) = common::get_with_court(&app, "/api/attorneys/search?q=NonExistentXYZ", "district12").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(results["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_search_requires_district_header() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, _) = common::get_no_court(&app, "/api/attorneys/search?q=test").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_search_partial_matches() {
    let (app, _pool, _guard) = common::test_app().await;
    create_searchable(&app, "district9", "PARTIAL1", "Christopher", "Washington", "cw@l.com", Some("Washington Legal")).await;
    for q in &["Chris", "Wash", "Legal"] {
        let (status, results) = common::get_with_court(&app, &format!("/api/attorneys/search?q={}", q), "district9").await;
        assert_eq!(status, StatusCode::OK);
        assert!(results["data"].as_array().unwrap().len() >= 1, "Partial match failed for: {}", q);
    }
}
