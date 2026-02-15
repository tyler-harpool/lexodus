use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, DeadlineCheckResponse, SpeedyTrialResponse, StartSpeedyTrialRequest,
    UpdateSpeedyTrialClockRequest, ExcludableDelayResponse,
    CreateExcludableDelayRequest,
};
use crate::tenant::CourtId;

// ── Speedy Trial Clock handlers ──────────────────────────────────

/// POST /api/cases/{id}/speedy-trial/start
#[utoipa::path(
    post,
    path = "/api/cases/{id}/speedy-trial/start",
    request_body = StartSpeedyTrialRequest,
    params(
        ("id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Speedy trial clock started", body = SpeedyTrialResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "speedy-trial"
)]
pub async fn start_speedy_trial(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(_id): Path<String>,
    Json(body): Json<StartSpeedyTrialRequest>,
) -> Result<(StatusCode, Json<SpeedyTrialResponse>), AppError> {
    let clock = crate::repo::speedy_trial::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(SpeedyTrialResponse::from(clock))))
}

/// GET /api/cases/{case_id}/speedy-trial
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/speedy-trial",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Speedy trial clock", body = SpeedyTrialResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "speedy-trial"
)]
pub async fn get_speedy_trial(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<SpeedyTrialResponse>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let clock = crate::repo::speedy_trial::find_by_case_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!(
            "Speedy trial clock not found for case {}", case_id
        )))?;

    Ok(Json(SpeedyTrialResponse::from(clock)))
}

/// PUT /api/cases/{case_id}/speedy-trial
#[utoipa::path(
    put,
    path = "/api/cases/{case_id}/speedy-trial",
    request_body = UpdateSpeedyTrialClockRequest,
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Clock updated", body = SpeedyTrialResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "speedy-trial"
)]
pub async fn update_speedy_trial_clock(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
    Json(body): Json<UpdateSpeedyTrialClockRequest>,
) -> Result<Json<SpeedyTrialResponse>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let clock = crate::repo::speedy_trial::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!(
            "Speedy trial clock not found for case {}", case_id
        )))?;

    Ok(Json(SpeedyTrialResponse::from(clock)))
}

/// Query params for approaching deadline endpoint.
#[derive(Debug, Deserialize)]
pub struct ApproachingParams {
    /// Number of days within which to flag (default: 14).
    #[serde(default = "default_approaching_days")]
    pub within_days: i64,
}

fn default_approaching_days() -> i64 {
    14
}

/// GET /api/speedy-trial/deadlines/approaching
#[utoipa::path(
    get,
    path = "/api/speedy-trial/deadlines/approaching",
    params(
        ("within_days" = Option<i64>, Query, description = "Days threshold (default 14)"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Cases approaching deadline", body = Vec<SpeedyTrialResponse>)
    ),
    tag = "speedy-trial"
)]
pub async fn list_approaching(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<ApproachingParams>,
) -> Result<Json<Vec<SpeedyTrialResponse>>, AppError> {
    let clocks = crate::repo::speedy_trial::list_approaching(
        &pool, &court.0, params.within_days,
    ).await?;

    let responses: Vec<SpeedyTrialResponse> = clocks
        .into_iter()
        .map(SpeedyTrialResponse::from)
        .collect();

    Ok(Json(responses))
}

/// GET /api/speedy-trial/violations
#[utoipa::path(
    get,
    path = "/api/speedy-trial/violations",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Cases that violated the deadline", body = Vec<SpeedyTrialResponse>)
    ),
    tag = "speedy-trial"
)]
pub async fn list_violations(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<SpeedyTrialResponse>>, AppError> {
    let clocks = crate::repo::speedy_trial::list_violations(&pool, &court.0).await?;

    let responses: Vec<SpeedyTrialResponse> = clocks
        .into_iter()
        .map(SpeedyTrialResponse::from)
        .collect();

    Ok(Json(responses))
}

// ── Excludable Delay handlers ────────────────────────────────────

/// POST /api/cases/{id}/speedy-trial/exclude
#[utoipa::path(
    post,
    path = "/api/cases/{id}/speedy-trial/exclude",
    request_body = CreateExcludableDelayRequest,
    params(
        ("id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Delay created", body = ExcludableDelayResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "speedy-trial"
)]
pub async fn create_delay(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
    Json(body): Json<CreateExcludableDelayRequest>,
) -> Result<(StatusCode, Json<ExcludableDelayResponse>), AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if body.reason.trim().is_empty() {
        return Err(AppError::bad_request("reason must not be empty"));
    }

    let delay = crate::repo::speedy_trial::create_delay(&pool, &court.0, uuid, body).await?;
    Ok((StatusCode::CREATED, Json(ExcludableDelayResponse::from(delay))))
}

/// GET /api/cases/{case_id}/speedy-trial/delays
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/speedy-trial/delays",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Delays for case", body = Vec<ExcludableDelayResponse>)
    ),
    tag = "speedy-trial"
)]
pub async fn list_delays(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<ExcludableDelayResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let delays = crate::repo::speedy_trial::list_delays_by_case(&pool, &court.0, uuid).await?;
    let responses: Vec<ExcludableDelayResponse> = delays
        .into_iter()
        .map(ExcludableDelayResponse::from)
        .collect();

    Ok(Json(responses))
}

/// GET /api/cases/{case_id}/speedy-trial/deadline-check
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/speedy-trial/deadline-check",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Deadline check result", body = DeadlineCheckResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "speedy-trial"
)]
pub async fn deadline_check(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<DeadlineCheckResponse>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let clock = crate::repo::speedy_trial::find_by_case_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!(
            "Speedy trial record not found for case {}", case_id
        )))?;

    let deadline_days: i64 = 70; // Speedy Trial Act default

    let total_excluded: i64 = sqlx::query_scalar!(
        r#"SELECT COALESCE(SUM(days_excluded)::BIGINT, 0) as "sum!: i64" FROM excludable_delays WHERE court_id = $1 AND case_id = $2"#,
        &court.0,
        uuid,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    // Use the earliest available milestone as the clock start date
    let start = clock.arrest_date
        .or(clock.indictment_date)
        .or(clock.arraignment_date)
        .unwrap_or(clock.trial_start_deadline - chrono::Duration::days(deadline_days));

    let now = chrono::Utc::now();
    let calendar_days = (now - start).num_days();
    let days_elapsed = (calendar_days - total_excluded).max(0);
    let days_remaining = (deadline_days - days_elapsed).max(0);
    let is_approaching = days_remaining <= 14;
    let is_violated = days_remaining == 0;

    Ok(Json(DeadlineCheckResponse {
        case_id,
        days_elapsed,
        days_remaining,
        deadline_days,
        is_approaching,
        is_violated,
    }))
}

/// DELETE /api/speedy-trial/delays/{id}
#[utoipa::path(
    delete,
    path = "/api/speedy-trial/delays/{id}",
    params(
        ("id" = String, Path, description = "Delay UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Delay deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "speedy-trial"
)]
pub async fn delete_delay(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::speedy_trial::delete_delay(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Delay {} not found", id)))
    }
}
