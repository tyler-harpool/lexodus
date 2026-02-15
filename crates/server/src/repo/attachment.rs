use shared_types::{AppError, DocketAttachment};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a pending attachment row (uploaded_at = NULL).
/// The row becomes visible to list queries only after `mark_uploaded` is called.
pub async fn create_pending(
    pool: &Pool<Postgres>,
    court_id: &str,
    docket_entry_id: Uuid,
    filename: &str,
    file_size: i64,
    content_type: &str,
    storage_key: &str,
) -> Result<DocketAttachment, AppError> {
    sqlx::query_as!(
        DocketAttachment,
        r#"
        INSERT INTO docket_attachments
            (court_id, docket_entry_id, filename, file_size, content_type, storage_key)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, court_id, docket_entry_id, filename, file_size, content_type,
                  storage_key, sealed, encryption, sha256, uploaded_at, created_at, updated_at
        "#,
        court_id,
        docket_entry_id,
        filename,
        file_size,
        content_type,
        storage_key,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Mark an attachment as uploaded (sets uploaded_at = NOW()).
pub async fn mark_uploaded(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        r#"
        UPDATE docket_attachments
        SET uploaded_at = NOW(), updated_at = NOW()
        WHERE id = $1 AND court_id = $2 AND uploaded_at IS NULL
        "#,
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// List uploaded attachments for a docket entry (only rows where uploaded_at IS NOT NULL).
pub async fn list_by_entry(
    pool: &Pool<Postgres>,
    court_id: &str,
    docket_entry_id: Uuid,
) -> Result<Vec<DocketAttachment>, AppError> {
    sqlx::query_as!(
        DocketAttachment,
        r#"
        SELECT id, court_id, docket_entry_id, filename, file_size, content_type,
               storage_key, sealed, encryption, sha256, uploaded_at, created_at, updated_at
        FROM docket_attachments
        WHERE court_id = $1
          AND docket_entry_id = $2
          AND uploaded_at IS NOT NULL
        ORDER BY created_at DESC
        "#,
        court_id,
        docket_entry_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Find a single attachment by ID within a court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<DocketAttachment>, AppError> {
    sqlx::query_as!(
        DocketAttachment,
        r#"
        SELECT id, court_id, docket_entry_id, filename, file_size, content_type,
               storage_key, sealed, encryption, sha256, uploaded_at, created_at, updated_at
        FROM docket_attachments
        WHERE id = $1 AND court_id = $2
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Count uploaded attachments for a docket entry (for badge display).
pub async fn count_by_entry(
    pool: &Pool<Postgres>,
    court_id: &str,
    docket_entry_id: Uuid,
) -> Result<i64, AppError> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM docket_attachments
        WHERE court_id = $1
          AND docket_entry_id = $2
          AND uploaded_at IS NOT NULL
        "#,
        court_id,
        docket_entry_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}
