use axum::extract::{Path, State};
use axum::Json;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{AppError, NefResponse};
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// GET /api/filings/{filing_id}/nef
// ---------------------------------------------------------------------------

/// GET /api/filings/{filing_id}/nef
///
/// Retrieve the persisted Notice of Electronic Filing for a given filing.
#[utoipa::path(
    get,
    path = "/api/filings/{filing_id}/nef",
    params(
        ("filing_id" = String, Path, description = "Filing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "NEF found", body = NefResponse),
        (status = 404, description = "NEF not found", body = AppError)
    ),
    tag = "nefs"
)]
pub async fn get_nef(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(filing_id): Path<String>,
) -> Result<Json<NefResponse>, AppError> {
    let filing_uuid = Uuid::parse_str(&filing_id)
        .map_err(|_| AppError::bad_request("Invalid filing UUID format"))?;

    let nef = crate::repo::nef::find_by_filing(&pool, &court.0, filing_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("NEF not found for filing {}", filing_id)))?;

    Ok(Json(NefResponse::from(nef)))
}

// ---------------------------------------------------------------------------
// GET /api/nef/{id}
// ---------------------------------------------------------------------------

/// GET /api/nef/{id}
///
/// Retrieve a NEF by its primary ID.
#[utoipa::path(
    get,
    path = "/api/nef/{id}",
    params(
        ("id" = String, Path, description = "NEF UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "NEF found", body = NefResponse),
        (status = 404, description = "NEF not found", body = AppError)
    ),
    tag = "nefs"
)]
pub async fn get_nef_by_id(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(nef_id): Path<String>,
) -> Result<Json<NefResponse>, AppError> {
    let nef_uuid = Uuid::parse_str(&nef_id)
        .map_err(|_| AppError::bad_request("Invalid NEF UUID format"))?;

    let nef = crate::repo::nef::find_by_id(&pool, &court.0, nef_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("NEF not found: {}", nef_id)))?;

    Ok(Json(NefResponse::from(nef)))
}

// ---------------------------------------------------------------------------
// GET /api/nef/docket-entry/{docket_entry_id}
// ---------------------------------------------------------------------------

/// GET /api/nef/docket-entry/{docket_entry_id}
///
/// Retrieve a NEF by its associated docket entry ID.
#[utoipa::path(
    get,
    path = "/api/nef/docket-entry/{docket_entry_id}",
    params(
        ("docket_entry_id" = String, Path, description = "Docket entry UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "NEF found", body = NefResponse),
        (status = 404, description = "NEF not found", body = AppError)
    ),
    tag = "nefs"
)]
pub async fn get_nef_by_docket_entry(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(docket_entry_id): Path<String>,
) -> Result<Json<NefResponse>, AppError> {
    let entry_uuid = Uuid::parse_str(&docket_entry_id)
        .map_err(|_| AppError::bad_request("Invalid docket entry UUID format"))?;

    let nef = crate::repo::nef::find_by_docket_entry(&pool, &court.0, entry_uuid)
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "NEF not found for docket entry {}",
                docket_entry_id
            ))
        })?;

    Ok(Json(NefResponse::from(nef)))
}
