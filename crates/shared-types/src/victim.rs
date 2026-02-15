use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A victim in a criminal case, tracked per the Crime Victims' Rights Act.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Victim {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub name: String,
    /// VictimType enum stored as text (e.g. "Individual", "Organization").
    pub victim_type: String,
    pub notification_email: Option<String>,
    pub notification_mail: bool,
    pub notification_phone: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A notification sent to a victim per CVRA requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct VictimNotification {
    pub id: Uuid,
    pub court_id: String,
    pub victim_id: Uuid,
    /// NotificationType enum stored as text (e.g. "StatusChange", "Hearing", "Sentencing").
    pub notification_type: String,
    pub sent_at: DateTime<Utc>,
    /// NotificationMethod enum stored as text (e.g. "Email", "Mail", "Phone").
    pub method: String,
    pub content_summary: Option<String>,
    pub acknowledged: bool,
    pub acknowledged_at: Option<DateTime<Utc>>,
}

// ── Victim validation constants ─────────────────────────────────────

/// Valid victim type values.
pub const VICTIM_TYPES: &[&str] = &["Individual", "Organization", "Government", "Minor", "Deceased", "Anonymous"];

/// Valid notification type values matching DB CHECK constraint.
pub const NOTIFICATION_TYPES: &[&str] = &[
    "Case Filed", "Hearing Scheduled", "Plea Agreement", "Sentencing",
    "Release", "Restitution", "Appeal", "Status Change", "Other",
];

/// Valid notification method values matching DB CHECK constraint.
pub const NOTIFICATION_METHODS: &[&str] = &["Email", "Mail", "Phone", "In-App", "Fax"];

/// Check whether a victim type string is valid.
pub fn is_valid_victim_type(s: &str) -> bool {
    VICTIM_TYPES.contains(&s)
}

/// Check whether a notification type string is valid.
pub fn is_valid_notification_type(s: &str) -> bool {
    NOTIFICATION_TYPES.contains(&s)
}

/// Check whether a notification method string is valid.
pub fn is_valid_notification_method(s: &str) -> bool {
    NOTIFICATION_METHODS.contains(&s)
}

// ── Victim API response ─────────────────────────────────────────────

/// API response shape for a victim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VictimResponse {
    pub id: String,
    pub case_id: String,
    pub name: String,
    pub victim_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    pub notification_mail: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_phone: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Victim> for VictimResponse {
    fn from(v: Victim) -> Self {
        Self {
            id: v.id.to_string(),
            case_id: v.case_id.to_string(),
            name: v.name,
            victim_type: v.victim_type,
            notification_email: v.notification_email,
            notification_mail: v.notification_mail,
            notification_phone: v.notification_phone,
            created_at: v.created_at.to_rfc3339(),
            updated_at: v.updated_at.to_rfc3339(),
        }
    }
}

// ── Victim request types ────────────────────────────────────────────

/// Request to create a new victim record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateVictimRequest {
    pub case_id: Uuid,
    pub name: String,
    pub victim_type: String,
    #[serde(default)]
    pub notification_email: Option<String>,
    #[serde(default)]
    pub notification_phone: Option<String>,
    #[serde(default)]
    pub notification_mail: Option<bool>,
}

// ── VictimNotification API response ─────────────────────────────────

/// API response shape for a victim notification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VictimNotificationResponse {
    pub id: String,
    pub victim_id: String,
    pub notification_type: String,
    pub sent_at: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_summary: Option<String>,
    pub acknowledged: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acknowledged_at: Option<String>,
}

impl From<VictimNotification> for VictimNotificationResponse {
    fn from(n: VictimNotification) -> Self {
        Self {
            id: n.id.to_string(),
            victim_id: n.victim_id.to_string(),
            notification_type: n.notification_type,
            sent_at: n.sent_at.to_rfc3339(),
            method: n.method,
            content_summary: n.content_summary,
            acknowledged: n.acknowledged,
            acknowledged_at: n.acknowledged_at.map(|d| d.to_rfc3339()),
        }
    }
}

/// Request to send a victim notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SendVictimNotificationRequest {
    pub notification_type: String,
    pub method: String,
    pub content_summary: String,
}
