use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A scheduled court event on the calendar (DB row).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct CalendarEvent {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub judge_id: Uuid,
    /// CalendarEventType enum stored as snake_case text (e.g. "motion_hearing").
    pub event_type: String,
    pub scheduled_date: DateTime<Utc>,
    pub duration_minutes: i32,
    pub courtroom: String,
    pub description: String,
    pub participants: Vec<String>,
    pub court_reporter: Option<String>,
    pub is_public: bool,
    /// EventStatus enum stored as snake_case text (e.g. "scheduled", "in_progress").
    pub status: String,
    pub notes: String,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,
    pub call_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Resolved case number from LEFT JOIN criminal_cases/civil_cases.
    pub case_number: Option<String>,
}

/// API response for a calendar entry (matches OpenAPI CalendarEntry schema).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CalendarEntryResponse {
    pub id: String,
    pub case_id: String,
    pub judge_id: String,
    pub event_type: String,
    pub scheduled_date: String,
    pub duration_minutes: i32,
    pub courtroom: String,
    pub description: String,
    pub participants: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub court_reporter: Option<String>,
    pub is_public: bool,
    pub status: String,
    pub notes: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_time: Option<String>,
    /// Resolved case number from criminal_cases or civil_cases.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,
}

impl From<CalendarEvent> for CalendarEntryResponse {
    fn from(e: CalendarEvent) -> Self {
        Self {
            id: e.id.to_string(),
            case_id: e.case_id.to_string(),
            judge_id: e.judge_id.to_string(),
            event_type: e.event_type,
            scheduled_date: e.scheduled_date.to_rfc3339(),
            duration_minutes: e.duration_minutes,
            courtroom: e.courtroom,
            description: e.description,
            participants: e.participants,
            court_reporter: e.court_reporter,
            is_public: e.is_public,
            status: e.status,
            notes: e.notes,
            actual_start: e.actual_start.map(|t| t.to_rfc3339()),
            actual_end: e.actual_end.map(|t| t.to_rfc3339()),
            call_time: e.call_time.map(|t| t.to_rfc3339()),
            case_number: e.case_number,
        }
    }
}

/// Search response for calendar entries (matches OpenAPI CalendarSearchResponse).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CalendarSearchResponse {
    pub events: Vec<CalendarEntryResponse>,
    pub total: i64,
}

/// Calendar event type values matching the DB CHECK constraint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum CalendarEventType {
    #[serde(rename = "initial_appearance")]
    InitialAppearance,
    #[serde(rename = "arraignment")]
    Arraignment,
    #[serde(rename = "bail_hearing")]
    BailHearing,
    #[serde(rename = "plea_hearing")]
    PleaHearing,
    #[serde(rename = "trial_date")]
    TrialDate,
    #[serde(rename = "sentencing")]
    Sentencing,
    #[serde(rename = "violation_hearing")]
    ViolationHearing,
    #[serde(rename = "status_conference")]
    StatusConference,
    #[serde(rename = "scheduling_conference")]
    SchedulingConference,
    #[serde(rename = "settlement_conference")]
    SettlementConference,
    #[serde(rename = "pretrial_conference")]
    PretrialConference,
    #[serde(rename = "motion_hearing")]
    MotionHearing,
    #[serde(rename = "evidentiary_hearing")]
    EvidentiaryHearing,
    #[serde(rename = "jury_selection")]
    JurySelection,
    #[serde(rename = "jury_trial")]
    JuryTrial,
    #[serde(rename = "bench_trial")]
    BenchTrial,
    #[serde(rename = "show_cause_hearing")]
    ShowCauseHearing,
    #[serde(rename = "contempt_hearing")]
    ContemptHearing,
    #[serde(rename = "emergency_hearing")]
    EmergencyHearing,
    #[serde(rename = "telephonic")]
    Telephonic,
    #[serde(rename = "video_conference")]
    VideoConference,
}

impl CalendarEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InitialAppearance => "initial_appearance",
            Self::Arraignment => "arraignment",
            Self::BailHearing => "bail_hearing",
            Self::PleaHearing => "plea_hearing",
            Self::TrialDate => "trial_date",
            Self::Sentencing => "sentencing",
            Self::ViolationHearing => "violation_hearing",
            Self::StatusConference => "status_conference",
            Self::SchedulingConference => "scheduling_conference",
            Self::SettlementConference => "settlement_conference",
            Self::PretrialConference => "pretrial_conference",
            Self::MotionHearing => "motion_hearing",
            Self::EvidentiaryHearing => "evidentiary_hearing",
            Self::JurySelection => "jury_selection",
            Self::JuryTrial => "jury_trial",
            Self::BenchTrial => "bench_trial",
            Self::ShowCauseHearing => "show_cause_hearing",
            Self::ContemptHearing => "contempt_hearing",
            Self::EmergencyHearing => "emergency_hearing",
            Self::Telephonic => "telephonic",
            Self::VideoConference => "video_conference",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "initial_appearance" => Some(Self::InitialAppearance),
            "arraignment" => Some(Self::Arraignment),
            "bail_hearing" => Some(Self::BailHearing),
            "plea_hearing" => Some(Self::PleaHearing),
            "trial_date" => Some(Self::TrialDate),
            "sentencing" => Some(Self::Sentencing),
            "violation_hearing" => Some(Self::ViolationHearing),
            "status_conference" => Some(Self::StatusConference),
            "scheduling_conference" => Some(Self::SchedulingConference),
            "settlement_conference" => Some(Self::SettlementConference),
            "pretrial_conference" => Some(Self::PretrialConference),
            "motion_hearing" => Some(Self::MotionHearing),
            "evidentiary_hearing" => Some(Self::EvidentiaryHearing),
            "jury_selection" => Some(Self::JurySelection),
            "jury_trial" => Some(Self::JuryTrial),
            "bench_trial" => Some(Self::BenchTrial),
            "show_cause_hearing" => Some(Self::ShowCauseHearing),
            "contempt_hearing" => Some(Self::ContemptHearing),
            "emergency_hearing" => Some(Self::EmergencyHearing),
            "telephonic" => Some(Self::Telephonic),
            "video_conference" => Some(Self::VideoConference),
            _ => None,
        }
    }
}

/// Event status values matching the DB CHECK constraint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum EventStatus {
    #[serde(rename = "scheduled")]
    Scheduled,
    #[serde(rename = "confirmed")]
    Confirmed,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "postponed")]
    Postponed,
    #[serde(rename = "recessed")]
    Recessed,
    #[serde(rename = "continued")]
    Continued,
}

impl EventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Confirmed => "confirmed",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Postponed => "postponed",
            Self::Recessed => "recessed",
            Self::Continued => "continued",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "scheduled" => Some(Self::Scheduled),
            "confirmed" => Some(Self::Confirmed),
            "in_progress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            "cancelled" => Some(Self::Cancelled),
            "postponed" => Some(Self::Postponed),
            "recessed" => Some(Self::Recessed),
            "continued" => Some(Self::Continued),
            _ => None,
        }
    }
}

/// Request to schedule a new court event (matches OpenAPI ScheduleEventRequest).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ScheduleEventRequest {
    pub case_id: Uuid,
    pub judge_id: Uuid,
    pub event_type: String,
    pub scheduled_date: DateTime<Utc>,
    pub duration_minutes: i32,
    pub courtroom: String,
    pub description: String,
    pub participants: Vec<String>,
    pub is_public: bool,
}

/// Request to update event status (matches OpenAPI UpdateEventStatusRequest).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateEventStatusRequest {
    pub status: String,
    #[serde(default)]
    pub actual_start: Option<DateTime<Utc>>,
    #[serde(default)]
    pub actual_end: Option<DateTime<Utc>>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Query parameters for calendar search (matches OpenAPI search_calendar params).
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct CalendarSearchParams {
    pub judge_id: Option<Uuid>,
    pub courtroom: Option<String>,
    pub event_type: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

// ── Calendar utilization and availability ───────────────────────────

/// Court utilization metrics for a time range.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CourtUtilization {
    pub total_events: i64,
    pub by_courtroom: serde_json::Value,
    pub by_judge: serde_json::Value,
    pub utilization_rate: f64,
}

/// An available scheduling slot for a judge.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AvailableSlot {
    pub judge_id: String,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub courtroom: Option<String>,
}
