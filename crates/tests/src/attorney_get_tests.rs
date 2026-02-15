use axum::http::StatusCode;
use crate::common;

#[tokio::test]
async fn test_get_attorney_success() {
    let (app, _pool, _guard) = common::test_app().await;
    let id = common::create_test_attorney(&app, "district9", "GET001").await;
    let (status, response) = common::get_with_court(&app, &format!("/api/attorneys/{}", id), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"], id);
    assert_eq!(response["bar_number"], "GET001");
}

#[tokio::test]
async fn test_get_attorney_not_found() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, _) = common::get_with_court(&app, "/api/attorneys/00000000-0000-0000-0000-000000000000", "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_attorney_missing_header() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, _) = common::get_no_court(&app, "/api/attorneys/00000000-0000-0000-0000-000000000000").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_attorney_full_fields() {
    let (app, _pool, _guard) = common::test_app().await;
    let body = serde_json::json!({
        "bar_number": "FULL001",
        "first_name": "Full",
        "last_name": "Fields",
        "middle_name": "Middle",
        "firm_name": "Full Firm",
        "email": "full@test.com",
        "phone": "555-1234",
        "fax": "555-5678",
        "address": {"street1": "1 Full St", "street2": "Apt 2", "city": "City", "state": "ST", "zip_code": "12345", "country": "USA"}
    });
    let (status, response) = common::post_json(&app, "/api/attorneys", &body.to_string(), "district9").await;
    assert!(status == StatusCode::OK || status == StatusCode::CREATED);
    let id = response["id"].as_str().unwrap();

    let (status, detail) = common::get_with_court(&app, &format!("/api/attorneys/{}", id), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["middle_name"], "Middle");
    assert_eq!(detail["firm_name"], "Full Firm");
    assert_eq!(detail["fax"], "555-5678");
    assert_eq!(detail["address"]["street2"], "Apt 2");
}
