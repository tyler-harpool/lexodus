use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, ConflictCheckResponse, CreateConflictCheckRequest,
    RunConflictCheckRequest, RunConflictCheckResult,
};
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// POST /api/conflict-checks
// ---------------------------------------------------------------------------

/// Create a new conflict check record.
#[utoipa::path(
    post,
    path = "/api/conflict-checks",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = CreateConflictCheckRequest,
    responses(
        (status = 201, description = "Conflict check created", body = ConflictCheckResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "conflict-checks"
)]
pub async fn create_conflict_check(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateConflictCheckRequest>,
) -> Result<(StatusCode, Json<ConflictCheckResponse>), AppError> {
    let check = crate::repo::conflict_check::create(
        &pool,
        &court.0,
        body.attorney_id,
        body.case_id,
        &body.party_names,
        &body.adverse_parties,
        body.notes.as_deref(),
    )
    .await?;

    Ok((StatusCode::CREATED, Json(ConflictCheckResponse::from(check))))
}

// ---------------------------------------------------------------------------
// GET /api/conflict-checks/attorney/{attorney_id}
// ---------------------------------------------------------------------------

/// List all conflict checks for a given attorney.
#[utoipa::path(
    get,
    path = "/api/conflict-checks/attorney/{attorney_id}",
    params(
        ("attorney_id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Conflict checks", body = Vec<ConflictCheckResponse>)
    ),
    tag = "conflict-checks"
)]
pub async fn list_by_attorney(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(attorney_id): Path<String>,
) -> Result<Json<Vec<ConflictCheckResponse>>, AppError> {
    let uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let checks = crate::repo::conflict_check::list_by_attorney(&pool, &court.0, uuid).await?;
    let response: Vec<ConflictCheckResponse> = checks.into_iter().map(ConflictCheckResponse::from).collect();
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// POST /api/conflict-checks/check
// ---------------------------------------------------------------------------

/// Run a live conflict check against existing representations.
#[utoipa::path(
    post,
    path = "/api/conflict-checks/check",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = RunConflictCheckRequest,
    responses(
        (status = 200, description = "Conflict check result", body = RunConflictCheckResult)
    ),
    tag = "conflict-checks"
)]
pub async fn run_conflict_check(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<RunConflictCheckRequest>,
) -> Result<Json<RunConflictCheckResult>, AppError> {
    let conflicts = crate::repo::conflict_check::run_check(
        &pool, &court.0, body.attorney_id, &body.party_names,
    )
    .await?;

    Ok(Json(RunConflictCheckResult {
        has_conflict: !conflicts.is_empty(),
        conflicts,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/conflict-checks/{id}/clear
// ---------------------------------------------------------------------------

/// Clear (resolve) a conflict check.
#[utoipa::path(
    post,
    path = "/api/conflict-checks/{id}/clear",
    params(
        ("id" = String, Path, description = "Conflict check UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Conflict check cleared", body = ConflictCheckResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "conflict-checks"
)]
pub async fn clear_conflict(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<ConflictCheckResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let check = crate::repo::conflict_check::clear(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Conflict check {} not found", id)))?;

    Ok(Json(ConflictCheckResponse::from(check)))
}
