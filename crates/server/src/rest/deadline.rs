use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Datelike;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CalculateDeadlineRequest, CalculateDeadlineResponse,
    CreateDeadlineRequest, Deadline, DeadlineResponse, DeadlineSearchParams,
    DeadlineSearchResponse, UpdateDeadlineRequest, UpdateDeadlineStatusRequest,
    is_valid_deadline_status,
};
use crate::error_convert::SqlxErrorExt;
use crate::tenant::CourtId;

/// POST /api/deadlines
#[utoipa::path(
    post,
    path = "/api/deadlines",
    request_body = CreateDeadlineRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Deadline created", body = DeadlineResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "deadlines"
)]
pub async fn create_deadline(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateDeadlineRequest>,
) -> Result<(StatusCode, Json<DeadlineResponse>), AppError> {
    if body.title.trim().is_empty() {
        return Err(AppError::bad_request("title must not be empty"));
    }

    let deadline = crate::repo::deadline::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(DeadlineResponse::from(deadline))))
}

/// GET /api/deadlines/{id}
#[utoipa::path(
    get,
    path = "/api/deadlines/{id}",
    params(
        ("id" = String, Path, description = "Deadline UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Deadline found", body = DeadlineResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "deadlines"
)]
pub async fn get_deadline(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<DeadlineResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deadline = crate::repo::deadline::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Deadline {} not found", id)))?;

    Ok(Json(DeadlineResponse::from(deadline)))
}

/// PUT /api/deadlines/{id}
#[utoipa::path(
    put,
    path = "/api/deadlines/{id}",
    request_body = UpdateDeadlineRequest,
    params(
        ("id" = String, Path, description = "Deadline UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Deadline updated", body = DeadlineResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "deadlines"
)]
pub async fn update_deadline(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateDeadlineRequest>,
) -> Result<Json<DeadlineResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deadline = crate::repo::deadline::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Deadline {} not found", id)))?;

    Ok(Json(DeadlineResponse::from(deadline)))
}

/// DELETE /api/deadlines/{id}
#[utoipa::path(
    delete,
    path = "/api/deadlines/{id}",
    params(
        ("id" = String, Path, description = "Deadline UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Deadline deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "deadlines"
)]
pub async fn delete_deadline(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::deadline::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Deadline {} not found", id)))
    }
}

/// PATCH /api/deadlines/{id}/status
#[utoipa::path(
    patch,
    path = "/api/deadlines/{id}/status",
    request_body = UpdateDeadlineStatusRequest,
    params(
        ("id" = String, Path, description = "Deadline UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Status updated", body = DeadlineResponse),
        (status = 400, description = "Invalid status", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "deadlines"
)]
pub async fn update_deadline_status(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateDeadlineStatusRequest>,
) -> Result<Json<DeadlineResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !is_valid_deadline_status(&body.status) {
        return Err(AppError::bad_request(format!(
            "Invalid status: {}. Valid values: open, met, extended, cancelled, expired",
            body.status
        )));
    }

    let deadline = crate::repo::deadline::update_status(&pool, &court.0, uuid, &body.status)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Deadline {} not found", id)))?;

    Ok(Json(DeadlineResponse::from(deadline)))
}

/// GET /api/deadlines/search
#[utoipa::path(
    get,
    path = "/api/deadlines/search",
    params(
        DeadlineSearchParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Search results", body = DeadlineSearchResponse)
    ),
    tag = "deadlines"
)]
pub async fn search_deadlines(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<DeadlineSearchParams>,
) -> Result<Json<DeadlineSearchResponse>, AppError> {
    let offset = params.offset.unwrap_or(0).max(0);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);

    if let Some(ref s) = params.status {
        if !is_valid_deadline_status(s) {
            return Err(AppError::bad_request(format!("Invalid status: {}", s)));
        }
    }

    let (deadlines, total) = crate::repo::deadline::search(
        &pool,
        &court.0,
        params.status.as_deref(),
        params.case_id,
        params.date_from,
        params.date_to,
        offset,
        limit,
    )
    .await?;

    let response = DeadlineSearchResponse {
        deadlines: deadlines.into_iter().map(DeadlineResponse::from).collect(),
        total,
    };

    Ok(Json(response))
}

/// GET /api/cases/{case_id}/deadlines
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/deadlines",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Deadlines for case", body = Vec<DeadlineResponse>)
    ),
    tag = "deadlines"
)]
pub async fn list_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<DeadlineResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    let rows = sqlx::query_as!(
        Deadline,
        r#"
        SELECT id, court_id, case_id, title, rule_code, due_at, status, notes, created_at, updated_at
        FROM deadlines
        WHERE court_id = $1 AND case_id = $2
        ORDER BY due_at ASC
        "#,
        court.0,
        uuid,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let response: Vec<DeadlineResponse> = rows.into_iter().map(DeadlineResponse::from).collect();
    Ok(Json(response))
}

/// PATCH /api/deadlines/{id}/complete
#[utoipa::path(
    patch,
    path = "/api/deadlines/{id}/complete",
    params(
        ("id" = String, Path, description = "Deadline UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Deadline completed", body = DeadlineResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "deadlines"
)]
pub async fn complete_deadline(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<DeadlineResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deadline = crate::repo::deadline::update_status(&pool, &court.0, uuid, "met")
        .await?
        .ok_or_else(|| AppError::not_found(format!("Deadline {} not found", id)))?;

    Ok(Json(DeadlineResponse::from(deadline)))
}

/// GET /api/cases/{case_id}/deadlines/type/{deadline_type}
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/deadlines/type/{deadline_type}",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("deadline_type" = String, Path, description = "Deadline status type"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Deadlines filtered by case and type", body = Vec<DeadlineResponse>)
    ),
    tag = "deadlines"
)]
pub async fn list_by_case_and_type(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((case_id, deadline_type)): Path<(String, String)>,
) -> Result<Json<Vec<DeadlineResponse>>, AppError> {
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    if !is_valid_deadline_status(&deadline_type) {
        return Err(AppError::bad_request(format!(
            "Invalid deadline_type: {}",
            deadline_type
        )));
    }

    let rows = sqlx::query_as!(
        Deadline,
        r#"
        SELECT id, court_id, case_id, title, rule_code, due_at, status, notes, created_at, updated_at
        FROM deadlines
        WHERE court_id = $1 AND case_id = $2 AND status = $3
        ORDER BY due_at ASC
        "#,
        court.0,
        case_uuid,
        deadline_type,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let response: Vec<DeadlineResponse> = rows.into_iter().map(DeadlineResponse::from).collect();
    Ok(Json(response))
}

/// GET /api/deadlines/upcoming
#[utoipa::path(
    get,
    path = "/api/deadlines/upcoming",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Upcoming deadlines (next 30 days)", body = Vec<DeadlineResponse>)
    ),
    tag = "deadlines"
)]
pub async fn list_upcoming(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<DeadlineResponse>>, AppError> {
    let rows = sqlx::query_as!(
        Deadline,
        r#"
        SELECT id, court_id, case_id, title, rule_code, due_at, status, notes, created_at, updated_at
        FROM deadlines
        WHERE court_id = $1
          AND status = 'open'
          AND due_at >= NOW()
          AND due_at <= NOW() + INTERVAL '30 days'
        ORDER BY due_at ASC
        "#,
        court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let response: Vec<DeadlineResponse> = rows.into_iter().map(DeadlineResponse::from).collect();
    Ok(Json(response))
}

/// GET /api/deadlines/urgent
#[utoipa::path(
    get,
    path = "/api/deadlines/urgent",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Urgent deadlines (next 7 days)", body = Vec<DeadlineResponse>)
    ),
    tag = "deadlines"
)]
pub async fn list_urgent(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<DeadlineResponse>>, AppError> {
    let rows = sqlx::query_as!(
        Deadline,
        r#"
        SELECT id, court_id, case_id, title, rule_code, due_at, status, notes, created_at, updated_at
        FROM deadlines
        WHERE court_id = $1
          AND status = 'open'
          AND due_at >= NOW()
          AND due_at <= NOW() + INTERVAL '7 days'
        ORDER BY due_at ASC
        "#,
        court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let response: Vec<DeadlineResponse> = rows.into_iter().map(DeadlineResponse::from).collect();
    Ok(Json(response))
}

/// POST /api/deadlines/calculate
#[utoipa::path(
    post,
    path = "/api/deadlines/calculate",
    request_body = CalculateDeadlineRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Calculated deadline", body = CalculateDeadlineResponse),
        (status = 400, description = "Unknown rule", body = AppError)
    ),
    tag = "deadlines"
)]
pub async fn calculate_deadline(
    Json(body): Json<CalculateDeadlineRequest>,
) -> Result<Json<CalculateDeadlineResponse>, AppError> {
    // Lookup the rule from hardcoded federal rules
    let (days, description, business_days_only) = match body.rule_code.as_str() {
        "FRCP-5" => (1, "Initial Appearance", false),
        "FRCP-5.1" => (14, "Preliminary Hearing", false),
        "FRCP-10" => (14, "Arraignment", false),
        "FRCP-12" => (14, "Pretrial Motions", true),
        "FRCP-16" => (14, "Discovery Disclosure", false),
        "FRCP-29" => (14, "Motion for Judgment of Acquittal", false),
        "FRCP-33" => (14, "Motion for New Trial", false),
        "FRCP-35" => (14, "Correcting or Reducing a Sentence", false),
        "STA-70" => (70, "Speedy Trial Act - 70 Day Rule", false),
        "STA-30" => (30, "Speedy Trial Act - 30 Day Indictment", false),
        "FRAP-4" => (14, "Notice of Appeal", false),
        "18USC3161" => (70, "Speedy Trial Act Compliance", false),
        _ => {
            return Err(AppError::bad_request(format!(
                "Unknown rule_code: {}",
                body.rule_code
            )));
        }
    };

    let calculated = if business_days_only {
        // Add business days (skip weekends)
        let mut date = body.trigger_date;
        let mut added = 0;
        while added < days {
            date = date + chrono::Duration::days(1);
            let weekday = date.weekday();
            if weekday != chrono::Weekday::Sat && weekday != chrono::Weekday::Sun {
                added += 1;
            }
        }
        date
    } else {
        body.trigger_date + chrono::Duration::days(days as i64)
    };

    Ok(Json(CalculateDeadlineResponse {
        rule_code: body.rule_code,
        calculated_date: calculated.to_rfc3339(),
        description: description.to_string(),
    }))
}
