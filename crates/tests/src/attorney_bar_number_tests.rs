use axum::http::StatusCode;
use crate::common;

#[tokio::test]
async fn test_get_by_bar_number_success() {
    let (app, _pool, _guard) = common::test_app().await;
    common::create_test_attorney(&app, "district9", "BARNR001").await;
    let (status, response) = common::get_with_court(
        &app,
        "/api/attorneys/bar-number/BARNR001",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["bar_number"], "BARNR001");
}

#[tokio::test]
async fn test_get_by_bar_number_not_found() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, _) = common::get_with_court(
        &app,
        "/api/attorneys/bar-number/NONEXISTENT",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_by_bar_number_wrong_tenant() {
    let (app, _pool, _guard) = common::test_app().await;
    common::create_test_attorney(&app, "district9", "BARNR002").await;
    let (status, _) = common::get_with_court(
        &app,
        "/api/attorneys/bar-number/BARNR002",
        "district12",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "Bar number from another tenant should not be visible");
}

#[tokio::test]
async fn test_get_by_bar_number_missing_header() {
    let (app, _pool, _guard) = common::test_app().await;
    let (status, _) = common::get_no_court(
        &app,
        "/api/attorneys/bar-number/BARNR001",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
