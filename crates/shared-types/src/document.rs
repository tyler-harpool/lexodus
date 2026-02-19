use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Sealing level
// ---------------------------------------------------------------------------

/// Document sealing level — controls visibility beyond the boolean `is_sealed`.
///
/// - `Public` — visible to all with case access.
/// - `SealedCourtOnly` — only court staff (clerks + judges) can view.
/// - `SealedCaseParticipants` — court staff + case attorneys.
/// - `SealedAttorneysOnly` — only case attorneys (not public, not other staff).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum SealingLevel {
    #[default]
    Public,
    SealedCourtOnly,
    SealedCaseParticipants,
    SealedAttorneysOnly,
}

impl SealingLevel {
    /// Database-compatible text representation (matches CHECK constraint).
    pub fn as_db_str(&self) -> &'static str {
        match self {
            SealingLevel::Public => "Public",
            SealingLevel::SealedCourtOnly => "SealedCourtOnly",
            SealingLevel::SealedCaseParticipants => "SealedCaseParticipants",
            SealingLevel::SealedAttorneysOnly => "SealedAttorneysOnly",
        }
    }

    /// Parse from database TEXT column. Unknown values default to Public.
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "SealedCourtOnly" => SealingLevel::SealedCourtOnly,
            "SealedCaseParticipants" => SealingLevel::SealedCaseParticipants,
            "SealedAttorneysOnly" => SealingLevel::SealedAttorneysOnly,
            _ => SealingLevel::Public,
        }
    }

    /// Returns true for any non-public sealing level.
    pub fn is_sealed(&self) -> bool {
        !matches!(self, SealingLevel::Public)
    }
}

// ---------------------------------------------------------------------------
// Document
// ---------------------------------------------------------------------------

/// A document stored in the system (metadata only, no blob).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Document {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub title: String,
    /// DocumentType enum stored as text (e.g. "Pleading", "Order", "Exhibit").
    pub document_type: String,
    pub storage_key: String,
    pub checksum: String,
    pub file_size: i64,
    pub content_type: String,
    pub is_sealed: bool,
    pub uploaded_by: String,
    /// The docket attachment this document was promoted from, if any.
    pub source_attachment_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    /// Granular sealing level (application-level policy beyond boolean is_sealed).
    pub sealing_level: String,
    /// Reason code for sealing (e.g. "JuvenileRecord", "TradeSecret").
    pub seal_reason_code: Option<String>,
    /// Optional motion that authorized the sealing.
    pub seal_motion_id: Option<Uuid>,
    /// If this document has been replaced, points to the replacement document.
    pub replaced_by_document_id: Option<Uuid>,
    /// Whether this document has been stricken from the record.
    pub is_stricken: bool,
}

/// An electronic filing submitted to the court.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Filing {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub filing_type: String,
    pub filed_by: String,
    pub filed_date: DateTime<Utc>,
    /// FilingStatus enum stored as text (e.g. "Pending", "Accepted", "Rejected").
    pub status: String,
    pub validation_errors: serde_json::Value,
    pub document_id: Option<Uuid>,
    pub docket_entry_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Lightweight response shape for filing list views (avoids raw JSON fields).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FilingListItem {
    pub id: String,
    pub court_id: String,
    pub case_id: String,
    pub filing_type: String,
    pub filed_by: String,
    pub filed_date: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docket_entry_id: Option<String>,
    pub created_at: String,
}

impl From<Filing> for FilingListItem {
    fn from(f: Filing) -> Self {
        Self {
            id: f.id.to_string(),
            court_id: f.court_id,
            case_id: f.case_id.to_string(),
            filing_type: f.filing_type,
            filed_by: f.filed_by,
            filed_date: f.filed_date.to_rfc3339(),
            status: f.status,
            document_id: f.document_id.map(|u| u.to_string()),
            docket_entry_id: f.docket_entry_id.map(|u| u.to_string()),
            created_at: f.created_at.to_rfc3339(),
        }
    }
}

/// Valid filing types matching the DB CHECK constraint.
pub const VALID_FILING_TYPES: &[&str] = &[
    "Initial", "Response", "Reply", "Motion", "Notice",
    "Stipulation", "Supplement", "Amendment", "Exhibit",
    "Certificate", "Other",
];

/// Valid filing statuses matching the DB CHECK constraint.
pub const FILING_STATUSES: &[&str] = &[
    "Pending", "Accepted", "Rejected", "Under Review", "Returned", "Filed",
];

// ---------------------------------------------------------------------------
// Document validation constants
// ---------------------------------------------------------------------------

/// Valid document types matching the DB CHECK constraint.
pub const VALID_DOCUMENT_TYPES: &[&str] = &[
    "Motion", "Order", "Brief", "Memorandum", "Declaration", "Affidavit",
    "Exhibit", "Transcript", "Notice", "Subpoena", "Warrant", "Indictment",
    "Plea Agreement", "Judgment", "Verdict", "Other",
];

// ---------------------------------------------------------------------------
// Filing API types
// ---------------------------------------------------------------------------

/// Request to validate a filing before submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ValidateFilingRequest {
    pub case_id: String,
    pub document_type: String,
    pub title: String,
    pub filed_by: String,
    /// Staged upload ID from POST /api/filings/upload/init + finalize flow.
    #[serde(default)]
    pub upload_id: Option<String>,
    #[serde(default)]
    pub is_sealed: Option<bool>,
    /// Granular sealing level. Defaults to "Public" if omitted.
    #[serde(default)]
    pub sealing_level: Option<String>,
    /// Reason code for sealing (e.g. "JuvenileRecord", "TradeSecret").
    #[serde(default)]
    pub reason_code: Option<String>,
}

/// A single validation error or warning for a filing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FilingValidationError {
    pub field: String,
    pub message: String,
    /// "error" or "warning"
    pub severity: String,
}

/// Response from filing validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ValidateFilingResponse {
    pub valid: bool,
    pub errors: Vec<FilingValidationError>,
    pub warnings: Vec<FilingValidationError>,
}

/// Request to submit an electronic filing (same shape as validate).
pub type SubmitFilingRequest = ValidateFilingRequest;

/// Response from a successful filing submission.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FilingResponse {
    pub filing_id: String,
    pub document_id: String,
    pub docket_entry_id: String,
    pub case_id: String,
    pub status: String,
    pub filed_date: String,
    pub nef: NefSummary,
}

/// Notice of Electronic Filing summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NefSummary {
    pub case_number: String,
    pub document_title: String,
    pub filed_by: String,
    pub filed_date: String,
    pub docket_number: i32,
}

/// Information about a court jurisdiction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JurisdictionInfo {
    pub court_id: String,
    pub name: String,
    pub court_type: String,
}

// ---------------------------------------------------------------------------
// Notice of Electronic Filing (NEF)
// ---------------------------------------------------------------------------

/// A persisted Notice of Electronic Filing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Nef {
    pub id: Uuid,
    pub court_id: String,
    pub filing_id: Uuid,
    pub document_id: Uuid,
    pub case_id: Uuid,
    pub docket_entry_id: Uuid,
    pub recipients: serde_json::Value,
    pub html_snapshot: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// API response shape for a NEF.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NefResponse {
    pub id: String,
    pub filing_id: String,
    pub document_id: String,
    pub case_id: String,
    pub docket_entry_id: String,
    pub recipients: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_snapshot: Option<String>,
    pub created_at: String,
}

impl From<Nef> for NefResponse {
    fn from(n: Nef) -> Self {
        Self {
            id: n.id.to_string(),
            filing_id: n.filing_id.to_string(),
            document_id: n.document_id.to_string(),
            case_id: n.case_id.to_string(),
            docket_entry_id: n.docket_entry_id.to_string(),
            recipients: n.recipients,
            html_snapshot: n.html_snapshot,
            created_at: n.created_at.to_rfc3339(),
        }
    }
}

// ---------------------------------------------------------------------------
// Filing upload staging types
// ---------------------------------------------------------------------------

/// A staged upload for a filing (presign + finalize pattern, no docket entry needed).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct FilingUpload {
    pub id: Uuid,
    pub court_id: String,
    pub filename: String,
    pub file_size: i64,
    pub content_type: String,
    pub storage_key: String,
    pub sha256: Option<String>,
    pub uploaded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Request to initiate a staged filing upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct InitFilingUploadRequest {
    pub filename: String,
    pub content_type: String,
    pub file_size: i64,
}

/// Response from initiating a staged filing upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct InitFilingUploadResponse {
    pub upload_id: String,
    pub presign_url: String,
    pub object_key: String,
    pub required_headers: std::collections::HashMap<String, String>,
}

/// Response from finalizing a staged filing upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FinalizeFilingUploadResponse {
    pub upload_id: String,
    pub filename: String,
    pub file_size: i64,
    pub content_type: String,
    pub uploaded_at: String,
}

// ---------------------------------------------------------------------------
// Document API types
// ---------------------------------------------------------------------------

/// Request to promote a docket attachment into a canonical document.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PromoteAttachmentRequest {
    pub docket_attachment_id: String,
    pub title: Option<String>,
    pub document_type: Option<String>,
}

/// Request to seal a document. Requires clerk or judge role.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SealDocumentRequest {
    /// One of: SealedCourtOnly, SealedCaseParticipants, SealedAttorneysOnly.
    pub sealing_level: String,
    /// Reason code for the seal (e.g. "JuvenileRecord", "TradeSecret").
    pub reason_code: String,
    /// Optional UUID of the motion that authorized the sealing.
    #[serde(default)]
    pub motion_id: Option<String>,
}

/// Request to replace a document with a corrected version.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReplaceDocumentRequest {
    /// Staged upload ID of the replacement file.
    pub upload_id: String,
    /// Optional new title. Falls back to the original document title.
    #[serde(default)]
    pub title: Option<String>,
}

/// API response shape for a document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DocumentResponse {
    pub id: String,
    pub court_id: String,
    pub case_id: String,
    pub title: String,
    pub document_type: String,
    pub storage_key: String,
    pub checksum: String,
    pub file_size: i64,
    pub content_type: String,
    pub is_sealed: bool,
    pub uploaded_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_attachment_id: Option<String>,
    pub created_at: String,
    pub sealing_level: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seal_reason_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seal_motion_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_by_document_id: Option<String>,
    pub is_stricken: bool,
}

// ---------------------------------------------------------------------------
// Document Events (audit trail)
// ---------------------------------------------------------------------------

/// A document event recording a lifecycle action (seal, unseal, replace, strike).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct DocumentEvent {
    pub id: Uuid,
    pub court_id: String,
    pub document_id: Uuid,
    pub event_type: String,
    pub actor: String,
    pub detail: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// API response shape for a document event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DocumentEventResponse {
    pub id: String,
    pub court_id: String,
    pub document_id: String,
    pub event_type: String,
    pub actor: String,
    pub detail: serde_json::Value,
    pub created_at: String,
}

impl From<DocumentEvent> for DocumentEventResponse {
    fn from(e: DocumentEvent) -> Self {
        Self {
            id: e.id.to_string(),
            court_id: e.court_id,
            document_id: e.document_id.to_string(),
            event_type: e.event_type,
            actor: e.actor,
            detail: e.detail,
            created_at: e.created_at.to_rfc3339(),
        }
    }
}

impl From<Document> for DocumentResponse {
    fn from(d: Document) -> Self {
        Self {
            id: d.id.to_string(),
            court_id: d.court_id,
            case_id: d.case_id.to_string(),
            title: d.title,
            document_type: d.document_type,
            storage_key: d.storage_key,
            checksum: d.checksum,
            file_size: d.file_size,
            content_type: d.content_type,
            is_sealed: d.is_sealed,
            uploaded_by: d.uploaded_by,
            source_attachment_id: d.source_attachment_id.map(|u| u.to_string()),
            created_at: d.created_at.to_rfc3339(),
            sealing_level: d.sealing_level,
            seal_reason_code: d.seal_reason_code,
            seal_motion_id: d.seal_motion_id.map(|u| u.to_string()),
            replaced_by_document_id: d.replaced_by_document_id.map(|u| u.to_string()),
            is_stricken: d.is_stricken,
        }
    }
}
