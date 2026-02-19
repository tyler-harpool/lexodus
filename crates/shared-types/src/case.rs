use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Validation constants ────────────────────────────────────────────

/// Valid case status values matching the DB CHECK constraint.
pub const CASE_STATUSES: &[&str] = &[
    "filed", "arraigned", "discovery", "pretrial_motions", "plea_negotiations",
    "trial_ready", "in_trial", "awaiting_sentencing", "sentenced", "dismissed", "on_appeal",
];

/// Valid crime type values matching the DB CHECK constraint.
pub const CRIME_TYPES: &[&str] = &[
    "fraud", "drug_offense", "racketeering", "cybercrime", "tax_offense",
    "money_laundering", "immigration", "firearms", "other",
];

/// Valid case priority values matching the DB CHECK constraint.
pub const CASE_PRIORITIES: &[&str] = &["low", "medium", "high", "critical"];

/// Check whether a status string is a valid case status.
pub fn is_valid_case_status(s: &str) -> bool {
    CASE_STATUSES.contains(&s)
}

/// Check whether a crime type string is valid.
pub fn is_valid_crime_type(s: &str) -> bool {
    CRIME_TYPES.contains(&s)
}

/// Check whether a priority string is valid.
pub fn is_valid_case_priority(s: &str) -> bool {
    CASE_PRIORITIES.contains(&s)
}

// ── DB row struct ───────────────────────────────────────────────────

/// A federal criminal case record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct CriminalCase {
    pub id: Uuid,
    pub court_id: String,
    pub case_number: String,
    pub title: String,
    pub description: String,
    pub crime_type: String,
    pub status: String,
    pub priority: String,
    pub assigned_judge_id: Option<Uuid>,
    pub district_code: String,
    pub location: String,
    pub is_sealed: bool,
    pub sealed_date: Option<DateTime<Utc>>,
    pub sealed_by: Option<String>,
    pub seal_reason: Option<String>,
    pub opened_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

// ── API response types ──────────────────────────────────────────────

/// API response shape for a criminal case.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CaseResponse {
    pub id: String,
    pub case_number: String,
    pub title: String,
    pub description: String,
    /// "criminal" or "civil"
    #[serde(default = "default_case_type")]
    pub case_type: String,
    /// For criminal: crime_type. For civil: nature_of_suit code.
    pub crime_type: String,
    pub status: String,
    pub priority: String,
    pub district_code: String,
    pub location: String,
    pub opened_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_judge_id: Option<String>,
    pub is_sealed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sealed_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sealed_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seal_reason: Option<String>,
    /// Civil-only: jurisdiction basis (federal_question, diversity, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jurisdiction_basis: Option<String>,
    /// Civil-only: jury demand (none, plaintiff, defendant, both)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jury_demand: Option<String>,
    /// Civil-only: class action flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_action: Option<bool>,
    /// Civil-only: amount in controversy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_in_controversy: Option<f64>,
    /// Civil-only: consent to magistrate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent_to_magistrate: Option<bool>,
    /// Civil-only: pro se litigant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pro_se: Option<bool>,
}

fn default_case_type() -> String {
    "criminal".to_string()
}

impl From<CriminalCase> for CaseResponse {
    fn from(c: CriminalCase) -> Self {
        Self {
            id: c.id.to_string(),
            case_number: c.case_number,
            title: c.title,
            description: c.description,
            case_type: "criminal".to_string(),
            crime_type: c.crime_type,
            status: c.status,
            priority: c.priority,
            district_code: c.district_code,
            location: c.location,
            opened_at: c.opened_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
            closed_at: c.closed_at.map(|d| d.to_rfc3339()),
            assigned_judge_id: c.assigned_judge_id.map(|u| u.to_string()),
            is_sealed: c.is_sealed,
            sealed_by: c.sealed_by,
            sealed_date: c.sealed_date.map(|d| d.to_rfc3339()),
            seal_reason: c.seal_reason,
            jurisdiction_basis: None,
            jury_demand: None,
            class_action: None,
            amount_in_controversy: None,
            consent_to_magistrate: None,
            pro_se: None,
        }
    }
}

/// Search response for cases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CaseSearchResponse {
    pub cases: Vec<CaseResponse>,
    pub total: i64,
}

// ── Request types ───────────────────────────────────────────────────

/// Request to create a new criminal case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateCaseRequest {
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub crime_type: String,
    pub district_code: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub assigned_judge_id: Option<Uuid>,
}

/// Request to update case status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateCaseStatusRequest {
    pub status: String,
}

/// Request to update a criminal case (all fields optional — only provided fields are changed).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateCaseRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub district_code: Option<String>,
}

/// Query parameters for case search.
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct CaseSearchParams {
    pub status: Option<String>,
    pub crime_type: Option<String>,
    pub priority: Option<String>,
    pub q: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

/// A defendant in a criminal case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Defendant {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub name: String,
    pub aliases: Vec<String>,
    pub usm_number: Option<String>,
    pub fbi_number: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    /// CitizenshipStatus enum stored as text.
    pub citizenship_status: String,
    /// CustodyStatus enum stored as text (e.g. "InCustody", "Released", "Fugitive").
    pub custody_status: String,
    /// BailType enum stored as text (e.g. "Personal", "Corporate", "Cash").
    pub bail_type: Option<String>,
    pub bail_amount: Option<f64>,
    pub bond_conditions: Vec<String>,
    pub bond_posted_date: Option<DateTime<Utc>>,
    pub surety_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single criminal charge (count) against a defendant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Charge {
    pub id: Uuid,
    pub court_id: String,
    pub defendant_id: Uuid,
    pub count_number: i32,
    pub statute: String,
    pub offense_description: String,
    pub statutory_max_months: Option<i32>,
    pub statutory_min_months: Option<i32>,
    /// PleaType enum stored as text (e.g. "NotGuilty", "Guilty", "NoloCon").
    pub plea: String,
    pub plea_date: Option<DateTime<Utc>>,
    /// Verdict enum stored as text (e.g. "Guilty", "NotGuilty", "Mistrial").
    pub verdict: String,
    pub verdict_date: Option<DateTime<Utc>>,
}

/// A piece of evidence in a criminal case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Evidence {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub description: String,
    /// EvidenceType enum stored as text (e.g. "Physical", "Documentary", "Digital").
    pub evidence_type: String,
    pub seized_date: Option<DateTime<Utc>>,
    pub seized_by: Option<String>,
    pub location: String,
    pub is_sealed: bool,
    pub created_at: DateTime<Utc>,
}

/// Chain-of-custody transfer record for evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct CustodyTransfer {
    pub id: Uuid,
    pub court_id: String,
    pub evidence_id: Uuid,
    pub transferred_from: String,
    pub transferred_to: String,
    pub date: DateTime<Utc>,
    pub location: String,
    /// EvidenceCondition enum stored as text.
    pub condition: String,
    pub notes: Option<String>,
}

/// A motion filed in a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Motion {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    /// MotionType enum stored as text (e.g. "Suppress", "Dismiss", "Compel").
    pub motion_type: String,
    pub filed_by: String,
    pub description: String,
    pub filed_date: DateTime<Utc>,
    /// MotionStatus enum stored as text (e.g. "Pending", "Granted", "Denied").
    pub status: String,
    pub ruling_date: Option<DateTime<Utc>>,
    pub ruling_text: Option<String>,
}

/// An internal note attached to a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct CaseNote {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub author: String,
    pub content: String,
    pub note_type: String,
    pub is_private: bool,
    pub created_at: DateTime<Utc>,
}

// ── Defendant validation constants ──────────────────────────────────

/// Valid citizenship status values matching the DB CHECK constraint.
pub const CITIZENSHIP_STATUSES: &[&str] = &[
    "Citizen", "Permanent Resident", "Visa Holder", "Undocumented", "Unknown",
];

/// Valid custody status values matching the DB CHECK constraint.
pub const CUSTODY_STATUSES: &[&str] = &[
    "In Custody", "Released", "Bail", "Bond", "Fugitive", "Supervised Release", "Unknown",
];

/// Valid bail type values matching the DB CHECK constraint.
pub const BAIL_TYPES: &[&str] = &[
    "Cash", "Surety", "Property", "Personal Recognizance", "Unsecured", "Denied", "None",
];

/// Check whether a citizenship status string is valid.
pub fn is_valid_citizenship_status(s: &str) -> bool {
    CITIZENSHIP_STATUSES.contains(&s)
}

/// Check whether a custody status string is valid.
pub fn is_valid_custody_status(s: &str) -> bool {
    CUSTODY_STATUSES.contains(&s)
}

/// Check whether a bail type string is valid.
pub fn is_valid_bail_type(s: &str) -> bool {
    BAIL_TYPES.contains(&s)
}

// ── Defendant API response ─────────────────────────────────────────

/// API response shape for a defendant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DefendantResponse {
    pub id: String,
    pub case_id: String,
    pub name: String,
    pub aliases: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usm_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fbi_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>,
    pub citizenship_status: String,
    pub custody_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bail_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bail_amount: Option<f64>,
    pub bond_conditions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bond_posted_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surety_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Defendant> for DefendantResponse {
    fn from(d: Defendant) -> Self {
        Self {
            id: d.id.to_string(),
            case_id: d.case_id.to_string(),
            name: d.name,
            aliases: d.aliases,
            usm_number: d.usm_number,
            fbi_number: d.fbi_number,
            date_of_birth: d.date_of_birth.map(|dt| dt.to_string()),
            citizenship_status: d.citizenship_status,
            custody_status: d.custody_status,
            bail_type: d.bail_type,
            bail_amount: d.bail_amount,
            bond_conditions: d.bond_conditions,
            bond_posted_date: d.bond_posted_date.map(|dt| dt.to_rfc3339()),
            surety_name: d.surety_name,
            created_at: d.created_at.to_rfc3339(),
            updated_at: d.updated_at.to_rfc3339(),
        }
    }
}

// ── Defendant request types ────────────────────────────────────────

/// Request to create a new defendant in a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateDefendantRequest {
    pub case_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub usm_number: Option<String>,
    #[serde(default)]
    pub fbi_number: Option<String>,
    #[serde(default)]
    pub date_of_birth: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub citizenship_status: Option<String>,
    #[serde(default)]
    pub custody_status: Option<String>,
    #[serde(default)]
    pub bail_type: Option<String>,
    #[serde(default)]
    pub bail_amount: Option<f64>,
    #[serde(default)]
    pub bond_conditions: Vec<String>,
    #[serde(default)]
    pub bond_posted_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub surety_name: Option<String>,
}

/// Request to update a defendant (all fields optional — only provided fields change).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateDefendantRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usm_number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fbi_number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<chrono::NaiveDate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citizenship_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custody_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bail_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bail_amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bond_conditions: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bond_posted_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surety_name: Option<String>,
}

// ── Charge validation constants ─────────────────────────────────────

/// Valid plea type values matching the DB CHECK constraint.
pub const PLEA_TYPES: &[&str] = &[
    "Not Guilty", "Guilty", "No Contest", "Alford", "Not Yet Entered",
];

/// Valid verdict type values matching the DB CHECK constraint.
pub const VERDICT_TYPES: &[&str] = &[
    "Guilty", "Not Guilty", "Dismissed", "Mistrial", "Acquitted", "Hung Jury",
];

/// Check whether a plea type string is valid.
pub fn is_valid_plea_type(s: &str) -> bool {
    PLEA_TYPES.contains(&s)
}

/// Check whether a verdict type string is valid.
pub fn is_valid_verdict_type(s: &str) -> bool {
    VERDICT_TYPES.contains(&s)
}

// ── Charge API response ────────────────────────────────────────────

/// API response shape for a criminal charge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChargeResponse {
    pub id: String,
    pub defendant_id: String,
    pub count_number: i32,
    pub statute: String,
    pub offense_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statutory_max_months: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statutory_min_months: Option<i32>,
    pub plea: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plea_date: Option<String>,
    pub verdict: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verdict_date: Option<String>,
}

impl From<Charge> for ChargeResponse {
    fn from(c: Charge) -> Self {
        Self {
            id: c.id.to_string(),
            defendant_id: c.defendant_id.to_string(),
            count_number: c.count_number,
            statute: c.statute,
            offense_description: c.offense_description,
            statutory_max_months: c.statutory_max_months,
            statutory_min_months: c.statutory_min_months,
            plea: c.plea,
            plea_date: c.plea_date.map(|dt| dt.to_rfc3339()),
            verdict: c.verdict,
            verdict_date: c.verdict_date.map(|dt| dt.to_rfc3339()),
        }
    }
}

// ── Charge request types ───────────────────────────────────────────

/// Request to create a new charge against a defendant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateChargeRequest {
    pub defendant_id: Uuid,
    pub count_number: i32,
    pub statute: String,
    pub offense_description: String,
    #[serde(default)]
    pub statutory_max_months: Option<i32>,
    #[serde(default)]
    pub statutory_min_months: Option<i32>,
    #[serde(default)]
    pub plea: Option<String>,
    #[serde(default)]
    pub plea_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub verdict: Option<String>,
    #[serde(default)]
    pub verdict_date: Option<DateTime<Utc>>,
}

/// Request to update a charge (all fields optional — only provided fields change).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateChargeRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count_number: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statute: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offense_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statutory_max_months: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statutory_min_months: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plea: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plea_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verdict: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verdict_date: Option<DateTime<Utc>>,
}

// ── Motion validation constants ─────────────────────────────────────

/// Valid motion type values matching the DB CHECK constraint.
pub const MOTION_TYPES: &[&str] = &[
    "Dismiss", "Suppress", "Compel", "Summary Judgment", "Continuance",
    "Change of Venue", "Reconsideration", "Limine", "Severance", "Joinder",
    "Discovery", "New Trial", "Other",
];

/// Valid motion status values matching the DB CHECK constraint.
pub const MOTION_STATUSES: &[&str] = &[
    "Pending", "Granted", "Denied", "Withdrawn", "Moot", "Deferred", "Partially Granted",
];

/// Check whether a motion type string is valid.
pub fn is_valid_motion_type(s: &str) -> bool {
    MOTION_TYPES.contains(&s)
}

/// Check whether a motion status string is valid.
pub fn is_valid_motion_status(s: &str) -> bool {
    MOTION_STATUSES.contains(&s)
}

// ── Motion API response ────────────────────────────────────────────

/// API response shape for a motion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MotionResponse {
    pub id: String,
    pub case_id: String,
    pub motion_type: String,
    pub filed_by: String,
    pub description: String,
    pub filed_date: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruling_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruling_text: Option<String>,
}

impl From<Motion> for MotionResponse {
    fn from(m: Motion) -> Self {
        Self {
            id: m.id.to_string(),
            case_id: m.case_id.to_string(),
            motion_type: m.motion_type,
            filed_by: m.filed_by,
            description: m.description,
            filed_date: m.filed_date.to_rfc3339(),
            status: m.status,
            ruling_date: m.ruling_date.map(|dt| dt.to_rfc3339()),
            ruling_text: m.ruling_text,
        }
    }
}

// ── Motion request types ───────────────────────────────────────────

/// Request to create a new motion in a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateMotionRequest {
    pub case_id: Uuid,
    pub motion_type: String,
    pub filed_by: String,
    pub description: String,
    #[serde(default)]
    pub filed_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub ruling_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ruling_text: Option<String>,
}

/// Request to update a motion (all fields optional — only provided fields change).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateMotionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filed_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ruling_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ruling_text: Option<String>,
}

// ── Evidence validation constants ───────────────────────────────────

/// Valid evidence type values matching the DB CHECK constraint.
pub const EVIDENCE_TYPES: &[&str] = &[
    "Physical", "Documentary", "Digital", "Testimonial", "Demonstrative", "Forensic", "Other",
];

/// Check whether an evidence type string is valid.
pub fn is_valid_evidence_type(s: &str) -> bool {
    EVIDENCE_TYPES.contains(&s)
}

// ── Evidence API response ──────────────────────────────────────────

/// API response shape for evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EvidenceResponse {
    pub id: String,
    pub case_id: String,
    pub description: String,
    pub evidence_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seized_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seized_by: Option<String>,
    pub location: String,
    pub is_sealed: bool,
    pub created_at: String,
}

impl From<Evidence> for EvidenceResponse {
    fn from(e: Evidence) -> Self {
        Self {
            id: e.id.to_string(),
            case_id: e.case_id.to_string(),
            description: e.description,
            evidence_type: e.evidence_type,
            seized_date: e.seized_date.map(|dt| dt.to_rfc3339()),
            seized_by: e.seized_by,
            location: e.location,
            is_sealed: e.is_sealed,
            created_at: e.created_at.to_rfc3339(),
        }
    }
}

// ── Evidence request types ─────────────────────────────────────────

/// Request to create a new evidence item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateEvidenceRequest {
    pub case_id: Uuid,
    pub description: String,
    pub evidence_type: String,
    #[serde(default)]
    pub seized_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub seized_by: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub is_sealed: Option<bool>,
}

/// Request to update evidence (all fields optional — only provided fields change).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateEvidenceRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seized_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seized_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_sealed: Option<bool>,
}

// ── Custody Transfer API response ──────────────────────────────────

/// API response shape for a custody transfer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CustodyTransferResponse {
    pub id: String,
    pub evidence_id: String,
    pub transferred_from: String,
    pub transferred_to: String,
    pub date: String,
    pub location: String,
    pub condition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl From<CustodyTransfer> for CustodyTransferResponse {
    fn from(ct: CustodyTransfer) -> Self {
        Self {
            id: ct.id.to_string(),
            evidence_id: ct.evidence_id.to_string(),
            transferred_from: ct.transferred_from,
            transferred_to: ct.transferred_to,
            date: ct.date.to_rfc3339(),
            location: ct.location,
            condition: ct.condition,
            notes: ct.notes,
        }
    }
}

// ── Custody Transfer request types ─────────────────────────────────

/// Request to create a new custody transfer record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateCustodyTransferRequest {
    pub evidence_id: Uuid,
    pub transferred_from: String,
    pub transferred_to: String,
    pub date: DateTime<Utc>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

// ── Case Note validation constants ────────────────────────────────

/// Valid note type values matching the DB CHECK constraint.
pub const NOTE_TYPES: &[&str] = &[
    "General",
    "Legal Research",
    "Procedural",
    "Confidential",
    "Bench Note",
    "Clerk Note",
    "Other",
];

/// Check whether a note type string is valid.
pub fn is_valid_note_type(s: &str) -> bool {
    NOTE_TYPES.contains(&s)
}

// ── Case Note response / request types ────────────────────────────

/// API response for a case note.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CaseNoteResponse {
    pub id: String,
    pub case_id: String,
    pub author: String,
    pub content: String,
    pub note_type: String,
    pub is_private: bool,
    pub created_at: String,
}

impl From<CaseNote> for CaseNoteResponse {
    fn from(n: CaseNote) -> Self {
        Self {
            id: n.id.to_string(),
            case_id: n.case_id.to_string(),
            author: n.author,
            content: n.content,
            note_type: n.note_type,
            is_private: n.is_private,
            created_at: n.created_at.to_rfc3339(),
        }
    }
}

/// Request to create a new case note.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateCaseNoteRequest {
    pub case_id: Uuid,
    pub author: String,
    pub content: String,
    #[serde(default = "default_note_type")]
    pub note_type: String,
    #[serde(default)]
    pub is_private: bool,
}

fn default_note_type() -> String {
    "General".to_string()
}

/// Request to update an existing case note.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateCaseNoteRequest {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub note_type: Option<String>,
    #[serde(default)]
    pub is_private: Option<bool>,
}

// ── Case statistics and extras ──────────────────────────────────────

/// Aggregate statistics for cases in a court district.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CaseStatistics {
    pub total: i64,
    pub by_status: serde_json::Value,
    pub by_crime_type: serde_json::Value,
    pub avg_duration_days: Option<f64>,
}

/// A single plea entry used in a batch plea request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PleaEntry {
    pub charge_id: Uuid,
    pub plea: String,
    #[serde(default)]
    pub plea_date: Option<DateTime<Utc>>,
}

/// Request to enter pleas for one or more charges.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PleaRequest {
    pub charges: Vec<PleaEntry>,
}

/// Request to update case priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdatePriorityRequest {
    pub priority: String,
}

/// Request to seal a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SealCaseRequest {
    pub sealed_by: String,
    pub seal_reason: String,
}

/// Request to add a case event for timeline tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddCaseEventRequest {
    pub event_type: String,
    pub description: String,
}
