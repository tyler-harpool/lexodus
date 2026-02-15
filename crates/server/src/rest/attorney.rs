use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, AttorneyResponse, CreateAttorneyRequest, UpdateAttorneyRequest,
    BulkUpdateStatusRequest, AttorneySearchParams, AttorneyListParams,
    PaginatedResponse, AttorneyStatus, normalize_pagination,
    BarAdmissionResponse, CreateBarAdmissionRequest, BAR_ADMISSION_STATUSES,
    FederalAdmissionResponse, CreateFederalAdmissionRequest, FEDERAL_ADMISSION_STATUSES,
    DisciplineRecordResponse, CreateDisciplineRecordRequest, DISCIPLINE_ACTION_TYPES,
    ProHacViceResponse, CreateProHacViceRequest, UpdatePhvStatusRequest, PRO_HAC_VICE_STATUSES,
    CjaAppointmentResponse, CreateCjaAppointmentRequest,
    EcfRegistrationResponse, UpsertEcfRegistrationRequest, ECF_REGISTRATION_STATUSES,
    AttorneyMetrics, AttorneyCaseLoad, GoodStandingResult, CanPracticeResult,
    WinRateResult, ConflictCheckRequest, ConflictCheckResult, AttorneyAddToCaseRequest,
    RepresentationResponse,
    PracticeAreaResponse, AddPracticeAreaRequest,
};
use crate::tenant::CourtId;

/// Simple email validation matching existing behavior.
fn is_valid_email(email: &str) -> bool {
    if let Some(at_pos) = email.find('@') {
        if at_pos > 0 && at_pos < email.len() - 1 {
            let domain = &email[at_pos + 1..];
            return domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.');
        }
    }
    false
}

/// POST /api/attorneys
#[utoipa::path(
    post,
    path = "/api/attorneys",
    request_body = CreateAttorneyRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Attorney created", body = AttorneyResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 409, description = "Duplicate bar number", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn create_attorney(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateAttorneyRequest>,
) -> Result<(StatusCode, Json<AttorneyResponse>), AppError> {
    if !is_valid_email(&body.email) {
        return Err(AppError::bad_request("Invalid email format"));
    }

    let attorney = crate::repo::attorney::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(AttorneyResponse::from(attorney))))
}

/// GET /api/attorneys/{id}
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorney found", body = AttorneyResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn get_attorney(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<AttorneyResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let attorney = crate::repo::attorney::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", id)))?;

    Ok(Json(AttorneyResponse::from(attorney)))
}

/// GET /api/attorneys/bar-number/{bar_number}
#[utoipa::path(
    get,
    path = "/api/attorneys/bar-number/{bar_number}",
    params(
        ("bar_number" = String, Path, description = "Attorney bar number"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorney found", body = AttorneyResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn get_attorney_by_bar_number(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(bar_number): Path<String>,
) -> Result<Json<AttorneyResponse>, AppError> {
    let attorney = crate::repo::attorney::find_by_bar_number(&pool, &court.0, &bar_number)
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "Attorney with bar number {} not found",
                bar_number
            ))
        })?;

    Ok(Json(AttorneyResponse::from(attorney)))
}

/// GET /api/attorneys
#[utoipa::path(
    get,
    path = "/api/attorneys",
    params(
        AttorneyListParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Paginated list", body = PaginatedResponse<AttorneyResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_attorneys(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<AttorneyListParams>,
) -> Result<Json<PaginatedResponse<AttorneyResponse>>, AppError> {
    let (page, limit) = normalize_pagination(params.page, params.limit);

    let (attorneys, total) = crate::repo::attorney::list(&pool, &court.0, page, limit).await?;

    let response_items: Vec<AttorneyResponse> =
        attorneys.into_iter().map(AttorneyResponse::from).collect();

    Ok(Json(PaginatedResponse::new(response_items, page, limit, total)))
}

/// GET /api/attorneys/search
#[utoipa::path(
    get,
    path = "/api/attorneys/search",
    params(
        AttorneySearchParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Search results", body = PaginatedResponse<AttorneyResponse>)
    ),
    tag = "attorneys"
)]
pub async fn search_attorneys(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<AttorneySearchParams>,
) -> Result<Json<PaginatedResponse<AttorneyResponse>>, AppError> {
    let (page, limit) = normalize_pagination(params.page, params.limit);
    let query = params.q.unwrap_or_default();

    let (attorneys, total) = if query.is_empty() {
        crate::repo::attorney::list(&pool, &court.0, page, limit).await?
    } else {
        crate::repo::attorney::search(&pool, &court.0, &query, page, limit).await?
    };

    let response_items: Vec<AttorneyResponse> =
        attorneys.into_iter().map(AttorneyResponse::from).collect();

    Ok(Json(PaginatedResponse::new(response_items, page, limit, total)))
}

/// PUT /api/attorneys/{id}
#[utoipa::path(
    put,
    path = "/api/attorneys/{id}",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = UpdateAttorneyRequest,
    responses(
        (status = 200, description = "Attorney updated", body = AttorneyResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn update_attorney(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateAttorneyRequest>,
) -> Result<Json<AttorneyResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    // Validate email if provided
    if let Some(ref email) = body.email {
        if !is_valid_email(email) {
            return Err(AppError::bad_request("Invalid email format"));
        }
    }

    // Validate status if provided
    if let Some(ref status) = body.status {
        if AttorneyStatus::from_str_opt(status).is_none() {
            return Err(AppError::bad_request(format!("Invalid status: {}", status)));
        }
    }

    let attorney = crate::repo::attorney::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", id)))?;

    Ok(Json(AttorneyResponse::from(attorney)))
}

/// DELETE /api/attorneys/{id}
#[utoipa::path(
    delete,
    path = "/api/attorneys/{id}",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Attorney deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn delete_attorney(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::attorney::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Attorney {} not found", id)))
    }
}

/// POST /api/attorneys/bulk/update-status
#[utoipa::path(
    post,
    path = "/api/attorneys/bulk/update-status",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = BulkUpdateStatusRequest,
    responses(
        (status = 204, description = "Bulk update complete"),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn bulk_update_status(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<BulkUpdateStatusRequest>,
) -> Result<StatusCode, AppError> {
    if AttorneyStatus::from_str_opt(&body.status).is_none() {
        return Err(AppError::bad_request(format!("Invalid status: {}", body.status)));
    }

    crate::repo::attorney::bulk_update_status(
        &pool,
        &court.0,
        &body.attorney_ids,
        &body.status,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

// ── PR 13-14: Attorney Sub-Resource Handlers ────────────────────────

// ── Filtering ───────────────────────────────────────────────────────

/// GET /api/attorneys/status/{status}
#[utoipa::path(
    get,
    path = "/api/attorneys/status/{status}",
    params(
        ("status" = String, Path, description = "Attorney status filter"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorneys by status", body = Vec<AttorneyResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_attorneys_by_status(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(status): Path<String>,
) -> Result<Json<Vec<AttorneyResponse>>, AppError> {
    if AttorneyStatus::from_str_opt(&status).is_none() {
        return Err(AppError::bad_request(format!("Invalid status: {}", status)));
    }

    let (attorneys, _total) = crate::repo::attorney::list(&pool, &court.0, 1, 1000).await?;
    let filtered: Vec<AttorneyResponse> = attorneys
        .into_iter()
        .filter(|a| a.status == status)
        .map(AttorneyResponse::from)
        .collect();

    Ok(Json(filtered))
}

/// GET /api/attorneys/firm/{firm_name}
#[utoipa::path(
    get,
    path = "/api/attorneys/firm/{firm_name}",
    params(
        ("firm_name" = String, Path, description = "Firm name filter"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorneys by firm", body = Vec<AttorneyResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_attorneys_by_firm(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(firm_name): Path<String>,
) -> Result<Json<Vec<AttorneyResponse>>, AppError> {
    let (attorneys, _total) = crate::repo::attorney::search(&pool, &court.0, &firm_name, 1, 1000).await?;
    let filtered: Vec<AttorneyResponse> = attorneys
        .into_iter()
        .filter(|a| a.firm_name.as_deref() == Some(&firm_name))
        .map(AttorneyResponse::from)
        .collect();

    Ok(Json(filtered))
}

// ── Bar Admissions ──────────────────────────────────────────────────

/// POST /api/attorneys/{id}/bar-admissions
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/bar-admissions",
    request_body = CreateBarAdmissionRequest,
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Bar admission created", body = BarAdmissionResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn add_bar_admission(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<CreateBarAdmissionRequest>,
) -> Result<(StatusCode, Json<BarAdmissionResponse>), AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref status) = body.status {
        if !BAR_ADMISSION_STATUSES.contains(&status.as_str()) {
            return Err(AppError::bad_request(format!(
                "Invalid bar admission status: {}. Valid: {:?}",
                status, BAR_ADMISSION_STATUSES
            )));
        }
    }

    let admission = crate::repo::bar_admission::create(&pool, &court.0, attorney_id, body).await?;
    Ok((StatusCode::CREATED, Json(BarAdmissionResponse::from(admission))))
}

/// DELETE /api/attorneys/{id}/bar-admissions/{state}
#[utoipa::path(
    delete,
    path = "/api/attorneys/{id}/bar-admissions/{state}",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("state" = String, Path, description = "Bar admission state"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Bar admission removed"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn remove_bar_admission(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((id, state)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::bar_admission::delete_by_state(&pool, &court.0, attorney_id, &state).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "Bar admission in state {} not found for attorney {}",
            state, id
        )))
    }
}

/// GET /api/attorneys/bar-state/{state}
#[utoipa::path(
    get,
    path = "/api/attorneys/bar-state/{state}",
    params(
        ("state" = String, Path, description = "US state abbreviation"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorneys admitted in state", body = Vec<BarAdmissionResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_attorneys_by_bar_state(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(state): Path<String>,
) -> Result<Json<Vec<BarAdmissionResponse>>, AppError> {
    let admissions = crate::repo::bar_admission::list_by_state(&pool, &court.0, &state).await?;
    let responses: Vec<BarAdmissionResponse> = admissions
        .into_iter()
        .map(BarAdmissionResponse::from)
        .collect();

    Ok(Json(responses))
}

// ── Federal Admissions ──────────────────────────────────────────────

/// POST /api/attorneys/{id}/federal-admissions
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/federal-admissions",
    request_body = CreateFederalAdmissionRequest,
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Federal admission created", body = FederalAdmissionResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn add_federal_admission(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<CreateFederalAdmissionRequest>,
) -> Result<(StatusCode, Json<FederalAdmissionResponse>), AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref status) = body.status {
        if !FEDERAL_ADMISSION_STATUSES.contains(&status.as_str()) {
            return Err(AppError::bad_request(format!(
                "Invalid federal admission status: {}. Valid: {:?}",
                status, FEDERAL_ADMISSION_STATUSES
            )));
        }
    }

    let admission = crate::repo::federal_admission::create(&pool, &court.0, attorney_id, body).await?;
    Ok((StatusCode::CREATED, Json(FederalAdmissionResponse::from(admission))))
}

/// DELETE /api/attorneys/{id}/federal-admissions/{court}
#[utoipa::path(
    delete,
    path = "/api/attorneys/{id}/federal-admissions/{court}",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("court" = String, Path, description = "Federal court name"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Federal admission removed"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn remove_federal_admission(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((id, fed_court)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::federal_admission::delete_by_court_name(
        &pool, &court.0, attorney_id, &fed_court,
    )
    .await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "Federal admission to {} not found for attorney {}",
            fed_court, id
        )))
    }
}

/// GET /api/attorneys/federal-court/{court}
#[utoipa::path(
    get,
    path = "/api/attorneys/federal-court/{court}",
    params(
        ("court" = String, Path, description = "Federal court name"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorneys admitted to federal court", body = Vec<FederalAdmissionResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_attorneys_by_federal_court(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(fed_court): Path<String>,
) -> Result<Json<Vec<FederalAdmissionResponse>>, AppError> {
    let admissions = crate::repo::federal_admission::list_by_court_name(&pool, &court.0, &fed_court).await?;
    let responses: Vec<FederalAdmissionResponse> = admissions
        .into_iter()
        .map(FederalAdmissionResponse::from)
        .collect();

    Ok(Json(responses))
}

// ── Discipline Records ──────────────────────────────────────────────

/// GET /api/attorneys/{id}/disciplinary-actions
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/disciplinary-actions",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Discipline records", body = Vec<DisciplineRecordResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_discipline(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<Vec<DisciplineRecordResponse>>, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let records = crate::repo::discipline::list_by_attorney(&pool, &court.0, attorney_id).await?;
    let responses: Vec<DisciplineRecordResponse> = records
        .into_iter()
        .map(DisciplineRecordResponse::from)
        .collect();

    Ok(Json(responses))
}

/// POST /api/attorneys/{id}/disciplinary-actions
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/disciplinary-actions",
    request_body = CreateDisciplineRecordRequest,
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Discipline record created", body = DisciplineRecordResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn add_discipline(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<CreateDisciplineRecordRequest>,
) -> Result<(StatusCode, Json<DisciplineRecordResponse>), AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !DISCIPLINE_ACTION_TYPES.contains(&body.action_type.as_str()) {
        return Err(AppError::bad_request(format!(
            "Invalid action_type: {}. Valid: {:?}",
            body.action_type, DISCIPLINE_ACTION_TYPES
        )));
    }

    let record = crate::repo::discipline::create(&pool, &court.0, attorney_id, body).await?;
    Ok((StatusCode::CREATED, Json(DisciplineRecordResponse::from(record))))
}

/// GET /api/attorneys/with-discipline
#[utoipa::path(
    get,
    path = "/api/attorneys/with-discipline",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorneys with discipline records", body = Vec<DisciplineRecordResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_attorneys_with_discipline(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<DisciplineRecordResponse>>, AppError> {
    let records = crate::repo::discipline::list_with_discipline(&pool, &court.0).await?;
    let responses: Vec<DisciplineRecordResponse> = records
        .into_iter()
        .map(DisciplineRecordResponse::from)
        .collect();

    Ok(Json(responses))
}

// ── Pro Hac Vice ────────────────────────────────────────────────────

/// POST /api/attorneys/{id}/pro-hac-vice
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/pro-hac-vice",
    request_body = CreateProHacViceRequest,
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Pro hac vice admission created", body = ProHacViceResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn add_pro_hac_vice(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<CreateProHacViceRequest>,
) -> Result<(StatusCode, Json<ProHacViceResponse>), AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let phv = crate::repo::pro_hac_vice::create(&pool, &court.0, attorney_id, body).await?;
    Ok((StatusCode::CREATED, Json(ProHacViceResponse::from(phv))))
}

/// PATCH /api/attorneys/{id}/pro-hac-vice/{case_id}/status
#[utoipa::path(
    patch,
    path = "/api/attorneys/{id}/pro-hac-vice/{case_id}/status",
    request_body = UpdatePhvStatusRequest,
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "PHV status updated", body = ProHacViceResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn update_phv_status(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((id, case_id_str)): Path<(String, String)>,
    Json(body): Json<UpdatePhvStatusRequest>,
) -> Result<Json<ProHacViceResponse>, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid attorney UUID format"))?;
    let case_id = Uuid::parse_str(&case_id_str)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    if !PRO_HAC_VICE_STATUSES.contains(&body.status.as_str()) {
        return Err(AppError::bad_request(format!(
            "Invalid PHV status: {}. Valid: {:?}",
            body.status, PRO_HAC_VICE_STATUSES
        )));
    }

    let phv = crate::repo::pro_hac_vice::update_status(
        &pool, &court.0, attorney_id, case_id, &body.status,
    )
    .await?
    .ok_or_else(|| AppError::not_found(format!(
        "Pro hac vice admission not found for attorney {} on case {}",
        id, case_id_str
    )))?;

    Ok(Json(ProHacViceResponse::from(phv)))
}

/// GET /api/attorneys/pro-hac-vice/active
#[utoipa::path(
    get,
    path = "/api/attorneys/pro-hac-vice/active",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Active PHV admissions", body = Vec<ProHacViceResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_active_phv(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<ProHacViceResponse>>, AppError> {
    let phvs = crate::repo::pro_hac_vice::list_active(&pool, &court.0).await?;
    let responses: Vec<ProHacViceResponse> = phvs
        .into_iter()
        .map(ProHacViceResponse::from)
        .collect();

    Ok(Json(responses))
}

/// GET /api/attorneys/pro-hac-vice/case/{case_id}
#[utoipa::path(
    get,
    path = "/api/attorneys/pro-hac-vice/case/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "PHV admissions for case", body = Vec<ProHacViceResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_phv_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<ProHacViceResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let phvs = crate::repo::pro_hac_vice::list_by_case(&pool, &court.0, uuid).await?;
    let responses: Vec<ProHacViceResponse> = phvs
        .into_iter()
        .map(ProHacViceResponse::from)
        .collect();

    Ok(Json(responses))
}

// ── CJA Panel ───────────────────────────────────────────────────────

/// POST /api/attorneys/{id}/cja-panel/{cja_district}
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/cja-panel/{cja_district}",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("cja_district" = String, Path, description = "CJA district code"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorney added to CJA panel", body = AttorneyResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn add_cja_panel(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((id, cja_district)): Path<(String, String)>,
) -> Result<Json<AttorneyResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let attorney = crate::repo::attorney::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", id)))?;

    let mut districts = attorney.cja_panel_districts.clone();
    if !districts.contains(&cja_district) {
        districts.push(cja_district);
    }

    let update = shared_types::UpdateAttorneyRequest {
        cja_panel_member: Some(true),
        cja_panel_districts: Some(districts),
        ..Default::default()
    };

    let updated = crate::repo::attorney::update(&pool, &court.0, uuid, update)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", id)))?;

    Ok(Json(AttorneyResponse::from(updated)))
}

/// DELETE /api/attorneys/{id}/cja-panel/{cja_district}
#[utoipa::path(
    delete,
    path = "/api/attorneys/{id}/cja-panel/{cja_district}",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("cja_district" = String, Path, description = "CJA district code"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorney removed from CJA panel", body = AttorneyResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn remove_cja_panel(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((id, cja_district)): Path<(String, String)>,
) -> Result<Json<AttorneyResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let attorney = crate::repo::attorney::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", id)))?;

    let districts: Vec<String> = attorney
        .cja_panel_districts
        .into_iter()
        .filter(|d| d != &cja_district)
        .collect();

    let cja_panel_member = !districts.is_empty();

    let update = shared_types::UpdateAttorneyRequest {
        cja_panel_member: Some(cja_panel_member),
        cja_panel_districts: Some(districts),
        ..Default::default()
    };

    let updated = crate::repo::attorney::update(&pool, &court.0, uuid, update)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", id)))?;

    Ok(Json(AttorneyResponse::from(updated)))
}

/// GET /api/attorneys/cja-panel/{cja_district}
#[utoipa::path(
    get,
    path = "/api/attorneys/cja-panel/{cja_district}",
    params(
        ("cja_district" = String, Path, description = "CJA district code"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "CJA panel members for district", body = Vec<AttorneyResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_cja_panel(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(cja_district): Path<String>,
) -> Result<Json<Vec<AttorneyResponse>>, AppError> {
    let (attorneys, _total) = crate::repo::attorney::list(&pool, &court.0, 1, 1000).await?;
    let filtered: Vec<AttorneyResponse> = attorneys
        .into_iter()
        .filter(|a| a.cja_panel_member && a.cja_panel_districts.contains(&cja_district))
        .map(AttorneyResponse::from)
        .collect();

    Ok(Json(filtered))
}

// ── CJA Appointments ────────────────────────────────────────────────

/// GET /api/attorneys/{id}/cja-appointments
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/cja-appointments",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "CJA appointments", body = Vec<CjaAppointmentResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_cja_appts(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<Vec<CjaAppointmentResponse>>, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let appts = crate::repo::cja_appointment::list_by_attorney(&pool, &court.0, attorney_id).await?;
    let responses: Vec<CjaAppointmentResponse> = appts
        .into_iter()
        .map(CjaAppointmentResponse::from)
        .collect();

    Ok(Json(responses))
}

/// POST /api/attorneys/{id}/cja-appointments
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/cja-appointments",
    request_body = CreateCjaAppointmentRequest,
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "CJA appointment created", body = CjaAppointmentResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn add_cja_appt(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<CreateCjaAppointmentRequest>,
) -> Result<(StatusCode, Json<CjaAppointmentResponse>), AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let appt = crate::repo::cja_appointment::create(&pool, &court.0, attorney_id, body).await?;
    Ok((StatusCode::CREATED, Json(CjaAppointmentResponse::from(appt))))
}

/// GET /api/attorneys/cja/pending-vouchers
#[utoipa::path(
    get,
    path = "/api/attorneys/cja/pending-vouchers",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Pending CJA vouchers", body = Vec<CjaAppointmentResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_pending_vouchers(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<CjaAppointmentResponse>>, AppError> {
    let appts = crate::repo::cja_appointment::list_pending_vouchers(&pool, &court.0).await?;
    let responses: Vec<CjaAppointmentResponse> = appts
        .into_iter()
        .map(CjaAppointmentResponse::from)
        .collect();

    Ok(Json(responses))
}

// ── ECF Registration ────────────────────────────────────────────────

/// PUT /api/attorneys/{id}/ecf-registration
#[utoipa::path(
    put,
    path = "/api/attorneys/{id}/ecf-registration",
    request_body = UpsertEcfRegistrationRequest,
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "ECF registration upserted", body = EcfRegistrationResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn upsert_ecf_registration(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpsertEcfRegistrationRequest>,
) -> Result<Json<EcfRegistrationResponse>, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let status = body.status.unwrap_or_else(|| "Active".to_string());
    if !ECF_REGISTRATION_STATUSES.contains(&status.as_str()) {
        return Err(AppError::bad_request(format!(
            "Invalid ECF status: {}. Valid: {:?}",
            status, ECF_REGISTRATION_STATUSES
        )));
    }

    let reg = crate::repo::ecf_registration::upsert(&pool, &court.0, attorney_id, &status).await?;
    Ok(Json(EcfRegistrationResponse::from(reg)))
}

// ── Standing & Practice Checks ──────────────────────────────────────

/// GET /api/attorneys/{id}/good-standing
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/good-standing",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Good standing result", body = GoodStandingResult),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn check_good_standing(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<GoodStandingResult>, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let attorney = crate::repo::attorney::find_by_id(&pool, &court.0, attorney_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", id)))?;

    let mut reasons = Vec::new();
    let mut in_good_standing = true;

    // Check attorney status
    if attorney.status != "Active" {
        in_good_standing = false;
        reasons.push(format!("Attorney status is '{}'", attorney.status));
    }

    // Check for active discipline records
    let discipline = crate::repo::discipline::list_by_attorney(&pool, &court.0, attorney_id).await?;
    let active_discipline: Vec<_> = discipline
        .iter()
        .filter(|d| d.end_date.is_none())
        .collect();

    if !active_discipline.is_empty() {
        in_good_standing = false;
        reasons.push(format!(
            "{} active disciplinary action(s) on record",
            active_discipline.len()
        ));
    }

    // Check bar admissions
    let bar_admissions = crate::repo::bar_admission::list_by_attorney(&pool, &court.0, attorney_id).await?;
    let has_active_bar = bar_admissions.iter().any(|b| b.status == "Active");
    if !has_active_bar && !bar_admissions.is_empty() {
        in_good_standing = false;
        reasons.push("No active bar admissions".to_string());
    }

    if in_good_standing {
        reasons.push("All checks passed".to_string());
    }

    Ok(Json(GoodStandingResult {
        attorney_id: id,
        in_good_standing,
        reasons,
    }))
}

/// GET /api/attorneys/{id}/can-practice/{court}
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/can-practice/{court}",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("court" = String, Path, description = "Court name to check"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Can practice result", body = CanPracticeResult),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn check_can_practice(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((id, target_court)): Path<(String, String)>,
) -> Result<Json<CanPracticeResult>, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let attorney = crate::repo::attorney::find_by_id(&pool, &court.0, attorney_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", id)))?;

    let mut reasons = Vec::new();
    let mut can_practice = true;

    // Check attorney status
    if attorney.status != "Active" {
        can_practice = false;
        reasons.push(format!("Attorney status is '{}'", attorney.status));
    }

    // Check federal admissions for the target court
    let federal_admissions = crate::repo::federal_admission::list_by_attorney(
        &pool, &court.0, attorney_id,
    )
    .await?;
    let has_federal = federal_admissions
        .iter()
        .any(|f| f.court_name == target_court && f.status == "Active");

    // Check pro hac vice for the target court
    let phvs = crate::repo::pro_hac_vice::list_by_attorney(&pool, &court.0, attorney_id).await?;
    let has_phv = phvs.iter().any(|p| p.status == "Active");

    if !has_federal && !has_phv {
        can_practice = false;
        reasons.push(format!(
            "No active federal admission or pro hac vice for court '{}'",
            target_court
        ));
    }

    if can_practice {
        reasons.push("Attorney is authorized to practice".to_string());
    }

    Ok(Json(CanPracticeResult {
        attorney_id: id,
        court: target_court,
        can_practice,
        reasons,
    }))
}

/// GET /api/attorneys/{id}/has-ecf-privileges
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/has-ecf-privileges",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "ECF privilege check result", body = GoodStandingResult)
    ),
    tag = "attorneys"
)]
pub async fn check_ecf_privileges(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<GoodStandingResult>, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let ecf = crate::repo::ecf_registration::find_by_attorney(&pool, &court.0, attorney_id).await?;

    let (has_privileges, reasons) = match ecf {
        Some(reg) if reg.status == "Active" => {
            (true, vec!["ECF registration is active".to_string()])
        }
        Some(reg) => {
            (false, vec![format!("ECF registration status is '{}'", reg.status)])
        }
        None => {
            (false, vec!["No ECF registration found".to_string()])
        }
    };

    Ok(Json(GoodStandingResult {
        attorney_id: id,
        in_good_standing: has_privileges,
        reasons,
    }))
}

// ── Win Rate ────────────────────────────────────────────────────────

/// POST /api/attorneys/{id}/calculate-win-rate
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/calculate-win-rate",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Win rate calculated", body = WinRateResult),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn calculate_win_rate(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<WinRateResult>, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let attorney = crate::repo::attorney::find_by_id(&pool, &court.0, attorney_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", id)))?;

    // Use stored win rate from the attorney record
    let total_cases = attorney.cases_handled as i64;
    let win_rate_pct = attorney.win_rate_percentage.unwrap_or(0.0);
    let wins = ((total_cases as f64) * (win_rate_pct / 100.0)).round() as i64;
    let losses = total_cases - wins;
    let win_rate = if total_cases > 0 {
        win_rate_pct
    } else {
        0.0
    };

    Ok(Json(WinRateResult {
        attorney_id: id,
        total_cases,
        wins,
        losses,
        win_rate,
    }))
}

// ── ECF Access ──────────────────────────────────────────────────────

/// GET /api/attorneys/ecf-access
#[utoipa::path(
    get,
    path = "/api/attorneys/ecf-access",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorneys with ECF access", body = Vec<EcfRegistrationResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_ecf_access(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<EcfRegistrationResponse>>, AppError> {
    let regs = crate::repo::ecf_registration::list_active(&pool, &court.0).await?;
    let responses: Vec<EcfRegistrationResponse> = regs
        .into_iter()
        .map(EcfRegistrationResponse::from)
        .collect();

    Ok(Json(responses))
}

/// DELETE /api/attorneys/{id}/ecf-access
#[utoipa::path(
    delete,
    path = "/api/attorneys/{id}/ecf-access",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "ECF access revoked"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn revoke_ecf_access(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let revoked = crate::repo::ecf_registration::revoke(&pool, &court.0, attorney_id).await?;
    if revoked {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "ECF registration not found for attorney {}",
            id
        )))
    }
}

// ── Attorney Cases ──────────────────────────────────────────────────

/// GET /api/attorneys/{attorney_id}/cases
#[utoipa::path(
    get,
    path = "/api/attorneys/{attorney_id}/cases",
    params(
        ("attorney_id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorney case list", body = Vec<RepresentationResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_attorney_cases(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(attorney_id): Path<String>,
) -> Result<Json<Vec<RepresentationResponse>>, AppError> {
    let uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let reps = crate::repo::representation::list_active_by_attorney(&pool, &court.0, uuid).await?;
    let responses: Vec<RepresentationResponse> = reps
        .into_iter()
        .map(RepresentationResponse::from)
        .collect();

    Ok(Json(responses))
}

/// POST /api/attorneys/{attorney_id}/cases
#[utoipa::path(
    post,
    path = "/api/attorneys/{attorney_id}/cases",
    request_body = AttorneyAddToCaseRequest,
    params(
        ("attorney_id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Attorney added to case", body = RepresentationResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn add_to_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(attorney_id): Path<String>,
    Json(body): Json<AttorneyAddToCaseRequest>,
) -> Result<(StatusCode, Json<RepresentationResponse>), AppError> {
    let atty_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| AppError::bad_request("Invalid attorney UUID format"))?;

    // Verify attorney exists
    crate::repo::attorney::find_by_id(&pool, &court.0, atty_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", attorney_id)))?;

    // Find the first party on the case to create the representation link
    let parties = crate::repo::party::list_by_case(&pool, &court.0, body.case_id).await?;
    let party = parties
        .first()
        .ok_or_else(|| AppError::bad_request("No parties found on case; add a party first"))?;

    let rep_type = body.role.unwrap_or_else(|| "Private".to_string());

    let create_req = shared_types::CreateRepresentationRequest {
        attorney_id: attorney_id.clone(),
        party_id: party.id.to_string(),
        case_id: body.case_id.to_string(),
        representation_type: Some(rep_type),
        lead_counsel: None,
        local_counsel: None,
        limited_appearance: None,
        court_appointed: None,
        cja_appointment_id: None,
        scope_of_representation: None,
        notes: None,
    };

    let rep = crate::repo::representation::create(&pool, &court.0, &create_req).await?;
    Ok((StatusCode::CREATED, Json(RepresentationResponse::from(rep))))
}

/// DELETE /api/attorneys/{attorney_id}/cases/{case_id}
#[utoipa::path(
    delete,
    path = "/api/attorneys/{attorney_id}/cases/{case_id}",
    params(
        ("attorney_id" = String, Path, description = "Attorney UUID"),
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Attorney removed from case"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn remove_from_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((attorney_id, case_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let atty_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| AppError::bad_request("Invalid attorney UUID format"))?;
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    // Find active representations for this attorney on this case
    let reps = crate::repo::representation::list_active_by_attorney(&pool, &court.0, atty_uuid).await?;
    let case_reps: Vec<_> = reps.into_iter().filter(|r| r.case_id == case_uuid).collect();

    if case_reps.is_empty() {
        return Err(AppError::not_found(format!(
            "No active representation found for attorney {} on case {}",
            attorney_id, case_id
        )));
    }

    // End all representations for this attorney on this case
    for rep in case_reps {
        crate::repo::representation::end_representation(
            &pool,
            &court.0,
            rep.id,
            Some("Client Request"),
        )
        .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

// ── Analytics ───────────────────────────────────────────────────────

/// GET /api/attorneys/{attorney_id}/case-load
#[utoipa::path(
    get,
    path = "/api/attorneys/{attorney_id}/case-load",
    params(
        ("attorney_id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorney case load", body = AttorneyCaseLoad),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn get_case_load(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(attorney_id): Path<String>,
) -> Result<Json<AttorneyCaseLoad>, AppError> {
    let uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    // Verify attorney exists
    crate::repo::attorney::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", attorney_id)))?;

    let reps = crate::repo::representation::list_active_by_attorney(&pool, &court.0, uuid).await?;
    let total_active = reps.len() as i64;

    // Build a status breakdown from representation statuses
    let mut status_map = serde_json::Map::new();
    for rep in &reps {
        let count = status_map
            .entry(&rep.representation_type)
            .or_insert_with(|| serde_json::Value::Number(serde_json::Number::from(0)));
        if let Some(n) = count.as_i64() {
            *count = serde_json::Value::Number(serde_json::Number::from(n + 1));
        }
    }

    Ok(Json(AttorneyCaseLoad {
        attorney_id,
        total_active,
        by_status: serde_json::Value::Object(status_map),
    }))
}

/// GET /api/attorneys/{attorney_id}/representation-history
#[utoipa::path(
    get,
    path = "/api/attorneys/{attorney_id}/representation-history",
    params(
        ("attorney_id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Representation history", body = Vec<RepresentationResponse>)
    ),
    tag = "attorneys"
)]
pub async fn get_rep_history(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(attorney_id): Path<String>,
) -> Result<Json<Vec<RepresentationResponse>>, AppError> {
    let uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    // list_active_by_attorney returns only active; for full history we use the same
    // but ideally we'd have a list_all_by_attorney. For now, return active ones.
    let reps = crate::repo::representation::list_active_by_attorney(&pool, &court.0, uuid).await?;
    let responses: Vec<RepresentationResponse> = reps
        .into_iter()
        .map(RepresentationResponse::from)
        .collect();

    Ok(Json(responses))
}

/// POST /api/attorneys/{attorney_id}/conflict-check
#[utoipa::path(
    post,
    path = "/api/attorneys/{attorney_id}/conflict-check",
    request_body = ConflictCheckRequest,
    params(
        ("attorney_id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Conflict check result", body = ConflictCheckResult)
    ),
    tag = "attorneys"
)]
pub async fn run_conflict_check(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(attorney_id): Path<String>,
    Json(body): Json<ConflictCheckRequest>,
) -> Result<Json<ConflictCheckResult>, AppError> {
    let uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    // Get all active cases for this attorney
    let reps = crate::repo::representation::list_active_by_attorney(&pool, &court.0, uuid).await?;

    let mut conflicts = Vec::new();

    // Check if any provided party names appear in the attorney's existing cases
    for rep in &reps {
        let parties = crate::repo::party::list_by_case(&pool, &court.0, rep.case_id).await?;
        for party in &parties {
            for name in &body.party_names {
                if party.name.to_lowercase().contains(&name.to_lowercase()) {
                    conflicts.push(format!(
                        "Party '{}' matches existing case party '{}' (case {})",
                        name, party.name, rep.case_id
                    ));
                }
            }
        }
    }

    Ok(Json(ConflictCheckResult {
        has_conflict: !conflicts.is_empty(),
        conflicts,
    }))
}

/// GET /api/attorneys/{id}/metrics
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/metrics",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorney metrics", body = AttorneyMetrics),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn get_metrics(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<AttorneyMetrics>, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let attorney = crate::repo::attorney::find_by_id(&pool, &court.0, attorney_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", id)))?;

    let reps = crate::repo::representation::list_active_by_attorney(&pool, &court.0, attorney_id).await?;
    let active_cases = reps.len() as i64;

    Ok(Json(AttorneyMetrics {
        attorney_id: id,
        total_cases: attorney.cases_handled as i64,
        active_cases,
        win_rate: attorney.win_rate_percentage,
        avg_case_duration_days: attorney.avg_case_duration_days.map(|d| d as f64),
    }))
}

/// GET /api/attorneys/{id}/win-rate
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/win-rate",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attorney win rate", body = WinRateResult),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn get_win_rate(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<WinRateResult>, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let attorney = crate::repo::attorney::find_by_id(&pool, &court.0, attorney_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", id)))?;

    let total_cases = attorney.cases_handled as i64;
    let win_rate_pct = attorney.win_rate_percentage.unwrap_or(0.0);
    let wins = ((total_cases as f64) * (win_rate_pct / 100.0)).round() as i64;
    let losses = total_cases - wins;

    Ok(Json(WinRateResult {
        attorney_id: id,
        total_cases,
        wins,
        losses,
        win_rate: win_rate_pct,
    }))
}

/// GET /api/attorneys/{id}/case-count
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/case-count",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Case count", body = serde_json::Value),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn get_case_count(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let attorney = crate::repo::attorney::find_by_id(&pool, &court.0, attorney_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attorney {} not found", id)))?;

    let reps = crate::repo::representation::list_active_by_attorney(&pool, &court.0, attorney_id).await?;

    Ok(Json(serde_json::json!({
        "attorney_id": id,
        "total_cases": attorney.cases_handled,
        "active_cases": reps.len()
    })))
}

/// GET /api/attorneys/top-performers
#[utoipa::path(
    get,
    path = "/api/attorneys/top-performers",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Top performing attorneys", body = Vec<AttorneyResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_top_performers(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<AttorneyResponse>>, AppError> {
    let (mut attorneys, _total) = crate::repo::attorney::list(&pool, &court.0, 1, 1000).await?;

    // Sort by win rate descending, then by cases handled descending
    attorneys.sort_by(|a, b| {
        let wr_a = a.win_rate_percentage.unwrap_or(0.0);
        let wr_b = b.win_rate_percentage.unwrap_or(0.0);
        wr_b.partial_cmp(&wr_a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.cases_handled.cmp(&a.cases_handled))
    });

    // Return top 10
    let top: Vec<AttorneyResponse> = attorneys
        .into_iter()
        .filter(|a| a.status == "Active")
        .take(10)
        .map(AttorneyResponse::from)
        .collect();

    Ok(Json(top))
}

// ── Practice Areas ──────────────────────────────────────────────────

/// POST /api/attorneys/{id}/practice-areas
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/practice-areas",
    request_body = AddPracticeAreaRequest,
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Practice area added", body = PracticeAreaResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn add_practice_area(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<AddPracticeAreaRequest>,
) -> Result<(StatusCode, Json<PracticeAreaResponse>), AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let area = body.area.trim().to_string();
    if area.is_empty() {
        return Err(AppError::bad_request("Practice area cannot be empty"));
    }

    let practice_area =
        crate::repo::practice_area::add(&pool, &court.0, attorney_id, &area).await?;
    Ok((
        StatusCode::CREATED,
        Json(PracticeAreaResponse::from(practice_area)),
    ))
}

/// GET /api/attorneys/{id}/practice-areas
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/practice-areas",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Practice areas for attorney", body = Vec<PracticeAreaResponse>)
    ),
    tag = "attorneys"
)]
pub async fn list_practice_areas(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<Vec<PracticeAreaResponse>>, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let areas =
        crate::repo::practice_area::list_by_attorney(&pool, &court.0, attorney_id).await?;
    let responses: Vec<PracticeAreaResponse> = areas
        .into_iter()
        .map(PracticeAreaResponse::from)
        .collect();

    Ok(Json(responses))
}

/// DELETE /api/attorneys/{id}/practice-areas/{area}
#[utoipa::path(
    delete,
    path = "/api/attorneys/{id}/practice-areas/{area}",
    params(
        ("id" = String, Path, description = "Attorney UUID"),
        ("area" = String, Path, description = "Practice area name"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Practice area removed"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "attorneys"
)]
pub async fn remove_practice_area(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((id, area)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let attorney_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted =
        crate::repo::practice_area::remove(&pool, &court.0, attorney_id, &area).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "Practice area '{}' not found for attorney {}",
            area, id
        )))
    }
}
