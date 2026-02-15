use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CaseNoteResponse, CreateCaseNoteRequest, UpdateCaseNoteRequest,
    is_valid_note_type, NOTE_TYPES,
};
use crate::tenant::CourtId;

// ── Case Note handlers ───────────────────────────────────────────

/// POST /api/case-notes
#[utoipa::path(
    post,
    path = "/api/case-notes",
    request_body = CreateCaseNoteRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Case note created", body = CaseNoteResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "case-notes"
)]
pub async fn create_case_note(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateCaseNoteRequest>,
) -> Result<(StatusCode, Json<CaseNoteResponse>), AppError> {
    if body.content.trim().is_empty() {
        return Err(AppError::bad_request("content must not be empty"));
    }

    if body.author.trim().is_empty() {
        return Err(AppError::bad_request("author must not be empty"));
    }

    if !is_valid_note_type(&body.note_type) {
        return Err(AppError::bad_request(format!(
            "Invalid note_type: {}. Valid values: {}",
            body.note_type,
            NOTE_TYPES.join(", ")
        )));
    }

    let note = crate::repo::case_note::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(CaseNoteResponse::from(note))))
}

/// GET /api/case-notes/{id}
#[utoipa::path(
    get,
    path = "/api/case-notes/{id}",
    params(
        ("id" = String, Path, description = "Case note UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Case note found", body = CaseNoteResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "case-notes"
)]
pub async fn get_case_note(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<CaseNoteResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let note = crate::repo::case_note::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Case note {} not found", id)))?;

    Ok(Json(CaseNoteResponse::from(note)))
}

/// GET /api/case-notes/case/{case_id}
#[utoipa::path(
    get,
    path = "/api/case-notes/case/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Notes for case", body = Vec<CaseNoteResponse>)
    ),
    tag = "case-notes"
)]
pub async fn list_case_notes_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<CaseNoteResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let notes = crate::repo::case_note::list_by_case(&pool, &court.0, uuid).await?;
    let responses: Vec<CaseNoteResponse> = notes.into_iter().map(CaseNoteResponse::from).collect();

    Ok(Json(responses))
}

/// PUT /api/case-notes/{id}
#[utoipa::path(
    put,
    path = "/api/case-notes/{id}",
    request_body = UpdateCaseNoteRequest,
    params(
        ("id" = String, Path, description = "Case note UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Case note updated", body = CaseNoteResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "case-notes"
)]
pub async fn update_case_note(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateCaseNoteRequest>,
) -> Result<Json<CaseNoteResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref c) = body.content {
        if c.trim().is_empty() {
            return Err(AppError::bad_request("content must not be empty"));
        }
    }

    if let Some(ref nt) = body.note_type {
        if !is_valid_note_type(nt) {
            return Err(AppError::bad_request(format!(
                "Invalid note_type: {}. Valid values: {}",
                nt,
                NOTE_TYPES.join(", ")
            )));
        }
    }

    let note = crate::repo::case_note::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Case note {} not found", id)))?;

    Ok(Json(CaseNoteResponse::from(note)))
}

/// DELETE /api/case-notes/{id}
#[utoipa::path(
    delete,
    path = "/api/case-notes/{id}",
    params(
        ("id" = String, Path, description = "Case note UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Case note deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "case-notes"
)]
pub async fn delete_case_note(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::case_note::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Case note {} not found", id)))
    }
}
