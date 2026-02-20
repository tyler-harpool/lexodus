use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::Address;

/// Attorney row from the database (flattened address columns).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Attorney {
    pub id: Uuid,
    pub court_id: String,
    pub bar_number: String,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub firm_name: Option<String>,
    pub email: String,
    pub phone: String,
    pub fax: Option<String>,
    pub address_street1: String,
    pub address_street2: Option<String>,
    pub address_city: String,
    pub address_state: String,
    pub address_zip: String,
    pub address_country: String,
    pub status: String,
    pub cja_panel_member: bool,
    pub cja_panel_districts: Vec<String>,
    pub languages_spoken: Vec<String>,
    pub cases_handled: i32,
    pub win_rate_percentage: Option<f64>,
    pub avg_case_duration_days: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API response shape for an attorney (nested address).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AttorneyResponse {
    pub id: String,
    pub bar_number: String,
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firm_name: Option<String>,
    pub email: String,
    pub phone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fax: Option<String>,
    pub address: Address,
    pub status: String,
    pub cja_panel_member: bool,
    pub cja_panel_districts: Vec<String>,
    pub languages_spoken: Vec<String>,
    pub cases_handled: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_rate_percentage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_case_duration_days: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Attorney> for AttorneyResponse {
    fn from(a: Attorney) -> Self {
        Self {
            id: a.id.to_string(),
            bar_number: a.bar_number,
            first_name: a.first_name,
            last_name: a.last_name,
            middle_name: a.middle_name,
            firm_name: a.firm_name,
            email: a.email,
            phone: a.phone,
            fax: a.fax,
            address: Address {
                street1: a.address_street1,
                street2: a.address_street2,
                city: a.address_city,
                state: a.address_state,
                zip_code: a.address_zip,
                country: a.address_country,
            },
            status: a.status,
            cja_panel_member: a.cja_panel_member,
            cja_panel_districts: a.cja_panel_districts,
            languages_spoken: a.languages_spoken,
            cases_handled: a.cases_handled,
            win_rate_percentage: a.win_rate_percentage,
            avg_case_duration_days: a.avg_case_duration_days,
            created_at: a.created_at.to_rfc3339(),
            updated_at: a.updated_at.to_rfc3339(),
        }
    }
}

/// Attorney status values matching the DB CHECK constraint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum AttorneyStatus {
    Active,
    Inactive,
    Suspended,
    Disbarred,
    Retired,
    Deceased,
}

impl AttorneyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Inactive => "Inactive",
            Self::Suspended => "Suspended",
            Self::Disbarred => "Disbarred",
            Self::Retired => "Retired",
            Self::Deceased => "Deceased",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "Active" => Some(Self::Active),
            "Inactive" => Some(Self::Inactive),
            "Suspended" => Some(Self::Suspended),
            "Disbarred" => Some(Self::Disbarred),
            "Retired" => Some(Self::Retired),
            "Deceased" => Some(Self::Deceased),
            _ => None,
        }
    }
}

/// Request to create a new attorney.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateAttorneyRequest {
    pub bar_number: String,
    pub first_name: String,
    pub last_name: String,
    #[serde(default)]
    pub middle_name: Option<String>,
    #[serde(default)]
    pub firm_name: Option<String>,
    pub email: String,
    pub phone: String,
    #[serde(default)]
    pub fax: Option<String>,
    pub address: Address,
}

/// Request to update an existing attorney (all fields optional for partial update).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(default)]
pub struct UpdateAttorneyRequest {
    pub bar_number: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub firm_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub fax: Option<String>,
    pub address: Option<Address>,
    pub status: Option<String>,
    pub cja_panel_member: Option<bool>,
    pub cja_panel_districts: Option<Vec<String>>,
    pub languages_spoken: Option<Vec<String>>,
    pub cases_handled: Option<i32>,
    pub win_rate_percentage: Option<f64>,
    pub avg_case_duration_days: Option<i32>,
}

/// Request for bulk status update.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BulkUpdateStatusRequest {
    pub attorney_ids: Vec<Uuid>,
    pub status: String,
}

/// Query parameters for attorney search.
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct AttorneySearchParams {
    pub q: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

/// Query parameters for attorney listing.
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct AttorneyListParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

// ── Attorney Sub-Resource Types (PR 13-14) ──────────────────────────

/// Valid bar admission statuses.
pub const BAR_ADMISSION_STATUSES: &[&str] = &["Active", "Inactive", "Suspended", "Revoked", "Retired"];

/// Valid discipline action types.
pub const DISCIPLINE_ACTION_TYPES: &[&str] = &[
    "Reprimand", "Censure", "Suspension", "Disbarment", "Probation", "Fine", "Other",
];

/// Valid federal admission statuses.
pub const FEDERAL_ADMISSION_STATUSES: &[&str] = &["Active", "Inactive", "Suspended", "Revoked", "Retired"];

/// Valid pro hac vice statuses.
pub const PRO_HAC_VICE_STATUSES: &[&str] = &["Active", "Expired", "Revoked", "Pending"];

/// Valid CJA voucher statuses.
pub const CJA_VOUCHER_STATUSES: &[&str] = &["Pending", "Submitted", "Approved", "Denied", "Paid"];

/// Valid ECF registration statuses.
pub const ECF_REGISTRATION_STATUSES: &[&str] = &["Active", "Suspended", "Revoked", "Pending"];

// ── Bar Admission ───────────────────────────────────────────────────

/// Bar admission row from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct BarAdmission {
    pub id: Uuid,
    pub court_id: String,
    pub attorney_id: Uuid,
    pub state: String,
    pub bar_number: String,
    pub admission_date: DateTime<Utc>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// API response shape for a bar admission.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BarAdmissionResponse {
    pub id: String,
    pub attorney_id: String,
    pub state: String,
    pub bar_number: String,
    pub admission_date: String,
    pub status: String,
    pub created_at: String,
}

impl From<BarAdmission> for BarAdmissionResponse {
    fn from(b: BarAdmission) -> Self {
        Self {
            id: b.id.to_string(),
            attorney_id: b.attorney_id.to_string(),
            state: b.state,
            bar_number: b.bar_number,
            admission_date: b.admission_date.to_rfc3339(),
            status: b.status,
            created_at: b.created_at.to_rfc3339(),
        }
    }
}

/// Request to create a new bar admission.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateBarAdmissionRequest {
    pub state: String,
    pub bar_number: String,
    #[serde(default)]
    pub admission_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub status: Option<String>,
}

// ── Federal Admission ───────────────────────────────────────────────

/// Federal admission row from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct FederalAdmission {
    pub id: Uuid,
    pub court_id: String,
    pub attorney_id: Uuid,
    pub court_name: String,
    pub admission_date: DateTime<Utc>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// API response shape for a federal admission.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FederalAdmissionResponse {
    pub id: String,
    pub attorney_id: String,
    pub court_name: String,
    pub admission_date: String,
    pub status: String,
    pub created_at: String,
}

impl From<FederalAdmission> for FederalAdmissionResponse {
    fn from(f: FederalAdmission) -> Self {
        Self {
            id: f.id.to_string(),
            attorney_id: f.attorney_id.to_string(),
            court_name: f.court_name,
            admission_date: f.admission_date.to_rfc3339(),
            status: f.status,
            created_at: f.created_at.to_rfc3339(),
        }
    }
}

/// Request to create a new federal admission.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateFederalAdmissionRequest {
    pub court_name: String,
    #[serde(default)]
    pub admission_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub status: Option<String>,
}

// ── Discipline Record ───────────────────────────────────────────────

/// Discipline history row from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct DisciplineRecord {
    pub id: Uuid,
    pub court_id: String,
    pub attorney_id: Uuid,
    pub action_type: String,
    pub jurisdiction: String,
    pub description: String,
    pub action_date: DateTime<Utc>,
    pub effective_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// API response shape for a discipline record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DisciplineRecordResponse {
    pub id: String,
    pub attorney_id: String,
    pub action_type: String,
    pub jurisdiction: String,
    pub description: String,
    pub action_date: String,
    pub effective_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    pub created_at: String,
}

impl From<DisciplineRecord> for DisciplineRecordResponse {
    fn from(d: DisciplineRecord) -> Self {
        Self {
            id: d.id.to_string(),
            attorney_id: d.attorney_id.to_string(),
            action_type: d.action_type,
            jurisdiction: d.jurisdiction,
            description: d.description,
            action_date: d.action_date.to_rfc3339(),
            effective_date: d.effective_date.to_rfc3339(),
            end_date: d.end_date.map(|dt| dt.to_rfc3339()),
            created_at: d.created_at.to_rfc3339(),
        }
    }
}

/// Request to create a new discipline record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateDisciplineRecordRequest {
    pub action_type: String,
    pub jurisdiction: String,
    pub description: String,
    #[serde(default)]
    pub action_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub effective_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub end_date: Option<DateTime<Utc>>,
}

// ── Pro Hac Vice ────────────────────────────────────────────────────

/// Pro hac vice admission row from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct ProHacVice {
    pub id: Uuid,
    pub court_id: String,
    pub attorney_id: Uuid,
    pub case_id: Uuid,
    pub sponsoring_attorney_id: Option<Uuid>,
    pub admission_date: DateTime<Utc>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// API response shape for a pro hac vice admission.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ProHacViceResponse {
    pub id: String,
    pub attorney_id: String,
    pub case_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsoring_attorney_id: Option<String>,
    pub admission_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<String>,
    pub status: String,
    pub created_at: String,
}

impl From<ProHacVice> for ProHacViceResponse {
    fn from(p: ProHacVice) -> Self {
        Self {
            id: p.id.to_string(),
            attorney_id: p.attorney_id.to_string(),
            case_id: p.case_id.to_string(),
            sponsoring_attorney_id: p.sponsoring_attorney_id.map(|u| u.to_string()),
            admission_date: p.admission_date.to_rfc3339(),
            expiration_date: p.expiration_date.map(|dt| dt.to_rfc3339()),
            status: p.status,
            created_at: p.created_at.to_rfc3339(),
        }
    }
}

/// Request to create a new pro hac vice admission.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateProHacViceRequest {
    pub case_id: Uuid,
    #[serde(default)]
    pub sponsoring_attorney_id: Option<Uuid>,
    #[serde(default)]
    pub expiration_date: Option<DateTime<Utc>>,
}

/// Request to update the status of a pro hac vice admission.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdatePhvStatusRequest {
    pub status: String,
}

// ── CJA Appointment ─────────────────────────────────────────────────

/// CJA appointment row from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct CjaAppointment {
    pub id: Uuid,
    pub court_id: String,
    pub attorney_id: Uuid,
    pub case_id: Option<Uuid>,
    pub appointment_date: DateTime<Utc>,
    pub termination_date: Option<DateTime<Utc>>,
    pub voucher_status: String,
    pub voucher_amount: Option<f64>,
    pub created_at: DateTime<Utc>,
}

/// API response shape for a CJA appointment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CjaAppointmentResponse {
    pub id: String,
    pub attorney_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    pub appointment_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termination_date: Option<String>,
    pub voucher_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voucher_amount: Option<f64>,
    pub created_at: String,
}

impl From<CjaAppointment> for CjaAppointmentResponse {
    fn from(c: CjaAppointment) -> Self {
        Self {
            id: c.id.to_string(),
            attorney_id: c.attorney_id.to_string(),
            case_id: c.case_id.map(|u| u.to_string()),
            appointment_date: c.appointment_date.to_rfc3339(),
            termination_date: c.termination_date.map(|dt| dt.to_rfc3339()),
            voucher_status: c.voucher_status,
            voucher_amount: c.voucher_amount,
            created_at: c.created_at.to_rfc3339(),
        }
    }
}

/// Request to create a new CJA appointment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateCjaAppointmentRequest {
    #[serde(default)]
    pub case_id: Option<Uuid>,
    #[serde(default)]
    pub voucher_amount: Option<f64>,
}

// ── ECF Registration ────────────────────────────────────────────────

/// ECF registration row from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct EcfRegistration {
    pub id: Uuid,
    pub court_id: String,
    pub attorney_id: Uuid,
    pub registration_date: DateTime<Utc>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// API response shape for an ECF registration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EcfRegistrationResponse {
    pub id: String,
    pub attorney_id: String,
    pub registration_date: String,
    pub status: String,
    pub created_at: String,
}

impl From<EcfRegistration> for EcfRegistrationResponse {
    fn from(e: EcfRegistration) -> Self {
        Self {
            id: e.id.to_string(),
            attorney_id: e.attorney_id.to_string(),
            registration_date: e.registration_date.to_rfc3339(),
            status: e.status,
            created_at: e.created_at.to_rfc3339(),
        }
    }
}

/// Request to create or update an ECF registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpsertEcfRegistrationRequest {
    #[serde(default)]
    pub status: Option<String>,
}

// ── Analytics DTOs ──────────────────────────────────────────────────

/// High-level metrics for an attorney.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AttorneyMetrics {
    pub attorney_id: String,
    pub total_cases: i64,
    pub active_cases: i64,
    pub win_rate: Option<f64>,
    pub avg_case_duration_days: Option<f64>,
}

/// Case load breakdown for an attorney.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AttorneyCaseLoad {
    pub attorney_id: String,
    pub total_active: i64,
    pub by_status: serde_json::Value,
}

/// Result of a good-standing check on an attorney.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GoodStandingResult {
    pub attorney_id: String,
    pub in_good_standing: bool,
    pub reasons: Vec<String>,
}

/// Result of checking if an attorney can practice in a given court.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CanPracticeResult {
    pub attorney_id: String,
    pub court: String,
    pub can_practice: bool,
    pub reasons: Vec<String>,
}

/// Win rate calculation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct WinRateResult {
    pub attorney_id: String,
    pub total_cases: i64,
    pub wins: i64,
    pub losses: i64,
    pub win_rate: f64,
}

/// Request to run a conflict check for an attorney.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConflictCheckRequest {
    pub case_id: Option<Uuid>,
    pub party_names: Vec<String>,
}

/// Result of a conflict check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConflictCheckResult {
    pub has_conflict: bool,
    pub conflicts: Vec<String>,
}

/// Request to add an attorney to a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AttorneyAddToCaseRequest {
    pub case_id: Uuid,
    pub role: Option<String>,
}

// ── Practice Areas ─────────────────────────────────────────────────

/// Practice area row from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct PracticeArea {
    pub id: Uuid,
    pub court_id: String,
    pub attorney_id: Uuid,
    pub area: String,
    pub created_at: DateTime<Utc>,
}

/// API response shape for a practice area.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PracticeAreaResponse {
    pub id: String,
    pub attorney_id: String,
    pub area: String,
    pub created_at: String,
}

impl From<PracticeArea> for PracticeAreaResponse {
    fn from(p: PracticeArea) -> Self {
        Self {
            id: p.id.to_string(),
            attorney_id: p.attorney_id.to_string(),
            area: p.area,
            created_at: p.created_at.to_rfc3339(),
        }
    }
}

/// Request to add a practice area.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddPracticeAreaRequest {
    pub area: String,
}
