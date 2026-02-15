use shared_types::{AppError, ConfigOverride};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// List config overrides for a given scope within a court.
/// `scope` should be "district" or "judge", and `scope_id` the specific ID.
pub async fn list_by_scope(
    pool: &Pool<Postgres>,
    court_id: &str,
    scope: &str,
    scope_id: &str,
) -> Result<Vec<ConfigOverride>, AppError> {
    let rows = sqlx::query_as!(
        ConfigOverride,
        r#"
        SELECT id, court_id, scope, scope_id, config_key, config_value,
               created_at, updated_at
        FROM config_overrides
        WHERE court_id = $1 AND scope = $2 AND scope_id = $3
        ORDER BY config_key
        "#,
        court_id,
        scope,
        scope_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Insert or update a configuration override (upsert on the unique constraint).
pub async fn set_override(
    pool: &Pool<Postgres>,
    court_id: &str,
    scope: &str,
    scope_id: &str,
    config_key: &str,
    config_value: &serde_json::Value,
) -> Result<ConfigOverride, AppError> {
    let row = sqlx::query_as!(
        ConfigOverride,
        r#"
        INSERT INTO config_overrides (court_id, scope, scope_id, config_key, config_value)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (court_id, scope, scope_id, config_key)
        DO UPDATE SET config_value = EXCLUDED.config_value, updated_at = NOW()
        RETURNING id, court_id, scope, scope_id, config_key, config_value,
                  created_at, updated_at
        "#,
        court_id,
        scope,
        scope_id,
        config_key,
        config_value,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete a specific config override by scope.
/// Returns true if a row was actually deleted.
pub async fn delete_override(
    pool: &Pool<Postgres>,
    court_id: &str,
    scope: &str,
    scope_id: &str,
    config_key: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        r#"
        DELETE FROM config_overrides
        WHERE court_id = $1 AND scope = $2 AND scope_id = $3 AND config_key = $4
        "#,
        court_id,
        scope,
        scope_id,
        config_key,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// Delete all config overrides for a given scope.
pub async fn delete_all_for_scope(
    pool: &Pool<Postgres>,
    court_id: &str,
    scope: &str,
    scope_id: &str,
) -> Result<u64, AppError> {
    let result = sqlx::query!(
        r#"
        DELETE FROM config_overrides
        WHERE court_id = $1 AND scope = $2 AND scope_id = $3
        "#,
        court_id,
        scope,
        scope_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected())
}

/// List all config overrides for the court (all scopes).
pub async fn list_all(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<ConfigOverride>, AppError> {
    let rows = sqlx::query_as!(
        ConfigOverride,
        r#"
        SELECT id, court_id, scope, scope_id, config_key, config_value,
               created_at, updated_at
        FROM config_overrides
        WHERE court_id = $1
        ORDER BY scope, scope_id, config_key
        "#,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Find a specific override by its UUID.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<ConfigOverride>, AppError> {
    let row = sqlx::query_as!(
        ConfigOverride,
        r#"
        SELECT id, court_id, scope, scope_id, config_key, config_value,
               created_at, updated_at
        FROM config_overrides
        WHERE id = $1 AND court_id = $2
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}
