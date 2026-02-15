use axum::http::StatusCode;

use crate::common::{test_app, delete_with_court, get_with_court, create_test_deadline};

#[tokio::test]
async fn delete_deadline_success() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_deadline(&app, "district9", "To be deleted").await;
    let id = created["id"].as_str().unwrap();

    let (status, _) = delete_with_court(&app, &format!("/api/deadlines/{}", id), "district9").await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Confirm it's gone
    let (status, _) = get_with_court(&app, &format!("/api/deadlines/{}", id), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_deadline_not_found() {
    let (app, _pool, _guard) = test_app().await;

    let fake_id = "00000000-0000-0000-0000-000000000099";
    let (status, _) = delete_with_court(&app, &format!("/api/deadlines/{}", fake_id), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_deadline_invalid_uuid() {
    let (app, _pool, _guard) = test_app().await;

    let (status, _) = delete_with_court(&app, "/api/deadlines/bad-uuid", "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_deadline_wrong_court() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_deadline(&app, "district9", "Court 9 only").await;
    let id = created["id"].as_str().unwrap();

    // Attempt delete from a different court
    let (status, _) = delete_with_court(&app, &format!("/api/deadlines/{}", id), "district12").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Confirm still exists in original court
    let (status, _) = get_with_court(&app, &format!("/api/deadlines/{}", id), "district9").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn delete_deadline_idempotent() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_deadline(&app, "district9", "Delete twice").await;
    let id = created["id"].as_str().unwrap();

    let (status, _) = delete_with_court(&app, &format!("/api/deadlines/{}", id), "district9").await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Second delete returns 404
    let (status, _) = delete_with_court(&app, &format!("/api/deadlines/{}", id), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
