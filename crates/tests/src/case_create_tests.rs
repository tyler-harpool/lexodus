use axum::http::StatusCode;

use crate::common::{test_app, post_json, post_no_court, create_test_case_via_api};

#[tokio::test]
async fn create_case_success() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "United States v. Smith",
        "crime_type": "fraud",
        "district_code": "district9",
    });

    let (status, resp) = post_json(&app, "/api/cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["title"], "United States v. Smith");
    assert_eq!(resp["crime_type"], "fraud");
    assert_eq!(resp["status"], "filed");
    assert_eq!(resp["priority"], "medium");
    assert!(resp["id"].as_str().is_some());
    assert!(resp["case_number"].as_str().unwrap().contains("-cr-"));
}

#[tokio::test]
async fn create_case_returns_all_fields() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "United States v. Jones",
        "description": "Tax evasion case",
        "crime_type": "tax_offense",
        "district_code": "district9",
        "location": "Federal Courthouse Room 301",
        "priority": "high",
    });

    let (status, resp) = post_json(&app, "/api/cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["title"], "United States v. Jones");
    assert_eq!(resp["description"], "Tax evasion case");
    assert_eq!(resp["crime_type"], "tax_offense");
    assert_eq!(resp["priority"], "high");
    assert_eq!(resp["location"], "Federal Courthouse Room 301");
    assert_eq!(resp["district_code"], "district9");
    assert_eq!(resp["is_sealed"], false);
    assert!(resp["opened_at"].as_str().is_some());
    assert!(resp["updated_at"].as_str().is_some());
}

#[tokio::test]
async fn create_case_with_optional_judge_id() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = crate::common::create_test_judge(&pool, "district9", "Judge Adams").await;

    let body = serde_json::json!({
        "title": "United States v. Baker",
        "crime_type": "drug_offense",
        "district_code": "district9",
        "assigned_judge_id": judge_id,
    });

    let (status, resp) = post_json(&app, "/api/cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["assigned_judge_id"], judge_id);
}

#[tokio::test]
async fn create_case_empty_title_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "   ",
        "crime_type": "fraud",
        "district_code": "district9",
    });

    let (status, _) = post_json(&app, "/api/cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_case_invalid_crime_type_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "United States v. Test",
        "crime_type": "invalid_type",
        "district_code": "district9",
    });

    let (status, resp) = post_json(&app, "/api/cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(resp["message"].as_str().unwrap().contains("Invalid crime_type"));
}

#[tokio::test]
async fn create_case_invalid_priority_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "United States v. Test",
        "crime_type": "fraud",
        "district_code": "district9",
        "priority": "super_urgent",
    });

    let (status, resp) = post_json(&app, "/api/cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(resp["message"].as_str().unwrap().contains("Invalid priority"));
}

#[tokio::test]
async fn create_case_missing_court_header_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "United States v. Test",
        "crime_type": "fraud",
        "district_code": "district9",
    });

    let (status, _) = post_no_court(&app, "/api/cases", &body.to_string()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_case_auto_increments_case_number() {
    let (app, _pool, _guard) = test_app().await;

    let c1 = create_test_case_via_api(&app, "district9", "Case Alpha").await;
    let c2 = create_test_case_via_api(&app, "district9", "Case Beta").await;

    let num1 = c1["case_number"].as_str().unwrap();
    let num2 = c2["case_number"].as_str().unwrap();

    assert_ne!(num1, num2);
    // Both should contain -CR- pattern
    assert!(num1.contains("-cr-"));
    assert!(num2.contains("-cr-"));
}
