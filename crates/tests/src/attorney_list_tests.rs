use axum::http::StatusCode;
use crate::common;

#[tokio::test]
async fn test_list_attorneys_empty() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, response) = common::get_with_court(&app, "/api/attorneys", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert!(response["data"].is_array());
    assert!(response["meta"].is_object());
}

#[tokio::test]
async fn test_list_attorneys_with_data() {
    let (app, _pool, _guard) = common::test_app().await;
    common::create_test_attorney(&app, "district9", "LIST001").await;
    common::create_test_attorney(&app, "district9", "LIST002").await;
    let (status, response) = common::get_with_court(&app, "/api/attorneys", "district9").await;
    assert_eq!(status, StatusCode::OK);
    let data = response["data"].as_array().unwrap();
    assert!(data.len() >= 2);
}

#[tokio::test]
async fn test_list_attorneys_missing_header() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, _) = common::get_no_court(&app, "/api/attorneys").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_attorneys_full_objects() {
    let (app, _pool, _guard) = common::test_app().await;
    common::create_test_attorney(&app, "district9", "LISTFULL").await;
    let (status, response) = common::get_with_court(&app, "/api/attorneys", "district9").await;
    assert_eq!(status, StatusCode::OK);
    let first = &response["data"][0];
    assert!(first["id"].is_string());
    assert!(first["bar_number"].is_string());
    assert!(first["first_name"].is_string());
    assert!(first["address"].is_object());
}

#[tokio::test]
async fn test_list_attorneys_after_deletion() {
    let (app, _pool, _guard) = common::test_app().await;
    let id = common::create_test_attorney(&app, "district9", "LISTDEL").await;
    let (_, initial) = common::get_with_court(&app, "/api/attorneys", "district9").await;
    let initial_count = initial["data"].as_array().unwrap().len();
    common::delete_with_court(&app, &format!("/api/attorneys/{}", id), "district9").await;
    let (_, after) = common::get_with_court(&app, "/api/attorneys", "district9").await;
    let after_count = after["data"].as_array().unwrap().len();
    assert_eq!(after_count, initial_count - 1);
}
