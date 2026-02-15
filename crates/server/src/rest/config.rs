use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use shared_types::{
    AppError, ConfigOverrideResponse, SetConfigOverrideRequest, ConfigPreviewRequest,
};
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

/// Query parameters for scoped config override requests.
#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct ScopeParams {
    /// The scope_id (e.g. the district code or judge UUID string).
    pub scope_id: String,
    /// Optional config_key to filter on.
    pub config_key: Option<String>,
}

// ---------------------------------------------------------------------------
// GET /api/config
// ---------------------------------------------------------------------------

/// Returns the merged configuration for the court (base + overrides).
#[utoipa::path(
    get,
    path = "/api/config",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "All config overrides", body = Vec<ConfigOverrideResponse>)
    ),
    tag = "configuration"
)]
pub async fn get_config(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<ConfigOverrideResponse>>, AppError> {
    let overrides = crate::repo::config_override::list_all(&pool, &court.0).await?;
    let response: Vec<ConfigOverrideResponse> = overrides.into_iter().map(ConfigOverrideResponse::from).collect();
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// GET /api/config/overrides/district
// ---------------------------------------------------------------------------

/// List configuration overrides scoped to a district.
#[utoipa::path(
    get,
    path = "/api/config/overrides/district",
    params(
        ScopeParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "District overrides", body = Vec<ConfigOverrideResponse>)
    ),
    tag = "configuration"
)]
pub async fn get_district_overrides(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<ScopeParams>,
) -> Result<Json<Vec<ConfigOverrideResponse>>, AppError> {
    let overrides = crate::repo::config_override::list_by_scope(
        &pool, &court.0, "district", &params.scope_id,
    )
    .await?;

    let response: Vec<ConfigOverrideResponse> = overrides.into_iter().map(ConfigOverrideResponse::from).collect();
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// PUT /api/config/overrides/district
// ---------------------------------------------------------------------------

/// Set a district-scoped configuration override.
#[utoipa::path(
    put,
    path = "/api/config/overrides/district",
    params(
        ScopeParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = SetConfigOverrideRequest,
    responses(
        (status = 200, description = "Override set", body = ConfigOverrideResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "configuration"
)]
pub async fn set_district_override(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<ScopeParams>,
    Json(body): Json<SetConfigOverrideRequest>,
) -> Result<Json<ConfigOverrideResponse>, AppError> {
    let row = crate::repo::config_override::set_override(
        &pool, &court.0, "district", &params.scope_id,
        &body.config_key, &body.config_value,
    )
    .await?;

    Ok(Json(ConfigOverrideResponse::from(row)))
}

// ---------------------------------------------------------------------------
// DELETE /api/config/overrides/district
// ---------------------------------------------------------------------------

/// Delete a district-scoped configuration override.
#[utoipa::path(
    delete,
    path = "/api/config/overrides/district",
    params(
        ScopeParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Override deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "configuration"
)]
pub async fn delete_district_override(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<ScopeParams>,
) -> Result<StatusCode, AppError> {
    let config_key = params.config_key.as_deref().unwrap_or("");
    if config_key.is_empty() {
        return Err(AppError::bad_request("config_key query parameter is required"));
    }

    let deleted = crate::repo::config_override::delete_override(
        &pool, &court.0, "district", &params.scope_id, config_key,
    )
    .await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found("Config override not found"))
    }
}

// ---------------------------------------------------------------------------
// GET /api/config/overrides/judge
// ---------------------------------------------------------------------------

/// List configuration overrides scoped to a judge.
#[utoipa::path(
    get,
    path = "/api/config/overrides/judge",
    params(
        ScopeParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Judge overrides", body = Vec<ConfigOverrideResponse>)
    ),
    tag = "configuration"
)]
pub async fn get_judge_overrides(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<ScopeParams>,
) -> Result<Json<Vec<ConfigOverrideResponse>>, AppError> {
    let overrides = crate::repo::config_override::list_by_scope(
        &pool, &court.0, "judge", &params.scope_id,
    )
    .await?;

    let response: Vec<ConfigOverrideResponse> = overrides.into_iter().map(ConfigOverrideResponse::from).collect();
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// PUT /api/config/overrides/judge
// ---------------------------------------------------------------------------

/// Set a judge-scoped configuration override.
#[utoipa::path(
    put,
    path = "/api/config/overrides/judge",
    params(
        ScopeParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = SetConfigOverrideRequest,
    responses(
        (status = 200, description = "Override set", body = ConfigOverrideResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "configuration"
)]
pub async fn set_judge_override(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<ScopeParams>,
    Json(body): Json<SetConfigOverrideRequest>,
) -> Result<Json<ConfigOverrideResponse>, AppError> {
    let row = crate::repo::config_override::set_override(
        &pool, &court.0, "judge", &params.scope_id,
        &body.config_key, &body.config_value,
    )
    .await?;

    Ok(Json(ConfigOverrideResponse::from(row)))
}

// ---------------------------------------------------------------------------
// DELETE /api/config/overrides/judge
// ---------------------------------------------------------------------------

/// Delete a judge-scoped configuration override.
#[utoipa::path(
    delete,
    path = "/api/config/overrides/judge",
    params(
        ScopeParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Override deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "configuration"
)]
pub async fn delete_judge_override(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<ScopeParams>,
) -> Result<StatusCode, AppError> {
    let config_key = params.config_key.as_deref().unwrap_or("");
    if config_key.is_empty() {
        return Err(AppError::bad_request("config_key query parameter is required"));
    }

    let deleted = crate::repo::config_override::delete_override(
        &pool, &court.0, "judge", &params.scope_id, config_key,
    )
    .await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found("Config override not found"))
    }
}

// ---------------------------------------------------------------------------
// POST /api/config/preview
// ---------------------------------------------------------------------------

/// Preview a configuration with hypothetical overrides applied.
/// Returns the overrides merged with the request body.
#[utoipa::path(
    post,
    path = "/api/config/preview",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = ConfigPreviewRequest,
    responses(
        (status = 200, description = "Preview result", body = serde_json::Value)
    ),
    tag = "configuration"
)]
pub async fn preview_config(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<ConfigPreviewRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Load all existing overrides for the court
    let existing = crate::repo::config_override::list_all(&pool, &court.0).await?;

    // Build a merged JSON object: start with existing, layer on request overrides
    let mut merged = serde_json::Map::new();

    for ov in &existing {
        merged.insert(ov.config_key.clone(), ov.config_value.clone());
    }

    // Layer the preview overrides on top
    if let serde_json::Value::Object(preview_map) = &body.overrides {
        for (k, v) in preview_map {
            merged.insert(k.clone(), v.clone());
        }
    }

    Ok(Json(serde_json::Value::Object(merged)))
}
