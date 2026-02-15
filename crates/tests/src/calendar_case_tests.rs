use axum::http::StatusCode;

use crate::common::*;

#[tokio::test]
async fn test_get_case_calendar_success() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Case").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0050").await;

    create_test_calendar_event(&app, "district9", &case_id, &judge_id, "arraignment").await;
    create_test_calendar_event(&app, "district9", &case_id, &judge_id, "plea_hearing").await;
    create_test_calendar_event(&app, "district9", &case_id, &judge_id, "sentencing").await;

    let uri = format!("/api/cases/{}/calendar", case_id);
    let (status, resp) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(status, StatusCode::OK);

    let events = resp.as_array().unwrap();
    assert_eq!(events.len(), 3);

    // All events should belong to the same case
    for event in events {
        assert_eq!(event["case_id"], case_id);
    }
}

#[tokio::test]
async fn test_get_case_calendar_empty() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0051").await;

    let uri = format!("/api/cases/{}/calendar", case_id);
    let (status, resp) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_case_calendar_tenant_isolation() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Iso").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0052").await;

    create_test_calendar_event(&app, "district9", &case_id, &judge_id, "trial_date").await;

    // Querying from district12 should return empty
    let uri = format!("/api/cases/{}/calendar", case_id);
    let (status, resp) = get_with_court(&app, &uri, "district12").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_case_calendar_invalid_uuid() {
    let (app, _pool, _guard) = test_app().await;

    let (status, resp) = get_with_court(
        &app,
        "/api/cases/not-a-uuid/calendar",
        "district9",
    ).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(resp["message"].as_str().unwrap().contains("Invalid UUID"));
}

#[tokio::test]
async fn test_get_case_calendar_only_returns_case_events() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Multi").await;
    let case_a = create_test_case(&pool, "district9", "CR-2026-0053").await;
    let case_b = create_test_case(&pool, "district9", "CR-2026-0054").await;

    create_test_calendar_event(&app, "district9", &case_a, &judge_id, "arraignment").await;
    create_test_calendar_event(&app, "district9", &case_a, &judge_id, "sentencing").await;
    create_test_calendar_event(&app, "district9", &case_b, &judge_id, "plea_hearing").await;

    let uri = format!("/api/cases/{}/calendar", case_a);
    let (status, resp) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp.as_array().unwrap().len(), 2);
}
