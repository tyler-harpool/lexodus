use shared_types::{AppError, EcfRegistration};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert or update an ECF registration for an attorney.
/// Uses ON CONFLICT to upsert based on (court_id, attorney_id).
pub async fn upsert(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
    status: &str,
) -> Result<EcfRegistration, AppError> {
    let row = sqlx::query_as::<_, EcfRegistration>(
        r#"
        INSERT INTO attorney_ecf_registrations
            (court_id, attorney_id, status)
        VALUES ($1, $2, $3)
        ON CONFLICT (court_id, attorney_id)
        DO UPDATE SET status = $3
        RETURNING id, court_id, attorney_id, registration_date, status, created_at
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .bind(status)
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find the ECF registration for a specific attorney within a court.
pub async fn find_by_attorney(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
) -> Result<Option<EcfRegistration>, AppError> {
    let row = sqlx::query_as::<_, EcfRegistration>(
        r#"
        SELECT id, court_id, attorney_id, registration_date, status, created_at
        FROM attorney_ecf_registrations
        WHERE court_id = $1 AND attorney_id = $2
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all ECF registrations in a court.
pub async fn list_all(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<EcfRegistration>, AppError> {
    let rows = sqlx::query_as::<_, EcfRegistration>(
        r#"
        SELECT id, court_id, attorney_id, registration_date, status, created_at
        FROM attorney_ecf_registrations
        WHERE court_id = $1
        ORDER BY registration_date DESC
        "#,
    )
    .bind(court_id)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all ECF registrations with active status in a court.
pub async fn list_active(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<EcfRegistration>, AppError> {
    let rows = sqlx::query_as::<_, EcfRegistration>(
        r#"
        SELECT id, court_id, attorney_id, registration_date, status, created_at
        FROM attorney_ecf_registrations
        WHERE court_id = $1 AND status = 'Active'
        ORDER BY registration_date DESC
        "#,
    )
    .bind(court_id)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Revoke ECF access for a specific attorney by setting status to 'Revoked'.
pub async fn revoke(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
        UPDATE attorney_ecf_registrations
        SET status = 'Revoked'
        WHERE court_id = $1 AND attorney_id = $2
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
