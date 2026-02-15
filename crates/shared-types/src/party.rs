use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Address;

// ---------------------------------------------------------------------------
// Domain Structs
// ---------------------------------------------------------------------------

/// A party to a case (defendant, government, intervenor, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Party {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    /// PartyType enum stored as text (e.g. "Defendant", "Plaintiff", "Intervenor").
    pub party_type: String,
    /// PartyRole enum stored as text.
    pub party_role: String,
    pub name: String,
    /// EntityType enum stored as text (e.g. "Individual", "Corporation", "Government").
    pub entity_type: String,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub organization_name: Option<String>,
    pub address_street1: Option<String>,
    pub address_city: Option<String>,
    pub address_state: Option<String>,
    pub address_zip: Option<String>,
    pub address_country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub represented: bool,
    pub pro_se: bool,
    /// ServiceMethod enum stored as text (e.g. "ECF", "Mail", "PersonalService").
    /// Nullable in the DB — not all parties have a service method set.
    pub service_method: Option<String>,
    /// PartyStatus enum stored as text (e.g. "Active", "Terminated").
    pub status: String,
    /// When the party joined the case. Nullable in DB (defaults not set on all rows).
    pub joined_date: Option<DateTime<Utc>>,
    pub terminated_date: Option<DateTime<Utc>>,
    pub ssn_last_four: Option<String>,
    pub ein: Option<String>,
    pub nef_sms_opt_in: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An attorney's representation of a party in a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Representation {
    pub id: Uuid,
    pub court_id: String,
    pub attorney_id: Uuid,
    pub party_id: Uuid,
    pub case_id: Uuid,
    /// RepresentationType enum stored as text (e.g. "Private", "Court Appointed", "Pro Bono").
    pub representation_type: String,
    /// RepresentationStatus enum stored as text (e.g. "Active", "Withdrawn", "Terminated").
    pub status: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub lead_counsel: bool,
    pub local_counsel: bool,
    pub court_appointed: bool,
    pub limited_appearance: bool,
    pub cja_appointment_id: Option<Uuid>,
    pub scope_of_representation: Option<String>,
    /// WithdrawalReason enum stored as text (when applicable).
    pub withdrawal_reason: Option<String>,
    pub notes: Option<String>,
}

/// Record of service of process or documents on a party.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct ServiceRecord {
    pub id: Uuid,
    pub court_id: String,
    pub document_id: Uuid,
    pub party_id: Uuid,
    pub service_date: DateTime<Utc>,
    /// ServiceMethod enum stored as text.
    pub service_method: String,
    pub served_by: String,
    pub proof_of_service_filed: bool,
    pub successful: bool,
    pub attempts: i32,
    pub notes: Option<String>,
    pub certificate_of_service: Option<String>,
}

// ---------------------------------------------------------------------------
// Service Method Enum
// ---------------------------------------------------------------------------

/// Valid service methods matching the DB CHECK constraint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ServiceMethod {
    Electronic,
    Mail,
    PersonalService,
    Waiver,
    Publication,
    CertifiedMail,
    ExpressMail,
    ECF,
    Other,
}

impl ServiceMethod {
    /// Return the DB-stored string matching the CHECK constraint.
    pub fn as_db_str(&self) -> &str {
        match self {
            Self::Electronic => "Electronic",
            Self::Mail => "Mail",
            Self::PersonalService => "Personal Service",
            Self::Waiver => "Waiver",
            Self::Publication => "Publication",
            Self::CertifiedMail => "Certified Mail",
            Self::ExpressMail => "Express Mail",
            Self::ECF => "ECF",
            Self::Other => "Other",
        }
    }

    /// All valid DB string values (for error messages).
    pub fn all_db_values() -> &'static [&'static str] {
        &[
            "Electronic",
            "Mail",
            "Personal Service",
            "Waiver",
            "Publication",
            "Certified Mail",
            "Express Mail",
            "ECF",
            "Other",
        ]
    }
}

impl TryFrom<&str> for ServiceMethod {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Electronic" => Ok(Self::Electronic),
            "Mail" => Ok(Self::Mail),
            "Personal Service" | "PersonalService" => Ok(Self::PersonalService),
            "Waiver" => Ok(Self::Waiver),
            "Publication" => Ok(Self::Publication),
            "Certified Mail" | "CertifiedMail" => Ok(Self::CertifiedMail),
            "Express Mail" | "ExpressMail" => Ok(Self::ExpressMail),
            "ECF" => Ok(Self::ECF),
            "Other" => Ok(Self::Other),
            _ => Err(format!(
                "Invalid service method '{}'. Valid values: {}",
                value,
                ServiceMethod::all_db_values().join(", ")
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Validation Constants (merged DB + OpenAPI spec)
// ---------------------------------------------------------------------------

pub const VALID_PARTY_TYPES: &[&str] = &[
    "Plaintiff", "Defendant", "Appellant", "Appellee", "Petitioner", "Respondent",
    "Intervenor", "Amicus Curiae", "Third Party", "Government", "Witness",
    "Counter-Claimant", "Cross-Claimant", "Other",
];

pub const VALID_PARTY_ROLES: &[&str] = &[
    "Lead", "Co-Defendant", "Co-Plaintiff", "Cross-Claimant", "Counter-Claimant",
    "Garnishee", "Real Party in Interest", "Principal", "Co-Party", "Representative",
    "Guardian", "Trustee", "Executor", "Administrator", "Next Friend", "Other",
];

pub const VALID_PARTY_STATUSES: &[&str] = &[
    "Active", "Terminated", "Defaulted", "Dismissed", "Settled", "Deceased",
    "Unknown", "In Contempt",
];

pub const VALID_ENTITY_TYPES: &[&str] = &[
    "Individual", "Corporation", "Partnership", "LLC", "Government",
    "Non-Profit", "Trust", "Estate", "Other",
];

pub const VALID_REPRESENTATION_TYPES: &[&str] = &[
    "Private", "Court Appointed", "Pro Bono", "Public Defender", "CJA Panel",
    "Government", "General", "Limited", "Pro Hac Vice", "Standby", "Other",
];

pub const VALID_REPRESENTATION_STATUSES: &[&str] = &[
    "Active", "Withdrawn", "Terminated", "Substituted", "Suspended", "Completed",
];

pub const VALID_WITHDRAWAL_REASONS: &[&str] = &[
    "Client Request", "Conflict of Interest", "Non-Payment",
    "Completed Representation", "Breakdown in Communication",
    "Health Reasons", "Court Order", "Other",
];

// ---------------------------------------------------------------------------
// Party Request/Response DTOs
// ---------------------------------------------------------------------------

/// Request body for creating a new party.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreatePartyRequest {
    pub case_id: String,
    pub party_type: String,
    pub name: String,
    pub entity_type: String,
    pub party_role: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub organization_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<String>,
    pub ssn_last_four: Option<String>,
    pub ein: Option<String>,
    pub address: Option<Address>,
    pub service_method: Option<String>,
    pub pro_se: Option<bool>,
}

/// Request body for updating a party (all fields optional — read-modify-write pattern).
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdatePartyRequest {
    pub party_type: Option<String>,
    pub party_role: Option<String>,
    pub name: Option<String>,
    pub entity_type: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub organization_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<String>,
    pub ssn_last_four: Option<String>,
    pub ein: Option<String>,
    pub address: Option<Address>,
    pub service_method: Option<String>,
    pub status: Option<String>,
    pub pro_se: Option<bool>,
    pub nef_sms_opt_in: Option<bool>,
}

/// Request body for updating party status only.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdatePartyStatusRequest {
    pub status: String,
}

/// API response for a party.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PartyResponse {
    pub id: String,
    pub case_id: String,
    pub court_id: String,
    pub party_type: String,
    pub party_role: String,
    pub name: String,
    pub entity_type: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub organization_name: Option<String>,
    pub represented: bool,
    pub pro_se: bool,
    pub service_method: Option<String>,
    pub status: String,
    pub joined_date: Option<String>,
    pub terminated_date: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<String>,
    pub ssn_last_four: Option<String>,
    pub ein: Option<String>,
    pub nef_sms_opt_in: bool,
    pub address: Option<Address>,
    pub attorneys: Vec<RepresentationResponse>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Party> for PartyResponse {
    fn from(p: Party) -> Self {
        let address = p.address_street1.as_ref().map(|street1| Address {
            street1: street1.clone(),
            street2: None,
            city: p.address_city.clone().unwrap_or_default(),
            state: p.address_state.clone().unwrap_or_default(),
            zip_code: p.address_zip.clone().unwrap_or_default(),
            country: p.address_country.clone().unwrap_or_default(),
        });

        Self {
            id: p.id.to_string(),
            case_id: p.case_id.to_string(),
            court_id: p.court_id,
            party_type: p.party_type,
            party_role: p.party_role,
            name: p.name,
            entity_type: p.entity_type,
            first_name: p.first_name,
            last_name: p.last_name,
            middle_name: p.middle_name,
            organization_name: p.organization_name,
            represented: p.represented,
            pro_se: p.pro_se,
            service_method: p.service_method,
            status: p.status,
            joined_date: p.joined_date.map(|d| d.to_rfc3339()),
            terminated_date: p.terminated_date.map(|d| d.to_rfc3339()),
            email: p.email,
            phone: p.phone,
            date_of_birth: p.date_of_birth.map(|d| d.to_string()),
            ssn_last_four: p.ssn_last_four,
            ein: p.ein,
            nef_sms_opt_in: p.nef_sms_opt_in,
            address,
            // Attorneys are filled in at the REST layer via a second query
            attorneys: Vec::new(),
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

// ---------------------------------------------------------------------------
// Representation Request/Response DTOs
// ---------------------------------------------------------------------------

/// Request body for creating a representation.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateRepresentationRequest {
    pub attorney_id: String,
    pub party_id: String,
    pub case_id: String,
    pub representation_type: Option<String>,
    pub lead_counsel: Option<bool>,
    pub local_counsel: Option<bool>,
    pub limited_appearance: Option<bool>,
    pub court_appointed: Option<bool>,
    pub cja_appointment_id: Option<String>,
    pub scope_of_representation: Option<String>,
    pub notes: Option<String>,
}

/// Request body for ending a representation.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EndRepresentationRequest {
    pub reason: Option<String>,
}

/// Request body for substituting an attorney on a case.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SubstituteAttorneyRequest {
    pub case_id: String,
    pub old_attorney_id: String,
    pub new_attorney_id: String,
}

/// API response for a representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RepresentationResponse {
    pub id: String,
    pub attorney_id: String,
    pub party_id: String,
    pub case_id: String,
    pub court_id: String,
    pub representation_type: String,
    pub status: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub lead_counsel: bool,
    pub local_counsel: bool,
    pub limited_appearance: bool,
    pub court_appointed: bool,
    pub cja_appointment_id: Option<String>,
    pub scope_of_representation: Option<String>,
    pub withdrawal_reason: Option<String>,
    pub notes: Option<String>,
}

impl From<Representation> for RepresentationResponse {
    fn from(r: Representation) -> Self {
        Self {
            id: r.id.to_string(),
            attorney_id: r.attorney_id.to_string(),
            party_id: r.party_id.to_string(),
            case_id: r.case_id.to_string(),
            court_id: r.court_id,
            representation_type: r.representation_type,
            status: r.status,
            start_date: r.start_date.to_rfc3339(),
            end_date: r.end_date.map(|d| d.to_rfc3339()),
            lead_counsel: r.lead_counsel,
            local_counsel: r.local_counsel,
            limited_appearance: r.limited_appearance,
            court_appointed: r.court_appointed,
            cja_appointment_id: r.cja_appointment_id.map(|id| id.to_string()),
            scope_of_representation: r.scope_of_representation,
            withdrawal_reason: r.withdrawal_reason,
            notes: r.notes,
        }
    }
}

// ---------------------------------------------------------------------------
// Service Record DTOs
// ---------------------------------------------------------------------------

/// Request body for creating a new service record.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateServiceRecordRequest {
    pub document_id: String,
    pub party_id: String,
    pub service_method: String,
    pub served_by: String,
    pub service_date: Option<String>,
    pub notes: Option<String>,
    pub certificate_of_service: Option<String>,
}

/// Request body for bulk-creating service records for a document.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BulkCreateServiceRecordRequest {
    pub party_ids: Vec<String>,
    pub service_method: String,
    pub served_by: String,
    pub service_date: Option<String>,
    pub certificate_of_service: Option<String>,
}

/// API response for a service record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ServiceRecordResponse {
    pub id: String,
    pub court_id: String,
    pub document_id: String,
    pub party_id: String,
    pub party_name: String,
    pub party_type: String,
    pub service_date: String,
    pub service_method: String,
    pub served_by: String,
    pub proof_of_service_filed: bool,
    pub successful: bool,
    pub attempts: i32,
    pub notes: Option<String>,
    pub certificate_of_service: Option<String>,
}

impl From<ServiceRecord> for ServiceRecordResponse {
    fn from(r: ServiceRecord) -> Self {
        Self {
            id: r.id.to_string(),
            court_id: r.court_id,
            document_id: r.document_id.to_string(),
            party_id: r.party_id.to_string(),
            party_name: "Unknown".to_string(),
            party_type: "Unknown".to_string(),
            service_date: r.service_date.to_rfc3339(),
            service_method: r.service_method,
            served_by: r.served_by,
            proof_of_service_filed: r.proof_of_service_filed,
            successful: r.successful,
            attempts: r.attempts,
            notes: r.notes,
            certificate_of_service: r.certificate_of_service,
        }
    }
}

/// Query parameters for listing / filtering service records.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct ServiceRecordSearchParams {
    pub document_id: Option<String>,
    pub party_id: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

/// An attorney conflict-of-interest check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct ConflictCheck {
    pub id: Uuid,
    pub court_id: String,
    pub attorney_id: Uuid,
    pub check_date: DateTime<Utc>,
    pub case_id: Option<Uuid>,
    pub party_names: Vec<String>,
    pub adverse_parties: Vec<String>,
    pub cleared: bool,
    pub waiver_obtained: bool,
    pub notes: Option<String>,
}

// ---------------------------------------------------------------------------
// Conflict Check Request/Response DTOs
// ---------------------------------------------------------------------------

/// API response for a conflict check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConflictCheckResponse {
    pub id: String,
    pub attorney_id: String,
    pub check_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    pub party_names: Vec<String>,
    pub adverse_parties: Vec<String>,
    pub cleared: bool,
    pub waiver_obtained: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl From<ConflictCheck> for ConflictCheckResponse {
    fn from(c: ConflictCheck) -> Self {
        Self {
            id: c.id.to_string(),
            attorney_id: c.attorney_id.to_string(),
            check_date: c.check_date.to_rfc3339(),
            case_id: c.case_id.map(|id| id.to_string()),
            party_names: c.party_names,
            adverse_parties: c.adverse_parties,
            cleared: c.cleared,
            waiver_obtained: c.waiver_obtained,
            notes: c.notes,
        }
    }
}

/// Request body for creating a conflict check record.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateConflictCheckRequest {
    pub attorney_id: Uuid,
    #[serde(default)]
    pub case_id: Option<Uuid>,
    pub party_names: Vec<String>,
    #[serde(default)]
    pub adverse_parties: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Request body for running a live conflict check.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RunConflictCheckRequest {
    pub attorney_id: Uuid,
    pub party_names: Vec<String>,
}

/// Result of a live conflict check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RunConflictCheckResult {
    pub has_conflict: bool,
    pub conflicts: Vec<String>,
}

/// Request body for migrating representation from one attorney to another.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MigrateRepresentationRequest {
    pub old_attorney_id: Uuid,
    pub new_attorney_id: Uuid,
    pub case_id: Uuid,
}
