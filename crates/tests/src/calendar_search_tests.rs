use axum::http::StatusCode;

use crate::common::*;

#[tokio::test]
async fn test_search_calendar_no_filters() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Search").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0040").await;

    create_test_calendar_event(&app, "district9", &case_id, &judge_id, "motion_hearing").await;
    create_test_calendar_event(&app, "district9", &case_id, &judge_id, "sentencing").await;

    let (status, resp) = get_with_court(&app, "/api/calendar/search", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 2);
    assert_eq!(resp["events"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_search_calendar_by_event_type() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Filter").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0041").await;

    create_test_calendar_event(&app, "district9", &case_id, &judge_id, "motion_hearing").await;
    create_test_calendar_event(&app, "district9", &case_id, &judge_id, "sentencing").await;
    create_test_calendar_event(&app, "district9", &case_id, &judge_id, "motion_hearing").await;

    let (status, resp) = get_with_court(
        &app,
        "/api/calendar/search?event_type=motion_hearing",
        "district9",
    ).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 2);
}

#[tokio::test]
async fn test_search_calendar_by_status() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Status").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0042").await;

    let event = create_test_calendar_event(&app, "district9", &case_id, &judge_id, "trial_date").await;
    let event_id = event["id"].as_str().unwrap();

    // Confirm the event
    let body = serde_json::json!({ "status": "confirmed" });
    let uri = format!("/api/calendar/events/{}/status", event_id);
    patch_json(&app, &uri, &body.to_string(), "district9").await;

    // Create another that stays 'scheduled'
    create_test_calendar_event(&app, "district9", &case_id, &judge_id, "arraignment").await;

    let (status, resp) = get_with_court(
        &app,
        "/api/calendar/search?status=confirmed",
        "district9",
    ).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["events"][0]["status"], "confirmed");
}

#[tokio::test]
async fn test_search_calendar_by_judge_id() {
    let (app, pool, _guard) = test_app().await;
    let judge_a = create_test_judge(&pool, "district9", "Judge Alpha").await;
    let judge_b = create_test_judge(&pool, "district9", "Judge Beta").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0043").await;

    create_test_calendar_event(&app, "district9", &case_id, &judge_a, "motion_hearing").await;
    create_test_calendar_event(&app, "district9", &case_id, &judge_b, "sentencing").await;

    let uri = format!("/api/calendar/search?judge_id={}", judge_a);
    let (status, resp) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["events"][0]["judge_id"], judge_a);
}

#[tokio::test]
async fn test_search_calendar_by_courtroom() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Room").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0044").await;

    // Default helper creates events with "Courtroom 4A"
    create_test_calendar_event(&app, "district9", &case_id, &judge_id, "arraignment").await;

    let (status, resp) = get_with_court(
        &app,
        "/api/calendar/search?courtroom=Courtroom%204A",
        "district9",
    ).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);

    // Non-matching courtroom
    let (status, resp) = get_with_court(
        &app,
        "/api/calendar/search?courtroom=Courtroom%205B",
        "district9",
    ).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 0);
}

#[tokio::test]
async fn test_search_calendar_pagination() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Page").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0045").await;

    for _ in 0..5 {
        create_test_calendar_event(&app, "district9", &case_id, &judge_id, "status_conference").await;
    }

    // First page of 2
    let (status, resp) = get_with_court(
        &app,
        "/api/calendar/search?limit=2&offset=0",
        "district9",
    ).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 5);
    assert_eq!(resp["events"].as_array().unwrap().len(), 2);

    // Second page
    let (status, resp) = get_with_court(
        &app,
        "/api/calendar/search?limit=2&offset=2",
        "district9",
    ).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 5);
    assert_eq!(resp["events"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_search_calendar_invalid_event_type_filter() {
    let (app, _pool, _guard) = test_app().await;

    let (status, resp) = get_with_court(
        &app,
        "/api/calendar/search?event_type=not_real",
        "district9",
    ).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(resp["message"].as_str().unwrap().contains("Invalid event_type"));
}

#[tokio::test]
async fn test_search_calendar_empty_result() {
    let (app, _pool, _guard) = test_app().await;

    let (status, resp) = get_with_court(&app, "/api/calendar/search", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 0);
    assert_eq!(resp["events"].as_array().unwrap().len(), 0);
}
