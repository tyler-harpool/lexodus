use chrono::{DateTime, Utc};
use shared_types::{AppError, QueueItem, QueueStats};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    queue_type: &str,
    priority: i32,
    title: &str,
    description: Option<&str>,
    source_type: &str,
    source_id: Uuid,
    case_id: Option<Uuid>,
    case_number: Option<&str>,
    submitted_by: Option<i64>,
    metadata: Option<serde_json::Value>,
    first_step: &str,
) -> Result<QueueItem, AppError> {
    sqlx::query_as!(
        QueueItem,
        r#"
        INSERT INTO clerk_queue
            (court_id, queue_type, priority, title, description,
             source_type, source_id, case_id, case_number, submitted_by,
             metadata, current_step)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, court_id, queue_type, priority, status, title,
                  description, source_type, source_id, case_id, case_type, case_number,
                  assigned_to, submitted_by, current_step,
                  metadata, created_at, updated_at, completed_at
        "#,
        court_id,
        queue_type,
        priority,
        title,
        description,
        source_type,
        source_id,
        case_id,
        case_number,
        submitted_by,
        metadata.unwrap_or(serde_json::json!({})),
        first_step,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<QueueItem>, AppError> {
    sqlx::query_as!(
        QueueItem,
        r#"
        SELECT id, court_id, queue_type, priority, status, title,
               description, source_type, source_id, case_id, case_type, case_number,
               assigned_to, submitted_by, current_step,
               metadata, created_at, updated_at, completed_at
        FROM clerk_queue
        WHERE id = $1 AND court_id = $2
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

pub async fn search(
    pool: &Pool<Postgres>,
    court_id: &str,
    status: Option<&str>,
    queue_type: Option<&str>,
    priority: Option<i32>,
    assigned_to: Option<i64>,
    case_id: Option<Uuid>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<QueueItem>, i64), AppError> {
    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM clerk_queue
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR status = $2)
          AND ($3::TEXT IS NULL OR queue_type = $3)
          AND ($4::INT IS NULL OR priority = $4)
          AND ($5::BIGINT IS NULL OR assigned_to = $5)
          AND ($6::UUID IS NULL OR case_id = $6)
        "#,
        court_id,
        status,
        queue_type,
        priority,
        assigned_to,
        case_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        QueueItem,
        r#"
        SELECT id, court_id, queue_type, priority, status, title,
               description, source_type, source_id, case_id, case_type, case_number,
               assigned_to, submitted_by, current_step,
               metadata, created_at, updated_at, completed_at
        FROM clerk_queue
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR status = $2)
          AND ($3::TEXT IS NULL OR queue_type = $3)
          AND ($4::INT IS NULL OR priority = $4)
          AND ($5::BIGINT IS NULL OR assigned_to = $5)
          AND ($6::UUID IS NULL OR case_id = $6)
        ORDER BY priority ASC, created_at ASC
        LIMIT $7 OFFSET $8
        "#,
        court_id,
        status,
        queue_type,
        priority,
        assigned_to,
        case_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, total))
}

pub async fn stats(
    pool: &Pool<Postgres>,
    court_id: &str,
    user_id: Option<i64>,
) -> Result<QueueStats, AppError> {
    let pending_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM clerk_queue WHERE court_id = $1 AND status = 'pending'"#,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let my_count = match user_id {
        Some(uid) => sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM clerk_queue WHERE court_id = $1 AND assigned_to = $2 AND status IN ('in_review', 'processing')"#,
            court_id,
            uid,
        )
        .fetch_one(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?,
        None => 0,
    };

    let today_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM clerk_queue WHERE court_id = $1 AND created_at >= CURRENT_DATE AND status NOT IN ('completed', 'rejected')"#,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let urgent_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM clerk_queue WHERE court_id = $1 AND priority <= 2 AND status NOT IN ('completed', 'rejected')"#,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let avg_processing_mins: Option<f64> = sqlx::query_scalar!(
        r#"
        SELECT (EXTRACT(EPOCH FROM AVG(completed_at - created_at)) / 60.0)::FLOAT8 as "avg?"
        FROM clerk_queue
        WHERE court_id = $1 AND status = 'completed' AND completed_at IS NOT NULL
        "#,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(QueueStats {
        pending_count,
        my_count,
        today_count,
        urgent_count,
        avg_processing_mins,
    })
}

pub async fn claim(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    user_id: i64,
) -> Result<Option<QueueItem>, AppError> {
    sqlx::query_as!(
        QueueItem,
        r#"
        UPDATE clerk_queue SET
            assigned_to = $3,
            status = 'in_review',
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2 AND assigned_to IS NULL AND status = 'pending'
        RETURNING id, court_id, queue_type, priority, status, title,
                  description, source_type, source_id, case_id, case_type, case_number,
                  assigned_to, submitted_by, current_step,
                  metadata, created_at, updated_at, completed_at
        "#,
        id,
        court_id,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

pub async fn release(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    user_id: i64,
) -> Result<Option<QueueItem>, AppError> {
    sqlx::query_as!(
        QueueItem,
        r#"
        UPDATE clerk_queue SET
            assigned_to = NULL,
            status = 'pending',
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2 AND assigned_to = $3
        RETURNING id, court_id, queue_type, priority, status, title,
                  description, source_type, source_id, case_id, case_type, case_number,
                  assigned_to, submitted_by, current_step,
                  metadata, created_at, updated_at, completed_at
        "#,
        id,
        court_id,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

pub async fn advance(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    next_step: &str,
    new_status: &str,
    completed_at: Option<DateTime<Utc>>,
) -> Result<Option<QueueItem>, AppError> {
    sqlx::query_as!(
        QueueItem,
        r#"
        UPDATE clerk_queue SET
            current_step = $3,
            status = $4,
            completed_at = $5,
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, queue_type, priority, status, title,
                  description, source_type, source_id, case_id, case_type, case_number,
                  assigned_to, submitted_by, current_step,
                  metadata, created_at, updated_at, completed_at
        "#,
        id,
        court_id,
        next_step,
        new_status,
        completed_at,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

pub async fn reject(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    reason: &str,
) -> Result<Option<QueueItem>, AppError> {
    let metadata_patch = serde_json::json!({ "reject_reason": reason });
    sqlx::query_as!(
        QueueItem,
        r#"
        UPDATE clerk_queue SET
            status = 'rejected',
            metadata = metadata || $3,
            completed_at = NOW(),
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, queue_type, priority, status, title,
                  description, source_type, source_id, case_id, case_type, case_number,
                  assigned_to, submitted_by, current_step,
                  metadata, created_at, updated_at, completed_at
        "#,
        id,
        court_id,
        metadata_patch,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}
