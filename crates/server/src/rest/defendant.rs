use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CreateDefendantRequest, DefendantResponse, UpdateDefendantRequest,
    is_valid_bail_type, is_valid_citizenship_status, is_valid_custody_status,
    BAIL_TYPES, CITIZENSHIP_STATUSES, CUSTODY_STATUSES,
};
use crate::tenant::CourtId;

/// POST /api/defendants
#[utoipa::path(
    post,
    path = "/api/defendants",
    request_body = CreateDefendantRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Defendant created", body = DefendantResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "defendants"
)]
pub async fn create_defendant(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateDefendantRequest>,
) -> Result<(StatusCode, Json<DefendantResponse>), AppError> {
    if body.name.trim().is_empty() {
        return Err(AppError::bad_request("name must not be empty"));
    }

    if let Some(ref cs) = body.citizenship_status {
        if !is_valid_citizenship_status(cs) {
            return Err(AppError::bad_request(format!(
                "Invalid citizenship_status: {}. Valid values: {}",
                cs,
                CITIZENSHIP_STATUSES.join(", ")
            )));
        }
    }

    if let Some(ref cs) = body.custody_status {
        if !is_valid_custody_status(cs) {
            return Err(AppError::bad_request(format!(
                "Invalid custody_status: {}. Valid values: {}",
                cs,
                CUSTODY_STATUSES.join(", ")
            )));
        }
    }

    if let Some(ref bt) = body.bail_type {
        if !is_valid_bail_type(bt) {
            return Err(AppError::bad_request(format!(
                "Invalid bail_type: {}. Valid values: {}",
                bt,
                BAIL_TYPES.join(", ")
            )));
        }
    }

    let defendant = crate::repo::defendant::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(DefendantResponse::from(defendant))))
}

/// GET /api/defendants/{id}
#[utoipa::path(
    get,
    path = "/api/defendants/{id}",
    params(
        ("id" = String, Path, description = "Defendant UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Defendant found", body = DefendantResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "defendants"
)]
pub async fn get_defendant(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<DefendantResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let defendant = crate::repo::defendant::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Defendant {} not found", id)))?;

    Ok(Json(DefendantResponse::from(defendant)))
}

/// GET /api/defendants/case/{case_id}
#[utoipa::path(
    get,
    path = "/api/defendants/case/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Defendants for case", body = Vec<DefendantResponse>)
    ),
    tag = "defendants"
)]
pub async fn list_defendants_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<DefendantResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let defendants = crate::repo::defendant::list_by_case(&pool, &court.0, uuid).await?;
    let responses: Vec<DefendantResponse> = defendants.into_iter().map(DefendantResponse::from).collect();

    Ok(Json(responses))
}

/// PUT /api/defendants/{id}
#[utoipa::path(
    put,
    path = "/api/defendants/{id}",
    request_body = UpdateDefendantRequest,
    params(
        ("id" = String, Path, description = "Defendant UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Defendant updated", body = DefendantResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "defendants"
)]
pub async fn update_defendant(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateDefendantRequest>,
) -> Result<Json<DefendantResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref n) = body.name {
        if n.trim().is_empty() {
            return Err(AppError::bad_request("name must not be empty"));
        }
    }

    if let Some(ref cs) = body.citizenship_status {
        if !is_valid_citizenship_status(cs) {
            return Err(AppError::bad_request(format!(
                "Invalid citizenship_status: {}. Valid values: {}",
                cs,
                CITIZENSHIP_STATUSES.join(", ")
            )));
        }
    }

    if let Some(ref cs) = body.custody_status {
        if !is_valid_custody_status(cs) {
            return Err(AppError::bad_request(format!(
                "Invalid custody_status: {}. Valid values: {}",
                cs,
                CUSTODY_STATUSES.join(", ")
            )));
        }
    }

    if let Some(ref bt) = body.bail_type {
        if !is_valid_bail_type(bt) {
            return Err(AppError::bad_request(format!(
                "Invalid bail_type: {}. Valid values: {}",
                bt,
                BAIL_TYPES.join(", ")
            )));
        }
    }

    let defendant = crate::repo::defendant::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Defendant {} not found", id)))?;

    Ok(Json(DefendantResponse::from(defendant)))
}

/// DELETE /api/defendants/{id}
#[utoipa::path(
    delete,
    path = "/api/defendants/{id}",
    params(
        ("id" = String, Path, description = "Defendant UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Defendant deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "defendants"
)]
pub async fn delete_defendant(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::defendant::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Defendant {} not found", id)))
    }
}
