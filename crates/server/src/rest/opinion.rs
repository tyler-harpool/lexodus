use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError,
    // Opinion types
    CreateJudicialOpinionRequest, JudicialOpinionResponse, UpdateJudicialOpinionRequest,
    OpinionListParams,
    is_valid_opinion_type, is_valid_opinion_disposition, is_valid_opinion_status,
    OPINION_TYPES, OPINION_DISPOSITIONS, OPINION_STATUSES,
    // Vote types
    CreateOpinionVoteRequest, OpinionVoteResponse,
    is_valid_vote_type, VOTE_TYPES,
    // Citation types
    CreateOpinionCitationRequest, OpinionCitationResponse,
    is_valid_citation_type, CITATION_TYPES,
    // Headnote types
    CreateHeadnoteRequest, HeadnoteResponse,
    // Draft types
    CreateOpinionDraftRequest, OpinionDraftResponse,
    is_valid_draft_status, DRAFT_STATUSES,
    // Draft comment types
    CreateDraftCommentRequest, DraftCommentResponse,
    // Statistics types
    OpinionStatistics, CitationStatistics,
};
use crate::tenant::CourtId;

// ── Query params ────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

// ── Opinion CRUD ────────────────────────────────────────────────────

/// POST /api/opinions
#[utoipa::path(
    post,
    path = "/api/opinions",
    request_body = CreateJudicialOpinionRequest,
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses(
        (status = 201, description = "Opinion created", body = JudicialOpinionResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn create_opinion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateJudicialOpinionRequest>,
) -> Result<(StatusCode, Json<JudicialOpinionResponse>), AppError> {
    if !is_valid_opinion_type(&body.opinion_type) {
        return Err(AppError::bad_request(format!(
            "Invalid opinion_type: {}. Valid values: {}",
            body.opinion_type,
            OPINION_TYPES.join(", ")
        )));
    }

    if let Some(ref d) = body.disposition {
        if !is_valid_opinion_disposition(d) {
            return Err(AppError::bad_request(format!(
                "Invalid disposition: {}. Valid values: {}",
                d,
                OPINION_DISPOSITIONS.join(", ")
            )));
        }
    }

    if let Some(ref s) = body.status {
        if !is_valid_opinion_status(s) {
            return Err(AppError::bad_request(format!(
                "Invalid status: {}. Valid values: {}",
                s,
                OPINION_STATUSES.join(", ")
            )));
        }
    }

    let opinion = crate::repo::opinion::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(JudicialOpinionResponse::from(opinion))))
}

/// GET /api/opinions
#[utoipa::path(
    get,
    path = "/api/opinions",
    params(
        OpinionListParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "List of opinions", body = Vec<JudicialOpinionResponse>)
    ),
    tag = "opinions"
)]
pub async fn list_opinions(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<OpinionListParams>,
) -> Result<Json<Vec<JudicialOpinionResponse>>, AppError> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let case_uuid = params.case_id
        .as_deref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| AppError::bad_request("Invalid case_id UUID format"))?;
    let judge_uuid = params.author_judge_id
        .as_deref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| AppError::bad_request("Invalid author_judge_id UUID format"))?;

    let rows = sqlx::query_as!(
        shared_types::JudicialOpinion,
        r#"
        SELECT id, court_id, case_id, case_name, docket_number,
               author_judge_id, author_judge_name, opinion_type,
               COALESCE(disposition, '') as "disposition!",
               title,
               COALESCE(syllabus, '') as "syllabus!",
               content, status, is_published, is_precedential,
               citation_volume, citation_reporter, citation_page,
               filed_at, published_at, keywords, created_at, updated_at
        FROM judicial_opinions
        WHERE court_id = $1
          AND ($2::UUID IS NULL OR case_id = $2)
          AND ($3::UUID IS NULL OR author_judge_id = $3)
          AND ($4::BOOL IS NULL OR is_published = $4)
          AND ($5::BOOL IS NULL OR is_precedential = $5)
        ORDER BY created_at DESC
        LIMIT $6 OFFSET $7
        "#,
        &court.0,
        case_uuid,
        judge_uuid,
        params.is_published,
        params.is_precedential,
        limit,
        offset,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<JudicialOpinionResponse> = rows.into_iter().map(JudicialOpinionResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/opinions/search?q=
#[utoipa::path(
    get,
    path = "/api/opinions/search",
    params(
        ("q" = Option<String>, Query, description = "Search query"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses((status = 200, description = "Search results", body = Vec<JudicialOpinionResponse>)),
    tag = "opinions"
)]
pub async fn search_opinions(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<JudicialOpinionResponse>>, AppError> {
    let q = params.q.unwrap_or_default();
    if q.trim().is_empty() {
        return Ok(Json(vec![]));
    }
    let opinions = crate::repo::opinion::search(&pool, &court.0, &q).await?;
    let responses: Vec<JudicialOpinionResponse> = opinions
        .into_iter()
        .map(JudicialOpinionResponse::from)
        .collect();
    Ok(Json(responses))
}

/// GET /api/opinions/{opinion_id}
#[utoipa::path(
    get,
    path = "/api/opinions/{opinion_id}",
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Opinion found", body = JudicialOpinionResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn get_opinion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(opinion_id): Path<String>,
) -> Result<Json<JudicialOpinionResponse>, AppError> {
    let uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let opinion = crate::repo::opinion::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Opinion {} not found", opinion_id)))?;

    Ok(Json(JudicialOpinionResponse::from(opinion)))
}

/// PATCH /api/opinions/{opinion_id}
#[utoipa::path(
    patch,
    path = "/api/opinions/{opinion_id}",
    request_body = UpdateJudicialOpinionRequest,
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Opinion updated", body = JudicialOpinionResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn update_opinion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(opinion_id): Path<String>,
    Json(body): Json<UpdateJudicialOpinionRequest>,
) -> Result<Json<JudicialOpinionResponse>, AppError> {
    let uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref s) = body.status {
        if !is_valid_opinion_status(s) {
            return Err(AppError::bad_request(format!(
                "Invalid status: {}. Valid values: {}",
                s,
                OPINION_STATUSES.join(", ")
            )));
        }
    }

    if let Some(ref d) = body.disposition {
        if !is_valid_opinion_disposition(d) {
            return Err(AppError::bad_request(format!(
                "Invalid disposition: {}. Valid values: {}",
                d,
                OPINION_DISPOSITIONS.join(", ")
            )));
        }
    }

    let opinion = crate::repo::opinion::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Opinion {} not found", opinion_id)))?;

    Ok(Json(JudicialOpinionResponse::from(opinion)))
}

/// DELETE /api/opinions/{opinion_id}
#[utoipa::path(
    delete,
    path = "/api/opinions/{opinion_id}",
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Opinion deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn delete_opinion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(opinion_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::opinion::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Opinion {} not found", opinion_id)))
    }
}

/// GET /api/cases/{case_id}/opinions
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/opinions",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Opinions for case", body = Vec<JudicialOpinionResponse>)
    ),
    tag = "opinions"
)]
pub async fn list_opinions_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<JudicialOpinionResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let opinions = crate::repo::opinion::list_by_case(&pool, &court.0, uuid).await?;
    let responses: Vec<JudicialOpinionResponse> = opinions
        .into_iter()
        .map(JudicialOpinionResponse::from)
        .collect();

    Ok(Json(responses))
}

/// GET /api/judges/{judge_id}/opinions
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/opinions",
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Opinions by judge", body = Vec<JudicialOpinionResponse>)
    ),
    tag = "opinions"
)]
pub async fn list_opinions_by_judge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(judge_id): Path<String>,
) -> Result<Json<Vec<JudicialOpinionResponse>>, AppError> {
    let uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let opinions = crate::repo::opinion::list_by_judge(&pool, &court.0, uuid).await?;
    let responses: Vec<JudicialOpinionResponse> = opinions
        .into_iter()
        .map(JudicialOpinionResponse::from)
        .collect();

    Ok(Json(responses))
}

// ── Votes ───────────────────────────────────────────────────────────

/// POST /api/opinions/{opinion_id}/votes
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/votes",
    request_body = CreateOpinionVoteRequest,
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Vote added", body = OpinionVoteResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn add_vote(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(opinion_id): Path<String>,
    Json(body): Json<CreateOpinionVoteRequest>,
) -> Result<(StatusCode, Json<OpinionVoteResponse>), AppError> {
    let opinion_uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !is_valid_vote_type(&body.vote_type) {
        return Err(AppError::bad_request(format!(
            "Invalid vote_type: {}. Valid values: {}",
            body.vote_type,
            VOTE_TYPES.join(", ")
        )));
    }

    let vote = crate::repo::opinion_vote::create(&pool, &court.0, opinion_uuid, body).await?;
    Ok((StatusCode::CREATED, Json(OpinionVoteResponse::from(vote))))
}

// ── Citations ───────────────────────────────────────────────────────

/// POST /api/opinions/{opinion_id}/citations
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/citations",
    request_body = CreateOpinionCitationRequest,
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Citation added", body = OpinionCitationResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn add_citation(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(opinion_id): Path<String>,
    Json(body): Json<CreateOpinionCitationRequest>,
) -> Result<(StatusCode, Json<OpinionCitationResponse>), AppError> {
    let opinion_uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !is_valid_citation_type(&body.citation_type) {
        return Err(AppError::bad_request(format!(
            "Invalid citation_type: {}. Valid values: {}",
            body.citation_type,
            CITATION_TYPES.join(", ")
        )));
    }

    let citation = crate::repo::opinion_citation::create(&pool, &court.0, opinion_uuid, body).await?;
    Ok((StatusCode::CREATED, Json(OpinionCitationResponse::from(citation))))
}

// ── Headnotes ───────────────────────────────────────────────────────

/// POST /api/opinions/{opinion_id}/headnotes
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/headnotes",
    request_body = CreateHeadnoteRequest,
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Headnote added", body = HeadnoteResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn add_headnote(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(opinion_id): Path<String>,
    Json(body): Json<CreateHeadnoteRequest>,
) -> Result<(StatusCode, Json<HeadnoteResponse>), AppError> {
    let opinion_uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if body.topic.trim().is_empty() {
        return Err(AppError::bad_request("topic must not be empty"));
    }

    let headnote = crate::repo::headnote::create(&pool, &court.0, opinion_uuid, body).await?;
    Ok((StatusCode::CREATED, Json(HeadnoteResponse::from(headnote))))
}

// ── Drafts ──────────────────────────────────────────────────────────

/// POST /api/opinions/{opinion_id}/drafts
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/drafts",
    request_body = CreateOpinionDraftRequest,
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Draft created", body = OpinionDraftResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn create_draft(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(opinion_id): Path<String>,
    Json(body): Json<CreateOpinionDraftRequest>,
) -> Result<(StatusCode, Json<OpinionDraftResponse>), AppError> {
    let opinion_uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref s) = body.status {
        if !is_valid_draft_status(s) {
            return Err(AppError::bad_request(format!(
                "Invalid status: {}. Valid values: {}",
                s,
                DRAFT_STATUSES.join(", ")
            )));
        }
    }

    let draft = crate::repo::opinion_draft::create(&pool, &court.0, opinion_uuid, body).await?;
    Ok((StatusCode::CREATED, Json(OpinionDraftResponse::from(draft))))
}

/// GET /api/opinions/{opinion_id}/drafts
#[utoipa::path(
    get,
    path = "/api/opinions/{opinion_id}/drafts",
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Drafts for opinion", body = Vec<OpinionDraftResponse>)
    ),
    tag = "opinions"
)]
pub async fn list_drafts(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(opinion_id): Path<String>,
) -> Result<Json<Vec<OpinionDraftResponse>>, AppError> {
    let uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let drafts = crate::repo::opinion_draft::list_by_opinion(&pool, &court.0, uuid).await?;
    let responses: Vec<OpinionDraftResponse> = drafts
        .into_iter()
        .map(OpinionDraftResponse::from)
        .collect();

    Ok(Json(responses))
}

/// GET /api/opinions/{opinion_id}/drafts/current
#[utoipa::path(
    get,
    path = "/api/opinions/{opinion_id}/drafts/current",
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Current draft", body = OpinionDraftResponse),
        (status = 404, description = "No current draft", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn get_current_draft(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(opinion_id): Path<String>,
) -> Result<Json<OpinionDraftResponse>, AppError> {
    let uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let draft = crate::repo::opinion_draft::find_current(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!("No current draft for opinion {}", opinion_id))
        })?;

    Ok(Json(OpinionDraftResponse::from(draft)))
}

// ── Draft Comments ──────────────────────────────────────────────────

/// POST /api/opinions/{opinion_id}/drafts/{draft_id}/comments
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/drafts/{draft_id}/comments",
    request_body = CreateDraftCommentRequest,
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("draft_id" = String, Path, description = "Draft UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Comment added", body = DraftCommentResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn add_draft_comment(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((opinion_id, draft_id)): Path<(String, String)>,
    Json(body): Json<CreateDraftCommentRequest>,
) -> Result<(StatusCode, Json<DraftCommentResponse>), AppError> {
    let _opinion_uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format for opinion_id"))?;
    let draft_uuid = Uuid::parse_str(&draft_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format for draft_id"))?;

    if body.author.trim().is_empty() {
        return Err(AppError::bad_request("author must not be empty"));
    }

    let comment = crate::repo::draft_comment::create(&pool, &court.0, draft_uuid, body).await?;
    Ok((StatusCode::CREATED, Json(DraftCommentResponse::from(comment))))
}

/// PATCH /api/opinions/{opinion_id}/drafts/{draft_id}/comments/{comment_id}/resolve
#[utoipa::path(
    patch,
    path = "/api/opinions/{opinion_id}/drafts/{draft_id}/comments/{comment_id}/resolve",
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("draft_id" = String, Path, description = "Draft UUID"),
        ("comment_id" = String, Path, description = "Comment UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Comment resolved", body = DraftCommentResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn resolve_comment(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((opinion_id, draft_id, comment_id)): Path<(String, String, String)>,
) -> Result<Json<DraftCommentResponse>, AppError> {
    let _opinion_uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format for opinion_id"))?;
    let _draft_uuid = Uuid::parse_str(&draft_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format for draft_id"))?;
    let comment_uuid = Uuid::parse_str(&comment_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format for comment_id"))?;

    let comment = crate::repo::draft_comment::resolve(&pool, &court.0, comment_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Comment {} not found", comment_id)))?;

    Ok(Json(DraftCommentResponse::from(comment)))
}

// ── Extended opinion handlers ───────────────────────────────────────

/// POST /api/opinions/{opinion_id}/file
/// File an opinion (set status to "Filed" and record filed_at).
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/file",
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Opinion filed", body = JudicialOpinionResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn file_opinion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(opinion_id): Path<String>,
) -> Result<Json<JudicialOpinionResponse>, AppError> {
    let uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let opinion = sqlx::query_as!(
        shared_types::JudicialOpinion,
        r#"
        UPDATE judicial_opinions SET
            status = 'Filed',
            filed_at = NOW(),
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, case_name, docket_number,
                  author_judge_id, author_judge_name, opinion_type,
                  COALESCE(disposition, '') as "disposition!",
                  title,
                  COALESCE(syllabus, '') as "syllabus!",
                  content, status, is_published, is_precedential,
                  citation_volume, citation_reporter, citation_page,
                  filed_at, published_at, keywords, created_at, updated_at
        "#,
        uuid,
        &court.0,
    )
    .fetch_optional(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found(format!("Opinion {} not found", opinion_id)))?;

    Ok(Json(JudicialOpinionResponse::from(opinion)))
}

/// POST /api/opinions/{opinion_id}/publish
/// Publish an opinion (set is_published = true and record published_at).
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/publish",
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Opinion published", body = JudicialOpinionResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn publish_opinion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(opinion_id): Path<String>,
) -> Result<Json<JudicialOpinionResponse>, AppError> {
    let uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let opinion = sqlx::query_as!(
        shared_types::JudicialOpinion,
        r#"
        UPDATE judicial_opinions SET
            status = 'Published',
            is_published = true,
            published_at = NOW(),
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, case_name, docket_number,
                  author_judge_id, author_judge_name, opinion_type,
                  COALESCE(disposition, '') as "disposition!",
                  title,
                  COALESCE(syllabus, '') as "syllabus!",
                  content, status, is_published, is_precedential,
                  citation_volume, citation_reporter, citation_page,
                  filed_at, published_at, keywords, created_at, updated_at
        "#,
        uuid,
        &court.0,
    )
    .fetch_optional(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found(format!("Opinion {} not found", opinion_id)))?;

    Ok(Json(JudicialOpinionResponse::from(opinion)))
}

/// GET /api/opinions/{opinion_id}/is-majority
/// Check if an opinion is a majority opinion.
#[utoipa::path(
    get,
    path = "/api/opinions/{opinion_id}/is-majority",
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Majority check result", body = bool),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn check_is_majority(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(opinion_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let opinion = crate::repo::opinion::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Opinion {} not found", opinion_id)))?;

    let is_majority = opinion.opinion_type == "Majority" || opinion.opinion_type == "Per Curiam";
    Ok(Json(serde_json::json!({ "is_majority": is_majority })))
}

/// GET /api/opinions/{opinion_id}/is-binding
/// Check if an opinion is binding precedent (published + precedential).
#[utoipa::path(
    get,
    path = "/api/opinions/{opinion_id}/is-binding",
    params(
        ("opinion_id" = String, Path, description = "Opinion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Binding check result", body = bool),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "opinions"
)]
pub async fn check_is_binding(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(opinion_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uuid = Uuid::parse_str(&opinion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let opinion = crate::repo::opinion::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Opinion {} not found", opinion_id)))?;

    let is_binding = opinion.is_published && opinion.is_precedential;
    Ok(Json(serde_json::json!({ "is_binding": is_binding })))
}

/// GET /api/opinions/statistics
/// Calculate aggregate opinion statistics for a court.
#[utoipa::path(
    get,
    path = "/api/opinions/statistics",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Opinion statistics", body = OpinionStatistics)),
    tag = "opinions"
)]
pub async fn calculate_statistics(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<OpinionStatistics>, AppError> {
    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM judicial_opinions WHERE court_id = $1"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let by_type: serde_json::Value = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(json_object_agg(opinion_type, cnt), '{}')::TEXT as "json!"
        FROM (SELECT opinion_type, COUNT(*) as cnt FROM judicial_opinions WHERE court_id = $1 GROUP BY opinion_type) sub
        "#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map(|s| serde_json::from_str(&s).unwrap_or(serde_json::Value::Object(Default::default())))
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let precedential_count: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM judicial_opinions WHERE court_id = $1 AND is_precedential = true"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let avg_days_to_publish: Option<f64> = sqlx::query_scalar!(
        r#"
        SELECT AVG(EXTRACT(EPOCH FROM (published_at - created_at)) / 86400.0)::FLOAT8 as "avg: f64"
        FROM judicial_opinions
        WHERE court_id = $1 AND published_at IS NOT NULL
        "#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    Ok(Json(OpinionStatistics {
        total,
        by_type,
        precedential_count,
        avg_days_to_publish,
    }))
}

/// GET /api/opinions/precedential
/// List all precedential opinions.
#[utoipa::path(
    get,
    path = "/api/opinions/precedential",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Precedential opinions", body = Vec<JudicialOpinionResponse>)),
    tag = "opinions"
)]
pub async fn list_precedential(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<JudicialOpinionResponse>>, AppError> {
    let opinions = sqlx::query_as!(
        shared_types::JudicialOpinion,
        r#"
        SELECT id, court_id, case_id, case_name, docket_number,
               author_judge_id, author_judge_name, opinion_type,
               COALESCE(disposition, '') as "disposition!",
               title,
               COALESCE(syllabus, '') as "syllabus!",
               content, status, is_published, is_precedential,
               citation_volume, citation_reporter, citation_page,
               filed_at, published_at, keywords, created_at, updated_at
        FROM judicial_opinions
        WHERE court_id = $1 AND is_precedential = true
        ORDER BY published_at DESC NULLS LAST
        "#,
        &court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<JudicialOpinionResponse> = opinions
        .into_iter()
        .map(JudicialOpinionResponse::from)
        .collect();
    Ok(Json(responses))
}

/// GET /api/opinions/citations/statistics
/// Get citation statistics for a court.
#[utoipa::path(
    get,
    path = "/api/opinions/citations/statistics",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Citation statistics", body = CitationStatistics)),
    tag = "opinions"
)]
pub async fn citation_statistics(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<CitationStatistics>, AppError> {
    let total_citations: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM opinion_citations WHERE court_id = $1"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    // Get the most-cited opinions (top 10)
    let most_cited_rows = sqlx::query!(
        r#"
        SELECT oc.cited_opinion_id as "cited_opinion_id!", COUNT(*) as "cite_count!"
        FROM opinion_citations oc
        WHERE oc.court_id = $1 AND oc.cited_opinion_id IS NOT NULL
        GROUP BY oc.cited_opinion_id
        ORDER BY COUNT(*) DESC
        LIMIT 10
        "#,
        &court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let most_cited: Vec<serde_json::Value> = most_cited_rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "opinion_id": row.cited_opinion_id.to_string(),
                "citation_count": row.cite_count,
            })
        })
        .collect();

    Ok(Json(CitationStatistics {
        total_citations,
        most_cited,
    }))
}
