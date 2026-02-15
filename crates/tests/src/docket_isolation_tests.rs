use axum::http::StatusCode;

use crate::common::*;

#[tokio::test]
async fn district9_cannot_see_district12_docket_entries() {
    let (app, pool, _guard) = test_app().await;
    let case_d9 = create_test_case(&pool, "district9", "2026-CR-DI01").await;
    let entry = create_test_docket_entry(&app, "district9", &case_d9, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();

    // district12 cannot see district9's docket entry
    let (status, _) = get_with_court(
        &app,
        &format!("/api/docket/entries/{}", entry_id),
        "district12",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn district12_cannot_delete_district9_docket_entry() {
    let (app, pool, _guard) = test_app().await;
    let case_d9 = create_test_case(&pool, "district9", "2026-CR-DI02").await;
    let entry = create_test_docket_entry(&app, "district9", &case_d9, "order").await;
    let entry_id = entry["id"].as_str().unwrap();

    // district12 cannot delete district9's docket entry
    let (status, _) = delete_with_court(
        &app,
        &format!("/api/docket/entries/{}", entry_id),
        "district12",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Verify it still exists for district9
    let (status, _) = get_with_court(
        &app,
        &format!("/api/docket/entries/{}", entry_id),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn search_only_returns_own_court_docket_entries() {
    let (app, pool, _guard) = test_app().await;
    let case_d9 = create_test_case(&pool, "district9", "2026-CR-DI03").await;
    let case_d12 = create_test_case(&pool, "district12", "2026-CR-DI04").await;
    create_test_docket_entry(&app, "district9", &case_d9, "motion").await;
    create_test_docket_entry(&app, "district12", &case_d12, "order").await;

    let (status, response) = get_with_court(&app, "/api/docket/search", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["total"], 1);
    assert_eq!(response["entries"][0]["case_id"], case_d9);

    let (status, response2) = get_with_court(&app, "/api/docket/search", "district12").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response2["total"], 1);
    assert_eq!(response2["entries"][0]["case_id"], case_d12);
}
