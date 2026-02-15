use axum::http::StatusCode;

use crate::common::{test_app, get_with_court, create_test_deadline};

#[tokio::test]
async fn get_deadline_by_id() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_deadline(&app, "district9", "File Brief").await;
    let id = created["id"].as_str().unwrap();

    let (status, resp) = get_with_court(&app, &format!("/api/deadlines/{}", id), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["id"], id);
    assert_eq!(resp["title"], "File Brief");
    assert_eq!(resp["status"], "open");
}

#[tokio::test]
async fn get_deadline_not_found() {
    let (app, _pool, _guard) = test_app().await;

    let fake_id = "00000000-0000-0000-0000-000000000099";
    let (status, _) = get_with_court(&app, &format!("/api/deadlines/{}", fake_id), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_deadline_invalid_uuid() {
    let (app, _pool, _guard) = test_app().await;

    let (status, _) = get_with_court(&app, "/api/deadlines/not-a-uuid", "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_deadline_wrong_court_returns_not_found() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_deadline(&app, "district9", "Court 9 deadline").await;
    let id = created["id"].as_str().unwrap();

    // Same ID but different court should return 404
    let (status, _) = get_with_court(&app, &format!("/api/deadlines/{}", id), "district12").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
