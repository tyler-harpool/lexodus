use axum::http::StatusCode;

use crate::common::{test_app, get_with_court, create_test_civil_case_via_api};

#[tokio::test]
async fn civil_cases_isolated_by_court() {
    let (app, _pool, _guard) = test_app().await;

    // Create cases in different courts
    create_test_civil_case_via_api(&app, "district9", "District 9 Case").await;
    create_test_civil_case_via_api(&app, "district12", "District 12 Case").await;

    // Verify district9 only sees its case
    let (status, resp) = get_with_court(&app, "/api/civil-cases", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["cases"][0]["title"], "District 9 Case");

    // Verify district12 only sees its case
    let (status, resp) = get_with_court(&app, "/api/civil-cases", "district12").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["cases"][0]["title"], "District 12 Case");
}

#[tokio::test]
async fn civil_case_get_cross_court_404() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_civil_case_via_api(&app, "district9", "Private Case").await;
    let id = created["id"].as_str().unwrap();

    // Attempt to get district9's case from district12 should return 404
    let (status, _) = get_with_court(&app, &format!("/api/civil-cases/{}", id), "district12").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
