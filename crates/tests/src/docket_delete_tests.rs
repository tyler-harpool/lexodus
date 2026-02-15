use axum::http::StatusCode;

use crate::common::*;

#[tokio::test]
async fn delete_docket_entry_success_204() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-DD01").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();

    let (status, _) = delete_with_court(
        &app,
        &format!("/api/docket/entries/{}", entry_id),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's actually gone
    let (status, _) = get_with_court(
        &app,
        &format!("/api/docket/entries/{}", entry_id),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_docket_entry_not_found_404() {
    let (app, _pool, _guard) = test_app().await;

    let (status, _) = delete_with_court(
        &app,
        "/api/docket/entries/00000000-0000-0000-0000-000000000000",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_docket_entry_invalid_uuid_400() {
    let (app, _pool, _guard) = test_app().await;

    let (status, _) =
        delete_with_court(&app, "/api/docket/entries/not-a-uuid", "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
