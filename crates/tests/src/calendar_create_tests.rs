use axum::http::StatusCode;

use crate::common::*;

#[tokio::test]
async fn test_schedule_event_success() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Smith").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0001").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "event_type": "motion_hearing",
        "scheduled_date": "2026-06-15T09:00:00Z",
        "duration_minutes": 60,
        "courtroom": "Courtroom 4A",
        "description": "Motion to suppress evidence",
        "participants": ["Prosecutor", "Defense Counsel"],
        "is_public": true
    });

    let (status, resp) = post_json(&app, "/api/calendar/events", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(resp["id"].is_string());
    assert_eq!(resp["event_type"], "motion_hearing");
    assert_eq!(resp["status"], "scheduled");
    assert_eq!(resp["courtroom"], "Courtroom 4A");
    assert_eq!(resp["duration_minutes"], 60);
    assert_eq!(resp["is_public"], true);
    assert_eq!(resp["participants"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_schedule_event_invalid_event_type() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Jones").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0002").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "event_type": "invalid_type",
        "scheduled_date": "2026-06-15T09:00:00Z",
        "duration_minutes": 60,
        "courtroom": "Courtroom 1",
        "description": "Bad event type test",
        "participants": [],
        "is_public": true
    });

    let (status, resp) = post_json(&app, "/api/calendar/events", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(resp["message"].as_str().unwrap().contains("Invalid event_type"));
}

#[tokio::test]
async fn test_schedule_event_invalid_duration() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Lee").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0003").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "event_type": "arraignment",
        "scheduled_date": "2026-06-15T09:00:00Z",
        "duration_minutes": -10,
        "courtroom": "Courtroom 1",
        "description": "Negative duration test",
        "participants": [],
        "is_public": true
    });

    let (status, resp) = post_json(&app, "/api/calendar/events", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(resp["message"].as_str().unwrap().contains("duration_minutes"));
}

#[tokio::test]
async fn test_schedule_event_missing_court_header() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "case_id": "00000000-0000-0000-0000-000000000001",
        "judge_id": "00000000-0000-0000-0000-000000000002",
        "event_type": "sentencing",
        "scheduled_date": "2026-06-15T09:00:00Z",
        "duration_minutes": 60,
        "courtroom": "Courtroom 1",
        "description": "No court header test",
        "participants": [],
        "is_public": true
    });

    let (status, _resp) = post_no_court(&app, "/api/calendar/events", &body.to_string()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_schedule_event_all_event_types() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Chen").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0004").await;

    let event_types = [
        "initial_appearance", "arraignment", "bail_hearing", "plea_hearing",
        "trial_date", "sentencing", "violation_hearing", "status_conference",
        "scheduling_conference", "settlement_conference", "pretrial_conference",
        "motion_hearing", "evidentiary_hearing", "jury_selection", "jury_trial",
        "bench_trial", "show_cause_hearing", "contempt_hearing", "emergency_hearing",
        "telephonic", "video_conference",
    ];

    for et in &event_types {
        let body = serde_json::json!({
            "case_id": case_id,
            "judge_id": judge_id,
            "event_type": et,
            "scheduled_date": "2026-07-01T10:00:00Z",
            "duration_minutes": 30,
            "courtroom": "Courtroom 1",
            "description": format!("Test {}", et),
            "participants": [],
            "is_public": true
        });

        let (status, resp) = post_json(&app, "/api/calendar/events", &body.to_string(), "district9").await;
        assert_eq!(status, StatusCode::CREATED, "Failed for event_type: {}", et);
        assert_eq!(resp["event_type"], *et);
    }
}

#[tokio::test]
async fn test_schedule_event_tenant_isolation() {
    let (app, pool, _guard) = test_app().await;
    let judge_d9 = create_test_judge(&pool, "district9", "Judge D9").await;
    let case_d9 = create_test_case(&pool, "district9", "CR-2026-0010").await;
    let judge_d12 = create_test_judge(&pool, "district12", "Judge D12").await;
    let case_d12 = create_test_case(&pool, "district12", "CR-2026-0011").await;

    // Create event in district9
    let event_d9 = create_test_calendar_event(&app, "district9", &case_d9, &judge_d9, "arraignment").await;
    let event_id = event_d9["id"].as_str().unwrap();

    // Create event in district12
    create_test_calendar_event(&app, "district12", &case_d12, &judge_d12, "sentencing").await;

    // Search district9 should only find district9 events
    let (status, resp) = get_with_court(&app, "/api/calendar/search", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["events"][0]["id"], event_id);

    // Search district12 should only find district12 events
    let (status, resp) = get_with_court(&app, "/api/calendar/search", "district12").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["events"][0]["event_type"], "sentencing");
}
