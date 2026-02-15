use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Deadline row from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Deadline {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Option<Uuid>,
    pub title: String,
    pub rule_code: Option<String>,
    pub due_at: DateTime<Utc>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API response shape for a deadline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeadlineResponse {
    pub id: String,
    pub case_id: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_code: Option<String>,
    pub due_at: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Deadline> for DeadlineResponse {
    fn from(d: Deadline) -> Self {
        Self {
            id: d.id.to_string(),
            case_id: d.case_id.map(|u| u.to_string()),
            title: d.title,
            rule_code: d.rule_code,
            due_at: d.due_at.to_rfc3339(),
            status: d.status,
            notes: d.notes,
            created_at: d.created_at.to_rfc3339(),
            updated_at: d.updated_at.to_rfc3339(),
        }
    }
}

/// Search response for deadlines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeadlineSearchResponse {
    pub deadlines: Vec<DeadlineResponse>,
    pub total: i64,
}

/// Valid deadline status values matching the DB CHECK constraint.
pub const DEADLINE_STATUSES: &[&str] = &["open", "met", "extended", "cancelled", "expired"];

/// Check whether a status string is a valid deadline status.
pub fn is_valid_deadline_status(s: &str) -> bool {
    DEADLINE_STATUSES.contains(&s)
}

/// Request to create a new deadline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateDeadlineRequest {
    pub title: String,
    #[serde(default)]
    pub case_id: Option<Uuid>,
    #[serde(default)]
    pub rule_code: Option<String>,
    pub due_at: DateTime<Utc>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Request to update an existing deadline (partial update).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(default)]
pub struct UpdateDeadlineRequest {
    pub title: Option<String>,
    pub case_id: Option<Uuid>,
    pub rule_code: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// Request to update deadline status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateDeadlineStatusRequest {
    pub status: String,
}

/// Query parameters for deadline search.
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct DeadlineSearchParams {
    pub status: Option<String>,
    pub case_id: Option<Uuid>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

// ── Extension Request types ─────────────────────────────────────────

/// Valid extension request statuses.
pub const EXTENSION_STATUSES: &[&str] = &["Pending", "Granted", "Denied", "Withdrawn"];

/// Check whether an extension status string is valid.
pub fn is_valid_extension_status(s: &str) -> bool {
    EXTENSION_STATUSES.contains(&s)
}

/// A request for a deadline extension (DB row).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct ExtensionRequest {
    pub id: Uuid,
    pub court_id: String,
    pub deadline_id: Uuid,
    pub requested_by: String,
    pub reason: String,
    pub request_date: DateTime<Utc>,
    pub requested_new_date: DateTime<Utc>,
    pub status: String,
    pub ruling_by: Option<String>,
    pub ruling_date: Option<DateTime<Utc>>,
    pub new_deadline_date: Option<DateTime<Utc>>,
}

/// API response shape for an extension request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExtensionResponse {
    pub id: String,
    pub deadline_id: String,
    pub requested_by: String,
    pub reason: String,
    pub request_date: String,
    pub requested_new_date: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruling_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruling_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_deadline_date: Option<String>,
}

impl From<ExtensionRequest> for ExtensionResponse {
    fn from(e: ExtensionRequest) -> Self {
        Self {
            id: e.id.to_string(),
            deadline_id: e.deadline_id.to_string(),
            requested_by: e.requested_by,
            reason: e.reason,
            request_date: e.request_date.to_rfc3339(),
            requested_new_date: e.requested_new_date.to_rfc3339(),
            status: e.status,
            ruling_by: e.ruling_by,
            ruling_date: e.ruling_date.map(|d| d.to_rfc3339()),
            new_deadline_date: e.new_deadline_date.map(|d| d.to_rfc3339()),
        }
    }
}

/// Request to create a new extension request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateExtensionRequest {
    pub reason: String,
    pub requested_new_date: DateTime<Utc>,
    pub requested_by: String,
}

/// Request to update the ruling on an extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateExtensionRulingRequest {
    pub status: String,
    pub ruling_by: String,
    #[serde(default)]
    pub new_deadline_date: Option<DateTime<Utc>>,
}

// ── Deadline Reminder types ─────────────────────────────────────────

/// Valid reminder type values matching DB CHECK constraint.
pub const REMINDER_TYPES: &[&str] = &["Email", "SMS", "In-App", "Push", "Fax"];

/// Check whether a reminder type string is valid.
pub fn is_valid_reminder_type(s: &str) -> bool {
    REMINDER_TYPES.contains(&s)
}

/// A reminder sent for a deadline (DB row).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct DeadlineReminder {
    pub id: Uuid,
    pub court_id: String,
    pub deadline_id: Uuid,
    pub recipient: String,
    pub reminder_type: String,
    pub sent_at: DateTime<Utc>,
    pub acknowledged: bool,
    pub acknowledged_at: Option<DateTime<Utc>>,
}

/// API response shape for a deadline reminder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReminderResponse {
    pub id: String,
    pub deadline_id: String,
    pub recipient: String,
    pub reminder_type: String,
    pub sent_at: String,
    pub acknowledged: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acknowledged_at: Option<String>,
}

impl From<DeadlineReminder> for ReminderResponse {
    fn from(r: DeadlineReminder) -> Self {
        Self {
            id: r.id.to_string(),
            deadline_id: r.deadline_id.to_string(),
            recipient: r.recipient,
            reminder_type: r.reminder_type,
            sent_at: r.sent_at.to_rfc3339(),
            acknowledged: r.acknowledged,
            acknowledged_at: r.acknowledged_at.map(|d| d.to_rfc3339()),
        }
    }
}

/// Request to send a deadline reminder.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SendReminderRequest {
    pub deadline_id: Uuid,
    pub recipient: String,
    pub reminder_type: String,
}

// ── Deadline Calculation types ──────────────────────────────────────

/// Request to calculate a deadline from a rule code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CalculateDeadlineRequest {
    pub rule_code: String,
    pub trigger_date: DateTime<Utc>,
    #[serde(default)]
    pub jurisdiction: Option<String>,
}

/// Response for a calculated deadline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CalculateDeadlineResponse {
    pub rule_code: String,
    pub calculated_date: String,
    pub description: String,
}

// ── Compliance types ────────────────────────────────────────────────

/// Aggregate compliance statistics for deadlines.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ComplianceStats {
    pub total_deadlines: i64,
    pub met: i64,
    pub missed: i64,
    pub extended: i64,
    pub compliance_rate: f64,
}

/// Compliance report with per-type breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ComplianceReport {
    pub stats: ComplianceStats,
    pub by_type: serde_json::Value,
}

/// A federal rule entry for reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FederalRule {
    pub rule_code: String,
    pub title: String,
    pub description: String,
    pub days: i32,
    pub business_days_only: bool,
}
