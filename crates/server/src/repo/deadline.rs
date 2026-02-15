use chrono::{DateTime, Utc};
use shared_types::{AppError, CreateDeadlineRequest, Deadline, UpdateDeadlineRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new deadline. Returns the created row.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateDeadlineRequest,
) -> Result<Deadline, AppError> {
    let row = sqlx::query_as!(
        Deadline,
        r#"
        INSERT INTO deadlines (court_id, case_id, title, rule_code, due_at, notes)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, court_id, case_id, title, rule_code, due_at, status, notes, created_at, updated_at
        "#,
        court_id,
        req.case_id,
        req.title,
        req.rule_code,
        req.due_at,
        req.notes,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a deadline by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Deadline>, AppError> {
    let row = sqlx::query_as!(
        Deadline,
        r#"
        SELECT id, court_id, case_id, title, rule_code, due_at, status, notes, created_at, updated_at
        FROM deadlines
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

/// Update a deadline (partial update). Returns the updated row or None if not found.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: UpdateDeadlineRequest,
) -> Result<Option<Deadline>, AppError> {
    let row = sqlx::query_as!(
        Deadline,
        r#"
        UPDATE deadlines SET
            title     = COALESCE($3, title),
            case_id   = COALESCE($4, case_id),
            rule_code = COALESCE($5, rule_code),
            due_at    = COALESCE($6, due_at),
            notes     = COALESCE($7, notes),
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, title, rule_code, due_at, status, notes, created_at, updated_at
        "#,
        id,
        court_id,
        req.title,
        req.case_id,
        req.rule_code,
        req.due_at,
        req.notes,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete a deadline. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM deadlines WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// Update only the status of a deadline. Returns the updated row or None.
pub async fn update_status(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    status: &str,
) -> Result<Option<Deadline>, AppError> {
    let row = sqlx::query_as!(
        Deadline,
        r#"
        UPDATE deadlines SET
            status = $3,
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, title, rule_code, due_at, status, notes, created_at, updated_at
        "#,
        id,
        court_id,
        status,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Search deadlines with filters. Returns (deadlines, total_count).
pub async fn search(
    pool: &Pool<Postgres>,
    court_id: &str,
    status: Option<&str>,
    case_id: Option<Uuid>,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<Deadline>, i64), AppError> {
    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM deadlines
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR status = $2)
          AND ($3::UUID IS NULL OR case_id = $3)
          AND ($4::TIMESTAMPTZ IS NULL OR due_at >= $4)
          AND ($5::TIMESTAMPTZ IS NULL OR due_at <= $5)
        "#,
        court_id,
        status as Option<&str>,
        case_id as Option<Uuid>,
        date_from as Option<DateTime<Utc>>,
        date_to as Option<DateTime<Utc>>,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        Deadline,
        r#"
        SELECT id, court_id, case_id, title, rule_code, due_at, status, notes, created_at, updated_at
        FROM deadlines
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR status = $2)
          AND ($3::UUID IS NULL OR case_id = $3)
          AND ($4::TIMESTAMPTZ IS NULL OR due_at >= $4)
          AND ($5::TIMESTAMPTZ IS NULL OR due_at <= $5)
        ORDER BY due_at ASC
        LIMIT $6 OFFSET $7
        "#,
        court_id,
        status as Option<&str>,
        case_id as Option<Uuid>,
        date_from as Option<DateTime<Utc>>,
        date_to as Option<DateTime<Utc>>,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, total))
}
