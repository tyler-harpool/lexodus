use axum::http::StatusCode;
use crate::common::*;

/// Test: Rule on a pending motion with "Granted" disposition.
/// Verifies: order created with correct type/title/content/status, motion status updated.
#[tokio::test]
async fn test_rule_motion_granted() {
    let (app, pool, _guard) = test_app().await;
    let court = "district9";

    // Setup: create case, judge, and a pending motion
    let case_id = create_test_case(&pool, court, "9:26-cr-99001").await;
    let judge_id = create_test_judge(&pool, court, "Judge Ruling Granted").await;

    let motion_body = serde_json::json!({
        "case_id": case_id,
        "motion_type": "Suppress",
        "filed_by": "Defense Attorney",
        "description": "Motion to suppress evidence obtained without warrant"
    });
    let (status, motion) = post_json(&app, "/api/motions", &motion_body.to_string(), court).await;
    assert_eq!(status, StatusCode::CREATED, "Failed to create motion: {:?}", motion);
    let motion_id = motion["id"].as_str().unwrap();

    // Rule on the motion: Granted
    let rule_body = serde_json::json!({
        "disposition": "Granted",
        "ruling_text": "The motion to suppress is GRANTED. Evidence excluded.",
        "judge_id": judge_id,
        "judge_name": "Judge Ruling Granted"
    });
    let (status, order) = post_json(
        &app,
        &format!("/api/motions/{}/rule", motion_id),
        &rule_body.to_string(),
        court,
    ).await;
    assert_eq!(status, StatusCode::OK, "Rule motion failed: {:?}", order);

    // Verify the created order
    assert_eq!(order["order_type"].as_str().unwrap(), "Procedural");
    assert!(
        order["title"].as_str().unwrap().contains("Suppress"),
        "Order title should reference motion type: {}",
        order["title"]
    );
    assert!(
        order["title"].as_str().unwrap().contains("Granted"),
        "Order title should contain disposition: {}",
        order["title"]
    );
    assert_eq!(order["status"].as_str().unwrap(), "Pending Signature");
    assert!(
        order["content"].as_str().unwrap().contains("GRANTED"),
        "Order content should contain ruling text: {}",
        order["content"]
    );

    // Verify motion status was updated to "Granted"
    let (status, updated_motion) = get_with_court(
        &app,
        &format!("/api/motions/{}", motion_id),
        court,
    ).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated_motion["status"].as_str().unwrap(), "Granted");
    assert!(
        updated_motion["ruling_date"].as_str().is_some(),
        "Motion should have a ruling_date set"
    );
}

/// Test: Cannot rule on a motion that is not in "Pending" status.
/// Verifies: returns 400 Bad Request.
#[tokio::test]
async fn test_rule_motion_not_pending() {
    let (app, pool, _guard) = test_app().await;
    let court = "district9";

    let case_id = create_test_case(&pool, court, "9:26-cr-99002").await;
    let judge_id = create_test_judge(&pool, court, "Judge Non-Pending").await;

    // Create a motion with status "Granted" (not pending)
    let motion_body = serde_json::json!({
        "case_id": case_id,
        "motion_type": "Dismiss",
        "filed_by": "Defense Counsel",
        "description": "Motion to dismiss",
        "status": "Granted"
    });
    let (status, motion) = post_json(&app, "/api/motions", &motion_body.to_string(), court).await;
    assert_eq!(status, StatusCode::CREATED);
    let motion_id = motion["id"].as_str().unwrap();

    // Attempt to rule on a non-pending motion
    let rule_body = serde_json::json!({
        "disposition": "Denied",
        "judge_id": judge_id,
        "judge_name": "Judge Non-Pending"
    });
    let (status, _resp) = post_json(
        &app,
        &format!("/api/motions/{}/rule", motion_id),
        &rule_body.to_string(),
        court,
    ).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// Test: Invalid disposition value is rejected.
/// "Overruled" is not a valid ruling disposition.
/// Verifies: returns 400 Bad Request.
#[tokio::test]
async fn test_rule_motion_invalid_disposition() {
    let (app, pool, _guard) = test_app().await;
    let court = "district9";

    let case_id = create_test_case(&pool, court, "9:26-cr-99003").await;
    let judge_id = create_test_judge(&pool, court, "Judge Invalid Disp").await;

    let motion_body = serde_json::json!({
        "case_id": case_id,
        "motion_type": "Compel",
        "filed_by": "Plaintiff",
        "description": "Motion to compel discovery responses"
    });
    let (status, motion) = post_json(&app, "/api/motions", &motion_body.to_string(), court).await;
    assert_eq!(status, StatusCode::CREATED);
    let motion_id = motion["id"].as_str().unwrap();

    // Use an invalid disposition
    let rule_body = serde_json::json!({
        "disposition": "Overruled",
        "judge_id": judge_id,
        "judge_name": "Judge Invalid Disp"
    });
    let (status, _resp) = post_json(
        &app,
        &format!("/api/motions/{}/rule", motion_id),
        &rule_body.to_string(),
        court,
    ).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// Test: Rule on a motion with "Denied" disposition.
/// For a "Dismiss" motion type, the order_type should be "Dismissal".
/// Verifies: order created with "Dismissal" type and ruling text in content.
#[tokio::test]
async fn test_rule_motion_denied() {
    let (app, pool, _guard) = test_app().await;
    let court = "district9";

    let case_id = create_test_case(&pool, court, "9:26-cr-99004").await;
    let judge_id = create_test_judge(&pool, court, "Judge Denied").await;

    let motion_body = serde_json::json!({
        "case_id": case_id,
        "motion_type": "Dismiss",
        "filed_by": "Defense Counsel",
        "description": "Motion to dismiss for lack of evidence"
    });
    let (status, motion) = post_json(&app, "/api/motions", &motion_body.to_string(), court).await;
    assert_eq!(status, StatusCode::CREATED);
    let motion_id = motion["id"].as_str().unwrap();

    // Rule: Denied
    let rule_body = serde_json::json!({
        "disposition": "Denied",
        "ruling_text": "The Court finds insufficient grounds. Motion DENIED.",
        "judge_id": judge_id,
        "judge_name": "Judge Denied"
    });
    let (status, order) = post_json(
        &app,
        &format!("/api/motions/{}/rule", motion_id),
        &rule_body.to_string(),
        court,
    ).await;
    assert_eq!(status, StatusCode::OK, "Rule motion denied failed: {:?}", order);

    // Dismiss motion type maps to "Dismissal" order type
    assert_eq!(order["order_type"].as_str().unwrap(), "Dismissal");
    assert!(
        order["content"].as_str().unwrap().contains("DENIED"),
        "Order content should contain ruling text: {}",
        order["content"]
    );

    // Verify motion status updated to "Denied"
    let (status, updated_motion) = get_with_court(
        &app,
        &format!("/api/motions/{}", motion_id),
        court,
    ).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated_motion["status"].as_str().unwrap(), "Denied");
}
