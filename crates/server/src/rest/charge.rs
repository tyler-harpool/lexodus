use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, ChargeResponse, CreateChargeRequest, UpdateChargeRequest,
    is_valid_plea_type, is_valid_verdict_type,
    PLEA_TYPES, VERDICT_TYPES,
};
use crate::tenant::CourtId;

/// POST /api/charges
#[utoipa::path(
    post,
    path = "/api/charges",
    request_body = CreateChargeRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Charge created", body = ChargeResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "charges"
)]
pub async fn create_charge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateChargeRequest>,
) -> Result<(StatusCode, Json<ChargeResponse>), AppError> {
    if body.statute.trim().is_empty() {
        return Err(AppError::bad_request("statute must not be empty"));
    }

    if body.offense_description.trim().is_empty() {
        return Err(AppError::bad_request("offense_description must not be empty"));
    }

    if let Some(ref p) = body.plea {
        if !is_valid_plea_type(p) {
            return Err(AppError::bad_request(format!(
                "Invalid plea: {}. Valid values: {}",
                p,
                PLEA_TYPES.join(", ")
            )));
        }
    }

    if let Some(ref v) = body.verdict {
        if !is_valid_verdict_type(v) {
            return Err(AppError::bad_request(format!(
                "Invalid verdict: {}. Valid values: {}",
                v,
                VERDICT_TYPES.join(", ")
            )));
        }
    }

    let charge = crate::repo::charge::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(ChargeResponse::from(charge))))
}

/// GET /api/charges/{id}
#[utoipa::path(
    get,
    path = "/api/charges/{id}",
    params(
        ("id" = String, Path, description = "Charge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Charge found", body = ChargeResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "charges"
)]
pub async fn get_charge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<ChargeResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let charge = crate::repo::charge::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Charge {} not found", id)))?;

    Ok(Json(ChargeResponse::from(charge)))
}

/// GET /api/charges/defendant/{defendant_id}
#[utoipa::path(
    get,
    path = "/api/charges/defendant/{defendant_id}",
    params(
        ("defendant_id" = String, Path, description = "Defendant UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Charges for defendant", body = Vec<ChargeResponse>)
    ),
    tag = "charges"
)]
pub async fn list_charges_by_defendant(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(defendant_id): Path<String>,
) -> Result<Json<Vec<ChargeResponse>>, AppError> {
    let uuid = Uuid::parse_str(&defendant_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let charges = crate::repo::charge::list_by_defendant(&pool, &court.0, uuid).await?;
    let responses: Vec<ChargeResponse> = charges.into_iter().map(ChargeResponse::from).collect();

    Ok(Json(responses))
}

/// PUT /api/charges/{id}
#[utoipa::path(
    put,
    path = "/api/charges/{id}",
    request_body = UpdateChargeRequest,
    params(
        ("id" = String, Path, description = "Charge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Charge updated", body = ChargeResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "charges"
)]
pub async fn update_charge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateChargeRequest>,
) -> Result<Json<ChargeResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref s) = body.statute {
        if s.trim().is_empty() {
            return Err(AppError::bad_request("statute must not be empty"));
        }
    }

    if let Some(ref d) = body.offense_description {
        if d.trim().is_empty() {
            return Err(AppError::bad_request("offense_description must not be empty"));
        }
    }

    if let Some(ref p) = body.plea {
        if !is_valid_plea_type(p) {
            return Err(AppError::bad_request(format!(
                "Invalid plea: {}. Valid values: {}",
                p,
                PLEA_TYPES.join(", ")
            )));
        }
    }

    if let Some(ref v) = body.verdict {
        if !is_valid_verdict_type(v) {
            return Err(AppError::bad_request(format!(
                "Invalid verdict: {}. Valid values: {}",
                v,
                VERDICT_TYPES.join(", ")
            )));
        }
    }

    let charge = crate::repo::charge::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Charge {} not found", id)))?;

    Ok(Json(ChargeResponse::from(charge)))
}

/// DELETE /api/charges/{id}
#[utoipa::path(
    delete,
    path = "/api/charges/{id}",
    params(
        ("id" = String, Path, description = "Charge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Charge deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "charges"
)]
pub async fn delete_charge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::charge::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Charge {} not found", id)))
    }
}
