use shared_types::{AppError, DeadlineReminder};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Create (send) a new deadline reminder.
pub async fn send(
    pool: &Pool<Postgres>,
    court_id: &str,
    deadline_id: Uuid,
    recipient: &str,
    reminder_type: &str,
) -> Result<DeadlineReminder, AppError> {
    let row = sqlx::query_as!(
        DeadlineReminder,
        r#"
        INSERT INTO deadline_reminders (court_id, deadline_id, recipient, reminder_type)
        VALUES ($1, $2, $3, $4)
        RETURNING id, court_id, deadline_id, recipient, reminder_type,
                  sent_at, acknowledged, acknowledged_at
        "#,
        court_id,
        deadline_id,
        recipient,
        reminder_type,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all reminders for a specific deadline.
pub async fn list_by_deadline(
    pool: &Pool<Postgres>,
    court_id: &str,
    deadline_id: Uuid,
) -> Result<Vec<DeadlineReminder>, AppError> {
    let rows = sqlx::query_as!(
        DeadlineReminder,
        r#"
        SELECT id, court_id, deadline_id, recipient, reminder_type,
               sent_at, acknowledged, acknowledged_at
        FROM deadline_reminders
        WHERE court_id = $1 AND deadline_id = $2
        ORDER BY sent_at DESC
        "#,
        court_id,
        deadline_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all reminders for a specific recipient.
pub async fn list_by_recipient(
    pool: &Pool<Postgres>,
    court_id: &str,
    recipient: &str,
) -> Result<Vec<DeadlineReminder>, AppError> {
    let rows = sqlx::query_as!(
        DeadlineReminder,
        r#"
        SELECT id, court_id, deadline_id, recipient, reminder_type,
               sent_at, acknowledged, acknowledged_at
        FROM deadline_reminders
        WHERE court_id = $1 AND recipient = $2
        ORDER BY sent_at DESC
        "#,
        court_id,
        recipient,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all unacknowledged (pending) reminders for the court.
pub async fn list_pending(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<DeadlineReminder>, AppError> {
    let rows = sqlx::query_as!(
        DeadlineReminder,
        r#"
        SELECT id, court_id, deadline_id, recipient, reminder_type,
               sent_at, acknowledged, acknowledged_at
        FROM deadline_reminders
        WHERE court_id = $1 AND acknowledged = false
        ORDER BY sent_at ASC
        "#,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Acknowledge a reminder. Returns the updated row or None.
pub async fn acknowledge(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<DeadlineReminder>, AppError> {
    let row = sqlx::query_as!(
        DeadlineReminder,
        r#"
        UPDATE deadline_reminders SET
            acknowledged = true,
            acknowledged_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, deadline_id, recipient, reminder_type,
                  sent_at, acknowledged, acknowledged_at
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}
