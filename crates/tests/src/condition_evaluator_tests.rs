//! Condition evaluator tests -- ported from spin-lexodus

use server::compliance::condition_evaluator::evaluate_condition;
use shared_types::compliance::{FilingContext, RuleCondition};
use serde_json::json;

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

fn ctx_with_meta(case_type: &str, doc_type: &str, meta: serde_json::Value) -> FilingContext {
    FilingContext {
        metadata: meta,
        ..ctx(case_type, doc_type)
    }
}

#[test]
fn field_equals_match() {
    let cond = RuleCondition::FieldEquals {
        field: "case_type".into(),
        value: "civil".into(),
    };
    assert!(evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn field_equals_no_match() {
    let cond = RuleCondition::FieldEquals {
        field: "case_type".into(),
        value: "civil".into(),
    };
    assert!(!evaluate_condition(&cond, &ctx("criminal", "complaint")));
}

#[test]
fn field_contains_match() {
    let cond = RuleCondition::FieldContains {
        field: "document_type".into(),
        value: "motion".into(),
    };
    assert!(evaluate_condition(&cond, &ctx("civil", "motion_to_dismiss")));
}

#[test]
fn field_exists_direct() {
    let cond = RuleCondition::FieldExists {
        field: "case_type".into(),
    };
    assert!(evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn field_exists_missing() {
    let cond = RuleCondition::FieldExists {
        field: "nonexistent".into(),
    };
    assert!(!evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn field_exists_metadata() {
    let cond = RuleCondition::FieldExists {
        field: "party_count".into(),
    };
    assert!(evaluate_condition(
        &cond,
        &ctx_with_meta("civil", "c", json!({"party_count": 3}))
    ));
}

#[test]
fn field_greater_than_numeric() {
    let cond = RuleCondition::FieldGreaterThan {
        field: "page_count".into(),
        value: "20".into(),
    };
    assert!(evaluate_condition(
        &cond,
        &ctx_with_meta("civil", "c", json!({"page_count": "25"}))
    ));
    assert!(!evaluate_condition(
        &cond,
        &ctx_with_meta("civil", "c", json!({"page_count": "10"}))
    ));
}

#[test]
fn field_less_than_numeric() {
    let cond = RuleCondition::FieldLessThan {
        field: "page_count".into(),
        value: "20".into(),
    };
    assert!(evaluate_condition(
        &cond,
        &ctx_with_meta("civil", "c", json!({"page_count": "5"}))
    ));
}

#[test]
fn and_all_true() {
    let cond = RuleCondition::And {
        conditions: vec![
            RuleCondition::FieldEquals {
                field: "case_type".into(),
                value: "civil".into(),
            },
            RuleCondition::FieldEquals {
                field: "document_type".into(),
                value: "complaint".into(),
            },
        ],
    };
    assert!(evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn and_one_false() {
    let cond = RuleCondition::And {
        conditions: vec![
            RuleCondition::FieldEquals {
                field: "case_type".into(),
                value: "civil".into(),
            },
            RuleCondition::FieldEquals {
                field: "document_type".into(),
                value: "motion".into(),
            },
        ],
    };
    assert!(!evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn or_one_true() {
    let cond = RuleCondition::Or {
        conditions: vec![
            RuleCondition::FieldEquals {
                field: "case_type".into(),
                value: "criminal".into(),
            },
            RuleCondition::FieldEquals {
                field: "case_type".into(),
                value: "civil".into(),
            },
        ],
    };
    assert!(evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn or_all_false() {
    let cond = RuleCondition::Or {
        conditions: vec![
            RuleCondition::FieldEquals {
                field: "case_type".into(),
                value: "criminal".into(),
            },
            RuleCondition::FieldEquals {
                field: "case_type".into(),
                value: "bankruptcy".into(),
            },
        ],
    };
    assert!(!evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn not_negates() {
    let cond = RuleCondition::Not {
        condition: Box::new(RuleCondition::FieldEquals {
            field: "case_type".into(),
            value: "criminal".into(),
        }),
    };
    assert!(evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn always_returns_true() {
    assert!(evaluate_condition(&RuleCondition::Always, &ctx("civil", "c")));
}

#[test]
fn nested_compound() {
    // And(Or(civil, criminal), Not(doc_type == "brief"))
    let cond = RuleCondition::And {
        conditions: vec![
            RuleCondition::Or {
                conditions: vec![
                    RuleCondition::FieldEquals {
                        field: "case_type".into(),
                        value: "civil".into(),
                    },
                    RuleCondition::FieldEquals {
                        field: "case_type".into(),
                        value: "criminal".into(),
                    },
                ],
            },
            RuleCondition::Not {
                condition: Box::new(RuleCondition::FieldEquals {
                    field: "document_type".into(),
                    value: "brief".into(),
                }),
            },
        ],
    };
    assert!(evaluate_condition(&cond, &ctx("civil", "motion")));
    assert!(!evaluate_condition(&cond, &ctx("civil", "brief")));
}

#[test]
fn metadata_boolean_as_string() {
    let cond = RuleCondition::FieldEquals {
        field: "pro_se".into(),
        value: "true".into(),
    };
    assert!(evaluate_condition(
        &cond,
        &ctx_with_meta("civil", "c", json!({"pro_se": true}))
    ));
}
