use axum::http::StatusCode;

use crate::common::{
    test_app, post_json, get_with_court, patch_json,
    create_test_case, create_test_deadline, create_test_deadline_with_case,
};

#[tokio::test]
async fn search_deadlines_empty() {
    let (app, _pool, _guard) = test_app().await;

    let (status, resp) = get_with_court(&app, "/api/deadlines/search", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 0);
    assert_eq!(resp["deadlines"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn search_deadlines_returns_all() {
    let (app, _pool, _guard) = test_app().await;

    create_test_deadline(&app, "district9", "DL A").await;
    create_test_deadline(&app, "district9", "DL B").await;
    create_test_deadline(&app, "district9", "DL C").await;

    let (status, resp) = get_with_court(&app, "/api/deadlines/search", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 3);
    assert_eq!(resp["deadlines"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn search_deadlines_filter_by_status() {
    let (app, _pool, _guard) = test_app().await;

    let dl = create_test_deadline(&app, "district9", "Open DL").await;
    let dl_id = dl["id"].as_str().unwrap();

    create_test_deadline(&app, "district9", "Another Open DL").await;

    // Mark one as met
    let body = serde_json::json!({ "status": "met" });
    patch_json(
        &app,
        &format!("/api/deadlines/{}/status", dl_id),
        &body.to_string(),
        "district9",
    )
    .await;

    // Search for open only
    let (status, resp) = get_with_court(&app, "/api/deadlines/search?status=open", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);

    // Search for met only
    let (status, resp) = get_with_court(&app, "/api/deadlines/search?status=met", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
}

#[tokio::test]
async fn search_deadlines_filter_by_case_id() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "DL-SEARCH-001").await;

    create_test_deadline_with_case(&app, "district9", "Linked DL", &case_id).await;
    create_test_deadline(&app, "district9", "Unlinked DL").await;

    let uri = format!("/api/deadlines/search?case_id={}", case_id);
    let (status, resp) = get_with_court(&app, &uri, "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["deadlines"][0]["case_id"], case_id);
}

#[tokio::test]
async fn search_deadlines_filter_by_date_range() {
    let (app, _pool, _guard) = test_app().await;

    // Create deadlines with different due dates
    let body_early = serde_json::json!({
        "title": "Early DL",
        "due_at": "2026-03-01T12:00:00Z"
    });
    let body_late = serde_json::json!({
        "title": "Late DL",
        "due_at": "2026-12-01T12:00:00Z"
    });
    post_json(&app, "/api/deadlines", &body_early.to_string(), "district9").await;
    post_json(&app, "/api/deadlines", &body_late.to_string(), "district9").await;

    // Search for only after June
    let (status, resp) = get_with_court(
        &app,
        "/api/deadlines/search?date_from=2026-06-01T00:00:00Z",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["deadlines"][0]["title"], "Late DL");
}

#[tokio::test]
async fn search_deadlines_pagination() {
    let (app, _pool, _guard) = test_app().await;

    for i in 0..5 {
        create_test_deadline(&app, "district9", &format!("DL {}", i)).await;
    }

    // Page 1: limit 2
    let (status, resp) =
        get_with_court(&app, "/api/deadlines/search?limit=2&offset=0", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 5);
    assert_eq!(resp["deadlines"].as_array().unwrap().len(), 2);

    // Page 2: offset 2, limit 2
    let (status, resp) =
        get_with_court(&app, "/api/deadlines/search?limit=2&offset=2", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 5);
    assert_eq!(resp["deadlines"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn search_deadlines_invalid_status_rejected() {
    let (app, _pool, _guard) = test_app().await;

    let (status, _) = get_with_court(
        &app,
        "/api/deadlines/search?status=nonsense",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
