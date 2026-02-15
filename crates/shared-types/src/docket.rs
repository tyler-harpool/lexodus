use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Validation constants ────────────────────────────────────────────

/// Valid docket entry type values matching the DB CHECK constraint (OpenAPI aligned).
pub const DOCKET_ENTRY_TYPES: &[&str] = &[
    "complaint", "indictment", "information", "criminal_complaint",
    "answer", "motion", "response", "reply", "notice",
    "order", "minute_order", "scheduling_order", "protective_order", "sealing_order",
    "discovery_request", "discovery_response", "deposition", "interrogatories",
    "exhibit", "witness_list", "expert_report",
    "hearing_notice", "hearing_minutes", "transcript",
    "judgment", "verdict", "sentence",
    "summons", "subpoena", "service_return",
    "appearance", "withdrawal", "substitution",
    "notice_of_appeal", "appeal_brief", "appellate_order",
    "letter", "status", "other",
];

/// Check whether an entry type string is valid.
pub fn is_valid_entry_type(s: &str) -> bool {
    DOCKET_ENTRY_TYPES.contains(&s)
}

// ── DB row struct ───────────────────────────────────────────────────

/// An entry on the case docket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct DocketEntry {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub entry_number: i32,
    pub date_filed: DateTime<Utc>,
    pub date_entered: DateTime<Utc>,
    pub filed_by: Option<String>,
    /// DocketEntryType enum stored as text.
    pub entry_type: String,
    pub description: String,
    pub document_id: Option<Uuid>,
    pub is_sealed: bool,
    pub is_ex_parte: bool,
    pub page_count: Option<i32>,
    pub related_entries: Vec<i32>,
    pub service_list: Vec<String>,
}

// ── API response types ──────────────────────────────────────────────

/// API response shape for a docket entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DocketEntryResponse {
    pub id: String,
    pub case_id: String,
    pub entry_number: i32,
    pub date_filed: String,
    pub date_entered: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filed_by: Option<String>,
    pub entry_type: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    pub is_sealed: bool,
    pub is_ex_parte: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<i32>,
    pub related_entries: Vec<i32>,
    pub service_list: Vec<String>,
}

impl From<DocketEntry> for DocketEntryResponse {
    fn from(d: DocketEntry) -> Self {
        Self {
            id: d.id.to_string(),
            case_id: d.case_id.to_string(),
            entry_number: d.entry_number,
            date_filed: d.date_filed.to_rfc3339(),
            date_entered: d.date_entered.to_rfc3339(),
            filed_by: d.filed_by,
            entry_type: d.entry_type,
            description: d.description,
            document_id: d.document_id.map(|u| u.to_string()),
            is_sealed: d.is_sealed,
            is_ex_parte: d.is_ex_parte,
            page_count: d.page_count,
            related_entries: d.related_entries,
            service_list: d.service_list,
        }
    }
}

/// Search response for docket entries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DocketSearchResponse {
    pub entries: Vec<DocketEntryResponse>,
    pub total: i64,
}

// ── Request types ───────────────────────────────────────────────────

/// Request to create a new docket entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateDocketEntryRequest {
    pub case_id: Uuid,
    pub entry_type: String,
    pub description: String,
    #[serde(default)]
    pub filed_by: Option<String>,
    #[serde(default)]
    pub document_id: Option<Uuid>,
    #[serde(default)]
    pub is_sealed: bool,
    #[serde(default)]
    pub is_ex_parte: bool,
    #[serde(default)]
    pub page_count: Option<i32>,
    #[serde(default)]
    pub related_entries: Vec<i32>,
    #[serde(default)]
    pub service_list: Vec<String>,
}

/// Query parameters for docket entry search.
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct DocketSearchParams {
    pub case_id: Option<String>,
    pub entry_type: Option<String>,
    pub q: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

/// Request to link an existing document to a docket entry.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LinkDocumentRequest {
    pub document_id: String,
}

/// An attachment associated with a docket entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct DocketAttachment {
    pub id: Uuid,
    pub court_id: String,
    pub docket_entry_id: Uuid,
    pub filename: String,
    pub file_size: i64,
    pub content_type: String,
    pub storage_key: String,
    pub sealed: bool,
    pub encryption: String,
    pub sha256: Option<String>,
    pub uploaded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Attachment API types ────────────────────────────────────────────

/// API response shape for a docket attachment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DocketAttachmentResponse {
    pub id: String,
    pub docket_entry_id: String,
    pub filename: String,
    pub file_size: i64,
    pub content_type: String,
    pub sealed: bool,
    pub encryption: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploaded_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<DocketAttachment> for DocketAttachmentResponse {
    fn from(a: DocketAttachment) -> Self {
        Self {
            id: a.id.to_string(),
            docket_entry_id: a.docket_entry_id.to_string(),
            filename: a.filename,
            file_size: a.file_size,
            content_type: a.content_type,
            sealed: a.sealed,
            encryption: a.encryption,
            sha256: a.sha256,
            uploaded_at: a.uploaded_at.map(|t| t.to_rfc3339()),
            created_at: a.created_at.to_rfc3339(),
            updated_at: a.updated_at.to_rfc3339(),
        }
    }
}

/// Request body for initiating a docket attachment upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateAttachmentRequest {
    pub file_name: String,
    pub content_type: String,
    pub file_size: i64,
}

/// Response returned when a presigned upload is initiated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateAttachmentResponse {
    pub attachment_id: String,
    pub presign_url: String,
    pub object_key: String,
    pub required_headers: std::collections::HashMap<String, String>,
}

// ── Docket sheet and statistics ─────────────────────────────────────

/// A full docket sheet for a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DocketSheet {
    pub case_id: String,
    pub case_number: String,
    pub entries: Vec<DocketEntryResponse>,
    pub total: i64,
}

/// Aggregate statistics for docket entries in a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DocketStatistics {
    pub case_id: String,
    pub total_entries: i64,
    pub by_type: serde_json::Value,
    pub sealed_count: i64,
}

/// Filing statistics for a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FilingStatsResponse {
    pub case_id: String,
    pub total_filings: i64,
    pub by_type: std::collections::HashMap<String, i64>,
}

/// Service check response for an entry type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ServiceCheckResponse {
    pub entry_type: String,
    pub requires_service: bool,
    pub service_method: String,
    pub service_deadline_days: i32,
}
