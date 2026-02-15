use shared_types::{AppError, CreateMotionRequest, Motion};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new motion.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateMotionRequest,
) -> Result<Motion, AppError> {
    let status = req.status.as_deref().unwrap_or("Pending");

    let row = sqlx::query_as!(
        Motion,
        r#"
        INSERT INTO motions
            (court_id, case_id, motion_type, filed_by, description,
             filed_date, status, ruling_date, ruling_text)
        VALUES ($1, $2, $3, $4, $5, COALESCE($6, NOW()), $7, $8, $9)
        RETURNING id, court_id, case_id, motion_type, filed_by, description,
                  filed_date, status, ruling_date, ruling_text
        "#,
        court_id,
        req.case_id,
        req.motion_type,
        req.filed_by,
        req.description,
        req.filed_date,
        status,
        req.ruling_date,
        req.ruling_text.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a motion by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Motion>, AppError> {
    let row = sqlx::query_as!(
        Motion,
        r#"
        SELECT id, court_id, case_id, motion_type, filed_by, description,
               filed_date, status, ruling_date, ruling_text
        FROM motions
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

/// List all motions for a given case within a court.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<Motion>, AppError> {
    let rows = sqlx::query_as!(
        Motion,
        r#"
        SELECT id, court_id, case_id, motion_type, filed_by, description,
               filed_date, status, ruling_date, ruling_text
        FROM motions
        WHERE case_id = $1 AND court_id = $2
        ORDER BY filed_date DESC
        "#,
        case_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Update a motion with only the provided fields.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: shared_types::UpdateMotionRequest,
) -> Result<Option<Motion>, AppError> {
    let row = sqlx::query_as!(
        Motion,
        r#"
        UPDATE motions SET
            motion_type = COALESCE($3, motion_type),
            filed_by    = COALESCE($4, filed_by),
            description = COALESCE($5, description),
            status      = COALESCE($6, status),
            ruling_date = COALESCE($7, ruling_date),
            ruling_text = COALESCE($8, ruling_text)
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, motion_type, filed_by, description,
                  filed_date, status, ruling_date, ruling_text
        "#,
        id,
        court_id,
        req.motion_type.as_deref(),
        req.filed_by.as_deref(),
        req.description.as_deref(),
        req.status.as_deref(),
        req.ruling_date,
        req.ruling_text.as_deref(),
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete a motion. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM motions WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
