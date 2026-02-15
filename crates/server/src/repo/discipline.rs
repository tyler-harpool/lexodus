use shared_types::{AppError, CreateDisciplineRecordRequest, DisciplineRecord};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new discipline record for an attorney.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
    req: CreateDisciplineRecordRequest,
) -> Result<DisciplineRecord, AppError> {
    let action_date = req
        .action_date
        .unwrap_or_else(chrono::Utc::now);
    let effective_date = req
        .effective_date
        .unwrap_or(action_date);

    let row = sqlx::query_as::<_, DisciplineRecord>(
        r#"
        INSERT INTO attorney_discipline_history
            (court_id, attorney_id, action_type, jurisdiction, description,
             action_date, effective_date, end_date)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, court_id, attorney_id, action_type, jurisdiction,
                  description, action_date, effective_date, end_date, created_at
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .bind(&req.action_type)
    .bind(&req.jurisdiction)
    .bind(&req.description)
    .bind(action_date)
    .bind(effective_date)
    .bind(req.end_date)
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all discipline records for a specific attorney within a court.
pub async fn list_by_attorney(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
) -> Result<Vec<DisciplineRecord>, AppError> {
    let rows = sqlx::query_as::<_, DisciplineRecord>(
        r#"
        SELECT id, court_id, attorney_id, action_type, jurisdiction,
               description, action_date, effective_date, end_date, created_at
        FROM attorney_discipline_history
        WHERE court_id = $1 AND attorney_id = $2
        ORDER BY action_date DESC
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all attorneys with discipline records in the court.
pub async fn list_with_discipline(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<DisciplineRecord>, AppError> {
    let rows = sqlx::query_as::<_, DisciplineRecord>(
        r#"
        SELECT id, court_id, attorney_id, action_type, jurisdiction,
               description, action_date, effective_date, end_date, created_at
        FROM attorney_discipline_history
        WHERE court_id = $1
        ORDER BY action_date DESC
        "#,
    )
    .bind(court_id)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}
