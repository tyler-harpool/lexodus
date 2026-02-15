use axum::http::StatusCode;

use crate::common::{test_app, get_with_court, create_test_deadline};

#[tokio::test]
async fn deadlines_isolated_between_courts() {
    let (app, _pool, _guard) = test_app().await;

    // Create deadlines in separate courts
    create_test_deadline(&app, "district9", "District 9 DL A").await;
    create_test_deadline(&app, "district9", "District 9 DL B").await;
    create_test_deadline(&app, "district12", "District 12 DL A").await;

    // Search district9: should see 2
    let (status, resp) = get_with_court(&app, "/api/deadlines/search", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 2);

    // Search district12: should see 1
    let (status, resp) = get_with_court(&app, "/api/deadlines/search", "district12").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["deadlines"][0]["title"], "District 12 DL A");
}

#[tokio::test]
async fn deadline_get_isolated_between_courts() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_deadline(&app, "district9", "Court 9 only").await;
    let id = created["id"].as_str().unwrap();

    // Accessible from correct court
    let (status, _) = get_with_court(&app, &format!("/api/deadlines/{}", id), "district9").await;
    assert_eq!(status, StatusCode::OK);

    // Not accessible from wrong court
    let (status, _) = get_with_court(&app, &format!("/api/deadlines/{}", id), "district12").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
