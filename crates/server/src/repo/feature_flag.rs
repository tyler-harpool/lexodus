use shared_types::{AppError, FeatureFlag};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// List all feature flags.
pub async fn list_all(
    pool: &Pool<Postgres>,
) -> Result<Vec<FeatureFlag>, AppError> {
    let rows = sqlx::query_as!(
        FeatureFlag,
        r#"
        SELECT id, feature_path, enabled, description, created_at, updated_at
        FROM feature_flags
        ORDER BY feature_path
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Find a feature flag by its path.
pub async fn find_by_path(
    pool: &Pool<Postgres>,
    feature_path: &str,
) -> Result<Option<FeatureFlag>, AppError> {
    let row = sqlx::query_as!(
        FeatureFlag,
        r#"
        SELECT id, feature_path, enabled, description, created_at, updated_at
        FROM feature_flags
        WHERE feature_path = $1
        "#,
        feature_path,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Update a feature flag's enabled status. Returns the updated flag or None.
pub async fn update(
    pool: &Pool<Postgres>,
    feature_path: &str,
    enabled: bool,
) -> Result<Option<FeatureFlag>, AppError> {
    let row = sqlx::query_as!(
        FeatureFlag,
        r#"
        UPDATE feature_flags
        SET enabled = $2, updated_at = NOW()
        WHERE feature_path = $1
        RETURNING id, feature_path, enabled, description, created_at, updated_at
        "#,
        feature_path,
        enabled,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all feature flags that are currently disabled (blocked).
pub async fn list_blocked(
    pool: &Pool<Postgres>,
) -> Result<Vec<FeatureFlag>, AppError> {
    let rows = sqlx::query_as!(
        FeatureFlag,
        r#"
        SELECT id, feature_path, enabled, description, created_at, updated_at
        FROM feature_flags
        WHERE enabled = false
        ORDER BY feature_path
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all feature flags that are currently enabled (ready).
pub async fn list_ready(
    pool: &Pool<Postgres>,
) -> Result<Vec<FeatureFlag>, AppError> {
    let rows = sqlx::query_as!(
        FeatureFlag,
        r#"
        SELECT id, feature_path, enabled, description, created_at, updated_at
        FROM feature_flags
        WHERE enabled = true
        ORDER BY feature_path
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Find a feature flag by its UUID.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    id: Uuid,
) -> Result<Option<FeatureFlag>, AppError> {
    let row = sqlx::query_as!(
        FeatureFlag,
        r#"
        SELECT id, feature_path, enabled, description, created_at, updated_at
        FROM feature_flags
        WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}
