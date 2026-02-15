use axum::http::StatusCode;

use crate::common::*;

#[tokio::test]
async fn get_case_docket_empty() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-DC01").await;

    let (status, response) = get_with_court(
        &app,
        &format!("/api/cases/{}/docket", case_id),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["entries"].as_array().unwrap().len(), 0);
    assert_eq!(response["total"], 0);
}

#[tokio::test]
async fn get_case_docket_returns_entries_in_order() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-DC02").await;
    create_test_docket_entry(&app, "district9", &case_id, "complaint").await;
    create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    create_test_docket_entry(&app, "district9", &case_id, "order").await;

    let (status, response) = get_with_court(
        &app,
        &format!("/api/cases/{}/docket", case_id),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let entries = response["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(response["total"], 3);

    // Should be ordered by entry_number ascending
    assert_eq!(entries[0]["entry_number"], 1);
    assert_eq!(entries[1]["entry_number"], 2);
    assert_eq!(entries[2]["entry_number"], 3);
    assert_eq!(entries[0]["entry_type"], "complaint");
    assert_eq!(entries[1]["entry_type"], "motion");
    assert_eq!(entries[2]["entry_type"], "order");
}

#[tokio::test]
async fn get_case_docket_only_returns_own_case() {
    let (app, pool, _guard) = test_app().await;
    let case1 = create_test_case(&pool, "district9", "2026-CR-DC03").await;
    let case2 = create_test_case(&pool, "district9", "2026-CR-DC04").await;
    create_test_docket_entry(&app, "district9", &case1, "motion").await;
    create_test_docket_entry(&app, "district9", &case2, "order").await;

    let (status, response) = get_with_court(
        &app,
        &format!("/api/cases/{}/docket", case1),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let entries = response["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["case_id"], case1);
}

#[tokio::test]
async fn get_case_docket_invalid_uuid_400() {
    let (app, _pool, _guard) = test_app().await;

    let (status, _) = get_with_court(&app, "/api/cases/not-a-uuid/docket", "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_case_docket_pagination() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "2026-CR-DC05").await;

    for _ in 0..5 {
        create_test_docket_entry(&app, "district9", &case_id, "notice").await;
    }

    let (status, response) = get_with_court(
        &app,
        &format!("/api/cases/{}/docket?limit=3&offset=0", case_id),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["entries"].as_array().unwrap().len(), 3);
    assert_eq!(response["total"], 5);
}
