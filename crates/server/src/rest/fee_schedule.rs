use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::AppError;
use crate::repo::fee_schedule::{
    CreateFeeRequest, FeeScheduleEntry, UpdateFeeRequest,
};
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// GET /api/fee-schedule — list active fees for the court
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/api/fee-schedule",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Active fee schedule entries", body = Vec<FeeScheduleEntry>)
    ),
    tag = "fee-schedule"
)]
pub async fn list_fees(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<FeeScheduleEntry>>, AppError> {
    let entries = crate::repo::fee_schedule::list_active(&pool, &court.0).await?;
    Ok(Json(entries))
}

// ---------------------------------------------------------------------------
// GET /api/fee-schedule/{id} — get a single fee entry
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/api/fee-schedule/{id}",
    params(
        ("id" = String, Path, description = "Fee schedule entry UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Fee schedule entry", body = FeeScheduleEntry),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "fee-schedule"
)]
pub async fn get_fee(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<FeeScheduleEntry>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let entry = crate::repo::fee_schedule::get_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Fee schedule entry {} not found", id)))?;

    Ok(Json(entry))
}

// ---------------------------------------------------------------------------
// POST /api/fee-schedule — create a new fee entry
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/fee-schedule",
    request_body = CreateFeeRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Fee schedule entry created", body = FeeScheduleEntry),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 409, description = "Duplicate fee_id + effective_date", body = AppError)
    ),
    tag = "fee-schedule"
)]
pub async fn create_fee(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateFeeRequest>,
) -> Result<(StatusCode, Json<FeeScheduleEntry>), AppError> {
    // Validate required fields are not blank
    if body.fee_id.trim().is_empty() {
        return Err(AppError::bad_request("fee_id must not be empty"));
    }
    if body.category.trim().is_empty() {
        return Err(AppError::bad_request("category must not be empty"));
    }
    if body.description.trim().is_empty() {
        return Err(AppError::bad_request("description must not be empty"));
    }
    if body.amount_cents < 0 {
        return Err(AppError::bad_request("amount_cents must not be negative"));
    }

    let entry = crate::repo::fee_schedule::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

// ---------------------------------------------------------------------------
// PATCH /api/fee-schedule/{id} — partially update a fee entry
// ---------------------------------------------------------------------------

#[utoipa::path(
    patch,
    path = "/api/fee-schedule/{id}",
    request_body = UpdateFeeRequest,
    params(
        ("id" = String, Path, description = "Fee schedule entry UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Fee schedule entry updated", body = FeeScheduleEntry),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "fee-schedule"
)]
pub async fn update_fee(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateFeeRequest>,
) -> Result<Json<FeeScheduleEntry>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(cents) = body.amount_cents {
        if cents < 0 {
            return Err(AppError::bad_request("amount_cents must not be negative"));
        }
    }

    let entry = crate::repo::fee_schedule::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Fee schedule entry {} not found", id)))?;

    Ok(Json(entry))
}

// ---------------------------------------------------------------------------
// DELETE /api/fee-schedule/{id} — soft-delete (set active=false)
// ---------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/api/fee-schedule/{id}",
    params(
        ("id" = String, Path, description = "Fee schedule entry UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Fee schedule entry deactivated"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "fee-schedule"
)]
pub async fn delete_fee(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let updated = crate::repo::fee_schedule::soft_delete(&pool, &court.0, uuid).await?;

    if updated {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Fee schedule entry {} not found", id)))
    }
}
