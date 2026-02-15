use chrono::{DateTime, Utc};
use shared_types::{AppError, CreateExtensionRequest, ExtensionRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new extension request for a deadline.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    deadline_id: Uuid,
    req: CreateExtensionRequest,
) -> Result<ExtensionRequest, AppError> {
    let row = sqlx::query_as!(
        ExtensionRequest,
        r#"
        INSERT INTO extension_requests
            (court_id, deadline_id, requested_by, reason, requested_new_date)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, court_id, deadline_id, requested_by, reason,
                  request_date, requested_new_date, status,
                  ruling_by, ruling_date, new_deadline_date
        "#,
        court_id,
        deadline_id,
        req.requested_by,
        req.reason,
        req.requested_new_date,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find an extension request by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<ExtensionRequest>, AppError> {
    let row = sqlx::query_as!(
        ExtensionRequest,
        r#"
        SELECT id, court_id, deadline_id, requested_by, reason,
               request_date, requested_new_date, status,
               ruling_by, ruling_date, new_deadline_date
        FROM extension_requests
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

/// List all extension requests for a specific deadline.
pub async fn list_by_deadline(
    pool: &Pool<Postgres>,
    court_id: &str,
    deadline_id: Uuid,
) -> Result<Vec<ExtensionRequest>, AppError> {
    let rows = sqlx::query_as!(
        ExtensionRequest,
        r#"
        SELECT id, court_id, deadline_id, requested_by, reason,
               request_date, requested_new_date, status,
               ruling_by, ruling_date, new_deadline_date
        FROM extension_requests
        WHERE court_id = $1 AND deadline_id = $2
        ORDER BY request_date DESC
        "#,
        court_id,
        deadline_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all pending extension requests for the court.
pub async fn list_pending(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<ExtensionRequest>, AppError> {
    let rows = sqlx::query_as!(
        ExtensionRequest,
        r#"
        SELECT id, court_id, deadline_id, requested_by, reason,
               request_date, requested_new_date, status,
               ruling_by, ruling_date, new_deadline_date
        FROM extension_requests
        WHERE court_id = $1 AND status = 'Pending'
        ORDER BY request_date ASC
        "#,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Update the ruling on an extension request.
pub async fn update_ruling(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    status: &str,
    ruling_by: &str,
    new_deadline_date: Option<DateTime<Utc>>,
) -> Result<Option<ExtensionRequest>, AppError> {
    let row = sqlx::query_as!(
        ExtensionRequest,
        r#"
        UPDATE extension_requests SET
            status = $3,
            ruling_by = $4,
            ruling_date = NOW(),
            new_deadline_date = $5
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, deadline_id, requested_by, reason,
                  request_date, requested_new_date, status,
                  ruling_by, ruling_date, new_deadline_date
        "#,
        id,
        court_id,
        status,
        ruling_by,
        new_deadline_date,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}
