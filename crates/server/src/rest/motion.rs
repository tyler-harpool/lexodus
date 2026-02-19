use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CreateMotionRequest, MotionResponse, UpdateMotionRequest,
    is_valid_motion_type, is_valid_motion_status,
    MOTION_TYPES, MOTION_STATUSES,
};
use crate::tenant::CourtId;

/// POST /api/motions
#[utoipa::path(
    post,
    path = "/api/motions",
    request_body = CreateMotionRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Motion created", body = MotionResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "motions"
)]
pub async fn create_motion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateMotionRequest>,
) -> Result<(StatusCode, Json<MotionResponse>), AppError> {
    if !is_valid_motion_type(&body.motion_type) {
        return Err(AppError::bad_request(format!(
            "Invalid motion_type: {}. Valid values: {}",
            body.motion_type,
            MOTION_TYPES.join(", ")
        )));
    }

    if body.filed_by.trim().is_empty() {
        return Err(AppError::bad_request("filed_by must not be empty"));
    }

    if body.description.trim().is_empty() {
        return Err(AppError::bad_request("description must not be empty"));
    }

    if let Some(ref s) = body.status {
        if !is_valid_motion_status(s) {
            return Err(AppError::bad_request(format!(
                "Invalid status: {}. Valid values: {}",
                s,
                MOTION_STATUSES.join(", ")
            )));
        }
    }

    let motion = crate::repo::motion::create(&pool, &court.0, body).await?;

    // Auto-create queue item for clerk processing (motions are higher priority)
    let _ = crate::repo::queue::create(
        &pool,
        &court.0,
        "motion",
        2,
        &format!("{} - {}", motion.motion_type, motion.description),
        Some("Motion requires clerk review"),
        "motion",
        motion.id,
        Some(motion.case_id),
        None,
        None,
        None,
        shared_types::pipeline_steps("motion").first().copied().unwrap_or("review"),
    )
    .await;

    Ok((StatusCode::CREATED, Json(MotionResponse::from(motion))))
}

/// GET /api/motions/{id}
#[utoipa::path(
    get,
    path = "/api/motions/{id}",
    params(
        ("id" = String, Path, description = "Motion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Motion found", body = MotionResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "motions"
)]
pub async fn get_motion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<MotionResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let motion = crate::repo::motion::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Motion {} not found", id)))?;

    Ok(Json(MotionResponse::from(motion)))
}

/// GET /api/motions/case/{case_id}
#[utoipa::path(
    get,
    path = "/api/motions/case/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Motions for case", body = Vec<MotionResponse>)
    ),
    tag = "motions"
)]
pub async fn list_motions_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<MotionResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let motions = crate::repo::motion::list_by_case(&pool, &court.0, uuid).await?;
    let responses: Vec<MotionResponse> = motions.into_iter().map(MotionResponse::from).collect();

    Ok(Json(responses))
}

/// PUT /api/motions/{id}
#[utoipa::path(
    put,
    path = "/api/motions/{id}",
    request_body = UpdateMotionRequest,
    params(
        ("id" = String, Path, description = "Motion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Motion updated", body = MotionResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "motions"
)]
pub async fn update_motion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateMotionRequest>,
) -> Result<Json<MotionResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref mt) = body.motion_type {
        if !is_valid_motion_type(mt) {
            return Err(AppError::bad_request(format!(
                "Invalid motion_type: {}. Valid values: {}",
                mt,
                MOTION_TYPES.join(", ")
            )));
        }
    }

    if let Some(ref s) = body.status {
        if !is_valid_motion_status(s) {
            return Err(AppError::bad_request(format!(
                "Invalid status: {}. Valid values: {}",
                s,
                MOTION_STATUSES.join(", ")
            )));
        }
    }

    if let Some(ref fb) = body.filed_by {
        if fb.trim().is_empty() {
            return Err(AppError::bad_request("filed_by must not be empty"));
        }
    }

    if let Some(ref d) = body.description {
        if d.trim().is_empty() {
            return Err(AppError::bad_request("description must not be empty"));
        }
    }

    let motion = crate::repo::motion::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Motion {} not found", id)))?;

    Ok(Json(MotionResponse::from(motion)))
}

/// DELETE /api/motions/{id}
#[utoipa::path(
    delete,
    path = "/api/motions/{id}",
    params(
        ("id" = String, Path, description = "Motion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Motion deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "motions"
)]
pub async fn delete_motion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::motion::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Motion {} not found", id)))
    }
}
