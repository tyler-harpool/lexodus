use axum::http::StatusCode;

use crate::common::{test_app, post_json};

/// Creating a case with no rules seeded should still succeed with 201.
/// The compliance engine runs but finds no matching rules, so the case
/// is created normally and a case_filed event is logged.
#[tokio::test]
async fn create_case_no_rules_still_succeeds() {
    let (app, _pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "United States v. NoRules",
        "crime_type": "fraud",
        "district_code": "district9",
    });

    let (status, resp) = post_json(&app, "/api/cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["title"], "United States v. NoRules");
    assert!(resp["id"].as_str().is_some());
}

/// After creating a case, a `case_filed` event should be logged
/// in the `case_events` table with the correct case_id and trigger.
#[tokio::test]
async fn create_case_logs_case_event() {
    let (app, pool, _guard) = test_app().await;

    let body = serde_json::json!({
        "title": "United States v. EventLog",
        "crime_type": "fraud",
        "district_code": "district9",
    });

    let (status, resp) = post_json(&app, "/api/cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);

    let case_id: uuid::Uuid = resp["id"].as_str().unwrap().parse().unwrap();

    // Query the case_events table directly
    let events = sqlx::query!(
        r#"SELECT trigger_event, case_type, payload
           FROM case_events
           WHERE court_id = $1 AND case_id = $2"#,
        "district9",
        case_id,
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query case_events");

    assert!(
        !events.is_empty(),
        "Expected at least one case_event after case creation"
    );

    let event = &events[0];
    assert_eq!(event.trigger_event, "case_filed");
    assert_eq!(event.case_type, "criminal");

    // Verify payload contains case_number
    assert!(
        event.payload.get("case_number").is_some(),
        "Payload should contain case_number"
    );
}

/// When a rule with trigger `case_filed` and action `generate_deadline`
/// exists, creating a case should automatically create a deadline with
/// the correct case_id and computed due date.
#[tokio::test]
async fn create_case_with_matching_rule_creates_deadline() {
    let (app, pool, _guard) = test_app().await;

    // Seed a rule: generate a 90-day deadline when a criminal case is filed
    sqlx::query(
        r#"INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
    )
    .bind("district9")
    .bind("Test Rule: Case Filing Deadline")
    .bind("Auto-create 90-day service deadline on case filing")
    .bind("Federal Rules of Civil Procedure")
    .bind("Deadline")
    .bind(20)
    .bind("Active")
    .bind("district9")
    .bind("FRCP 4(m)")
    .bind(serde_json::json!([{"type": "field_equals", "field": "case_type", "value": "criminal"}]))
    .bind(serde_json::json!([{"type": "generate_deadline", "description": "Service of process", "days_from_trigger": 90}]))
    .bind(serde_json::json!(["case_filed"]))
    .execute(&pool)
    .await
    .expect("Failed to seed test rule");

    // Create a case — the compliance engine should fire the rule
    let body = serde_json::json!({
        "title": "United States v. DeadlineTest",
        "crime_type": "fraud",
        "district_code": "district9",
    });

    let (status, resp) = post_json(&app, "/api/cases", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED, "Case creation should succeed: {resp}");

    let case_id: uuid::Uuid = resp["id"].as_str().unwrap().parse().unwrap();

    // Verify a deadline was created for this case
    let deadlines = sqlx::query!(
        r#"SELECT title, case_id, rule_code, notes
           FROM deadlines
           WHERE court_id = $1 AND case_id = $2"#,
        "district9",
        case_id,
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query deadlines");

    assert!(
        !deadlines.is_empty(),
        "Expected at least one deadline created by the compliance engine"
    );

    let dl = &deadlines[0];
    assert_eq!(dl.title, "Service of process");
    assert_eq!(dl.rule_code.as_deref(), Some("FRCP 4(m)"));
    assert!(
        dl.notes.as_ref().unwrap().contains("90 days"),
        "Computation notes should mention 90 days, got: {:?}",
        dl.notes
    );
}

/// When a rule with action `block_filing` matches, case creation
/// should be rejected with a 422 status and an error message.
#[tokio::test]
async fn create_case_with_blocking_rule_returns_error() {
    let (app, pool, _guard) = test_app().await;

    // Seed a blocking rule that fires on any criminal case filed
    sqlx::query(
        r#"INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
    )
    .bind("district9")
    .bind("Test Rule: Block All Criminal Filings")
    .bind("Blocks all criminal case filings for testing")
    .bind("Local Rules")
    .bind("Filing")
    .bind(40)
    .bind("Active")
    .bind("district9")
    .bind("LR-TEST-001")
    .bind(serde_json::json!([{"type": "field_equals", "field": "case_type", "value": "criminal"}]))
    .bind(serde_json::json!([{"type": "block_filing", "reason": "Criminal filings suspended by order"}]))
    .bind(serde_json::json!(["case_filed"]))
    .execute(&pool)
    .await
    .expect("Failed to seed blocking rule");

    // Attempt to create a case — should be blocked
    let body = serde_json::json!({
        "title": "United States v. Blocked",
        "crime_type": "fraud",
        "district_code": "district9",
    });

    let (status, resp) = post_json(&app, "/api/cases", &body.to_string(), "district9").await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "Blocked case should return 422, got: {} - {resp}",
        status
    );
    assert!(
        resp["message"]
            .as_str()
            .unwrap()
            .contains("Case creation blocked"),
        "Error message should indicate blocking, got: {resp}"
    );
}
