use shared_types::{AppError, CaseAssignment, CreateCaseAssignmentRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new case assignment.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateCaseAssignmentRequest,
) -> Result<CaseAssignment, AppError> {
    let row = sqlx::query_as!(
        CaseAssignment,
        r#"
        INSERT INTO case_assignments
            (court_id, case_id, judge_id, assignment_type, reason,
             previous_judge_id, reassignment_reason)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, court_id, case_id, judge_id, assignment_type,
                  assigned_date, reason, previous_judge_id, reassignment_reason
        "#,
        court_id,
        req.case_id,
        req.judge_id,
        req.assignment_type,
        req.reason.as_deref(),
        req.previous_judge_id,
        req.reassignment_reason.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all assignments for a case.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<CaseAssignment>, AppError> {
    let rows = sqlx::query_as!(
        CaseAssignment,
        r#"
        SELECT id, court_id, case_id, judge_id, assignment_type,
               assigned_date, reason, previous_judge_id, reassignment_reason
        FROM case_assignments
        WHERE case_id = $1 AND court_id = $2
        ORDER BY assigned_date DESC
        "#,
        case_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all assignments for a judge.
pub async fn list_by_judge(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Uuid,
) -> Result<Vec<CaseAssignment>, AppError> {
    let rows = sqlx::query_as!(
        CaseAssignment,
        r#"
        SELECT id, court_id, case_id, judge_id, assignment_type,
               assigned_date, reason, previous_judge_id, reassignment_reason
        FROM case_assignments
        WHERE judge_id = $1 AND court_id = $2
        ORDER BY assigned_date DESC
        "#,
        judge_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Delete an assignment. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM case_assignments WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
