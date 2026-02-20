use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const QUEUE_TYPES: &[&str] = &["filing", "motion", "order", "deadline_alert", "general"];
pub const QUEUE_STATUSES: &[&str] = &["pending", "in_review", "processing", "completed", "rejected"];
pub const QUEUE_STEPS: &[&str] = &["review", "docket", "nef", "route_judge", "serve", "completed"];
pub const QUEUE_SOURCE_TYPES: &[&str] =
    &["filing", "motion", "order", "document", "deadline", "calendar_event"];

pub fn is_valid_queue_type(s: &str) -> bool {
    QUEUE_TYPES.contains(&s)
}

pub fn is_valid_queue_status(s: &str) -> bool {
    QUEUE_STATUSES.contains(&s)
}

pub fn is_valid_queue_step(s: &str) -> bool {
    QUEUE_STEPS.contains(&s)
}

// ---------------------------------------------------------------------------
// Database Row
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct QueueItem {
    pub id: Uuid,
    pub court_id: String,
    pub queue_type: String,
    pub priority: i32,
    pub status: String,
    pub title: String,
    pub description: Option<String>,
    pub source_type: String,
    pub source_id: Uuid,
    pub case_id: Option<Uuid>,
    pub case_type: String,
    pub case_number: Option<String>,
    pub assigned_to: Option<i64>,
    pub submitted_by: Option<i64>,
    pub current_step: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// API Response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct QueueItemResponse {
    pub id: String,
    pub court_id: String,
    pub queue_type: String,
    pub priority: i32,
    pub status: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub source_type: String,
    pub source_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    pub case_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_by: Option<i64>,
    pub current_step: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

impl From<QueueItem> for QueueItemResponse {
    fn from(q: QueueItem) -> Self {
        Self {
            id: q.id.to_string(),
            court_id: q.court_id,
            queue_type: q.queue_type,
            priority: q.priority,
            status: q.status,
            title: q.title,
            description: q.description,
            source_type: q.source_type,
            source_id: q.source_id.to_string(),
            case_id: q.case_id.map(|u| u.to_string()),
            case_type: q.case_type,
            case_number: q.case_number,
            assigned_to: q.assigned_to,
            submitted_by: q.submitted_by,
            current_step: q.current_step,
            metadata: Some(q.metadata),
            created_at: q.created_at.to_rfc3339(),
            updated_at: q.updated_at.to_rfc3339(),
            completed_at: q.completed_at.map(|d| d.to_rfc3339()),
        }
    }
}

// ---------------------------------------------------------------------------
// Search Response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct QueueSearchResponse {
    pub items: Vec<QueueItemResponse>,
    pub total: i64,
}

// ---------------------------------------------------------------------------
// Stats Response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct QueueStats {
    pub pending_count: i64,
    pub my_count: i64,
    pub today_count: i64,
    pub urgent_count: i64,
    pub avg_processing_mins: Option<f64>,
}

// ---------------------------------------------------------------------------
// Request Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateQueueItemRequest {
    pub queue_type: String,
    pub priority: Option<i32>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub source_type: String,
    pub source_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Search/filter params for GET /api/queue
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct QueueSearchParams {
    pub status: Option<String>,
    pub queue_type: Option<String>,
    pub priority: Option<i32>,
    pub assigned_to: Option<i64>,
    pub case_id: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

/// POST /api/queue/{id}/advance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdvanceQueueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_data: Option<serde_json::Value>,
}

/// POST /api/queue/{id}/reject
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RejectQueueRequest {
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Pipeline Step Mapping
// ---------------------------------------------------------------------------

/// Returns the ordered pipeline steps for a given queue_type.
pub fn pipeline_steps(queue_type: &str) -> Vec<&'static str> {
    match queue_type {
        "filing" => vec!["review", "docket", "nef", "serve"],
        "motion" => vec!["review", "docket", "nef", "route_judge", "serve"],
        "order" => vec!["docket", "nef", "serve"],
        "deadline_alert" | "general" => vec!["review"],
        _ => vec!["review"],
    }
}

/// Returns the next step after `current` for a given queue_type, or None if at the end.
pub fn next_step(queue_type: &str, current: &str) -> Option<&'static str> {
    let steps = pipeline_steps(queue_type);
    let pos = steps.iter().position(|&s| s == current)?;
    steps.get(pos + 1).copied()
}
