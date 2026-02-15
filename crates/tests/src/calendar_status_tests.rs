use axum::http::StatusCode;

use crate::common::*;

#[tokio::test]
async fn test_update_event_status_success() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Brown").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0020").await;
    let event = create_test_calendar_event(&app, "district9", &case_id, &judge_id, "motion_hearing").await;
    let event_id = event["id"].as_str().unwrap();

    let body = serde_json::json!({
        "status": "confirmed"
    });

    let uri = format!("/api/calendar/events/{}/status", event_id);
    let (status, resp) = patch_json(&app, &uri, &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["status"], "confirmed");
    assert_eq!(resp["id"], event_id);
}

#[tokio::test]
async fn test_update_event_status_with_timing() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge White").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0021").await;
    let event = create_test_calendar_event(&app, "district9", &case_id, &judge_id, "plea_hearing").await;
    let event_id = event["id"].as_str().unwrap();

    let body = serde_json::json!({
        "status": "in_progress",
        "actual_start": "2026-06-15T09:05:00Z"
    });

    let uri = format!("/api/calendar/events/{}/status", event_id);
    let (status, resp) = patch_json(&app, &uri, &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["status"], "in_progress");
    assert!(resp["actual_start"].is_string());
}

#[tokio::test]
async fn test_update_event_status_completed_with_end_time() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Green").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0022").await;
    let event = create_test_calendar_event(&app, "district9", &case_id, &judge_id, "sentencing").await;
    let event_id = event["id"].as_str().unwrap();

    let body = serde_json::json!({
        "status": "completed",
        "actual_start": "2026-06-15T09:00:00Z",
        "actual_end": "2026-06-15T10:30:00Z",
        "notes": "Sentencing completed, 36 months custody"
    });

    let uri = format!("/api/calendar/events/{}/status", event_id);
    let (status, resp) = patch_json(&app, &uri, &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["status"], "completed");
    assert!(resp["actual_start"].is_string());
    assert!(resp["actual_end"].is_string());
    assert_eq!(resp["notes"], "Sentencing completed, 36 months custody");
}

#[tokio::test]
async fn test_update_event_status_invalid_status() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Gray").await;
    let case_id = create_test_case(&pool, "district9", "CR-2026-0023").await;
    let event = create_test_calendar_event(&app, "district9", &case_id, &judge_id, "arraignment").await;
    let event_id = event["id"].as_str().unwrap();

    let body = serde_json::json!({
        "status": "invalid_status"
    });

    let uri = format!("/api/calendar/events/{}/status", event_id);
    let (status, resp) = patch_json(&app, &uri, &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(resp["message"].as_str().unwrap().contains("Invalid status"));
}

#[tokio::test]
async fn test_update_event_status_not_found() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "status": "confirmed"
    });

    let uri = "/api/calendar/events/00000000-0000-0000-0000-000000000099/status";
    let (status, _resp) = patch_json(&app, uri, &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_event_status_all_statuses() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Black").await;

    let statuses = [
        "scheduled", "confirmed", "in_progress", "completed",
        "cancelled", "postponed", "recessed", "continued",
    ];

    for s in &statuses {
        let case_id = create_test_case(
            &pool,
            "district9",
            &format!("CR-2026-ST-{}", s),
        ).await;
        let event = create_test_calendar_event(
            &app, "district9", &case_id, &judge_id, "status_conference",
        ).await;
        let event_id = event["id"].as_str().unwrap();

        let body = serde_json::json!({ "status": s });
        let uri = format!("/api/calendar/events/{}/status", event_id);
        let (status, resp) = patch_json(&app, &uri, &body.to_string(), "district9").await;
        assert_eq!(status, StatusCode::OK, "Failed for status: {}", s);
        assert_eq!(resp["status"], *s);
    }
}
