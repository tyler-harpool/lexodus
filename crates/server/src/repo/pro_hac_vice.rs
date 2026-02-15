use shared_types::{AppError, CreateProHacViceRequest, ProHacVice};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new pro hac vice admission for an attorney.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
    req: CreateProHacViceRequest,
) -> Result<ProHacVice, AppError> {
    let row = sqlx::query_as::<_, ProHacVice>(
        r#"
        INSERT INTO attorney_pro_hac_vice
            (court_id, attorney_id, case_id, sponsoring_attorney_id,
             expiration_date)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, court_id, attorney_id, case_id, sponsoring_attorney_id,
                  admission_date, expiration_date, status, created_at
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .bind(req.case_id)
    .bind(req.sponsoring_attorney_id)
    .bind(req.expiration_date)
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all pro hac vice admissions for a specific attorney within a court.
pub async fn list_by_attorney(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
) -> Result<Vec<ProHacVice>, AppError> {
    let rows = sqlx::query_as::<_, ProHacVice>(
        r#"
        SELECT id, court_id, attorney_id, case_id, sponsoring_attorney_id,
               admission_date, expiration_date, status, created_at
        FROM attorney_pro_hac_vice
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

/// Update the status of a specific pro hac vice admission.
pub async fn update_status(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
    case_id: Uuid,
    new_status: &str,
) -> Result<Option<ProHacVice>, AppError> {
    let row = sqlx::query_as::<_, ProHacVice>(
        r#"
        UPDATE attorney_pro_hac_vice
        SET status = $4
        WHERE court_id = $1 AND attorney_id = $2 AND case_id = $3
        RETURNING id, court_id, attorney_id, case_id, sponsoring_attorney_id,
                  admission_date, expiration_date, status, created_at
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .bind(case_id)
    .bind(new_status)
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all active pro hac vice admissions across the court.
pub async fn list_active(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<ProHacVice>, AppError> {
    let rows = sqlx::query_as::<_, ProHacVice>(
        r#"
        SELECT id, court_id, attorney_id, case_id, sponsoring_attorney_id,
               admission_date, expiration_date, status, created_at
        FROM attorney_pro_hac_vice
        WHERE court_id = $1 AND status = 'Active'
        ORDER BY admission_date DESC
        "#,
    )
    .bind(court_id)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all pro hac vice admissions for a specific case.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<ProHacVice>, AppError> {
    let rows = sqlx::query_as::<_, ProHacVice>(
        r#"
        SELECT id, court_id, attorney_id, case_id, sponsoring_attorney_id,
               admission_date, expiration_date, status, created_at
        FROM attorney_pro_hac_vice
        WHERE court_id = $1 AND case_id = $2
        ORDER BY admission_date DESC
        "#,
    )
    .bind(court_id)
    .bind(case_id)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}
