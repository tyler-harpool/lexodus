use shared_types::{AppError, CreateRecusalMotionRequest, RecusalMotion, UpdateRecusalRulingRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new recusal motion.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Uuid,
    req: CreateRecusalMotionRequest,
) -> Result<RecusalMotion, AppError> {
    let row = sqlx::query_as!(
        RecusalMotion,
        r#"
        INSERT INTO recusal_motions
            (court_id, case_id, judge_id, filed_by, reason, detailed_grounds)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, court_id, case_id, judge_id, filed_by, filed_date,
                  reason, detailed_grounds, status, ruling_date, ruling_text,
                  replacement_judge_id
        "#,
        court_id,
        req.case_id,
        judge_id,
        req.filed_by,
        req.reason,
        req.detailed_grounds.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Update a recusal ruling.
pub async fn update_ruling(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: UpdateRecusalRulingRequest,
) -> Result<Option<RecusalMotion>, AppError> {
    let row = sqlx::query_as!(
        RecusalMotion,
        r#"
        UPDATE recusal_motions SET
            status               = $3,
            ruling_date          = NOW(),
            ruling_text          = $4,
            replacement_judge_id = $5
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, judge_id, filed_by, filed_date,
                  reason, detailed_grounds, status, ruling_date, ruling_text,
                  replacement_judge_id
        "#,
        id,
        court_id,
        req.status,
        req.ruling_text.as_deref(),
        req.replacement_judge_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List pending recusal motions.
pub async fn list_pending(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<RecusalMotion>, AppError> {
    let rows = sqlx::query_as!(
        RecusalMotion,
        r#"
        SELECT id, court_id, case_id, judge_id, filed_by, filed_date,
               reason, detailed_grounds, status, ruling_date, ruling_text,
               replacement_judge_id
        FROM recusal_motions
        WHERE court_id = $1 AND status = 'Pending'
        ORDER BY filed_date DESC
        "#,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List recusals for a case.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<RecusalMotion>, AppError> {
    let rows = sqlx::query_as!(
        RecusalMotion,
        r#"
        SELECT id, court_id, case_id, judge_id, filed_by, filed_date,
               reason, detailed_grounds, status, ruling_date, ruling_text,
               replacement_judge_id
        FROM recusal_motions
        WHERE case_id = $1 AND court_id = $2
        ORDER BY filed_date DESC
        "#,
        case_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List recusals for a judge.
pub async fn list_by_judge(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Uuid,
) -> Result<Vec<RecusalMotion>, AppError> {
    let rows = sqlx::query_as!(
        RecusalMotion,
        r#"
        SELECT id, court_id, case_id, judge_id, filed_by, filed_date,
               reason, detailed_grounds, status, ruling_date, ruling_text,
               replacement_judge_id
        FROM recusal_motions
        WHERE judge_id = $1 AND court_id = $2
        ORDER BY filed_date DESC
        "#,
        judge_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}
