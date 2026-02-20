use shared_types::{AppError, CreateMotionRequest, Motion, MotionResponse};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Intermediate row for pending motions joined with case number.
#[derive(sqlx::FromRow)]
struct PendingMotionRow {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub case_number: Option<String>,
    pub motion_type: String,
    pub filed_by: String,
    pub description: String,
    pub filed_date: chrono::DateTime<chrono::Utc>,
    pub status: String,
    pub ruling_date: Option<chrono::DateTime<chrono::Utc>>,
    pub ruling_text: Option<String>,
}

/// Insert a new motion.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateMotionRequest,
) -> Result<Motion, AppError> {
    let status = req.status.as_deref().unwrap_or("Pending");

    let row = sqlx::query_as!(
        Motion,
        r#"
        INSERT INTO motions
            (court_id, case_id, motion_type, filed_by, description,
             filed_date, status, ruling_date, ruling_text)
        VALUES ($1, $2, $3, $4, $5, COALESCE($6, NOW()), $7, $8, $9)
        RETURNING id, court_id, case_id, motion_type, filed_by, description,
                  filed_date, status, ruling_date, ruling_text
        "#,
        court_id,
        req.case_id,
        req.motion_type,
        req.filed_by,
        req.description,
        req.filed_date,
        status,
        req.ruling_date,
        req.ruling_text.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a motion by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Motion>, AppError> {
    let row = sqlx::query_as!(
        Motion,
        r#"
        SELECT id, court_id, case_id, motion_type, filed_by, description,
               filed_date, status, ruling_date, ruling_text
        FROM motions
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

/// List all motions for a given case within a court.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<Motion>, AppError> {
    let rows = sqlx::query_as!(
        Motion,
        r#"
        SELECT id, court_id, case_id, motion_type, filed_by, description,
               filed_date, status, ruling_date, ruling_text
        FROM motions
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

/// Update a motion with only the provided fields.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: shared_types::UpdateMotionRequest,
) -> Result<Option<Motion>, AppError> {
    let row = sqlx::query_as!(
        Motion,
        r#"
        UPDATE motions SET
            motion_type = COALESCE($3, motion_type),
            filed_by    = COALESCE($4, filed_by),
            description = COALESCE($5, description),
            status      = COALESCE($6, status),
            ruling_date = COALESCE($7, ruling_date),
            ruling_text = COALESCE($8, ruling_text)
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, motion_type, filed_by, description,
                  filed_date, status, ruling_date, ruling_text
        "#,
        id,
        court_id,
        req.motion_type.as_deref(),
        req.filed_by.as_deref(),
        req.description.as_deref(),
        req.status.as_deref(),
        req.ruling_date,
        req.ruling_text.as_deref(),
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete a motion. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM motions WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// List all pending motions for cases assigned to a specific judge.
///
/// Joins motions with case_assignments to find motions on the judge's
/// active cases, and resolves the case number from criminal_cases or
/// civil_cases via COALESCE.
pub async fn list_pending_for_judge(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Uuid,
) -> Result<Vec<MotionResponse>, AppError> {
    let rows = sqlx::query_as!(
        PendingMotionRow,
        r#"
        SELECT m.id, m.court_id, m.case_id,
               COALESCE(cc.case_number, cv.case_number) as "case_number?",
               m.motion_type, m.filed_by, m.description,
               m.filed_date, m.status, m.ruling_date, m.ruling_text
        FROM motions m
        JOIN case_assignments ca
            ON m.case_id = ca.case_id
            AND ca.judge_id = $2
            AND ca.court_id = $1
        LEFT JOIN criminal_cases cc ON m.case_id = cc.id
        LEFT JOIN civil_cases cv ON m.case_id = cv.id
        WHERE m.court_id = $1 AND m.status = 'Pending'
        ORDER BY m.filed_date ASC
        "#,
        court_id,
        judge_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let responses = rows
        .into_iter()
        .map(|r| MotionResponse {
            id: r.id.to_string(),
            case_id: r.case_id.to_string(),
            case_number: r.case_number,
            motion_type: r.motion_type,
            filed_by: r.filed_by,
            description: r.description,
            filed_date: r.filed_date.to_rfc3339(),
            status: r.status,
            ruling_date: r.ruling_date.map(|dt| dt.to_rfc3339()),
            ruling_text: r.ruling_text,
        })
        .collect();

    Ok(responses)
}
