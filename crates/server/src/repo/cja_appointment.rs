use shared_types::{AppError, CjaAppointment, CreateCjaAppointmentRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new CJA appointment for an attorney.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
    req: CreateCjaAppointmentRequest,
) -> Result<CjaAppointment, AppError> {
    let row = sqlx::query_as::<_, CjaAppointment>(
        r#"
        INSERT INTO attorney_cja_appointments
            (court_id, attorney_id, case_id, voucher_amount)
        VALUES ($1, $2, $3, $4)
        RETURNING id, court_id, attorney_id, case_id,
                  appointment_date, termination_date,
                  voucher_status, voucher_amount, created_at
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .bind(req.case_id)
    .bind(req.voucher_amount)
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all CJA appointments for a specific attorney within a court.
pub async fn list_by_attorney(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
) -> Result<Vec<CjaAppointment>, AppError> {
    let rows = sqlx::query_as::<_, CjaAppointment>(
        r#"
        SELECT id, court_id, attorney_id, case_id,
               appointment_date, termination_date,
               voucher_status, voucher_amount, created_at
        FROM attorney_cja_appointments
        WHERE court_id = $1 AND attorney_id = $2
        ORDER BY appointment_date DESC
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all CJA appointments with pending voucher status across the court.
pub async fn list_pending_vouchers(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<CjaAppointment>, AppError> {
    let rows = sqlx::query_as::<_, CjaAppointment>(
        r#"
        SELECT id, court_id, attorney_id, case_id,
               appointment_date, termination_date,
               voucher_status, voucher_amount, created_at
        FROM attorney_cja_appointments
        WHERE court_id = $1 AND voucher_status = 'Pending'
        ORDER BY appointment_date DESC
        "#,
    )
    .bind(court_id)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}
