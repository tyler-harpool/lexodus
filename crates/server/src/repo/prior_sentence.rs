use shared_types::{AppError, CreatePriorSentenceRequest, PriorSentence};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new prior sentence record.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    sentencing_id: Uuid,
    req: CreatePriorSentenceRequest,
) -> Result<PriorSentence, AppError> {
    let points = req.points_assigned.unwrap_or(0);

    let row = sqlx::query_as!(
        PriorSentence,
        r#"
        INSERT INTO prior_sentences
            (court_id, sentencing_id, defendant_id, prior_case_number,
             jurisdiction, offense, conviction_date, sentence_length_months,
             points_assigned)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, court_id, sentencing_id, defendant_id, prior_case_number,
                  jurisdiction, offense, conviction_date, sentence_length_months,
                  points_assigned, created_at
        "#,
        court_id,
        sentencing_id,
        req.defendant_id,
        req.prior_case_number.as_deref(),
        req.jurisdiction,
        req.offense,
        req.conviction_date,
        req.sentence_length_months,
        points,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all prior sentences for a sentencing record.
pub async fn list_by_sentencing(
    pool: &Pool<Postgres>,
    court_id: &str,
    sentencing_id: Uuid,
) -> Result<Vec<PriorSentence>, AppError> {
    let rows = sqlx::query_as!(
        PriorSentence,
        r#"
        SELECT id, court_id, sentencing_id, defendant_id, prior_case_number,
               jurisdiction, offense, conviction_date, sentence_length_months,
               points_assigned, created_at
        FROM prior_sentences
        WHERE sentencing_id = $1 AND court_id = $2
        ORDER BY conviction_date DESC
        "#,
        sentencing_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Calculate total criminal-history points from prior sentences.
pub async fn calc_points(
    pool: &Pool<Postgres>,
    court_id: &str,
    sentencing_id: Uuid,
) -> Result<i64, AppError> {
    let result = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(SUM(points_assigned), 0) as "total!"
        FROM prior_sentences
        WHERE sentencing_id = $1 AND court_id = $2
        "#,
        sentencing_id,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result)
}
