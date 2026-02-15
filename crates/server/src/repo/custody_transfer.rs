use shared_types::{AppError, CreateCustodyTransferRequest, CustodyTransfer};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new custody transfer record.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateCustodyTransferRequest,
) -> Result<CustodyTransfer, AppError> {
    let row = sqlx::query_as!(
        CustodyTransfer,
        r#"
        INSERT INTO custody_transfers
            (court_id, evidence_id, transferred_from, transferred_to,
             date, location, condition, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, court_id, evidence_id, transferred_from, transferred_to,
                  date,
                  COALESCE(location, '') as "location!",
                  COALESCE(condition, '') as "condition!",
                  notes
        "#,
        court_id,
        req.evidence_id,
        req.transferred_from,
        req.transferred_to,
        req.date,
        req.location.as_deref(),
        req.condition.as_deref(),
        req.notes.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a custody transfer by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<CustodyTransfer>, AppError> {
    let row = sqlx::query_as!(
        CustodyTransfer,
        r#"
        SELECT id, court_id, evidence_id, transferred_from, transferred_to,
               date,
               COALESCE(location, '') as "location!",
               COALESCE(condition, '') as "condition!",
               notes
        FROM custody_transfers
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

/// List all custody transfers for a given evidence item within a court.
pub async fn list_by_evidence(
    pool: &Pool<Postgres>,
    court_id: &str,
    evidence_id: Uuid,
) -> Result<Vec<CustodyTransfer>, AppError> {
    let rows = sqlx::query_as!(
        CustodyTransfer,
        r#"
        SELECT id, court_id, evidence_id, transferred_from, transferred_to,
               date,
               COALESCE(location, '') as "location!",
               COALESCE(condition, '') as "condition!",
               notes
        FROM custody_transfers
        WHERE evidence_id = $1 AND court_id = $2
        ORDER BY date ASC
        "#,
        evidence_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Delete a custody transfer. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM custody_transfers WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
