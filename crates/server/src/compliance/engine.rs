//! Rules evaluation engine — 5-stage pipeline
//!
//! Ported from spin-lexodus. Stateless engine that:
//! 1. Selects rules by jurisdiction + trigger + in-effect status
//! 2. Sorts by priority weight (StandingOrder > Local > Admin > Federal > Statutory)
//! 3. Evaluates recursive condition trees against filing context
//! 4. Processes matched rule actions into compliance report
//! 5. Returns ComplianceReport (blocked, warnings, deadlines, fees)

use chrono::Utc;
use shared_types::compliance::*;
use shared_types::Rule;
use super::condition_evaluator::evaluate_condition;

/// Select applicable rules for a given jurisdiction and trigger event.
/// Filters by: in-effect status, jurisdiction match (or global), trigger match.
pub fn select_rules(jurisdiction: &str, trigger: &TriggerEvent, all_rules: &[Rule]) -> Vec<Rule> {
    let jurisdiction_lower = jurisdiction.to_lowercase();
    let trigger_str = serde_json::to_value(trigger)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    all_rules
        .iter()
        .filter(|rule| {
            // Must be active
            if rule.status != "Active" {
                return false;
            }

            // Check effective/expiration dates
            let now = Utc::now();
            if let Some(eff) = rule.effective_date {
                if now < eff {
                    return false;
                }
            }
            if let Some(exp) = rule.expiration_date {
                if now > exp {
                    return false;
                }
            }

            // Jurisdiction: matches if rule has no jurisdiction (global) or matches
            let jurisdiction_match = rule
                .jurisdiction
                .as_ref()
                .map_or(true, |j| j.to_lowercase() == jurisdiction_lower);
            if !jurisdiction_match {
                return false;
            }

            // Trigger: check if rule's triggers array contains this trigger
            if let Some(triggers) = rule.triggers.as_array() {
                triggers
                    .iter()
                    .any(|t| t.as_str().map_or(false, |s| s == trigger_str))
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

/// Sort rules by priority weight (highest first). Stable sort preserves order within same priority.
pub fn resolve_priority(mut rules: Vec<Rule>) -> Vec<Rule> {
    rules.sort_by(|a, b| {
        let wa = RulePriority::from_db_priority(a.priority).weight();
        let wb = RulePriority::from_db_priority(b.priority).weight();
        wb.cmp(&wa)
    });
    rules
}

/// Evaluate a set of rules against a filing context.
/// Returns a ComplianceReport with all results, blocks, warnings, deadlines, and fees.
pub fn evaluate(context: &FilingContext, rules: &[Rule]) -> ComplianceReport {
    let mut report = ComplianceReport::default();
    let today = Utc::now().date_naive();

    for rule in rules {
        // Parse conditions from JSONB
        let conditions: Vec<RuleCondition> = parse_conditions(&rule.conditions);

        // Evaluate: all conditions must match (AND semantics at top level)
        let all_match = if conditions.is_empty() {
            true // No conditions = always matches
        } else {
            conditions.iter().all(|c| evaluate_condition(c, context))
        };

        if all_match {
            // Parse and process actions
            let actions: Vec<RuleAction> = parse_actions(&rule.actions);
            process_actions(rule, &actions, &mut report, today);
        } else {
            report.results.push(RuleResult {
                rule_id: rule.id,
                rule_name: rule.name.clone(),
                matched: false,
                action_taken: "none".to_string(),
                message: "Conditions not met".to_string(),
            });
        }
    }

    report
}

/// Parse conditions from JSONB. Supports both:
/// - New format: array of RuleCondition objects `[{"type": "field_equals", ...}]`
/// - Legacy format: flat object `{"trigger": "case_filed", "case_type": "civil"}`
fn parse_conditions(value: &serde_json::Value) -> Vec<RuleCondition> {
    // Try array of typed conditions first
    if let Ok(conditions) = serde_json::from_value::<Vec<RuleCondition>>(value.clone()) {
        return conditions;
    }

    // Try single condition object
    if let Ok(condition) = serde_json::from_value::<RuleCondition>(value.clone()) {
        return vec![condition];
    }

    // Legacy format: convert flat object to FieldEquals conditions
    if let Some(obj) = value.as_object() {
        let mut conditions = Vec::new();
        for (key, val) in obj {
            if key == "trigger" {
                continue; // Triggers are now in separate column
            }
            if let Some(v) = val.as_str() {
                conditions.push(RuleCondition::FieldEquals {
                    field: key.clone(),
                    value: v.to_string(),
                });
            }
        }
        return conditions;
    }

    Vec::new()
}

/// Parse actions from JSONB. Supports both:
/// - New format: array of RuleAction objects `[{"type": "generate_deadline", ...}]`
/// - Legacy format: object with action keys `{"create_deadline": {"days": 90, ...}}`
fn parse_actions(value: &serde_json::Value) -> Vec<RuleAction> {
    // Try array of typed actions
    if let Ok(actions) = serde_json::from_value::<Vec<RuleAction>>(value.clone()) {
        return actions;
    }

    // Try single action object
    if let Ok(action) = serde_json::from_value::<RuleAction>(value.clone()) {
        return vec![action];
    }

    // Legacy format: convert known action keys
    if let Some(obj) = value.as_object() {
        let mut actions = Vec::new();
        if let Some(dl) = obj.get("create_deadline") {
            if let Some(days) = dl.get("days").and_then(|d| d.as_i64()) {
                let title = dl
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Deadline");
                actions.push(RuleAction::GenerateDeadline {
                    description: title.to_string(),
                    days_from_trigger: days as i32,
                });
            }
        }
        return actions;
    }

    Vec::new()
}

/// Process matched rule actions into the compliance report.
fn process_actions(
    rule: &Rule,
    actions: &[RuleAction],
    report: &mut ComplianceReport,
    today: chrono::NaiveDate,
) {
    for action in actions {
        match action {
            RuleAction::BlockFiling { reason } => {
                report.blocked = true;
                report
                    .block_reasons
                    .push(format!("[{}] {}", rule.name, reason));
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "block_filing".to_string(),
                    message: reason.clone(),
                });
            }
            RuleAction::FlagForReview { reason } => {
                report
                    .warnings
                    .push(format!("[{}] {}", rule.name, reason));
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "flag_for_review".to_string(),
                    message: reason.clone(),
                });
            }
            RuleAction::GenerateDeadline {
                description,
                days_from_trigger,
            } => {
                let due_date = today + chrono::Duration::days(*days_from_trigger as i64);
                report.deadlines.push(DeadlineResult {
                    due_date,
                    description: description.clone(),
                    rule_citation: rule.citation.clone().unwrap_or_default(),
                    computation_notes: format!(
                        "Generated by rule '{}': {} days from trigger",
                        rule.name, days_from_trigger
                    ),
                    is_short_period: *days_from_trigger <= 14,
                });
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "generate_deadline".to_string(),
                    message: format!("{} (due {})", description, due_date),
                });
            }
            RuleAction::RequireRedaction { fields } => {
                let field_list = fields.join(", ");
                report.warnings.push(format!(
                    "[{}] Redaction required for: {}",
                    rule.name, field_list
                ));
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "require_redaction".to_string(),
                    message: format!("Redaction required for: {}", field_list),
                });
            }
            RuleAction::SendNotification { recipient, message } => {
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "send_notification".to_string(),
                    message: format!("Notify {}: {}", recipient, message),
                });
            }
            RuleAction::RequireFee {
                amount_cents,
                description,
            } => {
                report.fees.push(FeeRequirement {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    amount_cents: *amount_cents,
                    description: description.clone(),
                });
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "require_fee".to_string(),
                    message: format!(
                        "{}: ${:.2}",
                        description, *amount_cents as f64 / 100.0
                    ),
                });
            }
            RuleAction::LogCompliance { message } => {
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "log_compliance".to_string(),
                    message: message.clone(),
                });
            }
        }
    }
}
