use shared_types::{AppError, JudgeSignature};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert or update a judge's signature (upsert on unique court_id + judge_id).
pub async fn create_or_update(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Uuid,
    signature_data: &str,
) -> Result<JudgeSignature, AppError> {
    let row = sqlx::query_as!(
        JudgeSignature,
        r#"
        INSERT INTO judge_signatures (court_id, judge_id, signature_data)
        VALUES ($1, $2, $3)
        ON CONFLICT (court_id, judge_id)
        DO UPDATE SET signature_data = EXCLUDED.signature_data, created_at = NOW()
        RETURNING id, court_id, judge_id, signature_data, created_at
        "#,
        court_id,
        judge_id,
        signature_data,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a judge's signature by judge ID within a court.
pub async fn find_by_judge(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Uuid,
) -> Result<Option<JudgeSignature>, AppError> {
    let row = sqlx::query_as!(
        JudgeSignature,
        r#"
        SELECT id, court_id, judge_id, signature_data, created_at
        FROM judge_signatures
        WHERE court_id = $1 AND judge_id = $2
        "#,
        court_id,
        judge_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}
