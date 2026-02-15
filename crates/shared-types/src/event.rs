use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Event type configuration — describes what each event type needs
// ---------------------------------------------------------------------------

/// The kind of docket event being composed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum EventKind {
    /// Clerk/judge text-only docket notation (no file, no filing record).
    TextEntry,
    /// Electronic filing: creates Document + DocketEntry + Filing + NEF.
    Filing,
    /// Promote an existing docket attachment to a canonical document.
    PromoteAttachment,
}

impl EventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventKind::TextEntry => "text_entry",
            EventKind::Filing => "filing",
            EventKind::PromoteAttachment => "promote_attachment",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text_entry" => Some(EventKind::TextEntry),
            "filing" => Some(EventKind::Filing),
            "promote_attachment" => Some(EventKind::PromoteAttachment),
            _ => None,
        }
    }

    /// Human-readable label for UI display.
    pub fn label(&self) -> &'static str {
        match self {
            EventKind::TextEntry => "Add Text Entry",
            EventKind::Filing => "File Document",
            EventKind::PromoteAttachment => "Register as Filed Document",
        }
    }
}

/// Whether a file attachment is required, optional, or forbidden for an event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileRequirement {
    Required,
    Optional,
    None,
}

/// Static configuration for an event type — drives the EventComposer UI and
/// server-side validation.
#[derive(Debug, Clone)]
pub struct EventTypeConfig {
    pub kind: EventKind,
    /// Minimum role required (Clerk, Judge, Attorney, etc.).
    pub min_role: &'static str,
    /// Whether a file attachment is required, optional, or not applicable.
    pub file_requirement: FileRequirement,
    /// Whether this event type creates a Filing record.
    pub creates_filing: bool,
    /// Whether this event type creates a Document record.
    pub creates_document: bool,
    /// Whether a NEF is generated.
    pub creates_nef: bool,
    /// Whether sealing options are shown.
    pub supports_sealing: bool,
    /// Fields required beyond the basics (case_id is always required).
    pub required_fields: &'static [&'static str],
}

/// All event type configurations. Used by both UI and server validation.
pub const EVENT_TYPE_CONFIGS: &[EventTypeConfig] = &[
    EventTypeConfig {
        kind: EventKind::TextEntry,
        min_role: "clerk",
        file_requirement: FileRequirement::None,
        creates_filing: false,
        creates_document: false,
        creates_nef: false,
        supports_sealing: false,
        required_fields: &["entry_type", "description"],
    },
    EventTypeConfig {
        kind: EventKind::Filing,
        min_role: "attorney",
        file_requirement: FileRequirement::Optional,
        creates_filing: true,
        creates_document: true,
        creates_nef: true,
        supports_sealing: true,
        required_fields: &["document_type", "title", "filed_by"],
    },
    EventTypeConfig {
        kind: EventKind::PromoteAttachment,
        min_role: "clerk",
        file_requirement: FileRequirement::None,
        creates_filing: false,
        creates_document: true,
        creates_nef: false,
        supports_sealing: false,
        required_fields: &["attachment_id"],
    },
];

/// Look up the config for a given event kind.
pub fn get_event_config(kind: EventKind) -> &'static EventTypeConfig {
    EVENT_TYPE_CONFIGS
        .iter()
        .find(|c| c.kind == kind)
        .expect("all EventKind variants must have a config entry")
}

// ---------------------------------------------------------------------------
// Unified event submission request
// ---------------------------------------------------------------------------

/// Request to submit a docket event through the unified composer.
/// Fields are optional because different event kinds use different subsets.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SubmitEventRequest {
    /// Which kind of event: "text_entry", "filing", or "promote_attachment".
    pub event_kind: String,
    /// Target case UUID.
    pub case_id: String,

    // ── Text entry fields ───────────────────────────────────────────
    /// Docket entry type (e.g. "motion", "order"). Required for TextEntry.
    #[serde(default)]
    pub entry_type: Option<String>,
    /// Free-text description. Required for TextEntry, used as docket description for Filing.
    #[serde(default)]
    pub description: Option<String>,

    // ── Filing fields ───────────────────────────────────────────────
    /// Document type (e.g. "Motion", "Order"). Required for Filing.
    #[serde(default)]
    pub document_type: Option<String>,
    /// Document title. Required for Filing.
    #[serde(default)]
    pub title: Option<String>,
    /// Who filed the document. Required for Filing.
    #[serde(default)]
    pub filed_by: Option<String>,
    /// Staged upload ID (from the filing upload flow). Optional for Filing.
    #[serde(default)]
    pub upload_id: Option<String>,
    /// Whether the filing is sealed.
    #[serde(default)]
    pub is_sealed: Option<bool>,
    /// Sealing level (e.g. "SealedCourtOnly").
    #[serde(default)]
    pub sealing_level: Option<String>,
    /// Reason code for sealing (e.g. "JuvenileRecord").
    #[serde(default)]
    pub reason_code: Option<String>,

    // ── Promote attachment fields ───────────────────────────────────
    /// Docket attachment ID to promote. Required for PromoteAttachment.
    #[serde(default)]
    pub attachment_id: Option<String>,
    /// Override title when promoting.
    #[serde(default)]
    pub promote_title: Option<String>,
    /// Override document type when promoting.
    #[serde(default)]
    pub promote_document_type: Option<String>,
}

// ---------------------------------------------------------------------------
// Unified event response
// ---------------------------------------------------------------------------

/// Response from submitting a docket event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SubmitEventResponse {
    pub event_kind: String,
    pub docket_entry_id: String,
    pub entry_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filing_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nef_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Timeline types
// ---------------------------------------------------------------------------

/// A single entry in the unified case timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TimelineEntry {
    /// Unique ID for this timeline entry (may be docket_entry.id or document_event.id).
    pub id: String,
    /// Source: "docket_entry", "document_event", "filing".
    pub source: String,
    /// Timestamp for ordering.
    pub timestamp: String,
    /// Human-readable summary.
    pub summary: String,
    /// Actor who created the event.
    pub actor: Option<String>,
    /// Entry type or event type.
    pub entry_type: String,
    /// Whether the entry is sealed.
    pub is_sealed: bool,
    /// Associated document ID, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    /// Docket entry number, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_number: Option<i32>,
    /// Additional metadata.
    #[serde(default)]
    pub detail: serde_json::Value,
}

/// Response for the unified timeline endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TimelineResponse {
    pub entries: Vec<TimelineEntry>,
    pub total: i64,
}
