use shared_types::{AppError, CreateEvidenceRequest, Evidence};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new evidence item.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateEvidenceRequest,
) -> Result<Evidence, AppError> {
    let is_sealed = req.is_sealed.unwrap_or(false);

    let row = sqlx::query_as!(
        Evidence,
        r#"
        INSERT INTO evidence
            (court_id, case_id, description, evidence_type, seized_date,
             seized_by, location, is_sealed)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, court_id, case_id, description, evidence_type,
                  seized_date, seized_by,
                  COALESCE(location, '') as "location!",
                  is_sealed, created_at
        "#,
        court_id,
        req.case_id,
        req.description,
        req.evidence_type,
        req.seized_date,
        req.seized_by.as_deref(),
        req.location.as_deref(),
        is_sealed,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find evidence by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Evidence>, AppError> {
    let row = sqlx::query_as!(
        Evidence,
        r#"
        SELECT id, court_id, case_id, description, evidence_type,
               seized_date, seized_by,
               COALESCE(location, '') as "location!",
               is_sealed, created_at
        FROM evidence
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

/// List all evidence for a given case within a court.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<Evidence>, AppError> {
    let rows = sqlx::query_as!(
        Evidence,
        r#"
        SELECT id, court_id, case_id, description, evidence_type,
               seized_date, seized_by,
               COALESCE(location, '') as "location!",
               is_sealed, created_at
        FROM evidence
        WHERE case_id = $1 AND court_id = $2
        ORDER BY created_at ASC
        "#,
        case_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all evidence for a court (across all cases), ordered by description.
/// Supports optional search by description and pagination.
pub async fn list_all(
    pool: &Pool<Postgres>,
    court_id: &str,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<Evidence>, i64), AppError> {
    let search = q.map(|s| format!("%{}%", s.to_lowercase()));

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!" FROM evidence
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR LOWER(description) LIKE $2)
        "#,
        court_id,
        search.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        Evidence,
        r#"
        SELECT id, court_id, case_id, description, evidence_type,
               seized_date, seized_by,
               COALESCE(location, '') as "location!",
               is_sealed, created_at
        FROM evidence
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR LOWER(description) LIKE $2)
        ORDER BY description ASC
        LIMIT $3 OFFSET $4
        "#,
        court_id,
        search.as_deref(),
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, total))
}

/// Update evidence with only the provided fields.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: shared_types::UpdateEvidenceRequest,
) -> Result<Option<Evidence>, AppError> {
    let row = sqlx::query_as!(
        Evidence,
        r#"
        UPDATE evidence SET
            description   = COALESCE($3, description),
            evidence_type = COALESCE($4, evidence_type),
            seized_date   = COALESCE($5, seized_date),
            seized_by     = COALESCE($6, seized_by),
            location      = COALESCE($7, location),
            is_sealed     = COALESCE($8, is_sealed)
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, description, evidence_type,
                  seized_date, seized_by,
                  COALESCE(location, '') as "location!",
                  is_sealed, created_at
        "#,
        id,
        court_id,
        req.description.as_deref(),
        req.evidence_type.as_deref(),
        req.seized_date,
        req.seized_by.as_deref(),
        req.location.as_deref(),
        req.is_sealed,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete evidence. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM evidence WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
