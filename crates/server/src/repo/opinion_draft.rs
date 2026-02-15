use shared_types::{AppError, CreateOpinionDraftRequest, OpinionDraft};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new opinion draft with auto-incremented version.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    opinion_id: Uuid,
    req: CreateOpinionDraftRequest,
) -> Result<OpinionDraft, AppError> {
    let status = req.status.as_deref().unwrap_or("Draft");

    let row = sqlx::query_as!(
        OpinionDraft,
        r#"
        INSERT INTO opinion_drafts
            (court_id, opinion_id, version, content, status, author_id)
        VALUES (
            $1, $2,
            (SELECT COALESCE(MAX(version), 0) + 1 FROM opinion_drafts WHERE opinion_id = $2 AND court_id = $1),
            $3, $4, $5
        )
        RETURNING id, court_id, opinion_id, version, content, status, author_id, created_at
        "#,
        court_id,
        opinion_id,
        req.content,
        status,
        req.author_id.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all drafts for a given opinion within a court, newest first.
pub async fn list_by_opinion(
    pool: &Pool<Postgres>,
    court_id: &str,
    opinion_id: Uuid,
) -> Result<Vec<OpinionDraft>, AppError> {
    let rows = sqlx::query_as!(
        OpinionDraft,
        r#"
        SELECT id, court_id, opinion_id, version, content, status, author_id, created_at
        FROM opinion_drafts
        WHERE opinion_id = $1 AND court_id = $2
        ORDER BY version DESC
        "#,
        opinion_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Find the most recent draft for a given opinion within a court.
pub async fn find_current(
    pool: &Pool<Postgres>,
    court_id: &str,
    opinion_id: Uuid,
) -> Result<Option<OpinionDraft>, AppError> {
    let row = sqlx::query_as!(
        OpinionDraft,
        r#"
        SELECT id, court_id, opinion_id, version, content, status, author_id, created_at
        FROM opinion_drafts
        WHERE opinion_id = $1 AND court_id = $2
        ORDER BY version DESC
        LIMIT 1
        "#,
        opinion_id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}
