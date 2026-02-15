use shared_types::{AppError, CreateChargeRequest, Charge};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new charge against a defendant.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateChargeRequest,
) -> Result<Charge, AppError> {
    let plea = req.plea.as_deref().unwrap_or("Not Yet Entered");

    let row = sqlx::query_as!(
        Charge,
        r#"
        INSERT INTO charges
            (court_id, defendant_id, count_number, statute, offense_description,
             statutory_max_months, statutory_min_months, plea, plea_date,
             verdict, verdict_date)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, court_id, defendant_id, count_number, statute,
                  offense_description, statutory_max_months, statutory_min_months,
                  COALESCE(plea, 'Not Yet Entered') as "plea!",
                  plea_date,
                  COALESCE(verdict, '') as "verdict!",
                  verdict_date
        "#,
        court_id,
        req.defendant_id,
        req.count_number,
        req.statute,
        req.offense_description,
        req.statutory_max_months,
        req.statutory_min_months,
        plea,
        req.plea_date,
        req.verdict.as_deref(),
        req.verdict_date,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a charge by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Charge>, AppError> {
    let row = sqlx::query_as!(
        Charge,
        r#"
        SELECT id, court_id, defendant_id, count_number, statute,
               offense_description, statutory_max_months, statutory_min_months,
               COALESCE(plea, 'Not Yet Entered') as "plea!",
               plea_date,
               COALESCE(verdict, '') as "verdict!",
               verdict_date
        FROM charges
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

/// List all charges for a given defendant within a court.
pub async fn list_by_defendant(
    pool: &Pool<Postgres>,
    court_id: &str,
    defendant_id: Uuid,
) -> Result<Vec<Charge>, AppError> {
    let rows = sqlx::query_as!(
        Charge,
        r#"
        SELECT id, court_id, defendant_id, count_number, statute,
               offense_description, statutory_max_months, statutory_min_months,
               COALESCE(plea, 'Not Yet Entered') as "plea!",
               plea_date,
               COALESCE(verdict, '') as "verdict!",
               verdict_date
        FROM charges
        WHERE defendant_id = $1 AND court_id = $2
        ORDER BY count_number ASC
        "#,
        defendant_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Update a charge with only the provided fields.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: shared_types::UpdateChargeRequest,
) -> Result<Option<Charge>, AppError> {
    let row = sqlx::query_as!(
        Charge,
        r#"
        UPDATE charges SET
            count_number        = COALESCE($3, count_number),
            statute             = COALESCE($4, statute),
            offense_description = COALESCE($5, offense_description),
            statutory_max_months = COALESCE($6, statutory_max_months),
            statutory_min_months = COALESCE($7, statutory_min_months),
            plea                = COALESCE($8, plea),
            plea_date           = COALESCE($9, plea_date),
            verdict             = COALESCE($10, verdict),
            verdict_date        = COALESCE($11, verdict_date)
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, defendant_id, count_number, statute,
                  offense_description, statutory_max_months, statutory_min_months,
                  COALESCE(plea, 'Not Yet Entered') as "plea!",
                  plea_date,
                  COALESCE(verdict, '') as "verdict!",
                  verdict_date
        "#,
        id,
        court_id,
        req.count_number,
        req.statute.as_deref(),
        req.offense_description.as_deref(),
        req.statutory_max_months,
        req.statutory_min_months,
        req.plea.as_deref(),
        req.plea_date,
        req.verdict.as_deref(),
        req.verdict_date,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete a charge. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM charges WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
