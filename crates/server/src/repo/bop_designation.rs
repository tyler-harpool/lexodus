use shared_types::{AppError, BopDesignation, CreateBopDesignationRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new BOP designation.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    sentencing_id: Uuid,
    req: CreateBopDesignationRequest,
) -> Result<BopDesignation, AppError> {
    let rdap_eligible = req.rdap_eligible.unwrap_or(false);

    let row = sqlx::query_as!(
        BopDesignation,
        r#"
        INSERT INTO bop_designations
            (court_id, sentencing_id, defendant_id, facility, security_level,
             designation_date, designation_reason, rdap_eligible, rdap_enrolled)
        VALUES ($1, $2, $3, $4, $5, NOW(), $6, $7, false)
        RETURNING id, court_id, sentencing_id, defendant_id, facility,
                  security_level, designation_date, designation_reason,
                  rdap_eligible, rdap_enrolled, created_at
        "#,
        court_id,
        sentencing_id,
        req.defendant_id,
        req.facility,
        req.security_level,
        req.designation_reason.as_deref(),
        rdap_eligible,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all BOP designations where the defendant is RDAP-eligible.
pub async fn list_rdap_eligible(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<BopDesignation>, AppError> {
    let rows = sqlx::query_as!(
        BopDesignation,
        r#"
        SELECT id, court_id, sentencing_id, defendant_id, facility,
               security_level, designation_date, designation_reason,
               rdap_eligible, rdap_enrolled, created_at
        FROM bop_designations
        WHERE court_id = $1 AND rdap_eligible = true
        ORDER BY designation_date DESC
        "#,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}
