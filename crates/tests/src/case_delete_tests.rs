use axum::http::StatusCode;

use crate::common::{test_app, delete_with_court, get_with_court, create_test_case_via_api};

#[tokio::test]
async fn delete_case_success_204() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_case_via_api(&app, "district9", "Case to Delete").await;
    let id = created["id"].as_str().unwrap();

    let (status, _) = delete_with_court(&app, &format!("/api/cases/{}", id), "district9").await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's gone
    let (status, _) = get_with_court(&app, &format!("/api/cases/{}", id), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_case_not_found_404() {
    let (app, _pool, _guard) = test_app().await;

    let fake_id = uuid::Uuid::new_v4();
    let (status, _) = delete_with_court(
        &app,
        &format!("/api/cases/{}", fake_id),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_case_invalid_uuid_400() {
    let (app, _pool, _guard) = test_app().await;

    let (status, _) = delete_with_court(&app, "/api/cases/not-a-uuid", "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
