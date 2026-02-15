use axum::{
    extract::State,
    Json,
};
use sqlx::{Pool, Postgres};

use shared_types::{AppError, MigrateRepresentationRequest, RepresentationResponse};
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// POST /api/representations/migrate
// ---------------------------------------------------------------------------

/// Migrate representation from one attorney to another on a case.
/// This ends the old attorney's active representations and creates new ones
/// for the new attorney on the same parties.
#[utoipa::path(
    post,
    path = "/api/representations/migrate",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = MigrateRepresentationRequest,
    responses(
        (status = 200, description = "Representations migrated", body = Vec<RepresentationResponse>),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "representations"
)]
pub async fn migrate_representation(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<MigrateRepresentationRequest>,
) -> Result<Json<Vec<RepresentationResponse>>, AppError> {
    let new_reps = crate::repo::representation::substitute(
        &pool,
        &court.0,
        body.case_id,
        body.old_attorney_id,
        body.new_attorney_id,
    )
    .await?;

    let response: Vec<RepresentationResponse> = new_reps
        .into_iter()
        .map(RepresentationResponse::from)
        .collect();

    Ok(Json(response))
}
