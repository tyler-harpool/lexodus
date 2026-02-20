//! Recursive condition tree evaluator
//!
//! Ported from spin-lexodus. Evaluates RuleCondition trees against
//! a FilingContext by resolving field values from the context struct
//! and falling back to the metadata JSON object.

use shared_types::compliance::{FilingContext, RuleCondition};

/// Evaluate a condition tree against a filing context.
pub fn evaluate_condition(condition: &RuleCondition, context: &FilingContext) -> bool {
    match condition {
        RuleCondition::And { conditions } => {
            conditions.iter().all(|c| evaluate_condition(c, context))
        }
        RuleCondition::Or { conditions } => {
            conditions.iter().any(|c| evaluate_condition(c, context))
        }
        RuleCondition::Not { condition } => !evaluate_condition(condition, context),
        RuleCondition::FieldEquals { field, value } => {
            get_field_value(field, context).map_or(false, |v| v == *value)
        }
        RuleCondition::FieldContains { field, value } => {
            get_field_value(field, context).map_or(false, |v| v.contains(value.as_str()))
        }
        RuleCondition::FieldExists { field } => field_exists(field, context),
        RuleCondition::FieldGreaterThan { field, value } => {
            get_field_value(field, context).map_or(false, |v| {
                match (v.parse::<f64>(), value.parse::<f64>()) {
                    (Ok(field_num), Ok(threshold)) => field_num > threshold,
                    _ => v.as_str() > value.as_str(),
                }
            })
        }
        RuleCondition::FieldLessThan { field, value } => {
            get_field_value(field, context).map_or(false, |v| {
                match (v.parse::<f64>(), value.parse::<f64>()) {
                    (Ok(field_num), Ok(threshold)) => field_num < threshold,
                    _ => v.as_str() < value.as_str(),
                }
            })
        }
        RuleCondition::Always => true,
    }
}

/// Resolve a field value from the filing context.
/// Checks direct struct fields first, falls back to metadata JSON.
fn get_field_value(field: &str, context: &FilingContext) -> Option<String> {
    match field {
        "case_type" => Some(context.case_type.clone()),
        "document_type" => Some(context.document_type.clone()),
        "filer_role" => Some(context.filer_role.clone()),
        "jurisdiction_id" => Some(context.jurisdiction_id.clone()),
        "division" => context.division.clone(),
        "assigned_judge" => context.assigned_judge.clone(),
        _ => context.metadata.get(field).and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            _ => Some(v.to_string()),
        }),
    }
}

/// Check whether a field exists in the filing context.
fn field_exists(field: &str, context: &FilingContext) -> bool {
    match field {
        "case_type" | "document_type" | "filer_role" | "jurisdiction_id" => true,
        "division" => context.division.is_some(),
        "assigned_judge" => context.assigned_judge.is_some(),
        "service_method" => context.service_method.is_some(),
        _ => context
            .metadata
            .get(field)
            .map_or(false, |v| !v.is_null()),
    }
}
