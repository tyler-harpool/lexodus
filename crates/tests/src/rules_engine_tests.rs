//! Rules engine pipeline tests
//! Tests the full select -> prioritize -> evaluate -> report pipeline

use chrono::Utc;
use serde_json::json;
use server::compliance::engine;
use shared_types::compliance::*;
use shared_types::Rule;
use uuid::Uuid;

fn make_rule(
    name: &str,
    priority: i32,
    triggers: &[&str],
    conditions: serde_json::Value,
    actions: serde_json::Value,
) -> Rule {
    let now = Utc::now();
    Rule {
        id: Uuid::new_v4(),
        court_id: "district9".to_string(),
        name: name.to_string(),
        description: Some(format!("Test rule: {}", name)),
        source: "Federal Rules of Civil Procedure".to_string(),
        category: "Deadline".to_string(),
        priority,
        status: "Active".to_string(),
        jurisdiction: Some("district9".to_string()),
        citation: Some("Test Citation".to_string()),
        effective_date: None,
        expiration_date: None,
        supersedes_rule_id: None,
        conditions,
        actions,
        triggers: json!(triggers),
        created_at: now,
        updated_at: now,
    }
}

fn ctx(case_type: &str, doc_type: &str) -> FilingContext {
    FilingContext {
        case_type: case_type.to_string(),
        document_type: doc_type.to_string(),
        filer_role: "attorney".to_string(),
        jurisdiction_id: "district9".to_string(),
        division: None,
        assigned_judge: None,
        service_method: None,
        metadata: json!({}),
    }
}

#[test]
fn select_rules_jurisdiction_match() {
    let rule = make_rule("test", 20, &["case_filed"], json!({}), json!({}));
    let selected = engine::select_rules("district9", &TriggerEvent::CaseFiled, &[rule]);
    assert_eq!(selected.len(), 1);
}

#[test]
fn select_rules_jurisdiction_mismatch() {
    let rule = make_rule("test", 20, &["case_filed"], json!({}), json!({}));
    let selected = engine::select_rules("district12", &TriggerEvent::CaseFiled, &[rule]);
    assert_eq!(selected.len(), 0);
}

#[test]
fn select_rules_trigger_mismatch() {
    let rule = make_rule("test", 20, &["case_filed"], json!({}), json!({}));
    let selected = engine::select_rules("district9", &TriggerEvent::MotionFiled, &[rule]);
    assert_eq!(selected.len(), 0);
}

#[test]
fn select_rules_inactive_filtered() {
    let mut rule = make_rule("inactive", 20, &["case_filed"], json!({}), json!({}));
    rule.status = "Inactive".to_string();
    let selected = engine::select_rules("district9", &TriggerEvent::CaseFiled, &[rule]);
    assert_eq!(selected.len(), 0);
}

#[test]
fn resolve_priority_orders_by_weight() {
    let standing = make_rule("standing", 50, &["case_filed"], json!({}), json!({}));
    let local = make_rule("local", 40, &["case_filed"], json!({}), json!({}));
    let federal = make_rule("federal", 20, &["case_filed"], json!({}), json!({}));
    let statutory = make_rule("statutory", 10, &["case_filed"], json!({}), json!({}));

    let sorted = engine::resolve_priority(vec![statutory, federal, local, standing]);
    assert_eq!(sorted[0].name, "standing");
    assert_eq!(sorted[1].name, "local");
    assert_eq!(sorted[2].name, "federal");
    assert_eq!(sorted[3].name, "statutory");
}

#[test]
fn evaluate_matching_rule_generates_deadline() {
    let rule = make_rule(
        "FRCP 4(m)",
        20,
        &["case_filed"],
        json!([{"type": "field_equals", "field": "case_type", "value": "civil"}]),
        json!([{"type": "generate_deadline", "description": "Service of process", "days_from_trigger": 90}]),
    );

    let report = engine::evaluate(&ctx("civil", "complaint"), &[rule]);
    assert_eq!(report.deadlines.len(), 1);
    assert_eq!(report.deadlines[0].description, "Service of process");
    assert!(!report.deadlines[0].is_short_period);
    assert!(report.results[0].matched);
}

#[test]
fn evaluate_non_matching_conditions() {
    let rule = make_rule(
        "civil-only",
        20,
        &["case_filed"],
        json!([{"type": "field_equals", "field": "case_type", "value": "civil"}]),
        json!([{"type": "block_filing", "reason": "Should not fire"}]),
    );

    let report = engine::evaluate(&ctx("criminal", "indictment"), &[rule]);
    assert!(!report.blocked);
    assert!(!report.results[0].matched);
}

#[test]
fn evaluate_block_filing() {
    let rule = make_rule(
        "block-rule",
        20,
        &["document_filed"],
        json!([{"type": "always"}]),
        json!([{"type": "block_filing", "reason": "Missing cover sheet"}]),
    );

    let report = engine::evaluate(&ctx("civil", "complaint"), &[rule]);
    assert!(report.blocked);
    assert!(report.block_reasons[0].contains("Missing cover sheet"));
}

#[test]
fn evaluate_require_fee() {
    let rule = make_rule(
        "filing-fee",
        40,
        &["case_filed"],
        json!([{"type": "field_equals", "field": "case_type", "value": "civil"}]),
        json!([{"type": "require_fee", "amount_cents": 40500, "description": "Civil filing fee"}]),
    );

    let report = engine::evaluate(&ctx("civil", "complaint"), &[rule]);
    assert_eq!(report.fees.len(), 1);
    assert_eq!(report.fees[0].amount_cents, 40500);
}

#[test]
fn evaluate_legacy_condition_format() {
    // Tests backward compatibility with existing seeded rules
    let rule = make_rule(
        "legacy",
        20,
        &["case_filed"],
        json!({"case_type": "civil"}),
        json!({"create_deadline": {"days": 90, "title": "Service of process"}}),
    );

    let report = engine::evaluate(&ctx("civil", "complaint"), &[rule]);
    assert_eq!(report.deadlines.len(), 1);
    assert_eq!(report.deadlines[0].description, "Service of process");
}

#[test]
fn evaluate_empty_rules() {
    let report = engine::evaluate(&ctx("civil", "complaint"), &[]);
    assert!(report.results.is_empty());
    assert!(!report.blocked);
}

#[test]
fn evaluate_multiple_rules_mixed_match() {
    let matching = make_rule(
        "matches",
        20,
        &["case_filed"],
        json!([{"type": "field_equals", "field": "case_type", "value": "civil"}]),
        json!([{"type": "flag_for_review", "reason": "Matched"}]),
    );
    let not_matching = make_rule(
        "no-match",
        20,
        &["case_filed"],
        json!([{"type": "field_equals", "field": "case_type", "value": "criminal"}]),
        json!([{"type": "block_filing", "reason": "Should not fire"}]),
    );

    let report = engine::evaluate(&ctx("civil", "complaint"), &[matching, not_matching]);
    assert_eq!(report.results.len(), 2);
    assert!(report.results[0].matched);
    assert!(!report.results[1].matched);
    assert!(!report.blocked);
    assert_eq!(report.warnings.len(), 1);
}
