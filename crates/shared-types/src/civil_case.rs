use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Validation constants ────────────────────────────────────────────

/// Valid civil case status values matching the DB CHECK constraint.
pub const CIVIL_CASE_STATUSES: &[&str] = &[
    "filed", "pending", "discovery", "pretrial", "trial_ready", "in_trial",
    "settled", "judgment_entered", "on_appeal", "closed", "dismissed", "transferred",
];

/// Valid jurisdiction basis values matching the DB CHECK constraint.
pub const CIVIL_JURISDICTION_BASES: &[&str] = &[
    "federal_question", "diversity", "us_government_plaintiff", "us_government_defendant",
];

/// Valid jury demand values matching the DB CHECK constraint.
pub const CIVIL_JURY_DEMANDS: &[&str] = &["none", "plaintiff", "defendant", "both"];

/// Valid civil case priority values matching the DB CHECK constraint.
pub const CIVIL_CASE_PRIORITIES: &[&str] = &["low", "medium", "high", "critical"];

/// Check whether a status string is a valid civil case status.
pub fn is_valid_civil_status(s: &str) -> bool {
    CIVIL_CASE_STATUSES.contains(&s)
}

/// Check whether a jurisdiction basis string is valid.
pub fn is_valid_jurisdiction_basis(s: &str) -> bool {
    CIVIL_JURISDICTION_BASES.contains(&s)
}

/// Check whether a jury demand string is valid.
pub fn is_valid_jury_demand(s: &str) -> bool {
    CIVIL_JURY_DEMANDS.contains(&s)
}

// ── DB row struct ───────────────────────────────────────────────────

/// A federal civil case record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct CivilCase {
    pub id: Uuid,
    pub court_id: String,
    pub case_number: String,
    pub title: String,
    pub description: String,
    pub nature_of_suit: String,
    pub cause_of_action: String,
    pub jurisdiction_basis: String,
    pub jury_demand: String,
    pub class_action: bool,
    pub amount_in_controversy: Option<f64>,
    pub status: String,
    pub priority: String,
    pub assigned_judge_id: Option<Uuid>,
    pub district_code: String,
    pub location: String,
    pub is_sealed: bool,
    pub sealed_date: Option<DateTime<Utc>>,
    pub sealed_by: Option<String>,
    pub seal_reason: Option<String>,
    pub related_case_id: Option<Uuid>,
    pub consent_to_magistrate: bool,
    pub pro_se: bool,
    pub opened_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

// ── API response types ──────────────────────────────────────────────

/// API response shape for a civil case.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CivilCaseResponse {
    pub id: String,
    pub case_number: String,
    pub title: String,
    pub description: String,
    pub nature_of_suit: String,
    pub cause_of_action: String,
    pub jurisdiction_basis: String,
    pub jury_demand: String,
    pub class_action: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_in_controversy: Option<f64>,
    pub status: String,
    pub priority: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_judge_id: Option<String>,
    pub district_code: String,
    pub location: String,
    pub is_sealed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sealed_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sealed_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seal_reason: Option<String>,
    pub opened_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_case_id: Option<String>,
    pub consent_to_magistrate: bool,
    pub pro_se: bool,
}

impl From<CivilCase> for CivilCaseResponse {
    fn from(c: CivilCase) -> Self {
        Self {
            id: c.id.to_string(),
            case_number: c.case_number,
            title: c.title,
            description: c.description,
            nature_of_suit: c.nature_of_suit,
            cause_of_action: c.cause_of_action,
            jurisdiction_basis: c.jurisdiction_basis,
            jury_demand: c.jury_demand,
            class_action: c.class_action,
            amount_in_controversy: c.amount_in_controversy,
            status: c.status,
            priority: c.priority,
            assigned_judge_id: c.assigned_judge_id.map(|u| u.to_string()),
            district_code: c.district_code,
            location: c.location,
            is_sealed: c.is_sealed,
            sealed_date: c.sealed_date.map(|d| d.to_rfc3339()),
            sealed_by: c.sealed_by,
            seal_reason: c.seal_reason,
            opened_at: c.opened_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
            closed_at: c.closed_at.map(|d| d.to_rfc3339()),
            related_case_id: c.related_case_id.map(|u| u.to_string()),
            consent_to_magistrate: c.consent_to_magistrate,
            pro_se: c.pro_se,
        }
    }
}

/// Search response for civil cases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CivilCaseSearchResponse {
    pub cases: Vec<CivilCaseResponse>,
    pub total: i64,
}

// ── Request types ───────────────────────────────────────────────────

/// Request to create a new civil case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateCivilCaseRequest {
    pub title: String,
    pub nature_of_suit: String,
    #[serde(default)]
    pub cause_of_action: Option<String>,
    pub jurisdiction_basis: String,
    #[serde(default)]
    pub jury_demand: Option<String>,
    #[serde(default)]
    pub class_action: Option<bool>,
    #[serde(default)]
    pub amount_in_controversy: Option<f64>,
    #[serde(default)]
    pub district_code: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub assigned_judge_id: Option<String>,
    #[serde(default)]
    pub consent_to_magistrate: Option<bool>,
    #[serde(default)]
    pub pro_se: Option<bool>,
}

/// Request to update civil case status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateCivilCaseStatusRequest {
    pub status: String,
}

/// Query parameters for civil case search.
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct CivilCaseSearchParams {
    pub status: Option<String>,
    pub nature_of_suit: Option<String>,
    pub jurisdiction_basis: Option<String>,
    pub class_action: Option<bool>,
    pub assigned_judge_id: Option<String>,
    pub q: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

// ── Nature of Suit reference ────────────────────────────────────────

/// A JS-44 Nature of Suit code entry for the reference table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct NatureOfSuitCode {
    pub code: String,
    pub title: String,
    pub category: String,
    pub description: String,
}
