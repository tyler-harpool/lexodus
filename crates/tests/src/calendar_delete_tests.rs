use axum::http::StatusCode;

use crate::common::*;

#[tokio::test]
async fn test_delete_event_success() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Del").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0030").await;
    let event = create_test_calendar_event(&app, "district9", &case_id, &judge_id, "bail_hearing").await;
    let event_id = event["id"].as_str().unwrap();

    let uri = format!("/api/calendar/events/{}", event_id);
    let (status, _resp) = delete_with_court(&app, &uri, "district9").await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's gone via search
    let (status, resp) = get_with_court(&app, "/api/calendar/search", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 0);
}

#[tokio::test]
async fn test_delete_event_not_found() {
    let (app, _pool, _guard) = test_app().await;

    let (status, _resp) = delete_with_court(
        &app,
        "/api/calendar/events/00000000-0000-0000-0000-000000000099",
        "district9",
    ).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_event_invalid_uuid() {
    let (app, _pool, _guard) = test_app().await;

    let (status, resp) = delete_with_court(
        &app,
        "/api/calendar/events/not-a-uuid",
        "district9",
    ).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(resp["message"].as_str().unwrap().contains("Invalid UUID"));
}

#[tokio::test]
async fn test_delete_event_wrong_court() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Cross").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0031").await;
    let event = create_test_calendar_event(&app, "district9", &case_id, &judge_id, "arraignment").await;
    let event_id = event["id"].as_str().unwrap();

    // Try to delete from district12 — should fail (tenant isolation)
    let uri = format!("/api/calendar/events/{}", event_id);
    let (status, _resp) = delete_with_court(&app, &uri, "district12").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
