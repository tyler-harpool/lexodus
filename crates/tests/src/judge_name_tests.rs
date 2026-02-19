use axum::http::StatusCode;

use crate::common::*;

const COURT: &str = "district9";
const OTHER_COURT: &str = "district12";

// ── Order judge_name resolution ─────────────────────────────────────

#[tokio::test]
async fn order_create_includes_judge_name() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, COURT, "Hon. Sarah Mitchell").await;
    let case_id = create_test_case(&pool, COURT, "CR-2026-JN-001").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "order_type": "Scheduling",
        "title": "Scheduling Order",
        "content": "The court hereby schedules the following dates..."
    });

    let (status, resp) = post_json(&app, "/api/orders", &body.to_string(), COURT).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["judge_name"], "Hon. Sarah Mitchell");
    assert_eq!(resp["judge_id"], judge_id);
}

#[tokio::test]
async fn order_get_includes_judge_name() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, COURT, "Hon. Robert Chen").await;
    let case_id = create_test_case(&pool, COURT, "CR-2026-JN-002").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "order_type": "Protective",
        "title": "Protective Order",
        "content": "It is hereby ordered..."
    });

    let (_, created) = post_json(&app, "/api/orders", &body.to_string(), COURT).await;
    let order_id = created["id"].as_str().unwrap();

    let (status, resp) = get_with_court(&app, &format!("/api/orders/{}", order_id), COURT).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["judge_name"], "Hon. Robert Chen");
}

#[tokio::test]
async fn order_list_by_case_includes_judge_name() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, COURT, "Hon. Lisa Park").await;
    let case_id = create_test_case(&pool, COURT, "CR-2026-JN-003").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "order_type": "Discovery",
        "title": "Discovery Order",
        "content": "Discovery shall proceed..."
    });
    post_json(&app, "/api/orders", &body.to_string(), COURT).await;

    let (status, resp) = get_with_court(
        &app,
        &format!("/api/cases/{}/orders", case_id),
        COURT,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let orders = resp.as_array().unwrap();
    assert!(!orders.is_empty());
    assert_eq!(orders[0]["judge_name"], "Hon. Lisa Park");
}

#[tokio::test]
async fn order_list_all_includes_judge_name() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, COURT, "Hon. David Kim").await;
    let case_id = create_test_case(&pool, COURT, "CR-2026-JN-004").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "order_type": "Scheduling",
        "title": "Scheduling Order #2",
        "content": "Scheduling details..."
    });
    post_json(&app, "/api/orders", &body.to_string(), COURT).await;

    let (status, resp) = get_with_court(&app, "/api/orders", COURT).await;
    assert_eq!(status, StatusCode::OK);
    let orders = resp.as_array().unwrap();
    assert!(!orders.is_empty());
    assert_eq!(orders[0]["judge_name"], "Hon. David Kim");
}

#[tokio::test]
async fn order_update_preserves_judge_name() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, COURT, "Hon. Maria Gonzalez").await;
    let case_id = create_test_case(&pool, COURT, "CR-2026-JN-005").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "order_type": "Scheduling",
        "title": "Original Title",
        "content": "Original content"
    });
    let (_, created) = post_json(&app, "/api/orders", &body.to_string(), COURT).await;
    let order_id = created["id"].as_str().unwrap();

    let update = serde_json::json!({ "title": "Updated Title" });
    let (status, resp) = patch_json(
        &app,
        &format!("/api/orders/{}", order_id),
        &update.to_string(),
        COURT,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["title"], "Updated Title");
    assert_eq!(resp["judge_name"], "Hon. Maria Gonzalez");
}

#[tokio::test]
async fn order_sign_includes_judge_name() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, COURT, "Hon. James Wright").await;
    let case_id = create_test_case(&pool, COURT, "CR-2026-JN-006").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "order_type": "Scheduling",
        "title": "Order to Sign",
        "content": "Content",
        "status": "Pending Signature"
    });
    let (_, created) = post_json(&app, "/api/orders", &body.to_string(), COURT).await;
    let order_id = created["id"].as_str().unwrap();

    let sign = serde_json::json!({ "signed_by": "Hon. James Wright" });
    let (status, resp) = post_json(
        &app,
        &format!("/api/orders/{}/sign", order_id),
        &sign.to_string(),
        COURT,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["status"], "Signed");
    assert_eq!(resp["judge_name"], "Hon. James Wright");
    assert!(resp["signer_name"].as_str().is_some());
}

#[tokio::test]
async fn order_issue_includes_judge_name() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, COURT, "Hon. Elena Torres").await;
    let case_id = create_test_case(&pool, COURT, "CR-2026-JN-007").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "order_type": "Scheduling",
        "title": "Order to Issue",
        "content": "Content",
        "status": "Signed"
    });
    let (_, created) = post_json(&app, "/api/orders", &body.to_string(), COURT).await;
    let order_id = created["id"].as_str().unwrap();

    let issue = serde_json::json!({ "issued_by": "Clerk Thompson" });
    let (status, resp) = post_json(
        &app,
        &format!("/api/orders/{}/issue", order_id),
        &issue.to_string(),
        COURT,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["status"], "Filed");
    assert_eq!(resp["judge_name"], "Hon. Elena Torres");
}

// ── Assignment judge_name resolution ────────────────────────────────

#[tokio::test]
async fn assignment_create_includes_judge_name() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, COURT, "Hon. William Davis").await;
    let case_id = create_test_case(&pool, COURT, "CR-2026-JN-008").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "assignment_type": "Initial"
    });

    let (status, resp) = post_json(&app, "/api/judges/assignments", &body.to_string(), COURT).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["judge_name"], "Hon. William Davis");
    assert_eq!(resp["judge_id"], judge_id);
}

#[tokio::test]
async fn assignment_list_by_case_includes_judge_name() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, COURT, "Hon. Patricia Brown").await;
    let case_id = create_test_case(&pool, COURT, "CR-2026-JN-009").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "assignment_type": "Initial"
    });
    post_json(&app, "/api/judges/assignments", &body.to_string(), COURT).await;

    let (status, resp) = get_with_court(
        &app,
        &format!("/api/cases/{}/assignment", case_id),
        COURT,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let assignments = resp.as_array().unwrap();
    assert!(!assignments.is_empty());
    assert_eq!(assignments[0]["judge_name"], "Hon. Patricia Brown");
}

#[tokio::test]
async fn assignment_history_includes_judge_name() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, COURT, "Hon. Thomas Lee").await;
    let case_id = create_test_case(&pool, COURT, "CR-2026-JN-010").await;

    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "assignment_type": "Initial"
    });
    post_json(&app, "/api/judges/assignments", &body.to_string(), COURT).await;

    let (status, resp) = get_with_court(
        &app,
        &format!("/api/cases/{}/assignment-history", case_id),
        COURT,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let entries = resp["entries"].as_array().unwrap();
    assert!(!entries.is_empty());
    assert_eq!(entries[0]["judge_name"], "Hon. Thomas Lee");
}

// ── Tenant isolation ────────────────────────────────────────────────

#[tokio::test]
async fn order_judge_name_tenant_isolation() {
    let (app, pool, _guard) = test_app().await;

    // Create judges with same name in different courts
    let judge_d9 = create_test_judge(&pool, COURT, "Hon. Shared Name").await;
    let judge_d12 = create_test_judge(&pool, OTHER_COURT, "Hon. Different Name").await;

    let case_d9 = create_test_case(&pool, COURT, "CR-2026-JN-011").await;
    let case_d12 = create_test_case(&pool, OTHER_COURT, "CR-2026-JN-012").await;

    // Create order in district9
    let body_d9 = serde_json::json!({
        "case_id": case_d9,
        "judge_id": judge_d9,
        "order_type": "Scheduling",
        "title": "D9 Order",
        "content": "Content"
    });
    let (_, resp_d9) = post_json(&app, "/api/orders", &body_d9.to_string(), COURT).await;
    assert_eq!(resp_d9["judge_name"], "Hon. Shared Name");

    // Create order in district12
    let body_d12 = serde_json::json!({
        "case_id": case_d12,
        "judge_id": judge_d12,
        "order_type": "Scheduling",
        "title": "D12 Order",
        "content": "Content"
    });
    let (_, resp_d12) = post_json(&app, "/api/orders", &body_d12.to_string(), OTHER_COURT).await;
    assert_eq!(resp_d12["judge_name"], "Hon. Different Name");

    // Verify district9 orders don't show district12 judge names
    let (_, d9_orders) = get_with_court(&app, "/api/orders", COURT).await;
    for order in d9_orders.as_array().unwrap() {
        assert_ne!(order["judge_name"], "Hon. Different Name");
    }
}

#[tokio::test]
async fn assignment_judge_name_tenant_isolation() {
    let (app, pool, _guard) = test_app().await;

    let judge_d9 = create_test_judge(&pool, COURT, "Hon. Court9 Judge").await;
    let judge_d12 = create_test_judge(&pool, OTHER_COURT, "Hon. Court12 Judge").await;

    let case_d9 = create_test_case(&pool, COURT, "CR-2026-JN-013").await;
    let case_d12 = create_test_case(&pool, OTHER_COURT, "CR-2026-JN-014").await;

    // Assign in both courts
    let body_d9 = serde_json::json!({
        "case_id": case_d9,
        "judge_id": judge_d9,
        "assignment_type": "Initial"
    });
    let (_, resp_d9) = post_json(&app, "/api/judges/assignments", &body_d9.to_string(), COURT).await;
    assert_eq!(resp_d9["judge_name"], "Hon. Court9 Judge");

    let body_d12 = serde_json::json!({
        "case_id": case_d12,
        "judge_id": judge_d12,
        "assignment_type": "Initial"
    });
    let (_, resp_d12) = post_json(&app, "/api/judges/assignments", &body_d12.to_string(), OTHER_COURT).await;
    assert_eq!(resp_d12["judge_name"], "Hon. Court12 Judge");

    // Verify district9 assignments don't leak district12 names
    let (_, d9_assignments) = get_with_court(
        &app,
        &format!("/api/cases/{}/assignment", case_d9),
        COURT,
    )
    .await;
    for a in d9_assignments.as_array().unwrap() {
        assert_eq!(a["judge_name"], "Hon. Court9 Judge");
    }
}

// ── Assignment syncs case assigned_judge_id ─────────────────────────

#[tokio::test]
async fn assignment_syncs_case_assigned_judge_id() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, COURT, "Hon. Sync Judge").await;
    let case_id = create_test_case(&pool, COURT, "CR-2026-JN-015").await;

    // Before assignment, case has no assigned_judge_id
    let (status, case_before) = get_with_court(&app, &format!("/api/cases/{}", case_id), COURT).await;
    assert_eq!(status, StatusCode::OK);
    assert!(case_before["assigned_judge_id"].is_null());

    // Create assignment
    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "assignment_type": "Initial"
    });
    let (status, _) = post_json(&app, "/api/judges/assignments", &body.to_string(), COURT).await;
    assert_eq!(status, StatusCode::CREATED);

    // After assignment, case.assigned_judge_id should be set
    let (status, case_after) = get_with_court(&app, &format!("/api/cases/{}", case_id), COURT).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(case_after["assigned_judge_id"], judge_id);
}

#[tokio::test]
async fn reassignment_updates_case_assigned_judge_id() {
    let (app, pool, _guard) = test_app().await;
    let judge_a = create_test_judge(&pool, COURT, "Hon. Judge Alpha").await;
    let judge_b = create_test_judge(&pool, COURT, "Hon. Judge Beta").await;
    let case_id = create_test_case(&pool, COURT, "CR-2026-JN-016").await;

    // Initial assignment
    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_a,
        "assignment_type": "Initial"
    });
    post_json(&app, "/api/judges/assignments", &body.to_string(), COURT).await;

    // Reassignment to judge_b
    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_b,
        "assignment_type": "Reassignment",
        "previous_judge_id": judge_a,
        "reassignment_reason": "Recusal"
    });
    let (status, resp) = post_json(&app, "/api/judges/assignments", &body.to_string(), COURT).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["judge_name"], "Hon. Judge Beta");

    // Case should now point to judge_b
    let (_, case_data) = get_with_court(&app, &format!("/api/cases/{}", case_id), COURT).await;
    assert_eq!(case_data["assigned_judge_id"], judge_b);
}
