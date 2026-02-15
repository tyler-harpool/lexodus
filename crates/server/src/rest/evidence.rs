use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CreateEvidenceRequest, EvidenceResponse, UpdateEvidenceRequest,
    CreateCustodyTransferRequest, CustodyTransferResponse,
    is_valid_evidence_type, EVIDENCE_TYPES,
};
use crate::tenant::CourtId;

// ── Evidence handlers ──────────────────────────────────────────────

/// POST /api/evidence
#[utoipa::path(
    post,
    path = "/api/evidence",
    request_body = CreateEvidenceRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Evidence created", body = EvidenceResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "evidence"
)]
pub async fn create_evidence(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateEvidenceRequest>,
) -> Result<(StatusCode, Json<EvidenceResponse>), AppError> {
    if body.description.trim().is_empty() {
        return Err(AppError::bad_request("description must not be empty"));
    }

    if !is_valid_evidence_type(&body.evidence_type) {
        return Err(AppError::bad_request(format!(
            "Invalid evidence_type: {}. Valid values: {}",
            body.evidence_type,
            EVIDENCE_TYPES.join(", ")
        )));
    }

    let evidence = crate::repo::evidence::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(EvidenceResponse::from(evidence))))
}

/// GET /api/evidence/{id}
#[utoipa::path(
    get,
    path = "/api/evidence/{id}",
    params(
        ("id" = String, Path, description = "Evidence UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Evidence found", body = EvidenceResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "evidence"
)]
pub async fn get_evidence(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<EvidenceResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let evidence = crate::repo::evidence::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Evidence {} not found", id)))?;

    Ok(Json(EvidenceResponse::from(evidence)))
}

/// GET /api/evidence/case/{case_id}
#[utoipa::path(
    get,
    path = "/api/evidence/case/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Evidence for case", body = Vec<EvidenceResponse>)
    ),
    tag = "evidence"
)]
pub async fn list_evidence_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<EvidenceResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let evidence = crate::repo::evidence::list_by_case(&pool, &court.0, uuid).await?;
    let responses: Vec<EvidenceResponse> = evidence.into_iter().map(EvidenceResponse::from).collect();

    Ok(Json(responses))
}

/// PUT /api/evidence/{id}
#[utoipa::path(
    put,
    path = "/api/evidence/{id}",
    request_body = UpdateEvidenceRequest,
    params(
        ("id" = String, Path, description = "Evidence UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Evidence updated", body = EvidenceResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "evidence"
)]
pub async fn update_evidence(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateEvidenceRequest>,
) -> Result<Json<EvidenceResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref d) = body.description {
        if d.trim().is_empty() {
            return Err(AppError::bad_request("description must not be empty"));
        }
    }

    if let Some(ref et) = body.evidence_type {
        if !is_valid_evidence_type(et) {
            return Err(AppError::bad_request(format!(
                "Invalid evidence_type: {}. Valid values: {}",
                et,
                EVIDENCE_TYPES.join(", ")
            )));
        }
    }

    let evidence = crate::repo::evidence::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Evidence {} not found", id)))?;

    Ok(Json(EvidenceResponse::from(evidence)))
}

/// DELETE /api/evidence/{id}
#[utoipa::path(
    delete,
    path = "/api/evidence/{id}",
    params(
        ("id" = String, Path, description = "Evidence UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Evidence deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "evidence"
)]
pub async fn delete_evidence(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::evidence::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Evidence {} not found", id)))
    }
}

// ── Custody Transfer handlers ──────────────────────────────────────

/// POST /api/custody-transfers
#[utoipa::path(
    post,
    path = "/api/custody-transfers",
    request_body = CreateCustodyTransferRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Custody transfer created", body = CustodyTransferResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "evidence"
)]
pub async fn create_custody_transfer(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateCustodyTransferRequest>,
) -> Result<(StatusCode, Json<CustodyTransferResponse>), AppError> {
    if body.transferred_from.trim().is_empty() {
        return Err(AppError::bad_request("transferred_from must not be empty"));
    }

    if body.transferred_to.trim().is_empty() {
        return Err(AppError::bad_request("transferred_to must not be empty"));
    }

    let transfer = crate::repo::custody_transfer::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(CustodyTransferResponse::from(transfer))))
}

/// GET /api/custody-transfers/{id}
#[utoipa::path(
    get,
    path = "/api/custody-transfers/{id}",
    params(
        ("id" = String, Path, description = "Custody transfer UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Custody transfer found", body = CustodyTransferResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "evidence"
)]
pub async fn get_custody_transfer(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<CustodyTransferResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let transfer = crate::repo::custody_transfer::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Custody transfer {} not found", id)))?;

    Ok(Json(CustodyTransferResponse::from(transfer)))
}

/// GET /api/custody-transfers/evidence/{evidence_id}
#[utoipa::path(
    get,
    path = "/api/custody-transfers/evidence/{evidence_id}",
    params(
        ("evidence_id" = String, Path, description = "Evidence UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Custody chain for evidence", body = Vec<CustodyTransferResponse>)
    ),
    tag = "evidence"
)]
pub async fn list_custody_transfers_by_evidence(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(evidence_id): Path<String>,
) -> Result<Json<Vec<CustodyTransferResponse>>, AppError> {
    let uuid = Uuid::parse_str(&evidence_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let transfers = crate::repo::custody_transfer::list_by_evidence(&pool, &court.0, uuid).await?;
    let responses: Vec<CustodyTransferResponse> = transfers.into_iter().map(CustodyTransferResponse::from).collect();

    Ok(Json(responses))
}

/// DELETE /api/custody-transfers/{id}
#[utoipa::path(
    delete,
    path = "/api/custody-transfers/{id}",
    params(
        ("id" = String, Path, description = "Custody transfer UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Custody transfer deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "evidence"
)]
pub async fn delete_custody_transfer(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::custody_transfer::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Custody transfer {} not found", id)))
    }
}
