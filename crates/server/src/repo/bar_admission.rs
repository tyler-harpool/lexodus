use shared_types::{AppError, BarAdmission, CreateBarAdmissionRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new bar admission for an attorney.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
    req: CreateBarAdmissionRequest,
) -> Result<BarAdmission, AppError> {
    let admission_date = req
        .admission_date
        .unwrap_or_else(chrono::Utc::now);
    let status = req.status.unwrap_or_else(|| "Active".to_string());

    let row = sqlx::query_as::<_, BarAdmission>(
        r#"
        INSERT INTO attorney_bar_admissions
            (court_id, attorney_id, state, bar_number, admission_date, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, court_id, attorney_id, state, bar_number,
                  admission_date, status, created_at
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .bind(&req.state)
    .bind(&req.bar_number)
    .bind(admission_date)
    .bind(&status)
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all bar admissions for a specific attorney within a court.
pub async fn list_by_attorney(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
) -> Result<Vec<BarAdmission>, AppError> {
    let rows = sqlx::query_as::<_, BarAdmission>(
        r#"
        SELECT id, court_id, attorney_id, state, bar_number,
               admission_date, status, created_at
        FROM attorney_bar_admissions
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

/// Delete a bar admission by attorney and state within a court.
pub async fn delete_by_state(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
    state: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
        DELETE FROM attorney_bar_admissions
        WHERE court_id = $1 AND attorney_id = $2 AND state = $3
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .bind(state)
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// List all attorneys who hold bar admission in a specific state.
pub async fn list_by_state(
    pool: &Pool<Postgres>,
    court_id: &str,
    state: &str,
) -> Result<Vec<BarAdmission>, AppError> {
    let rows = sqlx::query_as::<_, BarAdmission>(
        r#"
        SELECT id, court_id, attorney_id, state, bar_number,
               admission_date, status, created_at
        FROM attorney_bar_admissions
        WHERE court_id = $1 AND state = $2 AND status = 'Active'
        ORDER BY admission_date DESC
        "#,
    )
    .bind(court_id)
    .bind(state)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}
