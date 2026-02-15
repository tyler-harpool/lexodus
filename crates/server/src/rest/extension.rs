use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CreateExtensionRequest, ExtensionResponse, UpdateExtensionRulingRequest,
    is_valid_extension_status, EXTENSION_STATUSES,
};
use crate::tenant::CourtId;

/// POST /api/deadlines/{deadline_id}/extensions
#[utoipa::path(
    post,
    path = "/api/deadlines/{deadline_id}/extensions",
    request_body = CreateExtensionRequest,
    params(
        ("deadline_id" = String, Path, description = "Deadline UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Extension request created", body = ExtensionResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Deadline not found", body = AppError)
    ),
    tag = "extensions"
)]
pub async fn request_extension(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(deadline_id): Path<String>,
    Json(body): Json<CreateExtensionRequest>,
) -> Result<(StatusCode, Json<ExtensionResponse>), AppError> {
    let deadline_uuid = Uuid::parse_str(&deadline_id)
        .map_err(|_| AppError::bad_request("Invalid deadline UUID format"))?;

    if body.reason.trim().is_empty() {
        return Err(AppError::bad_request("reason must not be empty"));
    }

    if body.requested_by.trim().is_empty() {
        return Err(AppError::bad_request("requested_by must not be empty"));
    }

    // Verify the deadline exists in this court
    crate::repo::deadline::find_by_id(&pool, &court.0, deadline_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Deadline {} not found", deadline_id)))?;

    let ext = crate::repo::extension_request::create(
        &pool, &court.0, deadline_uuid, body,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(ExtensionResponse::from(ext))))
}

/// PATCH /api/extensions/{extension_id}/ruling
#[utoipa::path(
    patch,
    path = "/api/extensions/{extension_id}/ruling",
    request_body = UpdateExtensionRulingRequest,
    params(
        ("extension_id" = String, Path, description = "Extension request UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Ruling updated", body = ExtensionResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Extension not found", body = AppError)
    ),
    tag = "extensions"
)]
pub async fn rule_on_extension(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(extension_id): Path<String>,
    Json(body): Json<UpdateExtensionRulingRequest>,
) -> Result<Json<ExtensionResponse>, AppError> {
    let uuid = Uuid::parse_str(&extension_id)
        .map_err(|_| AppError::bad_request("Invalid extension UUID format"))?;

    if !is_valid_extension_status(&body.status) {
        return Err(AppError::bad_request(format!(
            "Invalid status: {}. Valid values: {}",
            body.status,
            EXTENSION_STATUSES.join(", ")
        )));
    }

    if body.ruling_by.trim().is_empty() {
        return Err(AppError::bad_request("ruling_by must not be empty"));
    }

    let ext = crate::repo::extension_request::update_ruling(
        &pool,
        &court.0,
        uuid,
        &body.status,
        &body.ruling_by,
        body.new_deadline_date,
    )
    .await?
    .ok_or_else(|| AppError::not_found(format!("Extension {} not found", extension_id)))?;

    Ok(Json(ExtensionResponse::from(ext)))
}

/// GET /api/extensions/{id}
#[utoipa::path(
    get,
    path = "/api/extensions/{id}",
    params(
        ("id" = String, Path, description = "Extension request UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Extension found", body = ExtensionResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "extensions"
)]
pub async fn get_extension(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<ExtensionResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let ext = crate::repo::extension_request::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Extension {} not found", id)))?;

    Ok(Json(ExtensionResponse::from(ext)))
}

/// GET /api/deadlines/{deadline_id}/extensions
#[utoipa::path(
    get,
    path = "/api/deadlines/{deadline_id}/extensions",
    params(
        ("deadline_id" = String, Path, description = "Deadline UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Extensions for deadline", body = Vec<ExtensionResponse>)
    ),
    tag = "extensions"
)]
pub async fn list_extensions(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(deadline_id): Path<String>,
) -> Result<Json<Vec<ExtensionResponse>>, AppError> {
    let uuid = Uuid::parse_str(&deadline_id)
        .map_err(|_| AppError::bad_request("Invalid deadline UUID format"))?;

    let exts = crate::repo::extension_request::list_by_deadline(&pool, &court.0, uuid).await?;
    let response: Vec<ExtensionResponse> = exts.into_iter().map(ExtensionResponse::from).collect();

    Ok(Json(response))
}

/// GET /api/extensions/pending
#[utoipa::path(
    get,
    path = "/api/extensions/pending",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Pending extensions", body = Vec<ExtensionResponse>)
    ),
    tag = "extensions"
)]
pub async fn list_pending_extensions(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<ExtensionResponse>>, AppError> {
    let exts = crate::repo::extension_request::list_pending(&pool, &court.0).await?;
    let response: Vec<ExtensionResponse> = exts.into_iter().map(ExtensionResponse::from).collect();

    Ok(Json(response))
}
