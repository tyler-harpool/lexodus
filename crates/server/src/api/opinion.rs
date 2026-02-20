use dioxus::prelude::*;
use shared_types::{
    CreateHeadnoteRequest, CreateJudicialOpinionRequest, CreateOpinionDraftRequest,
    CreateOpinionVoteRequest, HeadnoteResponse, JudicialOpinionResponse, OpinionCitationResponse,
    OpinionDraftResponse, OpinionVoteResponse, PaginatedResponse, UpdateJudicialOpinionRequest,
};

// ── Opinion Server Functions ───────────────────────────

#[server]
pub async fn list_all_opinions(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<PaginatedResponse<JudicialOpinionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;
    use shared_types::PaginationMeta;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = opinion::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<JudicialOpinionResponse> =
        rows.into_iter().map(JudicialOpinionResponse::from).collect();

    let total_pages = if per_page > 0 { (total + per_page - 1) / per_page } else { 0 };
    let meta = PaginationMeta {
        total,
        page,
        limit: per_page,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    };

    Ok(PaginatedResponse {
        data: responses,
        meta,
    })
}

#[server]
pub async fn list_opinions_by_case(
    court_id: String,
    case_id: String,
) -> Result<Vec<JudicialOpinionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = opinion::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(JudicialOpinionResponse::from).collect())
}

#[server]
pub async fn list_opinions_by_judge(
    court_id: String,
    judge_id: String,
) -> Result<Vec<JudicialOpinionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid =
        Uuid::parse_str(&judge_id).map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let rows = opinion::list_by_judge(pool, &court_id, j_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(JudicialOpinionResponse::from).collect())
}

#[server]
pub async fn search_opinions(
    court_id: String,
    query: String,
) -> Result<Vec<JudicialOpinionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;

    let pool = get_db().await;
    let rows = opinion::search(pool, &court_id, &query)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(JudicialOpinionResponse::from).collect())
}

#[server]
pub async fn get_opinion(
    court_id: String,
    id: String,
) -> Result<JudicialOpinionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = opinion::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(JudicialOpinionResponse::from(row))
}

#[server]
pub async fn create_opinion(
    court_id: String,
    body: CreateJudicialOpinionRequest,
) -> Result<JudicialOpinionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;

    let pool = get_db().await;
    let row = opinion::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(JudicialOpinionResponse::from(row))
}

#[server]
pub async fn update_opinion(
    court_id: String,
    id: String,
    body: UpdateJudicialOpinionRequest,
) -> Result<JudicialOpinionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = opinion::update(pool, &court_id, uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(JudicialOpinionResponse::from(row))
}

#[server]
pub async fn delete_opinion(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    opinion::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Opinion Draft Server Functions ─────────────────────

#[server]
pub async fn list_opinion_drafts(
    court_id: String,
    opinion_id: String,
) -> Result<Vec<OpinionDraftResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion_draft;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let rows = opinion_draft::list_by_opinion(pool, &court_id, op_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(OpinionDraftResponse::from).collect())
}

#[server]
pub async fn create_opinion_draft(
    court_id: String,
    opinion_id: String,
    body: CreateOpinionDraftRequest,
) -> Result<OpinionDraftResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion_draft;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let row = opinion_draft::create(pool, &court_id, op_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(OpinionDraftResponse::from(row))
}

#[server]
pub async fn get_current_opinion_draft(
    court_id: String,
    opinion_id: String,
) -> Result<OpinionDraftResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion_draft;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let row = opinion_draft::find_current(pool, &court_id, op_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No current draft found"))?;
    Ok(OpinionDraftResponse::from(row))
}

// ── Opinion Vote Server Functions ──────────────────────

#[server]
pub async fn list_opinion_votes(
    court_id: String,
    opinion_id: String,
) -> Result<Vec<OpinionVoteResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion_vote;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let rows = opinion_vote::list_by_opinion(pool, &court_id, op_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(OpinionVoteResponse::from).collect())
}

#[server]
pub async fn create_opinion_vote(
    court_id: String,
    opinion_id: String,
    body: CreateOpinionVoteRequest,
) -> Result<OpinionVoteResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion_vote;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let row = opinion_vote::create(pool, &court_id, op_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(OpinionVoteResponse::from(row))
}

// ── Headnote Server Functions ──────────────────────────

#[server]
pub async fn list_headnotes(
    court_id: String,
    opinion_id: String,
) -> Result<Vec<HeadnoteResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::headnote;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let rows = headnote::list_by_opinion(pool, &court_id, op_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(HeadnoteResponse::from).collect())
}

#[server]
pub async fn create_headnote(
    court_id: String,
    opinion_id: String,
    body: CreateHeadnoteRequest,
) -> Result<HeadnoteResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::headnote;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let row = headnote::create(pool, &court_id, op_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(HeadnoteResponse::from(row))
}

// ── Opinion Citation Server Functions ───────────────────

#[server]
pub async fn list_opinion_citations(
    court_id: String,
    opinion_id: String,
) -> Result<Vec<OpinionCitationResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion_citation;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let rows = opinion_citation::list_by_opinion(pool, &court_id, op_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(OpinionCitationResponse::from).collect())
}
