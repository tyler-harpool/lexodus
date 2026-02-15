use shared_types::{AppError, CaseNote, CreateCaseNoteRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new case note.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateCaseNoteRequest,
) -> Result<CaseNote, AppError> {
    let row = sqlx::query_as!(
        CaseNote,
        r#"
        INSERT INTO case_notes
            (court_id, case_id, author, content, note_type, is_private)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, court_id, case_id, author, content, note_type,
                  is_private, created_at
        "#,
        court_id,
        req.case_id,
        req.author,
        req.content,
        req.note_type,
        req.is_private,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a case note by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<CaseNote>, AppError> {
    let row = sqlx::query_as!(
        CaseNote,
        r#"
        SELECT id, court_id, case_id, author, content, note_type,
               is_private, created_at
        FROM case_notes
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

/// List all notes for a given case within a court.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<CaseNote>, AppError> {
    let rows = sqlx::query_as!(
        CaseNote,
        r#"
        SELECT id, court_id, case_id, author, content, note_type,
               is_private, created_at
        FROM case_notes
        WHERE case_id = $1 AND court_id = $2
        ORDER BY created_at ASC
        "#,
        case_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Update a case note with only the provided fields.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: shared_types::UpdateCaseNoteRequest,
) -> Result<Option<CaseNote>, AppError> {
    let row = sqlx::query_as!(
        CaseNote,
        r#"
        UPDATE case_notes SET
            content   = COALESCE($3, content),
            note_type = COALESCE($4, note_type),
            is_private = COALESCE($5, is_private)
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, author, content, note_type,
                  is_private, created_at
        "#,
        id,
        court_id,
        req.content.as_deref(),
        req.note_type.as_deref(),
        req.is_private,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete a case note. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM case_notes WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
