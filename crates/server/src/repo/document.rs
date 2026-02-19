use shared_types::{AppError, Document, SealingLevel};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Promote a docket attachment into a canonical document.
///
/// This copies the attachment metadata (storage_key, file_size, content_type,
/// checksum) into the documents table and links back via `source_attachment_id`.
///
/// If a document already exists for this attachment, returns the existing document
/// (idempotent).
pub async fn promote_attachment(
    pool: &Pool<Postgres>,
    court_id: &str,
    attachment_id: Uuid,
    case_id: Uuid,
    title: &str,
    document_type: &str,
    storage_key: &str,
    file_size: i64,
    content_type: &str,
    checksum: &str,
) -> Result<Document, AppError> {
    // Use ON CONFLICT on the unique index (source_attachment_id) for idempotency.
    // If a document already exists for this attachment, return it unchanged.
    sqlx::query_as!(
        Document,
        r#"
        INSERT INTO documents
            (court_id, case_id, title, document_type, storage_key,
             checksum, file_size, content_type, is_sealed, uploaded_by,
             source_attachment_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, FALSE, 'system', $9)
        ON CONFLICT (source_attachment_id) WHERE source_attachment_id IS NOT NULL
        DO UPDATE SET court_id = documents.court_id
        RETURNING id, court_id, case_id, title, document_type, storage_key,
                  checksum, file_size, content_type, is_sealed, uploaded_by,
                  source_attachment_id, created_at,
                  sealing_level, seal_reason_code, seal_motion_id,
                  replaced_by_document_id, is_stricken
        "#,
        court_id,
        case_id,
        title,
        document_type,
        storage_key,
        checksum,
        file_size,
        content_type,
        attachment_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Find a document by its source attachment ID within a court.
pub async fn find_by_source_attachment(
    pool: &Pool<Postgres>,
    court_id: &str,
    attachment_id: Uuid,
) -> Result<Option<Document>, AppError> {
    sqlx::query_as!(
        Document,
        r#"
        SELECT id, court_id, case_id, title, document_type, storage_key,
               checksum, file_size, content_type, is_sealed, uploaded_by,
               source_attachment_id, created_at,
               sealing_level, seal_reason_code, seal_motion_id,
               replaced_by_document_id, is_stricken
        FROM documents
        WHERE court_id = $1 AND source_attachment_id = $2
        "#,
        court_id,
        attachment_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Find a document by ID within a court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Document>, AppError> {
    sqlx::query_as!(
        Document,
        r#"
        SELECT id, court_id, case_id, title, document_type, storage_key,
               checksum, file_size, content_type, is_sealed, uploaded_by,
               source_attachment_id, created_at,
               sealing_level, seal_reason_code, seal_motion_id,
               replaced_by_document_id, is_stricken
        FROM documents
        WHERE court_id = $1 AND id = $2
        "#,
        court_id,
        id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Seal a document: sets sealing_level, reason, optional motion, and is_sealed = true.
pub async fn seal(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    level: &SealingLevel,
    reason_code: &str,
    motion_id: Option<Uuid>,
) -> Result<Document, AppError> {
    sqlx::query_as!(
        Document,
        r#"
        UPDATE documents
        SET is_sealed = true,
            sealing_level = $3,
            seal_reason_code = $4,
            seal_motion_id = $5
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, title, document_type, storage_key,
                  checksum, file_size, content_type, is_sealed, uploaded_by,
                  source_attachment_id, created_at,
                  sealing_level, seal_reason_code, seal_motion_id,
                  replaced_by_document_id, is_stricken
        "#,
        id,
        court_id,
        level.as_db_str(),
        reason_code,
        motion_id as Option<Uuid>,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found("Document not found"))
}

/// Unseal a document: resets sealing to Public, clears reason and motion.
pub async fn unseal(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Document, AppError> {
    sqlx::query_as!(
        Document,
        r#"
        UPDATE documents
        SET is_sealed = false,
            sealing_level = 'Public',
            seal_reason_code = NULL,
            seal_motion_id = NULL
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, title, document_type, storage_key,
                  checksum, file_size, content_type, is_sealed, uploaded_by,
                  source_attachment_id, created_at,
                  sealing_level, seal_reason_code, seal_motion_id,
                  replaced_by_document_id, is_stricken
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found("Document not found"))
}

/// Grace period in minutes for in-place document replacement.
/// Within this window, the document is updated in-place (no new version created).
/// After the window, a full replace creates a new document and strikes the original.
/// Defaults to 15 minutes. Configured via `FILING_REPLACE_GRACE_MINUTES` env var.
fn grace_period_minutes() -> i64 {
    std::env::var("FILING_REPLACE_GRACE_MINUTES")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(15)
}

/// Replace a document. Within the grace period (default 15 min), updates the
/// document in-place. After the grace period, creates a new document and marks
/// the original as replaced + stricken. Original files are never deleted.
///
/// Returns the (potentially updated) document.
pub async fn replace(
    pool: &Pool<Postgres>,
    court_id: &str,
    original_id: Uuid,
    title: &str,
    storage_key: &str,
    file_size: i64,
    content_type: &str,
    checksum: &str,
) -> Result<Document, AppError> {
    // Fetch the original to inherit case, type, etc.
    let original = find_by_id(pool, court_id, original_id)
        .await?
        .ok_or_else(|| AppError::not_found("Document not found"))?;

    if original.is_stricken {
        return Err(AppError::bad_request("Cannot replace a stricken document"));
    }
    if original.replaced_by_document_id.is_some() {
        return Err(AppError::bad_request("Document has already been replaced"));
    }

    // Check if within grace period for in-place update
    let age_minutes = (chrono::Utc::now() - original.created_at).num_minutes();
    let grace = grace_period_minutes();

    if age_minutes < grace {
        // Within grace period: update the document row in-place
        let updated = sqlx::query_as!(
            Document,
            r#"
            UPDATE documents
            SET title = $3, storage_key = $4, checksum = $5,
                file_size = $6, content_type = $7
            WHERE id = $1 AND court_id = $2
            RETURNING id, court_id, case_id, title, document_type, storage_key,
                      checksum, file_size, content_type, is_sealed, uploaded_by,
                      source_attachment_id, created_at,
                      sealing_level, seal_reason_code, seal_motion_id,
                      replaced_by_document_id, is_stricken
            "#,
            original_id,
            court_id,
            title,
            storage_key,
            checksum,
            file_size,
            content_type,
        )
        .fetch_one(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?;

        return Ok(updated);
    }

    // After grace period: create replacement document + strike original
    let replacement = sqlx::query_as!(
        Document,
        r#"
        INSERT INTO documents
            (court_id, case_id, title, document_type, storage_key,
             checksum, file_size, content_type, is_sealed, uploaded_by,
             sealing_level, seal_reason_code, seal_motion_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id, court_id, case_id, title, document_type, storage_key,
                  checksum, file_size, content_type, is_sealed, uploaded_by,
                  source_attachment_id, created_at,
                  sealing_level, seal_reason_code, seal_motion_id,
                  replaced_by_document_id, is_stricken
        "#,
        court_id,
        original.case_id,
        title,
        original.document_type.as_str(),
        storage_key,
        checksum,
        file_size,
        content_type,
        original.is_sealed,
        original.uploaded_by.as_str(),
        original.sealing_level.as_str(),
        original.seal_reason_code.as_deref(),
        original.seal_motion_id as Option<Uuid>,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    // Mark original as replaced + stricken
    sqlx::query!(
        r#"
        UPDATE documents
        SET replaced_by_document_id = $3, is_stricken = true
        WHERE id = $1 AND court_id = $2
        "#,
        original_id,
        court_id,
        replacement.id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(replacement)
}

/// Strike a document from the record (without replacement).
pub async fn strike(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Document, AppError> {
    sqlx::query_as!(
        Document,
        r#"
        UPDATE documents
        SET is_stricken = true
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, title, document_type, storage_key,
                  checksum, file_size, content_type, is_sealed, uploaded_by,
                  source_attachment_id, created_at,
                  sealing_level, seal_reason_code, seal_motion_id,
                  replaced_by_document_id, is_stricken
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found("Document not found"))
}

/// List all documents for a court with optional title search, paginated.
pub async fn list_all(
    pool: &Pool<Postgres>,
    court_id: &str,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<Document>, i64), AppError> {
    let search = q.map(|s| format!("%{}%", s.to_lowercase()));

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!" FROM documents
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR LOWER(title) LIKE $2)
        "#,
        court_id,
        search.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        Document,
        r#"
        SELECT id, court_id, case_id, title, document_type, storage_key,
               checksum, file_size, content_type, is_sealed, uploaded_by,
               source_attachment_id, created_at,
               sealing_level, seal_reason_code, seal_motion_id,
               replaced_by_document_id, is_stricken
        FROM documents
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR LOWER(title) LIKE $2)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
        court_id,
        search.as_deref(),
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, total))
}
