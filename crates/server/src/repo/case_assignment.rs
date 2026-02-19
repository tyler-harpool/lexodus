use shared_types::{AppError, CaseAssignment, CreateCaseAssignmentRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new case assignment and sync `assigned_judge_id` on the parent case.
///
/// Both `criminal_cases` and `civil_cases` are updated — only the table
/// that actually contains the case_id will be affected (0-row update on the other).
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateCaseAssignmentRequest,
) -> Result<CaseAssignment, AppError> {
    let row = sqlx::query_as!(
        CaseAssignment,
        r#"
        WITH ins AS (
            INSERT INTO case_assignments
                (court_id, case_id, judge_id, assignment_type, reason,
                 previous_judge_id, reassignment_reason)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, court_id, case_id, judge_id, assignment_type,
                      assigned_date, reason, previous_judge_id, reassignment_reason
        )
        SELECT ins.id, ins.court_id, ins.case_id, ins.judge_id, ins.assignment_type,
               ins.assigned_date, ins.reason, ins.previous_judge_id, ins.reassignment_reason,
               j.name as judge_name
        FROM ins
        LEFT JOIN judges j ON ins.judge_id = j.id AND j.court_id = ins.court_id
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

    // Sync assigned_judge_id on the parent case so the UI always reflects
    // the current judge without a separate lookup.
    sqlx::query!(
        "UPDATE criminal_cases SET assigned_judge_id = $1, updated_at = NOW() WHERE id = $2 AND court_id = $3",
        req.judge_id,
        req.case_id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    sqlx::query!(
        "UPDATE civil_cases SET assigned_judge_id = $1, updated_at = NOW() WHERE id = $2 AND court_id = $3",
        req.judge_id,
        req.case_id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all assignments for a case with resolved judge names.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<CaseAssignment>, AppError> {
    let rows = sqlx::query_as!(
        CaseAssignment,
        r#"
        SELECT ca.id, ca.court_id, ca.case_id, ca.judge_id, ca.assignment_type,
               ca.assigned_date, ca.reason, ca.previous_judge_id, ca.reassignment_reason,
               j.name as judge_name
        FROM case_assignments ca
        LEFT JOIN judges j ON ca.judge_id = j.id AND j.court_id = ca.court_id
        WHERE ca.case_id = $1 AND ca.court_id = $2
        ORDER BY ca.assigned_date DESC
        "#,
        case_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all assignments for a judge with resolved judge names.
pub async fn list_by_judge(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Uuid,
) -> Result<Vec<CaseAssignment>, AppError> {
    let rows = sqlx::query_as!(
        CaseAssignment,
        r#"
        SELECT ca.id, ca.court_id, ca.case_id, ca.judge_id, ca.assignment_type,
               ca.assigned_date, ca.reason, ca.previous_judge_id, ca.reassignment_reason,
               j.name as judge_name
        FROM case_assignments ca
        LEFT JOIN judges j ON ca.judge_id = j.id AND j.court_id = ca.court_id
        WHERE ca.judge_id = $1 AND ca.court_id = $2
        ORDER BY ca.assigned_date DESC
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
