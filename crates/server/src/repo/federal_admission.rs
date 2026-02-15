use shared_types::{AppError, CreateFederalAdmissionRequest, FederalAdmission};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new federal court admission for an attorney.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
    req: CreateFederalAdmissionRequest,
) -> Result<FederalAdmission, AppError> {
    let admission_date = req
        .admission_date
        .unwrap_or_else(chrono::Utc::now);
    let status = req.status.unwrap_or_else(|| "Active".to_string());

    let row = sqlx::query_as::<_, FederalAdmission>(
        r#"
        INSERT INTO attorney_federal_admissions
            (court_id, attorney_id, court_name, admission_date, status)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, court_id, attorney_id, court_name,
                  admission_date, status, created_at
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .bind(&req.court_name)
    .bind(admission_date)
    .bind(&status)
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all federal admissions for a specific attorney within a court.
pub async fn list_by_attorney(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
) -> Result<Vec<FederalAdmission>, AppError> {
    let rows = sqlx::query_as::<_, FederalAdmission>(
        r#"
        SELECT id, court_id, attorney_id, court_name,
               admission_date, status, created_at
        FROM attorney_federal_admissions
        WHERE court_id = $1 AND attorney_id = $2
        ORDER BY admission_date DESC
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Delete a federal admission by attorney and court name within a court.
pub async fn delete_by_court_name(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
    court_name: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
        DELETE FROM attorney_federal_admissions
        WHERE court_id = $1 AND attorney_id = $2 AND court_name = $3
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .bind(court_name)
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// List all attorneys with active federal admission to a specific court.
pub async fn list_by_court_name(
    pool: &Pool<Postgres>,
    court_id: &str,
    court_name: &str,
) -> Result<Vec<FederalAdmission>, AppError> {
    let rows = sqlx::query_as::<_, FederalAdmission>(
        r#"
        SELECT id, court_id, attorney_id, court_name,
               admission_date, status, created_at
        FROM attorney_federal_admissions
        WHERE court_id = $1 AND court_name = $2 AND status = 'Active'
        ORDER BY admission_date DESC
        "#,
    )
    .bind(court_id)
    .bind(court_name)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}
