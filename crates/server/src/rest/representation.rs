use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CreateRepresentationRequest, EndRepresentationRequest,
    RepresentationResponse, SubstituteAttorneyRequest,
};
use crate::tenant::CourtId;

/// POST /api/representations
#[utoipa::path(
    post,
    path = "/api/representations",
    request_body = CreateRepresentationRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Representation created", body = RepresentationResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Attorney or party not found", body = AppError)
    ),
    tag = "representations"
)]
pub async fn add_representation(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateRepresentationRequest>,
) -> Result<(StatusCode, Json<RepresentationResponse>), AppError> {
    let rep = crate::repo::representation::create(&pool, &court.0, &body).await?;
    Ok((StatusCode::CREATED, Json(RepresentationResponse::from(rep))))
}

/// GET /api/representations/{id}
#[utoipa::path(
    get,
    path = "/api/representations/{id}",
    params(
        ("id" = String, Path, description = "Representation UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Representation found", body = RepresentationResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "representations"
)]
pub async fn get_representation(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<RepresentationResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let rep = crate::repo::representation::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Representation {} not found", id)))?;

    Ok(Json(RepresentationResponse::from(rep)))
}

/// POST /api/representations/{id}/end
#[utoipa::path(
    post,
    path = "/api/representations/{id}/end",
    params(
        ("id" = String, Path, description = "Representation UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = EndRepresentationRequest,
    responses(
        (status = 200, description = "Representation ended", body = RepresentationResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "representations"
)]
pub async fn end_representation(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<EndRepresentationRequest>,
) -> Result<Json<RepresentationResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let rep = crate::repo::representation::end_representation(
        &pool,
        &court.0,
        uuid,
        body.reason.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::not_found(format!("Representation {} not found", id)))?;

    Ok(Json(RepresentationResponse::from(rep)))
}

/// GET /api/representations/attorney/{attorney_id}/active
#[utoipa::path(
    get,
    path = "/api/representations/attorney/{attorney_id}/active",
    params(
        ("attorney_id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Active representations", body = Vec<RepresentationResponse>)
    ),
    tag = "representations"
)]
pub async fn list_active_by_attorney(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(attorney_id): Path<String>,
) -> Result<Json<Vec<RepresentationResponse>>, AppError> {
    let uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| AppError::bad_request("Invalid attorney_id UUID"))?;

    let reps = crate::repo::representation::list_active_by_attorney(&pool, &court.0, uuid).await?;
    let responses: Vec<RepresentationResponse> =
        reps.into_iter().map(RepresentationResponse::from).collect();

    Ok(Json(responses))
}

/// GET /api/representations/case/{case_id}
#[utoipa::path(
    get,
    path = "/api/representations/case/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Representations for case", body = Vec<RepresentationResponse>)
    ),
    tag = "representations"
)]
pub async fn list_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<RepresentationResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case_id UUID"))?;

    let reps = crate::repo::representation::list_by_case(&pool, &court.0, uuid).await?;
    let responses: Vec<RepresentationResponse> =
        reps.into_iter().map(RepresentationResponse::from).collect();

    Ok(Json(responses))
}

/// POST /api/representations/substitute
#[utoipa::path(
    post,
    path = "/api/representations/substitute",
    request_body = SubstituteAttorneyRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorney substituted", body = Vec<RepresentationResponse>),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "representations"
)]
pub async fn substitute_attorney(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<SubstituteAttorneyRequest>,
) -> Result<Json<Vec<RepresentationResponse>>, AppError> {
    let case_id = Uuid::parse_str(&body.case_id)
        .map_err(|_| AppError::bad_request("Invalid case_id UUID"))?;
    let old_atty = Uuid::parse_str(&body.old_attorney_id)
        .map_err(|_| AppError::bad_request("Invalid old_attorney_id UUID"))?;
    let new_atty = Uuid::parse_str(&body.new_attorney_id)
        .map_err(|_| AppError::bad_request("Invalid new_attorney_id UUID"))?;

    let new_reps = crate::repo::representation::substitute(
        &pool, &court.0, case_id, old_atty, new_atty,
    )
    .await?;

    let responses: Vec<RepresentationResponse> =
        new_reps.into_iter().map(RepresentationResponse::from).collect();

    Ok(Json(responses))
}
