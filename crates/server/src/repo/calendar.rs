use chrono::{DateTime, Utc};
use shared_types::{CalendarEvent, ScheduleEventRequest, UpdateEventStatusRequest, AppError};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new calendar event. Returns the created event.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: ScheduleEventRequest,
) -> Result<CalendarEvent, AppError> {
    let row = sqlx::query_as!(
        CalendarEvent,
        r#"
        INSERT INTO calendar_events (
            court_id, case_id, judge_id, event_type, scheduled_date,
            duration_minutes, courtroom, description, participants, is_public
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING
            id, court_id, case_id, judge_id, event_type, scheduled_date,
            duration_minutes, courtroom, description, participants,
            court_reporter, is_public, status, notes,
            actual_start, actual_end, call_time,
            created_at, updated_at
        "#,
        court_id,
        req.case_id,
        req.judge_id,
        req.event_type,
        req.scheduled_date,
        req.duration_minutes,
        req.courtroom,
        req.description,
        &req.participants,
        req.is_public,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a calendar event by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<CalendarEvent>, AppError> {
    let row = sqlx::query_as!(
        CalendarEvent,
        r#"
        SELECT
            id, court_id, case_id, judge_id, event_type, scheduled_date,
            duration_minutes, courtroom, description, participants,
            court_reporter, is_public, status, notes,
            actual_start, actual_end, call_time,
            created_at, updated_at
        FROM calendar_events
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

/// Update event status and optional timing fields.
pub async fn update_status(
    pool: &Pool<Postgres>,
    court_id: &str,
    event_id: Uuid,
    req: UpdateEventStatusRequest,
) -> Result<Option<CalendarEvent>, AppError> {
    let row = sqlx::query_as!(
        CalendarEvent,
        r#"
        UPDATE calendar_events SET
            status = $3,
            actual_start = COALESCE($4, actual_start),
            actual_end = COALESCE($5, actual_end),
            notes = COALESCE($6, notes),
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING
            id, court_id, case_id, judge_id, event_type, scheduled_date,
            duration_minutes, courtroom, description, participants,
            court_reporter, is_public, status, notes,
            actual_start, actual_end, call_time,
            created_at, updated_at
        "#,
        event_id,
        court_id,
        req.status,
        req.actual_start,
        req.actual_end,
        req.notes,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete a calendar event. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM calendar_events WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// List calendar events for a specific case.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<CalendarEvent>, AppError> {
    let rows = sqlx::query_as!(
        CalendarEvent,
        r#"
        SELECT
            id, court_id, case_id, judge_id, event_type, scheduled_date,
            duration_minutes, courtroom, description, participants,
            court_reporter, is_public, status, notes,
            actual_start, actual_end, call_time,
            created_at, updated_at
        FROM calendar_events
        WHERE court_id = $1 AND case_id = $2
        ORDER BY scheduled_date ASC
        "#,
        court_id,
        case_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Search calendar events with filters. Returns (events, total_count).
pub async fn search(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Option<Uuid>,
    courtroom: Option<&str>,
    event_type: Option<&str>,
    status: Option<&str>,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<CalendarEvent>, i64), AppError> {
    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM calendar_events
        WHERE court_id = $1
          AND ($2::UUID IS NULL OR judge_id = $2)
          AND ($3::TEXT IS NULL OR courtroom = $3)
          AND ($4::TEXT IS NULL OR event_type = $4)
          AND ($5::TEXT IS NULL OR status = $5)
          AND ($6::TIMESTAMPTZ IS NULL OR scheduled_date >= $6)
          AND ($7::TIMESTAMPTZ IS NULL OR scheduled_date <= $7)
        "#,
        court_id,
        judge_id as Option<Uuid>,
        courtroom as Option<&str>,
        event_type as Option<&str>,
        status as Option<&str>,
        date_from as Option<DateTime<Utc>>,
        date_to as Option<DateTime<Utc>>,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        CalendarEvent,
        r#"
        SELECT
            id, court_id, case_id, judge_id, event_type, scheduled_date,
            duration_minutes, courtroom, description, participants,
            court_reporter, is_public, status, notes,
            actual_start, actual_end, call_time,
            created_at, updated_at
        FROM calendar_events
        WHERE court_id = $1
          AND ($2::UUID IS NULL OR judge_id = $2)
          AND ($3::TEXT IS NULL OR courtroom = $3)
          AND ($4::TEXT IS NULL OR event_type = $4)
          AND ($5::TEXT IS NULL OR status = $5)
          AND ($6::TIMESTAMPTZ IS NULL OR scheduled_date >= $6)
          AND ($7::TIMESTAMPTZ IS NULL OR scheduled_date <= $7)
        ORDER BY scheduled_date ASC
        LIMIT $8 OFFSET $9
        "#,
        court_id,
        judge_id as Option<Uuid>,
        courtroom as Option<&str>,
        event_type as Option<&str>,
        status as Option<&str>,
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
