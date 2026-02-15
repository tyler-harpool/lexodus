use shared_types::{AppError, CreateOpinionCitationRequest, OpinionCitation};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new opinion citation.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    opinion_id: Uuid,
    req: CreateOpinionCitationRequest,
) -> Result<OpinionCitation, AppError> {
    let row = sqlx::query_as!(
        OpinionCitation,
        r#"
        INSERT INTO opinion_citations
            (court_id, opinion_id, cited_opinion_id, citation_text,
             citation_type, context, pinpoint_cite)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, court_id, opinion_id, cited_opinion_id,
                  citation_text, citation_type, context, pinpoint_cite
        "#,
        court_id,
        opinion_id,
        req.cited_opinion_id,
        req.citation_text,
        req.citation_type,
        req.context.as_deref(),
        req.pinpoint_cite.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all citations for a given opinion within a court.
pub async fn list_by_opinion(
    pool: &Pool<Postgres>,
    court_id: &str,
    opinion_id: Uuid,
) -> Result<Vec<OpinionCitation>, AppError> {
    let rows = sqlx::query_as!(
        OpinionCitation,
        r#"
        SELECT id, court_id, opinion_id, cited_opinion_id,
               citation_text, citation_type, context, pinpoint_cite
        FROM opinion_citations
        WHERE opinion_id = $1 AND court_id = $2
        "#,
        opinion_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}
