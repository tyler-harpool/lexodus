use axum::http::StatusCode;

use crate::common::{test_app, get_with_court, post_json, create_test_case_via_api};

#[tokio::test]
async fn search_empty_results() {
    let (app, _pool, _guard) = test_app().await;

    let (status, resp) = get_with_court(&app, "/api/cases", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 0);
    assert_eq!(resp["cases"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn search_returns_cases() {
    let (app, _pool, _guard) = test_app().await;

    create_test_case_via_api(&app, "district9", "Case Alpha").await;
    create_test_case_via_api(&app, "district9", "Case Beta").await;

    let (status, resp) = get_with_court(&app, "/api/cases", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 2);
    assert_eq!(resp["cases"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn search_filter_by_status() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_case_via_api(&app, "district9", "Filed Case").await;
    let id = created["id"].as_str().unwrap();

    // Transition one to arraigned
    let body = serde_json::json!({ "status": "arraigned" });
    crate::common::patch_json(
        &app,
        &format!("/api/cases/{}/status", id),
        &body.to_string(),
        "district9",
    )
    .await;

    // Create another that stays as filed
    create_test_case_via_api(&app, "district9", "Still Filed Case").await;

    // Filter by arraigned — should find 1
    let (status, resp) = get_with_court(&app, "/api/cases?status=arraigned", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["cases"][0]["status"], "arraigned");

    // Filter by filed — should find 1
    let (status, resp) = get_with_court(&app, "/api/cases?status=filed", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["cases"][0]["status"], "filed");
}

#[tokio::test]
async fn search_filter_by_crime_type() {
    let (app, _pool, _guard) = test_app().await;

    // Create a fraud case
    create_test_case_via_api(&app, "district9", "Fraud Case").await;

    // Create a cybercrime case
    let body = serde_json::json!({
        "title": "Cyber Case",
        "crime_type": "cybercrime",
        "district_code": "district9",
    });
    post_json(&app, "/api/cases", &body.to_string(), "district9").await;

    // Filter by cybercrime
    let (status, resp) = get_with_court(&app, "/api/cases?crime_type=cybercrime", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["cases"][0]["crime_type"], "cybercrime");
}

#[tokio::test]
async fn search_text_search_by_title() {
    let (app, _pool, _guard) = test_app().await;

    create_test_case_via_api(&app, "district9", "United States v. Alpha").await;
    create_test_case_via_api(&app, "district9", "United States v. Beta").await;

    let (status, resp) = get_with_court(&app, "/api/cases?q=Alpha", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["cases"][0]["title"], "United States v. Alpha");
}

#[tokio::test]
async fn search_pagination() {
    let (app, _pool, _guard) = test_app().await;

    for i in 0..5 {
        create_test_case_via_api(&app, "district9", &format!("Paginated Case {}", i)).await;
    }

    // Page 1: limit 2
    let (status, resp) = get_with_court(&app, "/api/cases?limit=2&offset=0", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 5);
    assert_eq!(resp["cases"].as_array().unwrap().len(), 2);

    // Page 2: offset 2, limit 2
    let (status, resp) = get_with_court(&app, "/api/cases?limit=2&offset=2", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 5);
    assert_eq!(resp["cases"].as_array().unwrap().len(), 2);

    // Page 3: offset 4, limit 2 → only 1 left
    let (status, resp) = get_with_court(&app, "/api/cases?limit=2&offset=4", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 5);
    assert_eq!(resp["cases"].as_array().unwrap().len(), 1);
}
