use axum::http::StatusCode;
use crate::common;

#[tokio::test]
async fn test_delete_attorney_success() {
    let (app, _pool, _guard) = common::test_app().await;
    let id = common::create_test_attorney(&app, "district9", "DEL001").await;
    let (status, _) = common::delete_with_court(&app, &format!("/api/attorneys/{}", id), "district9").await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_attorney_not_found() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, _) = common::delete_with_court(&app, "/api/attorneys/00000000-0000-0000-0000-000000000000", "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_attorney_missing_header() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, _) = common::get_no_court(&app, "/api/attorneys/00000000-0000-0000-0000-000000000000").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_then_get_returns_404() {
    let (app, _pool, _guard) = common::test_app().await;
    let id = common::create_test_attorney(&app, "district9", "DEL002").await;
    let (del_status, _) = common::delete_with_court(&app, &format!("/api/attorneys/{}", id), "district9").await;
    assert_eq!(del_status, StatusCode::NO_CONTENT);
    let (get_status, _) = common::get_with_court(&app, &format!("/api/attorneys/{}", id), "district9").await;
    assert_eq!(get_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_attorney_idempotent_gives_404() {
    let (app, _pool, _guard) = common::test_app().await;
    let id = common::create_test_attorney(&app, "district9", "DEL003").await;
    let (s1, _) = common::delete_with_court(&app, &format!("/api/attorneys/{}", id), "district9").await;
    assert_eq!(s1, StatusCode::NO_CONTENT);
    let (s2, _) = common::delete_with_court(&app, &format!("/api/attorneys/{}", id), "district9").await;
    assert_eq!(s2, StatusCode::NOT_FOUND);
}
