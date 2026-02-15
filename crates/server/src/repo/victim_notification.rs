use shared_types::{AppError, VictimNotification};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Send (create) a new victim notification.
pub async fn send(
    pool: &Pool<Postgres>,
    court_id: &str,
    victim_id: Uuid,
    notification_type: &str,
    method: &str,
    content_summary: &str,
) -> Result<VictimNotification, AppError> {
    let row = sqlx::query_as!(
        VictimNotification,
        r#"
        INSERT INTO victim_notifications
            (court_id, victim_id, notification_type, method, content_summary)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, court_id, victim_id, notification_type, sent_at,
                  method, content_summary, acknowledged, acknowledged_at
        "#,
        court_id,
        victim_id,
        notification_type,
        method,
        content_summary,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all notifications for a specific victim.
pub async fn list_by_victim(
    pool: &Pool<Postgres>,
    court_id: &str,
    victim_id: Uuid,
) -> Result<Vec<VictimNotification>, AppError> {
    let rows = sqlx::query_as!(
        VictimNotification,
        r#"
        SELECT id, court_id, victim_id, notification_type, sent_at,
               method, content_summary, acknowledged, acknowledged_at
        FROM victim_notifications
        WHERE court_id = $1 AND victim_id = $2
        ORDER BY sent_at DESC
        "#,
        court_id,
        victim_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}
