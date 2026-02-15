use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{AppError, CreateSignatureRequest, JudgeSignatureResponse};
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// POST /api/signatures
// ---------------------------------------------------------------------------

/// Upload (create or update) a judge's electronic signature.
#[utoipa::path(
    post,
    path = "/api/signatures",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = CreateSignatureRequest,
    responses(
        (status = 201, description = "Signature uploaded", body = JudgeSignatureResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "signatures"
)]
pub async fn upload_signature(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateSignatureRequest>,
) -> Result<(StatusCode, Json<JudgeSignatureResponse>), AppError> {
    if body.signature_data.trim().is_empty() {
        return Err(AppError::bad_request("signature_data cannot be empty"));
    }

    let sig = crate::repo::signature::create_or_update(
        &pool, &court.0, body.judge_id, &body.signature_data,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(JudgeSignatureResponse::from(sig))))
}

// ---------------------------------------------------------------------------
// GET /api/signatures/{judge_id}
// ---------------------------------------------------------------------------

/// Get a judge's electronic signature.
#[utoipa::path(
    get,
    path = "/api/signatures/{judge_id}",
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Signature found", body = JudgeSignatureResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "signatures"
)]
pub async fn get_signature(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(judge_id): Path<String>,
) -> Result<Json<JudgeSignatureResponse>, AppError> {
    let uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let sig = crate::repo::signature::find_by_judge(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Signature for judge {} not found", judge_id)))?;

    Ok(Json(JudgeSignatureResponse::from(sig)))
}
