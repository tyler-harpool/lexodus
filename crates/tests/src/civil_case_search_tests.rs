use axum::http::StatusCode;

use crate::common::{test_app, post_json, get_with_court, patch_json, create_test_civil_case_via_api, delete_with_court};

#[tokio::test]
async fn search_civil_cases_empty() {
    let (app, _pool, _guard) = test_app().await;

    let (status, resp) = get_with_court(&app, "/api/civil-cases", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 0);
    assert_eq!(resp["cases"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn search_civil_cases_returns_results() {
    let (app, _pool, _guard) = test_app().await;

    create_test_civil_case_via_api(&app, "district9", "Smith v. Jones").await;
    create_test_civil_case_via_api(&app, "district9", "Doe v. Roe").await;

    let (status, resp) = get_with_court(&app, "/api/civil-cases", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 2);
    assert_eq!(resp["cases"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn search_civil_cases_filter_by_status() {
    let (app, _pool, _guard) = test_app().await;

    let c1 = create_test_civil_case_via_api(&app, "district9", "Active Case").await;
    let _ = create_test_civil_case_via_api(&app, "district9", "Filed Case").await;

    // Update first case to discovery
    let id = c1["id"].as_str().unwrap();
    let body = serde_json::json!({ "status": "discovery" });
    patch_json(&app, &format!("/api/civil-cases/{}/status", id), &body.to_string(), "district9").await;

    let (status, resp) = get_with_court(&app, "/api/civil-cases?status=discovery", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["cases"][0]["title"], "Active Case");
}

#[tokio::test]
async fn search_civil_cases_filter_by_text() {
    let (app, _pool, _guard) = test_app().await;

    create_test_civil_case_via_api(&app, "district9", "Apple v. Samsung").await;
    create_test_civil_case_via_api(&app, "district9", "Google v. Oracle").await;

    let (status, resp) = get_with_court(&app, "/api/civil-cases?q=Apple", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["cases"][0]["title"], "Apple v. Samsung");
}

#[tokio::test]
async fn get_civil_case_by_id() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_civil_case_via_api(&app, "district9", "Test Civil Case").await;
    let id = created["id"].as_str().unwrap();

    let (status, resp) = get_with_court(&app, &format!("/api/civil-cases/{}", id), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["title"], "Test Civil Case");
    assert_eq!(resp["id"], id);
}

#[tokio::test]
async fn get_civil_case_not_found_404() {
    let (app, _pool, _guard) = test_app().await;

    let fake_id = "00000000-0000-0000-0000-000000000000";
    let (status, _) = get_with_court(&app, &format!("/api/civil-cases/{}", fake_id), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_civil_case_success() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_civil_case_via_api(&app, "district9", "To Delete").await;
    let id = created["id"].as_str().unwrap();

    let (status, _) = delete_with_court(&app, &format!("/api/civil-cases/{}", id), "district9").await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's gone
    let (status, _) = get_with_court(&app, &format!("/api/civil-cases/{}", id), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_civil_case_status_success() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_civil_case_via_api(&app, "district9", "Status Test").await;
    let id = created["id"].as_str().unwrap();

    let body = serde_json::json!({ "status": "settled" });
    let (status, resp) = patch_json(&app, &format!("/api/civil-cases/{}/status", id), &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["status"], "settled");
}

#[tokio::test]
async fn update_civil_case_status_invalid_400() {
    let (app, _pool, _guard) = test_app().await;

    let created = create_test_civil_case_via_api(&app, "district9", "Bad Status").await;
    let id = created["id"].as_str().unwrap();

    let body = serde_json::json!({ "status": "nonexistent" });
    let (status, resp) = patch_json(&app, &format!("/api/civil-cases/{}/status", id), &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(resp["message"].as_str().unwrap().contains("Invalid status"));
}

#[tokio::test]
async fn civil_case_statistics_empty() {
    let (app, _pool, _guard) = test_app().await;

    let (status, resp) = get_with_court(&app, "/api/civil-cases/statistics", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 0);
}

#[tokio::test]
async fn civil_case_statistics_with_data() {
    let (app, _pool, _guard) = test_app().await;

    create_test_civil_case_via_api(&app, "district9", "Stats Case 1").await;
    create_test_civil_case_via_api(&app, "district9", "Stats Case 2").await;

    let (status, resp) = get_with_court(&app, "/api/civil-cases/statistics", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 2);
    assert!(resp["by_status"].is_object());
    assert!(resp["by_nature_of_suit"].is_object());
}

#[tokio::test]
async fn list_civil_cases_by_judge() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = crate::common::create_test_judge(&pool, "district9", "Judge Martinez").await;

    let body = serde_json::json!({
        "title": "Assigned Case",
        "nature_of_suit": "440",
        "jurisdiction_basis": "federal_question",
        "assigned_judge_id": judge_id,
    });
    post_json(&app, "/api/civil-cases", &body.to_string(), "district9").await;

    // Create another case without this judge
    create_test_civil_case_via_api(&app, "district9", "Unassigned Case").await;

    let (status, resp) = get_with_court(&app, &format!("/api/civil-cases/by-judge/{}", judge_id), "district9").await;
    assert_eq!(status, StatusCode::OK);
    let cases = resp.as_array().unwrap();
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0]["title"], "Assigned Case");
}
