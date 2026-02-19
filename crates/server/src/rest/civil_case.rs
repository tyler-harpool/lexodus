use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CivilCase, CivilCaseResponse, CivilCaseSearchParams, CivilCaseSearchResponse,
    CreateCivilCaseRequest, UpdateCivilCaseStatusRequest,
    is_valid_civil_status, is_valid_jurisdiction_basis, is_valid_jury_demand,
    CIVIL_CASE_STATUSES, CIVIL_CASE_PRIORITIES, CIVIL_JURISDICTION_BASES, CIVIL_JURY_DEMANDS,
};
use crate::error_convert::SqlxErrorExt;
use crate::tenant::CourtId;

/// POST /api/civil-cases
#[utoipa::path(
    post,
    path = "/api/civil-cases",
    request_body = CreateCivilCaseRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Civil case created", body = CivilCaseResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "civil-cases"
)]
pub async fn create_civil_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateCivilCaseRequest>,
) -> Result<(StatusCode, Json<CivilCaseResponse>), AppError> {
    if body.title.trim().is_empty() {
        return Err(AppError::bad_request("title must not be empty"));
    }

    if body.nature_of_suit.trim().is_empty() {
        return Err(AppError::bad_request("nature_of_suit must not be empty"));
    }

    if !is_valid_jurisdiction_basis(&body.jurisdiction_basis) {
        return Err(AppError::bad_request(format!(
            "Invalid jurisdiction_basis: {}. Valid values: {}",
            body.jurisdiction_basis,
            CIVIL_JURISDICTION_BASES.join(", ")
        )));
    }

    if let Some(ref jd) = body.jury_demand {
        if !is_valid_jury_demand(jd) {
            return Err(AppError::bad_request(format!(
                "Invalid jury_demand: {}. Valid values: {}",
                jd,
                CIVIL_JURY_DEMANDS.join(", ")
            )));
        }
    }

    if let Some(ref p) = body.priority {
        if !CIVIL_CASE_PRIORITIES.contains(&p.as_str()) {
            return Err(AppError::bad_request(format!(
                "Invalid priority: {}. Valid values: {}",
                p,
                CIVIL_CASE_PRIORITIES.join(", ")
            )));
        }
    }

    let case = crate::repo::civil_case::create(&pool, &court.0, body).await?;

    // Auto-create a clerk queue item for the new civil filing
    sqlx::query!(
        r#"
        INSERT INTO clerk_queue
            (court_id, queue_type, priority, title, description,
             source_type, source_id, case_id, case_type, case_number, current_step)
        VALUES ($1, 'filing', 3, $2, $3, 'filing', $4, $4, 'civil', $5, 'review')
        ON CONFLICT DO NOTHING
        "#,
        case.court_id,
        format!("New Civil Complaint — {}", case.title),
        format!("Process initial civil filing. NOS {}. Verify filing fee, assign case number, and issue summons.", case.nature_of_suit),
        case.id,
        case.case_number,
    )
    .execute(&pool)
    .await
    .ok(); // Best-effort: don't fail the case creation if queue insert fails

    Ok((StatusCode::CREATED, Json(CivilCaseResponse::from(case))))
}

/// GET /api/civil-cases/{id}
#[utoipa::path(
    get,
    path = "/api/civil-cases/{id}",
    params(
        ("id" = String, Path, description = "Civil case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Civil case found", body = CivilCaseResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "civil-cases"
)]
pub async fn get_civil_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<CivilCaseResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let case = crate::repo::civil_case::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Civil case {} not found", id)))?;

    Ok(Json(CivilCaseResponse::from(case)))
}

/// DELETE /api/civil-cases/{id}
#[utoipa::path(
    delete,
    path = "/api/civil-cases/{id}",
    params(
        ("id" = String, Path, description = "Civil case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Civil case deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "civil-cases"
)]
pub async fn delete_civil_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::civil_case::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Civil case {} not found", id)))
    }
}

/// PATCH /api/civil-cases/{id}/status
#[utoipa::path(
    patch,
    path = "/api/civil-cases/{id}/status",
    request_body = UpdateCivilCaseStatusRequest,
    params(
        ("id" = String, Path, description = "Civil case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Status updated", body = CivilCaseResponse),
        (status = 400, description = "Invalid status", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "civil-cases"
)]
pub async fn update_civil_case_status(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateCivilCaseStatusRequest>,
) -> Result<Json<CivilCaseResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !is_valid_civil_status(&body.status) {
        return Err(AppError::bad_request(format!(
            "Invalid status: {}. Valid values: {}",
            body.status,
            CIVIL_CASE_STATUSES.join(", ")
        )));
    }

    let case = crate::repo::civil_case::update_status(&pool, &court.0, uuid, &body.status)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Civil case {} not found", id)))?;

    Ok(Json(CivilCaseResponse::from(case)))
}

/// GET /api/civil-cases
#[utoipa::path(
    get,
    path = "/api/civil-cases",
    params(
        CivilCaseSearchParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Search results", body = CivilCaseSearchResponse)
    ),
    tag = "civil-cases"
)]
pub async fn search_civil_cases(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<CivilCaseSearchParams>,
) -> Result<Json<CivilCaseSearchResponse>, AppError> {
    let offset = params.offset.unwrap_or(0).max(0);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);

    if let Some(ref s) = params.status {
        if !is_valid_civil_status(s) {
            return Err(AppError::bad_request(format!("Invalid status: {}", s)));
        }
    }

    if let Some(ref jb) = params.jurisdiction_basis {
        if !is_valid_jurisdiction_basis(jb) {
            return Err(AppError::bad_request(format!("Invalid jurisdiction_basis: {}", jb)));
        }
    }

    let (cases, total) = crate::repo::civil_case::search(
        &pool,
        &court.0,
        params.status.as_deref(),
        params.nature_of_suit.as_deref(),
        params.jurisdiction_basis.as_deref(),
        params.class_action,
        params.assigned_judge_id.as_deref(),
        params.q.as_deref(),
        offset,
        limit,
    )
    .await?;

    let response = CivilCaseSearchResponse {
        cases: cases.into_iter().map(CivilCaseResponse::from).collect(),
        total,
    };

    Ok(Json(response))
}

/// GET /api/civil-cases/statistics
#[utoipa::path(
    get,
    path = "/api/civil-cases/statistics",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Civil case statistics", body = serde_json::Value)
    ),
    tag = "civil-cases"
)]
pub async fn civil_case_statistics(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<serde_json::Value>, AppError> {
    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM civil_cases WHERE court_id = $1"#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let by_status_raw = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(
            json_object_agg(status, cnt),
            '{}'::json
        )::TEXT as "json!"
        FROM (
            SELECT status, COUNT(*) as cnt
            FROM civil_cases
            WHERE court_id = $1
            GROUP BY status
        ) sub
        "#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let by_nos_raw = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(
            json_object_agg(nature_of_suit, cnt),
            '{}'::json
        )::TEXT as "json!"
        FROM (
            SELECT nature_of_suit, COUNT(*) as cnt
            FROM civil_cases
            WHERE court_id = $1
            GROUP BY nature_of_suit
        ) sub
        "#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let avg_duration: Option<f64> = sqlx::query_scalar!(
        r#"
        SELECT AVG(EXTRACT(EPOCH FROM (COALESCE(closed_at, NOW()) - opened_at)) / 86400.0)::float8 as "avg"
        FROM civil_cases
        WHERE court_id = $1
        "#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let by_status: serde_json::Value = serde_json::from_str(&by_status_raw)
        .unwrap_or(serde_json::json!({}));
    let by_nature_of_suit: serde_json::Value = serde_json::from_str(&by_nos_raw)
        .unwrap_or(serde_json::json!({}));

    Ok(Json(serde_json::json!({
        "total": total,
        "by_status": by_status,
        "by_nature_of_suit": by_nature_of_suit,
        "avg_duration_days": avg_duration,
    })))
}

/// GET /api/civil-cases/by-judge/{judge_id}
#[utoipa::path(
    get,
    path = "/api/civil-cases/by-judge/{judge_id}",
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Civil cases for judge", body = Vec<CivilCaseResponse>)
    ),
    tag = "civil-cases"
)]
pub async fn list_civil_cases_by_judge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(judge_id): Path<String>,
) -> Result<Json<Vec<CivilCaseResponse>>, AppError> {
    let uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| AppError::bad_request("Invalid judge UUID format"))?;

    let rows = sqlx::query_as!(
        CivilCase,
        r#"
        SELECT id, court_id, case_number, title, description, nature_of_suit,
               cause_of_action, jurisdiction_basis, jury_demand, class_action,
               amount_in_controversy as "amount_in_controversy: f64",
               status, priority, assigned_judge_id, district_code, location,
               is_sealed, sealed_date, sealed_by, seal_reason, related_case_id,
               consent_to_magistrate, pro_se, opened_at, updated_at, closed_at
        FROM civil_cases
        WHERE court_id = $1 AND assigned_judge_id = $2
        ORDER BY opened_at DESC
        "#,
        court.0,
        uuid,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let response: Vec<CivilCaseResponse> = rows.into_iter().map(CivilCaseResponse::from).collect();
    Ok(Json(response))
}
