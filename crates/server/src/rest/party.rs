use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CreatePartyRequest, PartyResponse, RepresentationResponse,
    UpdatePartyRequest, UpdatePartyStatusRequest,
};
use crate::tenant::CourtId;

/// POST /api/parties
#[utoipa::path(
    post,
    path = "/api/parties",
    request_body = CreatePartyRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Party created", body = PartyResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "parties"
)]
pub async fn create_party(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreatePartyRequest>,
) -> Result<(StatusCode, Json<PartyResponse>), AppError> {
    let party = crate::repo::party::create(&pool, &court.0, &body).await?;
    Ok((StatusCode::CREATED, Json(PartyResponse::from(party))))
}

/// GET /api/parties/{id}
#[utoipa::path(
    get,
    path = "/api/parties/{id}",
    params(
        ("id" = String, Path, description = "Party UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Party found", body = PartyResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "parties"
)]
pub async fn get_party(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<PartyResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let party = crate::repo::party::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Party {} not found", id)))?;

    let reps = crate::repo::representation::list_by_party(&pool, &court.0, uuid).await?;
    let mut response = PartyResponse::from(party);
    response.attorneys = reps.into_iter().map(RepresentationResponse::from).collect();

    Ok(Json(response))
}

/// PUT /api/parties/{id}
#[utoipa::path(
    put,
    path = "/api/parties/{id}",
    params(
        ("id" = String, Path, description = "Party UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = UpdatePartyRequest,
    responses(
        (status = 200, description = "Party updated", body = PartyResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "parties"
)]
pub async fn update_party(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdatePartyRequest>,
) -> Result<Json<PartyResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let party = crate::repo::party::update(&pool, &court.0, uuid, &body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Party {} not found", id)))?;

    Ok(Json(PartyResponse::from(party)))
}

/// DELETE /api/parties/{id}
#[utoipa::path(
    delete,
    path = "/api/parties/{id}",
    params(
        ("id" = String, Path, description = "Party UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Party deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "parties"
)]
pub async fn delete_party(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::party::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Party {} not found", id)))
    }
}

/// GET /api/parties/case/{case_id}
#[utoipa::path(
    get,
    path = "/api/parties/case/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Parties for case", body = Vec<PartyResponse>)
    ),
    tag = "parties"
)]
pub async fn list_parties_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<PartyResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case_id UUID"))?;

    let parties = crate::repo::party::list_full_by_case(&pool, &court.0, uuid).await?;

    let mut responses: Vec<PartyResponse> = Vec::with_capacity(parties.len());
    for party in parties {
        let party_id = party.id;
        let mut resp = PartyResponse::from(party);
        let reps = crate::repo::representation::list_by_party(&pool, &court.0, party_id).await?;
        resp.attorneys = reps.into_iter().map(RepresentationResponse::from).collect();
        responses.push(resp);
    }

    Ok(Json(responses))
}

/// GET /api/parties/attorney/{attorney_id}
#[utoipa::path(
    get,
    path = "/api/parties/attorney/{attorney_id}",
    params(
        ("attorney_id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Parties represented by attorney", body = Vec<PartyResponse>)
    ),
    tag = "parties"
)]
pub async fn list_parties_by_attorney(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(attorney_id): Path<String>,
) -> Result<Json<Vec<PartyResponse>>, AppError> {
    let uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| AppError::bad_request("Invalid attorney_id UUID"))?;

    let parties = crate::repo::party::list_by_attorney(&pool, &court.0, uuid).await?;
    let responses: Vec<PartyResponse> = parties.into_iter().map(PartyResponse::from).collect();

    Ok(Json(responses))
}

/// PATCH /api/parties/{id}/status
#[utoipa::path(
    patch,
    path = "/api/parties/{id}/status",
    params(
        ("id" = String, Path, description = "Party UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = UpdatePartyStatusRequest,
    responses(
        (status = 200, description = "Status updated", body = PartyResponse),
        (status = 400, description = "Invalid status", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "parties"
)]
pub async fn update_party_status(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdatePartyStatusRequest>,
) -> Result<Json<PartyResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let party = crate::repo::party::update_status(&pool, &court.0, uuid, &body.status)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Party {} not found", id)))?;

    Ok(Json(PartyResponse::from(party)))
}

/// GET /api/parties/{id}/needs-service
#[utoipa::path(
    get,
    path = "/api/parties/{id}/needs-service",
    params(
        ("id" = String, Path, description = "Party UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Service check result", body = serde_json::Value)
    ),
    tag = "parties"
)]
pub async fn check_needs_service(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let needs = crate::repo::party::needs_service(&pool, &court.0, uuid).await?;

    Ok(Json(serde_json::json!({ "needs_service": needs })))
}

/// GET /api/parties/{id}/lead-counsel
#[utoipa::path(
    get,
    path = "/api/parties/{id}/lead-counsel",
    params(
        ("id" = String, Path, description = "Party UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Lead counsel found", body = RepresentationResponse),
        (status = 404, description = "No lead counsel", body = AppError)
    ),
    tag = "parties"
)]
pub async fn get_lead_counsel(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<RepresentationResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let rep = crate::repo::party::get_lead_counsel(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found("No lead counsel found for this party"))?;

    Ok(Json(RepresentationResponse::from(rep)))
}

/// GET /api/parties/{id}/is-represented
#[utoipa::path(
    get,
    path = "/api/parties/{id}/is-represented",
    params(
        ("id" = String, Path, description = "Party UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Representation check result", body = serde_json::Value)
    ),
    tag = "parties"
)]
pub async fn check_is_represented(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let represented = crate::repo::party::is_represented(&pool, &court.0, uuid).await?;

    Ok(Json(serde_json::json!({ "is_represented": represented })))
}

/// GET /api/parties/unrepresented
#[utoipa::path(
    get,
    path = "/api/parties/unrepresented",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Unrepresented parties", body = Vec<PartyResponse>)
    ),
    tag = "parties"
)]
pub async fn list_unrepresented(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<PartyResponse>>, AppError> {
    let parties = crate::repo::party::list_unrepresented(&pool, &court.0).await?;
    let responses: Vec<PartyResponse> = parties.into_iter().map(PartyResponse::from).collect();

    Ok(Json(responses))
}
