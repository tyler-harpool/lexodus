use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use shared_types::{
    AppError, FeatureFlagResponse, FeatureStatusResponse,
    SetFeatureOverrideRequest, UpdateFeatureFlagRequest,
};
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct FeatureOverrideParams {
    pub scope: Option<String>,
}

// ---------------------------------------------------------------------------
// GET /api/features
// ---------------------------------------------------------------------------

/// List all feature flags.
#[utoipa::path(
    get,
    path = "/api/features",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "All feature flags", body = Vec<FeatureFlagResponse>)
    ),
    tag = "features"
)]
pub async fn list_features(
    State(pool): State<Pool<Postgres>>,
    _court: CourtId,
) -> Result<Json<Vec<FeatureFlagResponse>>, AppError> {
    let flags = crate::repo::feature_flag::list_all(&pool).await?;
    let response: Vec<FeatureFlagResponse> = flags.into_iter().map(FeatureFlagResponse::from).collect();
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// PATCH /api/features
// ---------------------------------------------------------------------------

/// Update a feature flag's enabled status.
#[utoipa::path(
    patch,
    path = "/api/features",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = UpdateFeatureFlagRequest,
    responses(
        (status = 200, description = "Feature flag updated", body = FeatureFlagResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "features"
)]
pub async fn update_features(
    State(pool): State<Pool<Postgres>>,
    _court: CourtId,
    Json(body): Json<UpdateFeatureFlagRequest>,
) -> Result<Json<FeatureFlagResponse>, AppError> {
    let flag = crate::repo::feature_flag::update(&pool, &body.feature_path, body.enabled)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Feature flag '{}' not found", body.feature_path)))?;

    Ok(Json(FeatureFlagResponse::from(flag)))
}

// ---------------------------------------------------------------------------
// GET /api/features/implementation
// ---------------------------------------------------------------------------

/// List all feature flags (implementation view). Same data, different context.
#[utoipa::path(
    get,
    path = "/api/features/implementation",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "All feature flags (implementation view)", body = Vec<FeatureFlagResponse>)
    ),
    tag = "features"
)]
pub async fn get_impl(
    State(pool): State<Pool<Postgres>>,
    _court: CourtId,
) -> Result<Json<Vec<FeatureFlagResponse>>, AppError> {
    let flags = crate::repo::feature_flag::list_all(&pool).await?;
    let response: Vec<FeatureFlagResponse> = flags.into_iter().map(FeatureFlagResponse::from).collect();
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// PATCH /api/features/implementation
// ---------------------------------------------------------------------------

/// Update a feature flag (implementation view).
#[utoipa::path(
    patch,
    path = "/api/features/implementation",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = UpdateFeatureFlagRequest,
    responses(
        (status = 200, description = "Feature updated", body = FeatureFlagResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "features"
)]
pub async fn update_impl(
    State(pool): State<Pool<Postgres>>,
    _court: CourtId,
    Json(body): Json<UpdateFeatureFlagRequest>,
) -> Result<Json<FeatureFlagResponse>, AppError> {
    let flag = crate::repo::feature_flag::update(&pool, &body.feature_path, body.enabled)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Feature flag '{}' not found", body.feature_path)))?;

    Ok(Json(FeatureFlagResponse::from(flag)))
}

// ---------------------------------------------------------------------------
// GET /api/features/blocked
// ---------------------------------------------------------------------------

/// List all disabled (blocked) feature flags.
#[utoipa::path(
    get,
    path = "/api/features/blocked",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Blocked features", body = Vec<FeatureFlagResponse>)
    ),
    tag = "features"
)]
pub async fn list_blocked(
    State(pool): State<Pool<Postgres>>,
    _court: CourtId,
) -> Result<Json<Vec<FeatureFlagResponse>>, AppError> {
    let flags = crate::repo::feature_flag::list_blocked(&pool).await?;
    let response: Vec<FeatureFlagResponse> = flags.into_iter().map(FeatureFlagResponse::from).collect();
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// GET /api/features/ready
// ---------------------------------------------------------------------------

/// List all enabled (ready) feature flags.
#[utoipa::path(
    get,
    path = "/api/features/ready",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Ready features", body = Vec<FeatureFlagResponse>)
    ),
    tag = "features"
)]
pub async fn list_ready(
    State(pool): State<Pool<Postgres>>,
    _court: CourtId,
) -> Result<Json<Vec<FeatureFlagResponse>>, AppError> {
    let flags = crate::repo::feature_flag::list_ready(&pool).await?;
    let response: Vec<FeatureFlagResponse> = flags.into_iter().map(FeatureFlagResponse::from).collect();
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// POST /api/features/manager
// ---------------------------------------------------------------------------

/// Manage a feature flag (create or update).
#[utoipa::path(
    post,
    path = "/api/features/manager",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = UpdateFeatureFlagRequest,
    responses(
        (status = 200, description = "Feature managed", body = FeatureFlagResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "features"
)]
pub async fn manage_feature(
    State(pool): State<Pool<Postgres>>,
    _court: CourtId,
    Json(body): Json<UpdateFeatureFlagRequest>,
) -> Result<Json<FeatureFlagResponse>, AppError> {
    let flag = crate::repo::feature_flag::update(&pool, &body.feature_path, body.enabled)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Feature flag '{}' not found", body.feature_path)))?;

    Ok(Json(FeatureFlagResponse::from(flag)))
}

// ---------------------------------------------------------------------------
// GET /api/features/{feature_path}/enabled
// ---------------------------------------------------------------------------

/// Check if a specific feature is enabled.
#[utoipa::path(
    get,
    path = "/api/features/{feature_path}/enabled",
    params(
        ("feature_path" = String, Path, description = "Feature path"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Feature status", body = FeatureStatusResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "features"
)]
pub async fn check_enabled(
    State(pool): State<Pool<Postgres>>,
    _court: CourtId,
    Path(feature_path): Path<String>,
) -> Result<Json<FeatureStatusResponse>, AppError> {
    let flag = crate::repo::feature_flag::find_by_path(&pool, &feature_path)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Feature flag '{}' not found", feature_path)))?;

    Ok(Json(FeatureStatusResponse {
        feature_path: flag.feature_path,
        enabled: flag.enabled,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/features/override
// ---------------------------------------------------------------------------

/// Set a feature override scoped to a court or judge.
/// This stores the override in config_overrides with scope "feature".
#[utoipa::path(
    post,
    path = "/api/features/override",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = SetFeatureOverrideRequest,
    responses(
        (status = 200, description = "Override set", body = FeatureStatusResponse)
    ),
    tag = "features"
)]
pub async fn set_override(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<SetFeatureOverrideRequest>,
) -> Result<Json<FeatureStatusResponse>, AppError> {
    let scope = body.scope.as_deref().unwrap_or("district");
    let value = serde_json::json!({ "enabled": body.enabled });

    crate::repo::config_override::set_override(
        &pool, &court.0, scope, &court.0, &body.feature_path, &value,
    )
    .await?;

    Ok(Json(FeatureStatusResponse {
        feature_path: body.feature_path,
        enabled: body.enabled,
    }))
}

// ---------------------------------------------------------------------------
// DELETE /api/features/override
// ---------------------------------------------------------------------------

/// Clear all feature overrides for the court.
#[utoipa::path(
    delete,
    path = "/api/features/override",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Overrides cleared")
    ),
    tag = "features"
)]
pub async fn clear_overrides(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<StatusCode, AppError> {
    crate::repo::config_override::delete_all_for_scope(&pool, &court.0, "district", &court.0).await?;
    Ok(StatusCode::NO_CONTENT)
}
