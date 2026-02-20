use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Judge DB struct ─────────────────────────────────────────────────

/// A judicial officer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Judge {
    pub id: Uuid,
    pub court_id: String,
    pub name: String,
    /// Judge title (e.g. "Chief Judge", "Magistrate Judge").
    pub title: String,
    pub district: String,
    pub appointed_date: Option<DateTime<Utc>>,
    /// Judge status (e.g. "Active", "Senior", "Retired").
    pub status: String,
    pub senior_status_date: Option<DateTime<Utc>>,
    pub courtroom: Option<String>,
    pub current_caseload: i32,
    pub max_caseload: i32,
    pub specializations: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Judge validation constants ──────────────────────────────────────

/// Valid judge title values matching the DB CHECK constraint.
pub const JUDGE_TITLES: &[&str] = &[
    "Chief Judge", "Judge", "Senior Judge", "Magistrate Judge", "Visiting Judge",
];

/// Valid judge status values matching the DB CHECK constraint.
pub const JUDGE_STATUSES: &[&str] = &[
    "Active", "Senior", "Inactive", "Retired", "Deceased",
];

/// Check whether a judge title string is valid.
pub fn is_valid_judge_title(s: &str) -> bool {
    JUDGE_TITLES.contains(&s)
}

/// Check whether a judge status string is valid.
pub fn is_valid_judge_status(s: &str) -> bool {
    JUDGE_STATUSES.contains(&s)
}

// ── Judge API response ──────────────────────────────────────────────

/// API response shape for a judge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JudgeResponse {
    pub id: String,
    pub name: String,
    pub title: String,
    pub district: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appointed_date: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub senior_status_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub courtroom: Option<String>,
    pub current_caseload: i32,
    pub max_caseload: i32,
    pub specializations: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Judge> for JudgeResponse {
    fn from(j: Judge) -> Self {
        Self {
            id: j.id.to_string(),
            name: j.name,
            title: j.title,
            district: j.district,
            appointed_date: j.appointed_date.map(|dt| dt.to_rfc3339()),
            status: j.status,
            senior_status_date: j.senior_status_date.map(|dt| dt.to_rfc3339()),
            courtroom: j.courtroom,
            current_caseload: j.current_caseload,
            max_caseload: j.max_caseload,
            specializations: j.specializations,
            created_at: j.created_at.to_rfc3339(),
            updated_at: j.updated_at.to_rfc3339(),
        }
    }
}

// ── Judge request types ─────────────────────────────────────────────

/// Request to create a new judge.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateJudgeRequest {
    pub name: String,
    pub title: String,
    pub district: String,
    #[serde(default)]
    pub appointed_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub senior_status_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub courtroom: Option<String>,
    #[serde(default)]
    pub max_caseload: Option<i32>,
    #[serde(default)]
    pub specializations: Vec<String>,
}

/// Request to update a judge (all fields optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateJudgeRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub district: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appointed_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub senior_status_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub courtroom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_caseload: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub specializations: Option<Vec<String>>,
}

/// Request to update only a judge's status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateJudgeStatusRequest {
    pub status: String,
}

// ── Conflict constants ──────────────────────────────────────────────

/// Valid conflict type values matching the DB CHECK constraint.
pub const CONFLICT_TYPES: &[&str] = &[
    "Financial", "Familial", "Professional", "Prior Representation", "Organizational", "Other",
];

/// Check whether a conflict type string is valid.
pub fn is_valid_conflict_type(s: &str) -> bool {
    CONFLICT_TYPES.contains(&s)
}

// ── JudgeConflict DB struct ─────────────────────────────────────────

/// A declared conflict of interest for a judge.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct JudgeConflict {
    pub id: Uuid,
    pub court_id: String,
    pub judge_id: Uuid,
    pub party_name: Option<String>,
    pub law_firm: Option<String>,
    pub corporation: Option<String>,
    /// JudgeConflictType enum stored as text.
    pub conflict_type: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// API response shape for a judge conflict.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JudgeConflictResponse {
    pub id: String,
    pub judge_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub law_firm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corporation: Option<String>,
    pub conflict_type: String,
    pub start_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl From<JudgeConflict> for JudgeConflictResponse {
    fn from(c: JudgeConflict) -> Self {
        Self {
            id: c.id.to_string(),
            judge_id: c.judge_id.to_string(),
            party_name: c.party_name,
            law_firm: c.law_firm,
            corporation: c.corporation,
            conflict_type: c.conflict_type,
            start_date: c.start_date.to_rfc3339(),
            end_date: c.end_date.map(|dt| dt.to_rfc3339()),
            notes: c.notes,
        }
    }
}

/// Request to create a new judge conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateJudgeConflictRequest {
    pub conflict_type: String,
    pub start_date: DateTime<Utc>,
    #[serde(default)]
    pub party_name: Option<String>,
    #[serde(default)]
    pub law_firm: Option<String>,
    #[serde(default)]
    pub corporation: Option<String>,
    #[serde(default)]
    pub end_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub notes: Option<String>,
}

// ── Assignment constants ────────────────────────────────────────────

/// Valid assignment type values matching the DB CHECK constraint.
pub const ASSIGNMENT_TYPES: &[&str] = &[
    "Initial", "Reassignment", "Temporary", "Related Case", "Emergency",
];

/// Check whether an assignment type string is valid.
pub fn is_valid_assignment_type(s: &str) -> bool {
    ASSIGNMENT_TYPES.contains(&s)
}

// ── CaseAssignment DB struct ────────────────────────────────────────

/// Assignment of a judge to a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct CaseAssignment {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub judge_id: Uuid,
    /// AssignmentType enum stored as text (e.g. "Initial", "Reassignment", "Temporary").
    pub assignment_type: String,
    pub assigned_date: DateTime<Utc>,
    pub reason: Option<String>,
    pub previous_judge_id: Option<Uuid>,
    pub reassignment_reason: Option<String>,
    /// Resolved judge name from LEFT JOIN judges.
    pub judge_name: Option<String>,
}

/// API response shape for a case assignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CaseAssignmentResponse {
    pub id: String,
    pub case_id: String,
    pub judge_id: String,
    /// Resolved judge name from the judges table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub judge_name: Option<String>,
    pub assignment_type: String,
    pub assigned_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_judge_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reassignment_reason: Option<String>,
}

impl From<CaseAssignment> for CaseAssignmentResponse {
    fn from(a: CaseAssignment) -> Self {
        Self {
            id: a.id.to_string(),
            case_id: a.case_id.to_string(),
            judge_id: a.judge_id.to_string(),
            judge_name: a.judge_name,
            assignment_type: a.assignment_type,
            assigned_date: a.assigned_date.to_rfc3339(),
            reason: a.reason,
            previous_judge_id: a.previous_judge_id.map(|id| id.to_string()),
            reassignment_reason: a.reassignment_reason,
        }
    }
}

/// Request to create a new case assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateCaseAssignmentRequest {
    pub case_id: Uuid,
    pub judge_id: Uuid,
    pub assignment_type: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub previous_judge_id: Option<Uuid>,
    #[serde(default)]
    pub reassignment_reason: Option<String>,
}

// ── Recusal constants ───────────────────────────────────────────────

/// Valid recusal status values matching the DB CHECK constraint.
pub const RECUSAL_STATUSES: &[&str] = &[
    "Pending", "Granted", "Denied", "Withdrawn", "Moot",
];

/// Check whether a recusal status string is valid.
pub fn is_valid_recusal_status(s: &str) -> bool {
    RECUSAL_STATUSES.contains(&s)
}

// ── RecusalMotion DB struct ─────────────────────────────────────────

/// A motion for judicial recusal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct RecusalMotion {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub judge_id: Uuid,
    pub filed_by: String,
    pub filed_date: DateTime<Utc>,
    /// RecusalReason enum stored as text.
    pub reason: String,
    pub detailed_grounds: Option<String>,
    /// RecusalStatus enum stored as text (e.g. "Pending", "Granted", "Denied").
    pub status: String,
    pub ruling_date: Option<DateTime<Utc>>,
    pub ruling_text: Option<String>,
    pub replacement_judge_id: Option<Uuid>,
}

/// API response shape for a recusal motion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RecusalMotionResponse {
    pub id: String,
    pub case_id: String,
    pub judge_id: String,
    pub filed_by: String,
    pub filed_date: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed_grounds: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruling_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruling_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_judge_id: Option<String>,
}

impl From<RecusalMotion> for RecusalMotionResponse {
    fn from(r: RecusalMotion) -> Self {
        Self {
            id: r.id.to_string(),
            case_id: r.case_id.to_string(),
            judge_id: r.judge_id.to_string(),
            filed_by: r.filed_by,
            filed_date: r.filed_date.to_rfc3339(),
            reason: r.reason,
            detailed_grounds: r.detailed_grounds,
            status: r.status,
            ruling_date: r.ruling_date.map(|dt| dt.to_rfc3339()),
            ruling_text: r.ruling_text,
            replacement_judge_id: r.replacement_judge_id.map(|id| id.to_string()),
        }
    }
}

/// Request to create a new recusal motion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateRecusalMotionRequest {
    pub case_id: Uuid,
    pub filed_by: String,
    pub reason: String,
    #[serde(default)]
    pub detailed_grounds: Option<String>,
}

/// Request to update a recusal ruling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateRecusalRulingRequest {
    pub status: String,
    #[serde(default)]
    pub ruling_text: Option<String>,
    #[serde(default)]
    pub replacement_judge_id: Option<Uuid>,
}

// ── Judge workload ─────────────────────────────────────────────────

/// Summary of a judge's current workload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JudgeWorkload {
    pub judge_id: String,
    pub judge_name: String,
    pub active_cases: i64,
    pub pending_motions: i64,
    pub upcoming_hearings: i64,
}

/// Assignment history for a judge — wraps a list of case assignments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AssignmentHistory {
    pub entries: Vec<CaseAssignmentResponse>,
}
