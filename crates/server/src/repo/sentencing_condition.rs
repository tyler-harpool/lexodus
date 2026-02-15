use shared_types::{AppError, CreateSpecialConditionRequest, SentencingSpecialCondition};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new special condition for a sentencing record.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    sentencing_id: Uuid,
    req: CreateSpecialConditionRequest,
) -> Result<SentencingSpecialCondition, AppError> {
    let status = "Active";

    let row = sqlx::query_as!(
        SentencingSpecialCondition,
        r#"
        INSERT INTO sentencing_special_conditions
            (court_id, sentencing_id, condition_type, description, effective_date, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, court_id, sentencing_id, condition_type, description,
                  effective_date, status, created_at
        "#,
        court_id,
        sentencing_id,
        req.condition_type,
        req.description,
        req.effective_date,
        status,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all special conditions for a sentencing record.
pub async fn list_by_sentencing(
    pool: &Pool<Postgres>,
    court_id: &str,
    sentencing_id: Uuid,
) -> Result<Vec<SentencingSpecialCondition>, AppError> {
    let rows = sqlx::query_as!(
        SentencingSpecialCondition,
        r#"
        SELECT id, court_id, sentencing_id, condition_type, description,
               effective_date, status, created_at
        FROM sentencing_special_conditions
        WHERE sentencing_id = $1 AND court_id = $2
        ORDER BY created_at DESC
        "#,
        sentencing_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}
