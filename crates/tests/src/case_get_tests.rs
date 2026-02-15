use axum::http::StatusCode;

use crate::common::{test_app, get_with_court, create_test_case_via_api};

#[tokio::test]
async fn get_case_success() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_case_via_api(&app, "district9", "US v. Get Test").await;
    let id = created["id"].as_str().unwrap();

    let (status, resp) = get_with_court(&app, &format!("/api/cases/{}", id), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["id"], id);
    assert_eq!(resp["title"], "US v. Get Test");
    assert_eq!(resp["crime_type"], "fraud");
    assert_eq!(resp["status"], "filed");
}

#[tokio::test]
async fn get_case_not_found_404() {
    let (app, _pool, _guard) = test_app().await;

    let fake_id = uuid::Uuid::new_v4();
    let (status, _) = get_with_court(
        &app,
        &format!("/api/cases/{}", fake_id),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_case_invalid_uuid_400() {
    let (app, _pool, _guard) = test_app().await;

    let (status, _) = get_with_court(&app, "/api/cases/not-a-uuid", "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_case_wrong_court_404() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_case_via_api(&app, "district9", "Court 9 Only Case").await;
    let id = created["id"].as_str().unwrap();

    // Accessible from correct court
    let (status, _) = get_with_court(&app, &format!("/api/cases/{}", id), "district9").await;
    assert_eq!(status, StatusCode::OK);

    // Not accessible from wrong court
    let (status, _) = get_with_court(&app, &format!("/api/cases/{}", id), "district12").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
