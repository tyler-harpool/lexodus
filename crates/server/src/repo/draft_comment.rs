use shared_types::{AppError, CreateDraftCommentRequest, DraftComment};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new draft comment.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    draft_id: Uuid,
    req: CreateDraftCommentRequest,
) -> Result<DraftComment, AppError> {
    let row = sqlx::query_as!(
        DraftComment,
        r#"
        INSERT INTO draft_comments
            (court_id, draft_id, author, content)
        VALUES ($1, $2, $3, $4)
        RETURNING id, court_id, draft_id, author, content,
                  resolved, resolved_at, created_at
        "#,
        court_id,
        draft_id,
        req.author,
        req.content,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Mark a draft comment as resolved.
pub async fn resolve(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<DraftComment>, AppError> {
    let row = sqlx::query_as!(
        DraftComment,
        r#"
        UPDATE draft_comments SET
            resolved    = true,
            resolved_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, draft_id, author, content,
                  resolved, resolved_at, created_at
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}
