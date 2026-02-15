use shared_types::{AppError, CreateOpinionVoteRequest, OpinionVote};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new opinion vote.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    opinion_id: Uuid,
    req: CreateOpinionVoteRequest,
) -> Result<OpinionVote, AppError> {
    let row = sqlx::query_as!(
        OpinionVote,
        r#"
        INSERT INTO opinion_votes
            (court_id, opinion_id, judge_id, vote_type, notes)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, court_id, opinion_id, judge_id, vote_type, joined_at, notes
        "#,
        court_id,
        opinion_id,
        req.judge_id,
        req.vote_type,
        req.notes.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all votes for a given opinion within a court.
pub async fn list_by_opinion(
    pool: &Pool<Postgres>,
    court_id: &str,
    opinion_id: Uuid,
) -> Result<Vec<OpinionVote>, AppError> {
    let rows = sqlx::query_as!(
        OpinionVote,
        r#"
        SELECT id, court_id, opinion_id, judge_id, vote_type, joined_at, notes
        FROM opinion_votes
        WHERE opinion_id = $1 AND court_id = $2
        ORDER BY joined_at ASC
        "#,
        opinion_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}
