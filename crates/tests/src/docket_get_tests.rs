use axum::http::StatusCode;

use crate::common::*;

#[tokio::test]
async fn get_docket_entry_success() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-DG01").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();

    let (status, response) = get_with_court(
        &app,
        &format!("/api/docket/entries/{}", entry_id),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"], entry_id);
    assert_eq!(response["entry_type"], "motion");
    assert_eq!(response["case_id"], case_id);
}

#[tokio::test]
async fn get_docket_entry_not_found_404() {
    let (app, _pool, _guard) = test_app().await;

    let (status, _) = get_with_court(
        &app,
        "/api/docket/entries/00000000-0000-0000-0000-000000000000",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_docket_entry_invalid_uuid_400() {
    let (app, _pool, _guard) = test_app().await;

    let (status, _) = get_with_court(&app, "/api/docket/entries/not-a-uuid", "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_docket_entry_wrong_court_404() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-DG02").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();

    let (status, _) = get_with_court(
        &app,
        &format!("/api/docket/entries/{}", entry_id),
        "district12",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
