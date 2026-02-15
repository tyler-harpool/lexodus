use axum::http::StatusCode;

use crate::common::{
    test_app, get_with_court, delete_with_court, create_test_case_via_api,
};

#[tokio::test]
async fn district9_cannot_see_district12_cases() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_case_via_api(&app, "district12", "District 12 Case").await;
    let id = created["id"].as_str().unwrap();

    // district9 cannot see it
    let (status, _) = get_with_court(&app, &format!("/api/cases/{}", id), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // district12 can see it
    let (status, _) = get_with_court(&app, &format!("/api/cases/{}", id), "district12").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn district12_cannot_delete_district9_case() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_case_via_api(&app, "district9", "District 9 Protected").await;
    let id = created["id"].as_str().unwrap();

    // district12 cannot delete it
    let (status, _) = delete_with_court(&app, &format!("/api/cases/{}", id), "district12").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // district9 can still see it
    let (status, _) = get_with_court(&app, &format!("/api/cases/{}", id), "district9").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn search_only_returns_own_court() {
    let (app, _pool, _guard) = test_app().await;

    create_test_case_via_api(&app, "district9", "D9 Case A").await;
    create_test_case_via_api(&app, "district9", "D9 Case B").await;
    create_test_case_via_api(&app, "district12", "D12 Case A").await;

    // district9 sees 2
    let (status, resp) = get_with_court(&app, "/api/cases", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 2);

    // district12 sees 1
    let (status, resp) = get_with_court(&app, "/api/cases", "district12").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["cases"][0]["title"], "D12 Case A");
}
