use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Speedy Trial Act clock tracking for a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct SpeedyTrialClock {
    pub case_id: Uuid,
    pub court_id: String,
    pub arrest_date: Option<DateTime<Utc>>,
    pub indictment_date: Option<DateTime<Utc>>,
    pub arraignment_date: Option<DateTime<Utc>>,
    pub trial_start_deadline: DateTime<Utc>,
    pub days_elapsed: i64,
    pub days_remaining: i64,
    pub is_tolled: bool,
    pub waived: bool,
}

/// A period of excludable delay under the Speedy Trial Act.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct ExcludableDelay {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    /// DelayReason enum stored as text.
    pub reason: String,
    pub statutory_reference: String,
    pub days_excluded: i64,
    pub order_reference: Option<String>,
}

// ── Speedy Trial response / request types ─────────────────────────

/// API response for a speedy trial clock.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SpeedyTrialResponse {
    pub case_id: String,
    pub arrest_date: Option<String>,
    pub indictment_date: Option<String>,
    pub arraignment_date: Option<String>,
    pub trial_start_deadline: String,
    pub days_elapsed: i64,
    pub days_remaining: i64,
    pub is_tolled: bool,
    pub waived: bool,
}

impl From<SpeedyTrialClock> for SpeedyTrialResponse {
    fn from(c: SpeedyTrialClock) -> Self {
        Self {
            case_id: c.case_id.to_string(),
            arrest_date: c.arrest_date.map(|d| d.to_rfc3339()),
            indictment_date: c.indictment_date.map(|d| d.to_rfc3339()),
            arraignment_date: c.arraignment_date.map(|d| d.to_rfc3339()),
            trial_start_deadline: c.trial_start_deadline.to_rfc3339(),
            days_elapsed: c.days_elapsed,
            days_remaining: c.days_remaining,
            is_tolled: c.is_tolled,
            waived: c.waived,
        }
    }
}

/// Request to start the speedy trial clock for a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StartSpeedyTrialRequest {
    pub case_id: Uuid,
    #[serde(default)]
    pub arrest_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub indictment_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub arraignment_date: Option<DateTime<Utc>>,
    /// If not provided, computed as 70 days from the earliest milestone.
    #[serde(default)]
    pub trial_start_deadline: Option<DateTime<Utc>>,
}

/// Request to update a speedy trial clock.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateSpeedyTrialClockRequest {
    #[serde(default)]
    pub arrest_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub indictment_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub arraignment_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub trial_start_deadline: Option<DateTime<Utc>>,
    #[serde(default)]
    pub days_elapsed: Option<i64>,
    #[serde(default)]
    pub days_remaining: Option<i64>,
    #[serde(default)]
    pub is_tolled: Option<bool>,
    #[serde(default)]
    pub waived: Option<bool>,
}

// ── Excludable Delay response / request types ─────────────────────

/// API response for an excludable delay.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExcludableDelayResponse {
    pub id: String,
    pub case_id: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub reason: String,
    pub statutory_reference: String,
    pub days_excluded: i64,
    pub order_reference: Option<String>,
}

impl From<ExcludableDelay> for ExcludableDelayResponse {
    fn from(d: ExcludableDelay) -> Self {
        Self {
            id: d.id.to_string(),
            case_id: d.case_id.to_string(),
            start_date: d.start_date.to_rfc3339(),
            end_date: d.end_date.map(|dt| dt.to_rfc3339()),
            reason: d.reason,
            statutory_reference: d.statutory_reference,
            days_excluded: d.days_excluded,
            order_reference: d.order_reference,
        }
    }
}

/// Speedy trial deadline check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeadlineCheckResponse {
    pub case_id: String,
    pub days_elapsed: i64,
    pub days_remaining: i64,
    pub deadline_days: i64,
    pub is_approaching: bool,
    pub is_violated: bool,
}

/// Request to create an excludable delay.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateExcludableDelayRequest {
    pub start_date: DateTime<Utc>,
    #[serde(default)]
    pub end_date: Option<DateTime<Utc>>,
    pub reason: String,
    #[serde(default)]
    pub statutory_reference: Option<String>,
    #[serde(default)]
    pub days_excluded: Option<i64>,
    #[serde(default)]
    pub order_reference: Option<String>,
}
