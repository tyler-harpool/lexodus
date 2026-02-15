use shared_types::{AppError, CreateJudgeConflictRequest, JudgeConflict};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new judge conflict.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Uuid,
    req: CreateJudgeConflictRequest,
) -> Result<JudgeConflict, AppError> {
    let row = sqlx::query_as!(
        JudgeConflict,
        r#"
        INSERT INTO judge_conflicts
            (court_id, judge_id, party_name, law_firm, corporation,
             conflict_type, start_date, end_date, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, court_id, judge_id, party_name, law_firm, corporation,
                  conflict_type, start_date, end_date, notes
        "#,
        court_id,
        judge_id,
        req.party_name.as_deref(),
        req.law_firm.as_deref(),
        req.corporation.as_deref(),
        req.conflict_type,
        req.start_date,
        req.end_date,
        req.notes.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a conflict by ID within a court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<JudgeConflict>, AppError> {
    let row = sqlx::query_as!(
        JudgeConflict,
        r#"
        SELECT id, court_id, judge_id, party_name, law_firm, corporation,
               conflict_type, start_date, end_date, notes
        FROM judge_conflicts
        WHERE id = $1 AND court_id = $2
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all conflicts for a judge.
pub async fn list_by_judge(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Uuid,
) -> Result<Vec<JudgeConflict>, AppError> {
    let rows = sqlx::query_as!(
        JudgeConflict,
        r#"
        SELECT id, court_id, judge_id, party_name, law_firm, corporation,
               conflict_type, start_date, end_date, notes
        FROM judge_conflicts
        WHERE judge_id = $1 AND court_id = $2
        ORDER BY start_date DESC
        "#,
        judge_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Delete a conflict. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM judge_conflicts WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
