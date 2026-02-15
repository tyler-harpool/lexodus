use shared_types::{AppError, CreateHeadnoteRequest, Headnote};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new headnote.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    opinion_id: Uuid,
    req: CreateHeadnoteRequest,
) -> Result<Headnote, AppError> {
    let row = sqlx::query_as!(
        Headnote,
        r#"
        INSERT INTO opinion_headnotes
            (court_id, opinion_id, headnote_number, topic, text, key_number)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, court_id, opinion_id, headnote_number, topic, text, key_number
        "#,
        court_id,
        opinion_id,
        req.headnote_number,
        req.topic,
        req.text,
        req.key_number.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all headnotes for a given opinion within a court.
pub async fn list_by_opinion(
    pool: &Pool<Postgres>,
    court_id: &str,
    opinion_id: Uuid,
) -> Result<Vec<Headnote>, AppError> {
    let rows = sqlx::query_as!(
        Headnote,
        r#"
        SELECT id, court_id, opinion_id, headnote_number, topic, text, key_number
        FROM opinion_headnotes
        WHERE opinion_id = $1 AND court_id = $2
        ORDER BY headnote_number ASC
        "#,
        opinion_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}
