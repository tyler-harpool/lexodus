use axum::http::StatusCode;

use crate::common::{test_app, post_json, post_no_court, create_test_civil_case_via_api};

#[tokio::test]
async fn create_civil_case_success() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "Smith v. Acme Corp",
        "nature_of_suit": "360",
        "jurisdiction_basis": "diversity",
    });

    let (status, resp) = post_json(&app, "/api/civil-cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["title"], "Smith v. Acme Corp");
    assert_eq!(resp["nature_of_suit"], "360");
    assert_eq!(resp["jurisdiction_basis"], "diversity");
    assert_eq!(resp["status"], "filed");
    assert_eq!(resp["priority"], "medium");
    assert_eq!(resp["jury_demand"], "none");
    assert_eq!(resp["class_action"], false);
    assert_eq!(resp["consent_to_magistrate"], false);
    assert_eq!(resp["pro_se"], false);
    assert!(resp["id"].as_str().is_some());
    assert!(resp["case_number"].as_str().unwrap().contains("-cv-"));
}

#[tokio::test]
async fn create_civil_case_with_all_fields() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = crate::common::create_test_judge(&pool, "district9", "Judge Taylor").await;

    let body = serde_json::json!({
        "title": "Jones v. Federal Agency",
        "nature_of_suit": "895",
        "cause_of_action": "5 U.S.C. 552",
        "jurisdiction_basis": "federal_question",
        "jury_demand": "plaintiff",
        "class_action": true,
        "amount_in_controversy": 5000000.0,
        "district_code": "district9",
        "description": "FOIA enforcement action",
        "priority": "high",
        "assigned_judge_id": judge_id,
        "consent_to_magistrate": true,
        "pro_se": true,
    });

    let (status, resp) = post_json(&app, "/api/civil-cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["title"], "Jones v. Federal Agency");
    assert_eq!(resp["cause_of_action"], "5 U.S.C. 552");
    assert_eq!(resp["jury_demand"], "plaintiff");
    assert_eq!(resp["class_action"], true);
    assert_eq!(resp["amount_in_controversy"], 5000000.0);
    assert_eq!(resp["priority"], "high");
    assert_eq!(resp["assigned_judge_id"], judge_id);
    assert_eq!(resp["consent_to_magistrate"], true);
    assert_eq!(resp["pro_se"], true);
    assert_eq!(resp["description"], "FOIA enforcement action");
}

#[tokio::test]
async fn create_civil_case_empty_title_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "   ",
        "nature_of_suit": "110",
        "jurisdiction_basis": "federal_question",
    });

    let (status, _) = post_json(&app, "/api/civil-cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_civil_case_invalid_jurisdiction_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "Test Case",
        "nature_of_suit": "110",
        "jurisdiction_basis": "state_court",
    });

    let (status, resp) = post_json(&app, "/api/civil-cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(resp["message"].as_str().unwrap().contains("Invalid jurisdiction_basis"));
}

#[tokio::test]
async fn create_civil_case_invalid_jury_demand_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "Test Case",
        "nature_of_suit": "110",
        "jurisdiction_basis": "federal_question",
        "jury_demand": "invalid",
    });

    let (status, resp) = post_json(&app, "/api/civil-cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(resp["message"].as_str().unwrap().contains("Invalid jury_demand"));
}

#[tokio::test]
async fn create_civil_case_invalid_priority_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "Test Case",
        "nature_of_suit": "110",
        "jurisdiction_basis": "federal_question",
        "priority": "super_urgent",
    });

    let (status, resp) = post_json(&app, "/api/civil-cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(resp["message"].as_str().unwrap().contains("Invalid priority"));
}

#[tokio::test]
async fn create_civil_case_missing_court_header_400() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "Test Case",
        "nature_of_suit": "110",
        "jurisdiction_basis": "federal_question",
    });

    let (status, _) = post_no_court(&app, "/api/civil-cases", &body.to_string()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_civil_case_auto_increments_case_number() {
    let (app, _pool, _guard) = test_app().await;

    let c1 = create_test_civil_case_via_api(&app, "district9", "Case Alpha").await;
    let c2 = create_test_civil_case_via_api(&app, "district9", "Case Beta").await;

    let num1 = c1["case_number"].as_str().unwrap();
    let num2 = c2["case_number"].as_str().unwrap();

    assert_ne!(num1, num2);
    assert!(num1.contains("-cv-"));
    assert!(num2.contains("-cv-"));
}
