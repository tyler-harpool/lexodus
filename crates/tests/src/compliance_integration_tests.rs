use axum::http::StatusCode;
use serde_json::json;

use crate::common::{
    create_test_case_via_api, create_test_docket_entry, create_test_token_with_courts,
    delete_with_court, get_with_court, patch_json, post_json, post_json_authed, test_app,
};

/// Seed a compliance rule directly into the database.
///
/// This inserts a row into the `rules` table with the given parameters,
/// mirroring how production rules are stored. The rule is scoped to a
/// specific court and marked Active so the compliance engine picks it up.
async fn seed_rule(
    pool: &sqlx::Pool<sqlx::Postgres>,
    court: &str,
    name: &str,
    citation: &str,
    priority: i32,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    triggers: serde_json::Value,
) {
    sqlx::query(
        r#"INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
    )
    .bind(court)
    .bind(name)
    .bind(format!("Test rule: {}", name))
    .bind("Federal Rules of Civil Procedure")
    .bind("Deadline")
    .bind(priority)
    .bind("Active")
    .bind(court)
    .bind(citation)
    .bind(conditions)
    .bind(actions)
    .bind(triggers)
    .execute(pool)
    .await
    .expect("Failed to seed rule");
}

/// When a rule with trigger `case_filed` and action `generate_deadline` is
/// active, creating a criminal case should automatically create a deadline
/// with the correct title, rule_code, and computation notes.
#[tokio::test]
async fn create_case_auto_creates_deadline() {
    let (app, pool, _guard) = test_app().await;

    // Seed rule: 90-day service-of-process deadline on criminal case filing
    seed_rule(
        &pool,
        "district9",
        "FRCP 4(m) Service Deadline",
        "FRCP 4(m)",
        20,
        json!([{"type": "field_equals", "field": "case_type", "value": "criminal"}]),
        json!([{"type": "generate_deadline", "description": "Service of process", "days_from_trigger": 90}]),
        json!(["case_filed"]),
    )
    .await;

    // Create a criminal case — the compliance engine should fire the rule
    let case = create_test_case_via_api(&app, "district9", "United States v. AutoDeadline").await;
    let case_id: uuid::Uuid = case["id"].as_str().unwrap().parse().unwrap();

    // Query the deadlines table for this case
    let deadlines = sqlx::query!(
        r#"SELECT title, rule_code, notes
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
    assert_eq!(
        dl.rule_code.as_deref(),
        Some("FRCP 4(m)"),
        "Deadline rule_code should match the rule citation"
    );
    assert!(
        dl.notes.as_ref().unwrap().contains("90 days"),
        "Computation notes should mention 90 days, got: {:?}",
        dl.notes
    );
}

/// When a rule with trigger `answer_filed` is active and an answer docket
/// entry is filed, a discovery-conference deadline should be auto-created.
#[tokio::test]
async fn docket_answer_creates_discovery_deadline() {
    let (app, pool, _guard) = test_app().await;

    // Seed rule: 14-day discovery conference deadline on answer filing
    seed_rule(
        &pool,
        "district9",
        "FRCP 26(f) Discovery Conference",
        "FRCP 26(f)",
        20,
        json!([{"type": "field_equals", "field": "case_type", "value": "criminal"}]),
        json!([{"type": "generate_deadline", "description": "Discovery conference scheduling", "days_from_trigger": 14}]),
        json!(["answer_filed"]),
    )
    .await;

    // Create a case, then file an answer docket entry
    let case = create_test_case_via_api(&app, "district9", "United States v. AnswerTest").await;
    let case_id_str = case["id"].as_str().unwrap();
    let case_id: uuid::Uuid = case_id_str.parse().unwrap();

    create_test_docket_entry(&app, "district9", case_id_str, "answer").await;

    // Verify the deadline was created
    let deadlines = sqlx::query!(
        r#"SELECT title, rule_code
           FROM deadlines
           WHERE court_id = $1 AND case_id = $2 AND title = 'Discovery conference scheduling'"#,
        "district9",
        case_id,
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query deadlines");

    assert!(
        !deadlines.is_empty(),
        "Expected a 'Discovery conference scheduling' deadline after filing an answer"
    );
    assert_eq!(deadlines[0].rule_code.as_deref(), Some("FRCP 26(f)"));
}

/// When a rule with trigger `motion_filed` is active and a motion docket
/// entry is filed, a response-due deadline should be auto-created.
#[tokio::test]
async fn docket_motion_creates_response_deadline() {
    let (app, pool, _guard) = test_app().await;

    // Seed rule: 21-day response deadline on motion filing
    seed_rule(
        &pool,
        "district9",
        "FRCP 12(a) Response to Motion",
        "FRCP 12(a)",
        20,
        json!([{"type": "always"}]),
        json!([{"type": "generate_deadline", "description": "Response to motion due", "days_from_trigger": 21}]),
        json!(["motion_filed"]),
    )
    .await;

    // Create a case, then file a motion docket entry
    let case = create_test_case_via_api(&app, "district9", "United States v. MotionTest").await;
    let case_id_str = case["id"].as_str().unwrap();
    let case_id: uuid::Uuid = case_id_str.parse().unwrap();

    create_test_docket_entry(&app, "district9", case_id_str, "motion").await;

    // Verify the deadline was created
    let deadlines = sqlx::query!(
        r#"SELECT title
           FROM deadlines
           WHERE court_id = $1 AND case_id = $2 AND title = 'Response to motion due'"#,
        "district9",
        case_id,
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query deadlines");

    assert!(
        !deadlines.is_empty(),
        "Expected a 'Response to motion due' deadline after filing a motion"
    );
}

/// When a blocking rule matches `complaint_filed`, attempting to create a
/// complaint docket entry should be rejected with HTTP 422.
#[tokio::test]
async fn blocking_rule_prevents_docket_entry() {
    let (app, pool, _guard) = test_app().await;

    // Seed a blocking rule that fires on complaint_filed trigger
    seed_rule(
        &pool,
        "district9",
        "Block Complaint Filings",
        "LR-TEST-BLOCK",
        40,
        json!([{"type": "always"}]),
        json!([{"type": "block_filing", "reason": "Complaint filings suspended"}]),
        json!(["complaint_filed"]),
    )
    .await;

    // Create a case (triggers case_filed, not complaint_filed — should succeed)
    let case = create_test_case_via_api(&app, "district9", "United States v. BlockTest").await;
    let case_id = case["id"].as_str().unwrap();

    // Attempt to create a complaint docket entry — should be blocked with 422
    let body = json!({
        "case_id": case_id,
        "entry_type": "complaint",
        "description": "Test complaint entry",
    });

    let court_roles = std::collections::HashMap::from([("district9".to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);

    let (status, resp) = post_json_authed(
        &app,
        "/api/docket/entries",
        &body.to_string(),
        "district9",
        &token,
    )
    .await;

    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "Complaint docket entry should be blocked with 422, got: {} - {}",
        status,
        resp
    );
    assert!(
        resp["message"]
            .as_str()
            .unwrap_or("")
            .contains("Complaint filings suspended"),
        "Error should contain the blocking reason, got: {}",
        resp
    );
}

/// The POST /api/rules/evaluate endpoint should match seeded rules against
/// a provided context and return the matching rules and their actions.
#[tokio::test]
async fn evaluate_rules_via_api() {
    let (app, pool, _guard) = test_app().await;

    // Seed two rules: one for criminal cases, one for civil only
    seed_rule(
        &pool,
        "district9",
        "Criminal Case Deadline",
        "FRCP-CRIM-TEST",
        20,
        json!([{"type": "field_equals", "field": "case_type", "value": "criminal"}]),
        json!([{"type": "generate_deadline", "description": "Criminal processing deadline", "days_from_trigger": 30}]),
        json!(["case_filed"]),
    )
    .await;

    seed_rule(
        &pool,
        "district9",
        "Civil Only Rule",
        "FRCP-CIVIL-TEST",
        20,
        json!([{"type": "field_equals", "field": "case_type", "value": "civil"}]),
        json!([{"type": "generate_deadline", "description": "Civil deadline", "days_from_trigger": 60}]),
        json!(["case_filed"]),
    )
    .await;

    // Evaluate rules with a criminal case context
    let eval_body = json!({
        "context": {
            "trigger": "case_filed",
            "case_type": "criminal",
            "document_type": "case_initiation",
            "filer_role": "system"
        }
    });

    let (status, resp) = post_json(
        &app,
        "/api/rules/evaluate",
        &eval_body.to_string(),
        "district9",
    )
    .await;

    assert_eq!(status, StatusCode::OK, "Evaluate should return 200, got: {} - {}", status, resp);

    // Only the criminal rule should match
    let matched = resp["matched_rules"].as_array().expect("matched_rules should be an array");
    assert_eq!(
        matched.len(),
        1,
        "Only the criminal rule should match, got {} matches: {}",
        matched.len(),
        resp
    );
    assert_eq!(
        matched[0]["name"].as_str().unwrap(),
        "Criminal Case Deadline",
        "Matched rule should be the criminal one"
    );

    // Actions should include the matched rule's action
    let actions = resp["actions"].as_array().expect("actions should be an array");
    assert!(
        !actions.is_empty(),
        "Actions should be non-empty for the matched rule"
    );
}

/// After creating a case, the compliance engine logs a case_event. The
/// GET /api/cases/{case_id}/compliance-events endpoint should return
/// this audit trail.
#[tokio::test]
async fn case_events_audit_trail() {
    let (app, _pool, _guard) = test_app().await;

    // Create a case — this triggers the compliance engine and logs a case_event
    let case = create_test_case_via_api(&app, "district9", "United States v. AuditTrail").await;
    let case_id = case["id"].as_str().unwrap();

    // Fetch the compliance events via the API
    let uri = format!("/api/cases/{}/compliance-events", case_id);
    let (status, resp) = get_with_court(&app, &uri, "district9").await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Compliance events endpoint should return 200, got: {} - {}",
        status,
        resp
    );

    let events = resp.as_array().expect("Response should be an array of events");
    assert!(
        !events.is_empty(),
        "Expected at least one compliance event after case creation"
    );

    // Verify the first event has trigger_event = "case_filed"
    let has_case_filed = events.iter().any(|e| {
        e.get("trigger_event")
            .and_then(|v| v.as_str())
            .map(|s| s == "case_filed")
            .unwrap_or(false)
    });
    assert!(
        has_case_filed,
        "At least one event should have trigger_event = 'case_filed', got: {:?}",
        events
    );
}

/// Full CRUD lifecycle for fee schedule entries.
#[tokio::test]
async fn fee_schedule_crud() {
    let (app, _pool, _guard) = test_app().await;

    // Create a fee entry
    let create_body = json!({
        "fee_id": "TEST-FEE",
        "category": "Filing",
        "description": "Test filing fee",
        "amount_cents": 40500
    });

    let (status, created) = post_json(
        &app,
        "/api/fee-schedule",
        &create_body.to_string(),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "Fee creation should return 201: {}", created);

    let fee_id = created["id"].as_str().expect("Created fee should have an id");
    assert_eq!(created["fee_id"].as_str().unwrap(), "TEST-FEE");
    assert_eq!(created["category"].as_str().unwrap(), "Filing");
    assert_eq!(created["description"].as_str().unwrap(), "Test filing fee");
    assert_eq!(created["amount_cents"].as_i64().unwrap(), 40500);

    // List fees — verify the new entry appears
    let (status, list) = get_with_court(&app, "/api/fee-schedule", "district9").await;
    assert_eq!(status, StatusCode::OK);
    let fees = list.as_array().expect("Fee list should be an array");
    assert!(
        fees.iter().any(|f| f["fee_id"].as_str() == Some("TEST-FEE")),
        "Created fee should appear in the list"
    );

    // Get fee by ID
    let get_uri = format!("/api/fee-schedule/{}", fee_id);
    let (status, fetched) = get_with_court(&app, &get_uri, "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["amount_cents"].as_i64().unwrap(), 40500);

    // Update fee amount
    let update_body = json!({ "amount_cents": 41000 });
    let (status, updated) = patch_json(
        &app,
        &get_uri,
        &update_body.to_string(),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Fee update should return 200: {}", updated);
    assert_eq!(
        updated["amount_cents"].as_i64().unwrap(),
        41000,
        "Amount should be updated to 41000"
    );

    // Delete (soft-delete) the fee
    let (status, _) = delete_with_court(&app, &get_uri, "district9").await;
    assert_eq!(status, StatusCode::NO_CONTENT, "Fee delete should return 204");

    // List fees again — the soft-deleted fee should no longer appear
    let (status, list_after) = get_with_court(&app, "/api/fee-schedule", "district9").await;
    assert_eq!(status, StatusCode::OK);
    let fees_after = list_after.as_array().expect("Fee list should be an array");
    assert!(
        !fees_after.iter().any(|f| f["fee_id"].as_str() == Some("TEST-FEE")),
        "Soft-deleted fee should not appear in active fee list"
    );
}

/// Legacy rule format (flat object conditions/actions) should be backward-
/// compatible with the compliance engine. When a legacy-format rule matches,
/// it should still produce the expected deadline.
#[tokio::test]
async fn legacy_rule_format_backward_compatible() {
    let (app, pool, _guard) = test_app().await;

    // Seed a rule with legacy (flat-object) format for conditions and actions
    seed_rule(
        &pool,
        "district9",
        "Legacy Format Rule",
        "LEGACY-TEST",
        20,
        json!({"case_type": "criminal"}),
        json!({"create_deadline": {"days": 30, "title": "Legacy deadline"}}),
        json!(["case_filed"]),
    )
    .await;

    // Create a criminal case — the legacy rule should fire
    let case = create_test_case_via_api(&app, "district9", "United States v. LegacyFormat").await;
    let case_id: uuid::Uuid = case["id"].as_str().unwrap().parse().unwrap();

    // Verify the legacy-format deadline was auto-created
    let deadlines = sqlx::query!(
        r#"SELECT title
           FROM deadlines
           WHERE court_id = $1 AND case_id = $2 AND title = 'Legacy deadline'"#,
        "district9",
        case_id,
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query deadlines");

    assert!(
        !deadlines.is_empty(),
        "Legacy rule format should still produce a deadline with title 'Legacy deadline'"
    );
}
