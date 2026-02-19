use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError,
    // Judge types
    CreateJudgeRequest, JudgeResponse, UpdateJudgeRequest, UpdateJudgeStatusRequest,
    is_valid_judge_title, is_valid_judge_status, JUDGE_TITLES, JUDGE_STATUSES,
    JudgeWorkload, AssignmentHistory,
    // Conflict types
    CreateJudgeConflictRequest, JudgeConflictResponse,
    is_valid_conflict_type, CONFLICT_TYPES,
    // Assignment types
    CreateCaseAssignmentRequest, CaseAssignmentResponse,
    is_valid_assignment_type, ASSIGNMENT_TYPES,
    // Recusal types
    CreateRecusalMotionRequest, RecusalMotionResponse,
    UpdateRecusalRulingRequest,
    is_valid_recusal_status, RECUSAL_STATUSES,
};
use crate::tenant::CourtId;

// ── Query params ────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

// ── Judge CRUD ──────────────────────────────────────────────────────

/// POST /api/judges
#[utoipa::path(
    post,
    path = "/api/judges",
    request_body = CreateJudgeRequest,
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses(
        (status = 201, description = "Judge created", body = JudgeResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "judges"
)]
pub async fn create_judge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateJudgeRequest>,
) -> Result<(StatusCode, Json<JudgeResponse>), AppError> {
    if body.name.trim().is_empty() {
        return Err(AppError::bad_request("name must not be empty"));
    }
    if !is_valid_judge_title(&body.title) {
        return Err(AppError::bad_request(format!(
            "Invalid title: {}. Valid values: {}", body.title, JUDGE_TITLES.join(", ")
        )));
    }
    if let Some(ref s) = body.status {
        if !is_valid_judge_status(s) {
            return Err(AppError::bad_request(format!(
                "Invalid status: {}. Valid values: {}", s, JUDGE_STATUSES.join(", ")
            )));
        }
    }

    let judge = crate::repo::judge::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(JudgeResponse::from(judge))))
}

/// GET /api/judges
#[utoipa::path(
    get,
    path = "/api/judges",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "List of judges", body = Vec<JudgeResponse>)),
    tag = "judges"
)]
pub async fn list_judges(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<JudgeResponse>>, AppError> {
    let judges = crate::repo::judge::list_by_court(&pool, &court.0).await?;
    let responses: Vec<JudgeResponse> = judges.into_iter().map(JudgeResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/judges/search?q=
#[utoipa::path(
    get,
    path = "/api/judges/search",
    params(
        ("q" = Option<String>, Query, description = "Search query"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses((status = 200, description = "Search results", body = Vec<JudgeResponse>)),
    tag = "judges"
)]
pub async fn search_judges(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<JudgeResponse>>, AppError> {
    let q = params.q.unwrap_or_default();
    if q.trim().is_empty() {
        return Ok(Json(vec![]));
    }
    let judges = crate::repo::judge::search(&pool, &court.0, &q).await?;
    let responses: Vec<JudgeResponse> = judges.into_iter().map(JudgeResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/judges/status/{status}
#[utoipa::path(
    get,
    path = "/api/judges/status/{status}",
    params(
        ("status" = String, Path, description = "Judge status filter"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses((status = 200, description = "Judges with status", body = Vec<JudgeResponse>)),
    tag = "judges"
)]
pub async fn list_judges_by_status(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(status): Path<String>,
) -> Result<Json<Vec<JudgeResponse>>, AppError> {
    if !is_valid_judge_status(&status) {
        return Err(AppError::bad_request(format!(
            "Invalid status: {}. Valid values: {}", status, JUDGE_STATUSES.join(", ")
        )));
    }
    let judges = crate::repo::judge::list_by_status(&pool, &court.0, &status).await?;
    let responses: Vec<JudgeResponse> = judges.into_iter().map(JudgeResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/judges/{id}
#[utoipa::path(
    get,
    path = "/api/judges/{id}",
    params(
        ("id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Judge found", body = JudgeResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "judges"
)]
pub async fn get_judge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<JudgeResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let judge = crate::repo::judge::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Judge {} not found", id)))?;
    Ok(Json(JudgeResponse::from(judge)))
}

/// PUT /api/judges/{id}
#[utoipa::path(
    put,
    path = "/api/judges/{id}",
    request_body = UpdateJudgeRequest,
    params(
        ("id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Judge updated", body = JudgeResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "judges"
)]
pub async fn update_judge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateJudgeRequest>,
) -> Result<Json<JudgeResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref n) = body.name {
        if n.trim().is_empty() {
            return Err(AppError::bad_request("name must not be empty"));
        }
    }
    if let Some(ref t) = body.title {
        if !is_valid_judge_title(t) {
            return Err(AppError::bad_request(format!(
                "Invalid title: {}. Valid values: {}", t, JUDGE_TITLES.join(", ")
            )));
        }
    }
    if let Some(ref s) = body.status {
        if !is_valid_judge_status(s) {
            return Err(AppError::bad_request(format!(
                "Invalid status: {}. Valid values: {}", s, JUDGE_STATUSES.join(", ")
            )));
        }
    }

    let judge = crate::repo::judge::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Judge {} not found", id)))?;
    Ok(Json(JudgeResponse::from(judge)))
}

/// DELETE /api/judges/{id}
#[utoipa::path(
    delete,
    path = "/api/judges/{id}",
    params(
        ("id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Judge deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "judges"
)]
pub async fn delete_judge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let deleted = crate::repo::judge::delete(&pool, &court.0, uuid).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Judge {} not found", id)))
    }
}

/// PATCH /api/judges/{id}/status
#[utoipa::path(
    patch,
    path = "/api/judges/{id}/status",
    request_body = UpdateJudgeStatusRequest,
    params(
        ("id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Status updated", body = JudgeResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "judges"
)]
pub async fn update_judge_status(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateJudgeStatusRequest>,
) -> Result<Json<JudgeResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !is_valid_judge_status(&body.status) {
        return Err(AppError::bad_request(format!(
            "Invalid status: {}. Valid values: {}", body.status, JUDGE_STATUSES.join(", ")
        )));
    }

    let judge = crate::repo::judge::update_status(&pool, &court.0, uuid, &body.status)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Judge {} not found", id)))?;
    Ok(Json(JudgeResponse::from(judge)))
}

// ── Conflicts ───────────────────────────────────────────────────────

/// POST /api/judges/{judge_id}/conflicts
#[utoipa::path(
    post,
    path = "/api/judges/{judge_id}/conflicts",
    request_body = CreateJudgeConflictRequest,
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Conflict created", body = JudgeConflictResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "judges"
)]
pub async fn create_conflict(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(judge_id): Path<String>,
    Json(body): Json<CreateJudgeConflictRequest>,
) -> Result<(StatusCode, Json<JudgeConflictResponse>), AppError> {
    let judge_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !is_valid_conflict_type(&body.conflict_type) {
        return Err(AppError::bad_request(format!(
            "Invalid conflict_type: {}. Valid values: {}", body.conflict_type, CONFLICT_TYPES.join(", ")
        )));
    }

    let conflict = crate::repo::judge_conflict::create(&pool, &court.0, judge_uuid, body).await?;
    Ok((StatusCode::CREATED, Json(JudgeConflictResponse::from(conflict))))
}

/// GET /api/judges/{judge_id}/conflicts
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/conflicts",
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses((status = 200, description = "Conflicts for judge", body = Vec<JudgeConflictResponse>)),
    tag = "judges"
)]
pub async fn list_conflicts(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(judge_id): Path<String>,
) -> Result<Json<Vec<JudgeConflictResponse>>, AppError> {
    let judge_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let conflicts = crate::repo::judge_conflict::list_by_judge(&pool, &court.0, judge_uuid).await?;
    let responses: Vec<JudgeConflictResponse> = conflicts.into_iter().map(JudgeConflictResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/judges/{judge_id}/conflicts/{conflict_id}
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/conflicts/{conflict_id}",
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("conflict_id" = String, Path, description = "Conflict UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Conflict found", body = JudgeConflictResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "judges"
)]
pub async fn get_conflict(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((_judge_id, conflict_id)): Path<(String, String)>,
) -> Result<Json<JudgeConflictResponse>, AppError> {
    let conflict_uuid = Uuid::parse_str(&conflict_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let conflict = crate::repo::judge_conflict::find_by_id(&pool, &court.0, conflict_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Conflict {} not found", conflict_id)))?;
    Ok(Json(JudgeConflictResponse::from(conflict)))
}

/// DELETE /api/judges/{judge_id}/conflicts/{conflict_id}
#[utoipa::path(
    delete,
    path = "/api/judges/{judge_id}/conflicts/{conflict_id}",
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("conflict_id" = String, Path, description = "Conflict UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Conflict deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "judges"
)]
pub async fn delete_conflict(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((_judge_id, conflict_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let conflict_uuid = Uuid::parse_str(&conflict_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let deleted = crate::repo::judge_conflict::delete(&pool, &court.0, conflict_uuid).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Conflict {} not found", conflict_id)))
    }
}

// ── Assignments ─────────────────────────────────────────────────────

/// POST /api/judges/assignments
#[utoipa::path(
    post,
    path = "/api/judges/assignments",
    request_body = CreateCaseAssignmentRequest,
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses(
        (status = 201, description = "Assignment created", body = CaseAssignmentResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "judges"
)]
pub async fn create_assignment(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateCaseAssignmentRequest>,
) -> Result<(StatusCode, Json<CaseAssignmentResponse>), AppError> {
    if !is_valid_assignment_type(&body.assignment_type) {
        return Err(AppError::bad_request(format!(
            "Invalid assignment_type: {}. Valid values: {}", body.assignment_type, ASSIGNMENT_TYPES.join(", ")
        )));
    }
    let assignment = crate::repo::case_assignment::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(CaseAssignmentResponse::from(assignment))))
}

/// GET /api/cases/{case_id}/assignment
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/assignment",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses((status = 200, description = "Assignments for case", body = Vec<CaseAssignmentResponse>)),
    tag = "judges"
)]
pub async fn list_assignments_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<CaseAssignmentResponse>>, AppError> {
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let assignments = crate::repo::case_assignment::list_by_case(&pool, &court.0, case_uuid).await?;
    let responses: Vec<CaseAssignmentResponse> = assignments.into_iter().map(CaseAssignmentResponse::from).collect();
    Ok(Json(responses))
}

/// DELETE /api/assignments/{id}
#[utoipa::path(
    delete,
    path = "/api/assignments/{id}",
    params(
        ("id" = String, Path, description = "Assignment UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Assignment deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "judges"
)]
pub async fn delete_assignment(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let deleted = crate::repo::case_assignment::delete(&pool, &court.0, uuid).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Assignment {} not found", id)))
    }
}

// ── Recusals ────────────────────────────────────────────────────────

/// POST /api/judges/{judge_id}/recusals
#[utoipa::path(
    post,
    path = "/api/judges/{judge_id}/recusals",
    request_body = CreateRecusalMotionRequest,
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Recusal created", body = RecusalMotionResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "judges"
)]
pub async fn create_recusal(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(judge_id): Path<String>,
    Json(body): Json<CreateRecusalMotionRequest>,
) -> Result<(StatusCode, Json<RecusalMotionResponse>), AppError> {
    let judge_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    if body.filed_by.trim().is_empty() {
        return Err(AppError::bad_request("filed_by must not be empty"));
    }
    if body.reason.trim().is_empty() {
        return Err(AppError::bad_request("reason must not be empty"));
    }
    let recusal = crate::repo::recusal_motion::create(&pool, &court.0, judge_uuid, body).await?;
    Ok((StatusCode::CREATED, Json(RecusalMotionResponse::from(recusal))))
}

/// PATCH /api/recusals/{recusal_id}/ruling
#[utoipa::path(
    patch,
    path = "/api/recusals/{recusal_id}/ruling",
    request_body = UpdateRecusalRulingRequest,
    params(
        ("recusal_id" = String, Path, description = "Recusal UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Ruling updated", body = RecusalMotionResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "judges"
)]
pub async fn update_recusal_ruling(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(recusal_id): Path<String>,
    Json(body): Json<UpdateRecusalRulingRequest>,
) -> Result<Json<RecusalMotionResponse>, AppError> {
    let uuid = Uuid::parse_str(&recusal_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !is_valid_recusal_status(&body.status) {
        return Err(AppError::bad_request(format!(
            "Invalid status: {}. Valid values: {}", body.status, RECUSAL_STATUSES.join(", ")
        )));
    }

    let recusal = crate::repo::recusal_motion::update_ruling(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Recusal {} not found", recusal_id)))?;
    Ok(Json(RecusalMotionResponse::from(recusal)))
}

/// GET /api/recusals/pending
#[utoipa::path(
    get,
    path = "/api/recusals/pending",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Pending recusals", body = Vec<RecusalMotionResponse>)),
    tag = "judges"
)]
pub async fn list_pending_recusals(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<RecusalMotionResponse>>, AppError> {
    let recusals = crate::repo::recusal_motion::list_pending(&pool, &court.0).await?;
    let responses: Vec<RecusalMotionResponse> = recusals.into_iter().map(RecusalMotionResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/cases/{case_id}/recusals
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/recusals",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses((status = 200, description = "Recusals for case", body = Vec<RecusalMotionResponse>)),
    tag = "judges"
)]
pub async fn list_recusals_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<RecusalMotionResponse>>, AppError> {
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let recusals = crate::repo::recusal_motion::list_by_case(&pool, &court.0, case_uuid).await?;
    let responses: Vec<RecusalMotionResponse> = recusals.into_iter().map(RecusalMotionResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/recusals/judge/{judge_id}
#[utoipa::path(
    get,
    path = "/api/recusals/judge/{judge_id}",
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses((status = 200, description = "Recusals for judge", body = Vec<RecusalMotionResponse>)),
    tag = "judges"
)]
pub async fn list_recusals_by_judge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(judge_id): Path<String>,
) -> Result<Json<Vec<RecusalMotionResponse>>, AppError> {
    let judge_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let recusals = crate::repo::recusal_motion::list_by_judge(&pool, &court.0, judge_uuid).await?;
    let responses: Vec<RecusalMotionResponse> = recusals.into_iter().map(RecusalMotionResponse::from).collect();
    Ok(Json(responses))
}

// ── Extended judge handlers ─────────────────────────────────────────

/// GET /api/judges/available
/// List judges who have capacity (current_caseload < max_caseload) and are Active.
#[utoipa::path(
    get,
    path = "/api/judges/available",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Available judges", body = Vec<JudgeResponse>)),
    tag = "judges"
)]
pub async fn list_available(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<JudgeResponse>>, AppError> {
    let judges = crate::repo::judge::list_by_court(&pool, &court.0).await?;
    let available: Vec<JudgeResponse> = judges
        .into_iter()
        .filter(|j| j.status == "Active" && j.current_caseload < j.max_caseload)
        .map(JudgeResponse::from)
        .collect();
    Ok(Json(available))
}

/// GET /api/judges/{id}/workload
/// Get workload summary for a judge.
#[utoipa::path(
    get,
    path = "/api/judges/{id}/workload",
    params(
        ("id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Judge workload", body = JudgeWorkload),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "judges"
)]
pub async fn get_workload(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<JudgeWorkload>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let judge = crate::repo::judge::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Judge {} not found", id)))?;

    // Count active cases from assignments
    let active_cases: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM case_assignments WHERE judge_id = $1 AND court_id = $2"#,
        uuid,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    // Count pending motions via case_assignments join (motions table has no judge_id)
    let pending_motions: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!"
           FROM motions m
           JOIN case_assignments ca ON ca.case_id = m.case_id AND ca.court_id = m.court_id
           WHERE ca.judge_id = $1 AND m.court_id = $2 AND m.status = 'Pending'"#,
        uuid,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    // Count upcoming hearings
    let upcoming_hearings: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM calendar_events WHERE judge_id = $1 AND court_id = $2 AND scheduled_date > NOW()"#,
        uuid,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    Ok(Json(JudgeWorkload {
        judge_id: judge.id.to_string(),
        judge_name: judge.name,
        active_cases,
        pending_motions,
        upcoming_hearings,
    }))
}

/// GET /api/judges/district/{district}
/// List judges in a specific district.
#[utoipa::path(
    get,
    path = "/api/judges/district/{district}",
    params(
        ("district" = String, Path, description = "District name"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses((status = 200, description = "Judges in district", body = Vec<JudgeResponse>)),
    tag = "judges"
)]
pub async fn list_by_district(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(district): Path<String>,
) -> Result<Json<Vec<JudgeResponse>>, AppError> {
    let judges = crate::repo::judge::list_by_court(&pool, &court.0).await?;
    let filtered: Vec<JudgeResponse> = judges
        .into_iter()
        .filter(|j| j.district == district)
        .map(JudgeResponse::from)
        .collect();
    Ok(Json(filtered))
}

/// GET /api/judges/vacation
/// List judges currently on vacation (status = "Inactive" with a courtroom note).
#[utoipa::path(
    get,
    path = "/api/judges/vacation",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Judges on vacation", body = Vec<JudgeResponse>)),
    tag = "judges"
)]
pub async fn list_on_vacation(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<JudgeResponse>>, AppError> {
    let judges = crate::repo::judge::list_by_status(&pool, &court.0, "Inactive").await?;
    let responses: Vec<JudgeResponse> = judges.into_iter().map(JudgeResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/judges/conflicts/check/{party_name}
/// Check if any judge has a conflict involving a specific party name.
#[utoipa::path(
    get,
    path = "/api/judges/conflicts/check/{party_name}",
    params(
        ("party_name" = String, Path, description = "Party name to check"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses((status = 200, description = "Judges with conflicts for party", body = Vec<JudgeConflictResponse>)),
    tag = "judges"
)]
pub async fn check_conflicts_for_party(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(party_name): Path<String>,
) -> Result<Json<Vec<JudgeConflictResponse>>, AppError> {
    let pattern = format!("%{}%", party_name);
    let conflicts = sqlx::query_as!(
        shared_types::JudgeConflict,
        r#"
        SELECT id, court_id, judge_id, party_name, law_firm, corporation,
               conflict_type, start_date, end_date, notes
        FROM judge_conflicts
        WHERE court_id = $1
          AND (party_name ILIKE $2 OR law_firm ILIKE $2 OR corporation ILIKE $2)
          AND (end_date IS NULL OR end_date > NOW())
        ORDER BY start_date DESC
        "#,
        &court.0,
        pattern,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<JudgeConflictResponse> = conflicts
        .into_iter()
        .map(JudgeConflictResponse::from)
        .collect();
    Ok(Json(responses))
}

/// GET /api/cases/{case_id}/assignment-history
/// Get the full assignment history for a case.
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/assignment-history",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Assignment history", body = AssignmentHistory),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "judges"
)]
pub async fn get_assignment_history(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<AssignmentHistory>, AppError> {
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let assignments = sqlx::query_as!(
        shared_types::CaseAssignment,
        r#"
        SELECT ca.id, ca.court_id, ca.case_id, ca.judge_id, ca.assignment_type,
               ca.assigned_date, ca.reason, ca.previous_judge_id, ca.reassignment_reason,
               j.name as judge_name
        FROM case_assignments ca
        LEFT JOIN judges j ON ca.judge_id = j.id AND j.court_id = ca.court_id
        WHERE ca.case_id = $1 AND ca.court_id = $2
        ORDER BY ca.assigned_date DESC
        "#,
        case_uuid,
        &court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let entries: Vec<CaseAssignmentResponse> = assignments
        .into_iter()
        .map(CaseAssignmentResponse::from)
        .collect();

    Ok(Json(AssignmentHistory { entries }))
}

/// POST /api/recusals/{recusal_id}/process
/// Process a pending recusal (grant or deny).
#[utoipa::path(
    post,
    path = "/api/recusals/{recusal_id}/process",
    request_body = UpdateRecusalRulingRequest,
    params(
        ("recusal_id" = String, Path, description = "Recusal UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Recusal processed", body = RecusalMotionResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "judges"
)]
pub async fn process_recusal(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(recusal_id): Path<String>,
    Json(body): Json<UpdateRecusalRulingRequest>,
) -> Result<Json<RecusalMotionResponse>, AppError> {
    let uuid = Uuid::parse_str(&recusal_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !is_valid_recusal_status(&body.status) {
        return Err(AppError::bad_request(format!(
            "Invalid status: {}. Valid values: {}", body.status, RECUSAL_STATUSES.join(", ")
        )));
    }

    let recusal = crate::repo::recusal_motion::update_ruling(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Recusal {} not found", recusal_id)))?;

    Ok(Json(RecusalMotionResponse::from(recusal)))
}
