use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Sentencing validation constants ─────────────────────────────────

/// Valid criminal history category values matching the DB CHECK constraint.
pub const CRIMINAL_HISTORY_CATEGORIES: &[&str] = &["I", "II", "III", "IV", "V", "VI"];

/// Valid departure type values matching the DB CHECK constraint.
pub const DEPARTURE_TYPES: &[&str] = &["Upward", "Downward", "None"];

/// Valid variance type values matching the DB CHECK constraint.
pub const VARIANCE_TYPES: &[&str] = &["Upward", "Downward", "None"];

/// Check whether a criminal history category string is valid.
pub fn is_valid_criminal_history_category(s: &str) -> bool {
    CRIMINAL_HISTORY_CATEGORIES.contains(&s)
}

/// Check whether a departure type string is valid.
pub fn is_valid_departure_type(s: &str) -> bool {
    DEPARTURE_TYPES.contains(&s)
}

/// Check whether a variance type string is valid.
pub fn is_valid_variance_type(s: &str) -> bool {
    VARIANCE_TYPES.contains(&s)
}

// ── SentencingRecord DB struct ──────────────────────────────────────

/// Full sentencing record including guidelines calculations and imposed sentence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct SentencingRecord {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub defendant_id: Uuid,
    pub judge_id: Uuid,
    pub base_offense_level: Option<i32>,
    pub specific_offense_level: Option<i32>,
    pub adjusted_offense_level: Option<i32>,
    pub total_offense_level: Option<i32>,
    pub criminal_history_category: Option<String>,
    pub criminal_history_points: Option<i32>,
    pub guidelines_range_low_months: Option<i32>,
    pub guidelines_range_high_months: Option<i32>,
    pub custody_months: Option<i32>,
    pub probation_months: Option<i32>,
    pub fine_amount: Option<f64>,
    pub restitution_amount: Option<f64>,
    pub forfeiture_amount: Option<f64>,
    pub special_assessment: Option<f64>,
    /// DepartureType enum stored as text (e.g. "Upward", "Downward").
    pub departure_type: Option<String>,
    pub departure_reason: Option<String>,
    /// VarianceType enum stored as text.
    pub variance_type: Option<String>,
    pub variance_justification: Option<String>,
    pub supervised_release_months: Option<i32>,
    pub appeal_waiver: bool,
    pub sentencing_date: Option<DateTime<Utc>>,
    pub judgment_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── SentencingRecord API response ───────────────────────────────────

/// API response shape for a sentencing record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SentencingResponse {
    pub id: String,
    pub case_id: String,
    pub defendant_id: String,
    pub judge_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_offense_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specific_offense_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjusted_offense_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_offense_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criminal_history_category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criminal_history_points: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidelines_range_low_months: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidelines_range_high_months: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custody_months: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probation_months: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fine_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restitution_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forfeiture_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_assessment: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub departure_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub departure_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variance_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variance_justification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supervised_release_months: Option<i32>,
    pub appeal_waiver: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentencing_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub judgment_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<SentencingRecord> for SentencingResponse {
    fn from(s: SentencingRecord) -> Self {
        Self {
            id: s.id.to_string(),
            case_id: s.case_id.to_string(),
            defendant_id: s.defendant_id.to_string(),
            judge_id: s.judge_id.to_string(),
            base_offense_level: s.base_offense_level,
            specific_offense_level: s.specific_offense_level,
            adjusted_offense_level: s.adjusted_offense_level,
            total_offense_level: s.total_offense_level,
            criminal_history_category: s.criminal_history_category,
            criminal_history_points: s.criminal_history_points,
            guidelines_range_low_months: s.guidelines_range_low_months,
            guidelines_range_high_months: s.guidelines_range_high_months,
            custody_months: s.custody_months,
            probation_months: s.probation_months,
            fine_amount: s.fine_amount,
            restitution_amount: s.restitution_amount,
            forfeiture_amount: s.forfeiture_amount,
            special_assessment: s.special_assessment,
            departure_type: s.departure_type,
            departure_reason: s.departure_reason,
            variance_type: s.variance_type,
            variance_justification: s.variance_justification,
            supervised_release_months: s.supervised_release_months,
            appeal_waiver: s.appeal_waiver,
            sentencing_date: s.sentencing_date.map(|dt| dt.to_rfc3339()),
            judgment_date: s.judgment_date.map(|dt| dt.to_rfc3339()),
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
        }
    }
}

// ── Sentencing request types ────────────────────────────────────────

/// Request to create a new sentencing record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateSentencingRequest {
    pub case_id: Uuid,
    pub defendant_id: Uuid,
    pub judge_id: Uuid,
    #[serde(default)]
    pub base_offense_level: Option<i32>,
    #[serde(default)]
    pub specific_offense_level: Option<i32>,
    #[serde(default)]
    pub adjusted_offense_level: Option<i32>,
    #[serde(default)]
    pub total_offense_level: Option<i32>,
    #[serde(default)]
    pub criminal_history_category: Option<String>,
    #[serde(default)]
    pub criminal_history_points: Option<i32>,
    #[serde(default)]
    pub guidelines_range_low_months: Option<i32>,
    #[serde(default)]
    pub guidelines_range_high_months: Option<i32>,
    #[serde(default)]
    pub custody_months: Option<i32>,
    #[serde(default)]
    pub probation_months: Option<i32>,
    #[serde(default)]
    pub fine_amount: Option<f64>,
    #[serde(default)]
    pub restitution_amount: Option<f64>,
    #[serde(default)]
    pub forfeiture_amount: Option<f64>,
    #[serde(default)]
    pub special_assessment: Option<f64>,
    #[serde(default)]
    pub departure_type: Option<String>,
    #[serde(default)]
    pub departure_reason: Option<String>,
    #[serde(default)]
    pub variance_type: Option<String>,
    #[serde(default)]
    pub variance_justification: Option<String>,
    #[serde(default)]
    pub supervised_release_months: Option<i32>,
    #[serde(default)]
    pub appeal_waiver: Option<bool>,
    #[serde(default)]
    pub sentencing_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub judgment_date: Option<DateTime<Utc>>,
}

// ── BOP / special-condition / prior-sentence constants ──────────────

/// Valid BOP security-level values.
pub const BOP_SECURITY_LEVELS: &[&str] = &[
    "Minimum", "Low", "Medium", "High", "Administrative", "Unassigned",
];

/// Valid special-condition status values.
pub const SPECIAL_CONDITION_STATUSES: &[&str] = &[
    "Active", "Modified", "Terminated", "Expired",
];

// ── SentencingSpecialCondition DB struct ────────────────────────────

/// A special condition attached to a sentencing record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct SentencingSpecialCondition {
    pub id: Uuid,
    pub court_id: String,
    pub sentencing_id: Uuid,
    pub condition_type: String,
    pub description: String,
    pub effective_date: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// API response for a sentencing special condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SpecialConditionResponse {
    pub id: String,
    pub sentencing_id: String,
    pub condition_type: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_date: Option<String>,
    pub status: String,
    pub created_at: String,
}

impl From<SentencingSpecialCondition> for SpecialConditionResponse {
    fn from(c: SentencingSpecialCondition) -> Self {
        Self {
            id: c.id.to_string(),
            sentencing_id: c.sentencing_id.to_string(),
            condition_type: c.condition_type,
            description: c.description,
            effective_date: c.effective_date.map(|dt| dt.to_rfc3339()),
            status: c.status,
            created_at: c.created_at.to_rfc3339(),
        }
    }
}

/// Request to create a new special condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateSpecialConditionRequest {
    pub condition_type: String,
    pub description: String,
    #[serde(default)]
    pub effective_date: Option<DateTime<Utc>>,
}

// ── BopDesignation DB struct ───────────────────────────────────────

/// A Bureau of Prisons (BOP) designation for a defendant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct BopDesignation {
    pub id: Uuid,
    pub court_id: String,
    pub sentencing_id: Uuid,
    pub defendant_id: Uuid,
    pub facility: String,
    pub security_level: String,
    pub designation_date: DateTime<Utc>,
    pub designation_reason: Option<String>,
    pub rdap_eligible: bool,
    pub rdap_enrolled: bool,
    pub created_at: DateTime<Utc>,
}

/// API response for a BOP designation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BopDesignationResponse {
    pub id: String,
    pub sentencing_id: String,
    pub defendant_id: String,
    pub facility: String,
    pub security_level: String,
    pub designation_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub designation_reason: Option<String>,
    pub rdap_eligible: bool,
    pub rdap_enrolled: bool,
    pub created_at: String,
}

impl From<BopDesignation> for BopDesignationResponse {
    fn from(b: BopDesignation) -> Self {
        Self {
            id: b.id.to_string(),
            sentencing_id: b.sentencing_id.to_string(),
            defendant_id: b.defendant_id.to_string(),
            facility: b.facility,
            security_level: b.security_level,
            designation_date: b.designation_date.to_rfc3339(),
            designation_reason: b.designation_reason,
            rdap_eligible: b.rdap_eligible,
            rdap_enrolled: b.rdap_enrolled,
            created_at: b.created_at.to_rfc3339(),
        }
    }
}

/// Request to create a new BOP designation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateBopDesignationRequest {
    pub defendant_id: Uuid,
    pub facility: String,
    pub security_level: String,
    #[serde(default)]
    pub designation_reason: Option<String>,
    #[serde(default)]
    pub rdap_eligible: Option<bool>,
}

// ── PriorSentence DB struct ────────────────────────────────────────

/// A prior sentence record used for criminal-history calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct PriorSentence {
    pub id: Uuid,
    pub court_id: String,
    pub sentencing_id: Uuid,
    pub defendant_id: Uuid,
    pub prior_case_number: Option<String>,
    pub jurisdiction: String,
    pub offense: String,
    pub conviction_date: DateTime<Utc>,
    pub sentence_length_months: Option<i32>,
    pub points_assigned: i32,
    pub created_at: DateTime<Utc>,
}

/// API response for a prior sentence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PriorSentenceResponse {
    pub id: String,
    pub sentencing_id: String,
    pub defendant_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prior_case_number: Option<String>,
    pub jurisdiction: String,
    pub offense: String,
    pub conviction_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentence_length_months: Option<i32>,
    pub points_assigned: i32,
    pub created_at: String,
}

impl From<PriorSentence> for PriorSentenceResponse {
    fn from(p: PriorSentence) -> Self {
        Self {
            id: p.id.to_string(),
            sentencing_id: p.sentencing_id.to_string(),
            defendant_id: p.defendant_id.to_string(),
            prior_case_number: p.prior_case_number,
            jurisdiction: p.jurisdiction,
            offense: p.offense,
            conviction_date: p.conviction_date.to_rfc3339(),
            sentence_length_months: p.sentence_length_months,
            points_assigned: p.points_assigned,
            created_at: p.created_at.to_rfc3339(),
        }
    }
}

/// Request to create a prior sentence record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreatePriorSentenceRequest {
    pub defendant_id: Uuid,
    pub jurisdiction: String,
    pub offense: String,
    pub conviction_date: DateTime<Utc>,
    #[serde(default)]
    pub prior_case_number: Option<String>,
    #[serde(default)]
    pub sentence_length_months: Option<i32>,
    #[serde(default)]
    pub points_assigned: Option<i32>,
}

// ── Sentencing workflow request / response types ───────────────────

/// Request to record a departure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DepartureRequest {
    pub departure_type: String,
    pub departure_reason: String,
}

/// Request to record a variance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VarianceRequest {
    pub variance_type: String,
    pub variance_justification: String,
}

/// Request to update supervised-release months.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SupervisedReleaseRequest {
    pub supervised_release_months: i32,
}

/// Request to calculate sentencing guidelines.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GuidelinesRequest {
    pub base_offense_level: i32,
    pub specific_offense_adjustments: Vec<i32>,
    #[serde(default)]
    pub acceptance_reduction: Option<i32>,
    pub criminal_history_category: String,
}

/// Result of a guidelines calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GuidelinesResult {
    pub total_offense_level: i32,
    pub criminal_history_category: String,
    pub range_low_months: i32,
    pub range_high_months: i32,
}

/// Request to calculate total offense level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OffenseLevelRequest {
    pub base_level: i32,
    pub adjustments: Vec<serde_json::Value>,
}

/// Sentencing statistics for a court.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SentencingStatistics {
    pub total: i64,
    pub avg_custody_months: Option<f64>,
    pub departure_rate: f64,
    pub variance_rate: f64,
}

/// Query parameters for date-range searches.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct DateRangeParams {
    #[serde(default)]
    pub from: Option<DateTime<Utc>>,
    #[serde(default)]
    pub to: Option<DateTime<Utc>>,
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

/// Request to update a sentencing record (all fields optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateSentencingRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_offense_level: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub specific_offense_level: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjusted_offense_level: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_offense_level: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criminal_history_category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criminal_history_points: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guidelines_range_low_months: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guidelines_range_high_months: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custody_months: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probation_months: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fine_amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restitution_amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forfeiture_amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub special_assessment: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub departure_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub departure_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variance_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variance_justification: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supervised_release_months: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appeal_waiver: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentencing_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judgment_date: Option<DateTime<Utc>>,
}
