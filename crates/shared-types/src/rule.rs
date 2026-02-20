use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Domain Struct
// ---------------------------------------------------------------------------

/// A legal rule or local court rule tracked in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Rule {
    pub id: Uuid,
    pub court_id: String,
    pub name: String,
    /// Nullable in DB (TEXT without NOT NULL).
    pub description: Option<String>,
    /// RuleSource enum stored as text (e.g. "FRCP", "LocalRule", "Statute").
    pub source: String,
    /// RuleCategory enum stored as text.
    pub category: String,
    /// Priority as an integer (0 = lowest).
    pub priority: i32,
    /// RuleStatus enum stored as text (e.g. "Active", "Superseded", "Repealed").
    pub status: String,
    pub jurisdiction: Option<String>,
    pub citation: Option<String>,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub supersedes_rule_id: Option<Uuid>,
    /// JSON conditions that trigger this rule.
    pub conditions: serde_json::Value,
    /// JSON actions to perform when the rule matches.
    pub actions: serde_json::Value,
    /// Trigger events that activate this rule (JSONB array of strings)
    pub triggers: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Rule Request/Response DTOs
// ---------------------------------------------------------------------------

/// API response for a rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RuleResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub source: String,
    pub category: String,
    pub priority: i32,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes_rule_id: Option<String>,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub triggers: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Rule> for RuleResponse {
    fn from(r: Rule) -> Self {
        Self {
            id: r.id.to_string(),
            name: r.name,
            description: r.description,
            source: r.source,
            category: r.category,
            priority: r.priority,
            status: r.status,
            jurisdiction: r.jurisdiction,
            citation: r.citation,
            effective_date: r.effective_date.map(|d| d.to_rfc3339()),
            expiration_date: r.expiration_date.map(|d| d.to_rfc3339()),
            supersedes_rule_id: r.supersedes_rule_id.map(|id| id.to_string()),
            conditions: r.conditions,
            actions: r.actions,
            triggers: r.triggers,
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

/// Request body for creating a new rule.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateRuleRequest {
    pub name: String,
    pub description: String,
    pub source: String,
    pub category: String,
    pub priority: i32,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub jurisdiction: Option<String>,
    #[serde(default)]
    pub citation: Option<String>,
    #[serde(default)]
    pub effective_date: Option<String>,
    #[serde(default)]
    pub conditions: Option<serde_json::Value>,
    #[serde(default)]
    pub actions: Option<serde_json::Value>,
    #[serde(default)]
    pub triggers: Option<serde_json::Value>,
}

/// Request body for updating a rule (all fields optional).
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub category: Option<String>,
    pub priority: Option<i32>,
    pub status: Option<String>,
    pub jurisdiction: Option<String>,
    pub citation: Option<String>,
    pub effective_date: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    #[serde(default)]
    pub triggers: Option<serde_json::Value>,
}

/// Request body for evaluating rules against a context.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EvaluateRulesRequest {
    pub context: serde_json::Value,
    #[serde(default)]
    pub category: Option<String>,
}

/// Response body for a rule evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EvaluateRulesResponse {
    pub matched_rules: Vec<RuleResponse>,
    pub actions: Vec<serde_json::Value>,
}
