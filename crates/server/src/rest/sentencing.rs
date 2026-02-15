use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CreateSentencingRequest, SentencingResponse, UpdateSentencingRequest,
    is_valid_criminal_history_category, CRIMINAL_HISTORY_CATEGORIES,
    is_valid_departure_type, DEPARTURE_TYPES,
    is_valid_variance_type, VARIANCE_TYPES,
    // New sentencing types
    DepartureRequest, VarianceRequest, SupervisedReleaseRequest,
    GuidelinesRequest, GuidelinesResult, OffenseLevelRequest,
    SentencingStatistics, DateRangeParams,
    CreateSpecialConditionRequest, SpecialConditionResponse,
    CreateBopDesignationRequest, BopDesignationResponse, BOP_SECURITY_LEVELS,
    CreatePriorSentenceRequest, PriorSentenceResponse,
};
use crate::tenant::CourtId;

/// POST /api/sentencing
#[utoipa::path(
    post,
    path = "/api/sentencing",
    request_body = CreateSentencingRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Sentencing created", body = SentencingResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "sentencing"
)]
pub async fn create_sentencing(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateSentencingRequest>,
) -> Result<(StatusCode, Json<SentencingResponse>), AppError> {
    if let Some(ref chc) = body.criminal_history_category {
        if !is_valid_criminal_history_category(chc) {
            return Err(AppError::bad_request(format!(
                "Invalid criminal_history_category: {}. Valid values: {}",
                chc,
                CRIMINAL_HISTORY_CATEGORIES.join(", ")
            )));
        }
    }

    if let Some(ref dt) = body.departure_type {
        if !is_valid_departure_type(dt) {
            return Err(AppError::bad_request(format!(
                "Invalid departure_type: {}. Valid values: {}",
                dt,
                DEPARTURE_TYPES.join(", ")
            )));
        }
    }

    if let Some(ref vt) = body.variance_type {
        if !is_valid_variance_type(vt) {
            return Err(AppError::bad_request(format!(
                "Invalid variance_type: {}. Valid values: {}",
                vt,
                VARIANCE_TYPES.join(", ")
            )));
        }
    }

    let sentencing = crate::repo::sentencing::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(SentencingResponse::from(sentencing))))
}

/// GET /api/sentencing/{id}
#[utoipa::path(
    get,
    path = "/api/sentencing/{id}",
    params(
        ("id" = String, Path, description = "Sentencing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Sentencing found", body = SentencingResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "sentencing"
)]
pub async fn get_sentencing(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<SentencingResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let sentencing = crate::repo::sentencing::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Sentencing {} not found", id)))?;

    Ok(Json(SentencingResponse::from(sentencing)))
}

/// PUT /api/sentencing/{id}
#[utoipa::path(
    put,
    path = "/api/sentencing/{id}",
    request_body = UpdateSentencingRequest,
    params(
        ("id" = String, Path, description = "Sentencing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Sentencing updated", body = SentencingResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "sentencing"
)]
pub async fn update_sentencing(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateSentencingRequest>,
) -> Result<Json<SentencingResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref chc) = body.criminal_history_category {
        if !is_valid_criminal_history_category(chc) {
            return Err(AppError::bad_request(format!(
                "Invalid criminal_history_category: {}. Valid values: {}",
                chc,
                CRIMINAL_HISTORY_CATEGORIES.join(", ")
            )));
        }
    }

    if let Some(ref dt) = body.departure_type {
        if !is_valid_departure_type(dt) {
            return Err(AppError::bad_request(format!(
                "Invalid departure_type: {}. Valid values: {}",
                dt,
                DEPARTURE_TYPES.join(", ")
            )));
        }
    }

    if let Some(ref vt) = body.variance_type {
        if !is_valid_variance_type(vt) {
            return Err(AppError::bad_request(format!(
                "Invalid variance_type: {}. Valid values: {}",
                vt,
                VARIANCE_TYPES.join(", ")
            )));
        }
    }

    let sentencing = crate::repo::sentencing::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Sentencing {} not found", id)))?;

    Ok(Json(SentencingResponse::from(sentencing)))
}

/// DELETE /api/sentencing/{id}
#[utoipa::path(
    delete,
    path = "/api/sentencing/{id}",
    params(
        ("id" = String, Path, description = "Sentencing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Sentencing deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "sentencing"
)]
pub async fn delete_sentencing(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::sentencing::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Sentencing {} not found", id)))
    }
}

/// GET /api/cases/{case_id}/sentencing
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/sentencing",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Sentencings for case", body = Vec<SentencingResponse>)
    ),
    tag = "sentencing"
)]
pub async fn list_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<SentencingResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let sentencings = crate::repo::sentencing::list_by_case(&pool, &court.0, uuid).await?;
    let responses: Vec<SentencingResponse> = sentencings.into_iter().map(SentencingResponse::from).collect();

    Ok(Json(responses))
}

/// GET /api/sentencing/defendant/{defendant_id}
#[utoipa::path(
    get,
    path = "/api/sentencing/defendant/{defendant_id}",
    params(
        ("defendant_id" = String, Path, description = "Defendant UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Sentencings for defendant", body = Vec<SentencingResponse>)
    ),
    tag = "sentencing"
)]
pub async fn list_by_defendant(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(defendant_id): Path<String>,
) -> Result<Json<Vec<SentencingResponse>>, AppError> {
    let uuid = Uuid::parse_str(&defendant_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let sentencings = crate::repo::sentencing::list_by_defendant(&pool, &court.0, uuid).await?;
    let responses: Vec<SentencingResponse> = sentencings.into_iter().map(SentencingResponse::from).collect();

    Ok(Json(responses))
}

/// GET /api/sentencing/judge/{judge_id}
#[utoipa::path(
    get,
    path = "/api/sentencing/judge/{judge_id}",
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Sentencings for judge", body = Vec<SentencingResponse>)
    ),
    tag = "sentencing"
)]
pub async fn list_by_judge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(judge_id): Path<String>,
) -> Result<Json<Vec<SentencingResponse>>, AppError> {
    let uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let sentencings = crate::repo::sentencing::list_by_judge(&pool, &court.0, uuid).await?;
    let responses: Vec<SentencingResponse> = sentencings.into_iter().map(SentencingResponse::from).collect();

    Ok(Json(responses))
}

// ── Extended sentencing handlers ────────────────────────────────────

/// GET /api/sentencing/pending
/// List sentencing records that have no sentencing_date yet.
#[utoipa::path(
    get,
    path = "/api/sentencing/pending",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Pending sentencings", body = Vec<SentencingResponse>)),
    tag = "sentencing"
)]
pub async fn list_pending(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<SentencingResponse>>, AppError> {
    let rows = sqlx::query_as!(
        shared_types::SentencingRecord,
        r#"
        SELECT id, court_id, case_id, defendant_id, judge_id,
               base_offense_level, specific_offense_level,
               adjusted_offense_level, total_offense_level,
               criminal_history_category, criminal_history_points,
               guidelines_range_low_months, guidelines_range_high_months,
               custody_months, probation_months,
               fine_amount::FLOAT8 as "fine_amount: f64",
               restitution_amount::FLOAT8 as "restitution_amount: f64",
               forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
               special_assessment::FLOAT8 as "special_assessment: f64",
               departure_type, departure_reason,
               variance_type, variance_justification,
               supervised_release_months, appeal_waiver,
               sentencing_date, judgment_date,
               created_at, updated_at
        FROM sentencing
        WHERE court_id = $1 AND sentencing_date IS NULL
        ORDER BY created_at ASC
        "#,
        &court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<SentencingResponse> = rows.into_iter().map(SentencingResponse::from).collect();
    Ok(Json(responses))
}

/// POST /api/sentencing/calculate-guidelines
/// Calculate sentencing guidelines range from inputs.
#[utoipa::path(
    post,
    path = "/api/sentencing/calculate-guidelines",
    request_body = GuidelinesRequest,
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses(
        (status = 200, description = "Guidelines calculation result", body = GuidelinesResult),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "sentencing"
)]
pub async fn calculate_guidelines(
    State(_pool): State<Pool<Postgres>>,
    _court: CourtId,
    Json(body): Json<GuidelinesRequest>,
) -> Result<Json<GuidelinesResult>, AppError> {
    if !is_valid_criminal_history_category(&body.criminal_history_category) {
        return Err(AppError::bad_request(format!(
            "Invalid criminal_history_category: {}. Valid values: {}",
            body.criminal_history_category,
            CRIMINAL_HISTORY_CATEGORIES.join(", ")
        )));
    }

    // Calculate total offense level
    let adjustments_sum: i32 = body.specific_offense_adjustments.iter().sum();
    let acceptance = body.acceptance_reduction.unwrap_or(0);
    let total_offense_level = (body.base_offense_level + adjustments_sum - acceptance).max(1).min(43);

    // Simplified guidelines range lookup based on offense level and category
    let category_index = match body.criminal_history_category.as_str() {
        "I" => 0,
        "II" => 1,
        "III" => 2,
        "IV" => 3,
        "V" => 4,
        "VI" => 5,
        _ => 0,
    };

    // Base range estimates (simplified; real USSG tables have 258 cells)
    let base_low = match total_offense_level {
        1..=6 => 0,
        7..=10 => [2, 4, 6, 8, 9, 12][category_index],
        11..=15 => [8, 10, 12, 15, 18, 21][category_index],
        16..=20 => [21, 24, 27, 30, 33, 37][category_index],
        21..=25 => [37, 41, 46, 51, 57, 63][category_index],
        26..=30 => [63, 70, 78, 87, 97, 108][category_index],
        31..=35 => [108, 121, 135, 151, 168, 188][category_index],
        36..=40 => [188, 210, 235, 262, 292, 324][category_index],
        41..=43 => [360, 360, 360, 360, 360, 360][category_index],
        _ => 0,
    };

    let base_high = (base_low as f64 * 1.25).ceil() as i32;

    Ok(Json(GuidelinesResult {
        total_offense_level,
        criminal_history_category: body.criminal_history_category,
        range_low_months: base_low,
        range_high_months: base_high,
    }))
}

/// GET /api/sentencing/statistics/departures
/// Get departure statistics for a court.
#[utoipa::path(
    get,
    path = "/api/sentencing/statistics/departures",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Departure stats", body = serde_json::Value)),
    tag = "sentencing"
)]
pub async fn departure_stats(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<serde_json::Value>, AppError> {
    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM sentencing WHERE court_id = $1 AND sentencing_date IS NOT NULL"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let departures: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM sentencing WHERE court_id = $1 AND departure_type IS NOT NULL AND departure_type != 'None'"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let rate = if total > 0 { departures as f64 / total as f64 } else { 0.0 };

    Ok(Json(serde_json::json!({
        "total_sentenced": total,
        "total_departures": departures,
        "departure_rate": rate,
    })))
}

/// GET /api/sentencing/statistics/variances
/// Get variance statistics for a court.
#[utoipa::path(
    get,
    path = "/api/sentencing/statistics/variances",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Variance stats", body = serde_json::Value)),
    tag = "sentencing"
)]
pub async fn variance_stats(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<serde_json::Value>, AppError> {
    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM sentencing WHERE court_id = $1 AND sentencing_date IS NOT NULL"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let variances: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM sentencing WHERE court_id = $1 AND variance_type IS NOT NULL AND variance_type != 'None'"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let rate = if total > 0 { variances as f64 / total as f64 } else { 0.0 };

    Ok(Json(serde_json::json!({
        "total_sentenced": total,
        "total_variances": variances,
        "variance_rate": rate,
    })))
}

/// GET /api/sentencing/statistics/judge/{judge_id}
/// Get sentencing statistics for a specific judge.
#[utoipa::path(
    get,
    path = "/api/sentencing/statistics/judge/{judge_id}",
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses((status = 200, description = "Judge sentencing stats", body = SentencingStatistics)),
    tag = "sentencing"
)]
pub async fn judge_stats(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(judge_id): Path<String>,
) -> Result<Json<SentencingStatistics>, AppError> {
    let uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM sentencing WHERE court_id = $1 AND judge_id = $2"#,
        &court.0,
        uuid,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let avg_custody_months: Option<f64> = sqlx::query_scalar!(
        r#"SELECT AVG(custody_months)::FLOAT8 as "avg: f64" FROM sentencing WHERE court_id = $1 AND judge_id = $2 AND custody_months IS NOT NULL"#,
        &court.0,
        uuid,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let departures: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM sentencing WHERE court_id = $1 AND judge_id = $2 AND departure_type IS NOT NULL AND departure_type != 'None'"#,
        &court.0,
        uuid,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let variances: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM sentencing WHERE court_id = $1 AND judge_id = $2 AND variance_type IS NOT NULL AND variance_type != 'None'"#,
        &court.0,
        uuid,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let departure_rate = if total > 0 { departures as f64 / total as f64 } else { 0.0 };
    let variance_rate = if total > 0 { variances as f64 / total as f64 } else { 0.0 };

    Ok(Json(SentencingStatistics {
        total,
        avg_custody_months,
        departure_rate,
        variance_rate,
    }))
}

/// GET /api/sentencing/statistics/district
/// Get sentencing statistics for the entire court district.
#[utoipa::path(
    get,
    path = "/api/sentencing/statistics/district",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "District sentencing stats", body = SentencingStatistics)),
    tag = "sentencing"
)]
pub async fn district_stats(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<SentencingStatistics>, AppError> {
    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM sentencing WHERE court_id = $1"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let avg_custody_months: Option<f64> = sqlx::query_scalar!(
        r#"SELECT AVG(custody_months)::FLOAT8 as "avg: f64" FROM sentencing WHERE court_id = $1 AND custody_months IS NOT NULL"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let departures: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM sentencing WHERE court_id = $1 AND departure_type IS NOT NULL AND departure_type != 'None'"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let variances: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM sentencing WHERE court_id = $1 AND variance_type IS NOT NULL AND variance_type != 'None'"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let departure_rate = if total > 0 { departures as f64 / total as f64 } else { 0.0 };
    let variance_rate = if total > 0 { variances as f64 / total as f64 } else { 0.0 };

    Ok(Json(SentencingStatistics {
        total,
        avg_custody_months,
        departure_rate,
        variance_rate,
    }))
}

/// GET /api/sentencing/statistics/trial-penalty
/// Get trial penalty statistics (comparing plea vs trial sentences).
#[utoipa::path(
    get,
    path = "/api/sentencing/statistics/trial-penalty",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Trial penalty stats", body = serde_json::Value)),
    tag = "sentencing"
)]
pub async fn trial_penalty_stats(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<serde_json::Value>, AppError> {
    // Compare average custody months for records with vs without acceptance reduction
    let with_acceptance: Option<f64> = sqlx::query_scalar!(
        r#"
        SELECT AVG(custody_months)::FLOAT8 as "avg: f64"
        FROM sentencing
        WHERE court_id = $1 AND custody_months IS NOT NULL
          AND adjusted_offense_level IS NOT NULL
          AND total_offense_level IS NOT NULL
          AND adjusted_offense_level > total_offense_level
        "#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let without_acceptance: Option<f64> = sqlx::query_scalar!(
        r#"
        SELECT AVG(custody_months)::FLOAT8 as "avg: f64"
        FROM sentencing
        WHERE court_id = $1 AND custody_months IS NOT NULL
          AND (adjusted_offense_level IS NULL OR total_offense_level IS NULL
               OR adjusted_offense_level <= total_offense_level)
        "#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    Ok(Json(serde_json::json!({
        "avg_custody_with_acceptance": with_acceptance,
        "avg_custody_without_acceptance": without_acceptance,
    })))
}

/// GET /api/sentencing/statistics/offense/{offense_type}
/// Get offense-level distribution statistics.
#[utoipa::path(
    get,
    path = "/api/sentencing/statistics/offense/{offense_type}",
    params(
        ("offense_type" = String, Path, description = "Offense type"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses((status = 200, description = "Offense stats", body = serde_json::Value)),
    tag = "sentencing"
)]
pub async fn offense_stats(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(_offense_type): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let avg_base: Option<f64> = sqlx::query_scalar!(
        r#"SELECT AVG(base_offense_level)::FLOAT8 as "avg: f64" FROM sentencing WHERE court_id = $1 AND base_offense_level IS NOT NULL"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let avg_total: Option<f64> = sqlx::query_scalar!(
        r#"SELECT AVG(total_offense_level)::FLOAT8 as "avg: f64" FROM sentencing WHERE court_id = $1 AND total_offense_level IS NOT NULL"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    Ok(Json(serde_json::json!({
        "avg_base_offense_level": avg_base,
        "avg_total_offense_level": avg_total,
    })))
}

/// POST /api/sentencing/{id}/departure
/// Record a departure on a sentencing record.
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/departure",
    request_body = DepartureRequest,
    params(
        ("id" = String, Path, description = "Sentencing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Departure recorded", body = SentencingResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "sentencing"
)]
pub async fn record_departure(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<DepartureRequest>,
) -> Result<Json<SentencingResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !is_valid_departure_type(&body.departure_type) {
        return Err(AppError::bad_request(format!(
            "Invalid departure_type: {}. Valid values: {}",
            body.departure_type,
            DEPARTURE_TYPES.join(", ")
        )));
    }

    let sentencing = sqlx::query_as!(
        shared_types::SentencingRecord,
        r#"
        UPDATE sentencing SET
            departure_type = $3,
            departure_reason = $4,
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, defendant_id, judge_id,
                  base_offense_level, specific_offense_level,
                  adjusted_offense_level, total_offense_level,
                  criminal_history_category, criminal_history_points,
                  guidelines_range_low_months, guidelines_range_high_months,
                  custody_months, probation_months,
                  fine_amount::FLOAT8 as "fine_amount: f64",
                  restitution_amount::FLOAT8 as "restitution_amount: f64",
                  forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
                  special_assessment::FLOAT8 as "special_assessment: f64",
                  departure_type, departure_reason,
                  variance_type, variance_justification,
                  supervised_release_months, appeal_waiver,
                  sentencing_date, judgment_date,
                  created_at, updated_at
        "#,
        uuid,
        &court.0,
        body.departure_type,
        body.departure_reason,
    )
    .fetch_optional(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found(format!("Sentencing {} not found", id)))?;

    Ok(Json(SentencingResponse::from(sentencing)))
}

/// POST /api/sentencing/{id}/variance
/// Record a variance on a sentencing record.
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/variance",
    request_body = VarianceRequest,
    params(
        ("id" = String, Path, description = "Sentencing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Variance recorded", body = SentencingResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "sentencing"
)]
pub async fn record_variance(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<VarianceRequest>,
) -> Result<Json<SentencingResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !is_valid_variance_type(&body.variance_type) {
        return Err(AppError::bad_request(format!(
            "Invalid variance_type: {}. Valid values: {}",
            body.variance_type,
            VARIANCE_TYPES.join(", ")
        )));
    }

    let sentencing = sqlx::query_as!(
        shared_types::SentencingRecord,
        r#"
        UPDATE sentencing SET
            variance_type = $3,
            variance_justification = $4,
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, defendant_id, judge_id,
                  base_offense_level, specific_offense_level,
                  adjusted_offense_level, total_offense_level,
                  criminal_history_category, criminal_history_points,
                  guidelines_range_low_months, guidelines_range_high_months,
                  custody_months, probation_months,
                  fine_amount::FLOAT8 as "fine_amount: f64",
                  restitution_amount::FLOAT8 as "restitution_amount: f64",
                  forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
                  special_assessment::FLOAT8 as "special_assessment: f64",
                  departure_type, departure_reason,
                  variance_type, variance_justification,
                  supervised_release_months, appeal_waiver,
                  sentencing_date, judgment_date,
                  created_at, updated_at
        "#,
        uuid,
        &court.0,
        body.variance_type,
        body.variance_justification,
    )
    .fetch_optional(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found(format!("Sentencing {} not found", id)))?;

    Ok(Json(SentencingResponse::from(sentencing)))
}

/// GET /api/sentencing/substantial-assistance
/// List sentencing records with downward departures (substantial assistance).
#[utoipa::path(
    get,
    path = "/api/sentencing/substantial-assistance",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Substantial assistance cases", body = Vec<SentencingResponse>)),
    tag = "sentencing"
)]
pub async fn list_substantial_assistance(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<SentencingResponse>>, AppError> {
    let rows = sqlx::query_as!(
        shared_types::SentencingRecord,
        r#"
        SELECT id, court_id, case_id, defendant_id, judge_id,
               base_offense_level, specific_offense_level,
               adjusted_offense_level, total_offense_level,
               criminal_history_category, criminal_history_points,
               guidelines_range_low_months, guidelines_range_high_months,
               custody_months, probation_months,
               fine_amount::FLOAT8 as "fine_amount: f64",
               restitution_amount::FLOAT8 as "restitution_amount: f64",
               forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
               special_assessment::FLOAT8 as "special_assessment: f64",
               departure_type, departure_reason,
               variance_type, variance_justification,
               supervised_release_months, appeal_waiver,
               sentencing_date, judgment_date,
               created_at, updated_at
        FROM sentencing
        WHERE court_id = $1 AND departure_type = 'Downward'
        ORDER BY sentencing_date DESC NULLS LAST
        "#,
        &court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<SentencingResponse> = rows.into_iter().map(SentencingResponse::from).collect();
    Ok(Json(responses))
}

/// POST /api/sentencing/{id}/special-conditions
/// Add a special condition to a sentencing record.
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/special-conditions",
    request_body = CreateSpecialConditionRequest,
    params(
        ("id" = String, Path, description = "Sentencing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Special condition added", body = SpecialConditionResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "sentencing"
)]
pub async fn add_special_condition(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<CreateSpecialConditionRequest>,
) -> Result<(StatusCode, Json<SpecialConditionResponse>), AppError> {
    let sentencing_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if body.condition_type.trim().is_empty() {
        return Err(AppError::bad_request("condition_type must not be empty"));
    }
    if body.description.trim().is_empty() {
        return Err(AppError::bad_request("description must not be empty"));
    }

    let condition = crate::repo::sentencing_condition::create(&pool, &court.0, sentencing_uuid, body).await?;
    Ok((StatusCode::CREATED, Json(SpecialConditionResponse::from(condition))))
}

/// PUT /api/sentencing/{id}/supervised-release
/// Update supervised-release months on a sentencing record.
#[utoipa::path(
    put,
    path = "/api/sentencing/{id}/supervised-release",
    request_body = SupervisedReleaseRequest,
    params(
        ("id" = String, Path, description = "Sentencing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Supervised release updated", body = SentencingResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "sentencing"
)]
pub async fn update_supervised_release(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<SupervisedReleaseRequest>,
) -> Result<Json<SentencingResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let sentencing = sqlx::query_as!(
        shared_types::SentencingRecord,
        r#"
        UPDATE sentencing SET
            supervised_release_months = $3,
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, defendant_id, judge_id,
                  base_offense_level, specific_offense_level,
                  adjusted_offense_level, total_offense_level,
                  criminal_history_category, criminal_history_points,
                  guidelines_range_low_months, guidelines_range_high_months,
                  custody_months, probation_months,
                  fine_amount::FLOAT8 as "fine_amount: f64",
                  restitution_amount::FLOAT8 as "restitution_amount: f64",
                  forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
                  special_assessment::FLOAT8 as "special_assessment: f64",
                  departure_type, departure_reason,
                  variance_type, variance_justification,
                  supervised_release_months, appeal_waiver,
                  sentencing_date, judgment_date,
                  created_at, updated_at
        "#,
        uuid,
        &court.0,
        body.supervised_release_months,
    )
    .fetch_optional(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found(format!("Sentencing {} not found", id)))?;

    Ok(Json(SentencingResponse::from(sentencing)))
}

/// GET /api/sentencing/active-supervision
/// List sentencing records with active supervised release.
#[utoipa::path(
    get,
    path = "/api/sentencing/active-supervision",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Active supervision cases", body = Vec<SentencingResponse>)),
    tag = "sentencing"
)]
pub async fn list_active_supervision(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<SentencingResponse>>, AppError> {
    let rows = sqlx::query_as!(
        shared_types::SentencingRecord,
        r#"
        SELECT id, court_id, case_id, defendant_id, judge_id,
               base_offense_level, specific_offense_level,
               adjusted_offense_level, total_offense_level,
               criminal_history_category, criminal_history_points,
               guidelines_range_low_months, guidelines_range_high_months,
               custody_months, probation_months,
               fine_amount::FLOAT8 as "fine_amount: f64",
               restitution_amount::FLOAT8 as "restitution_amount: f64",
               forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
               special_assessment::FLOAT8 as "special_assessment: f64",
               departure_type, departure_reason,
               variance_type, variance_justification,
               supervised_release_months, appeal_waiver,
               sentencing_date, judgment_date,
               created_at, updated_at
        FROM sentencing
        WHERE court_id = $1
          AND supervised_release_months IS NOT NULL
          AND supervised_release_months > 0
        ORDER BY sentencing_date DESC NULLS LAST
        "#,
        &court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<SentencingResponse> = rows.into_iter().map(SentencingResponse::from).collect();
    Ok(Json(responses))
}

/// POST /api/sentencing/{id}/bop-designation
/// Add a BOP designation to a sentencing record.
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/bop-designation",
    request_body = CreateBopDesignationRequest,
    params(
        ("id" = String, Path, description = "Sentencing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "BOP designation added", body = BopDesignationResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "sentencing"
)]
pub async fn add_bop_designation(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<CreateBopDesignationRequest>,
) -> Result<(StatusCode, Json<BopDesignationResponse>), AppError> {
    let sentencing_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !BOP_SECURITY_LEVELS.contains(&body.security_level.as_str()) {
        return Err(AppError::bad_request(format!(
            "Invalid security_level: {}. Valid values: {}",
            body.security_level,
            BOP_SECURITY_LEVELS.join(", ")
        )));
    }

    let designation = crate::repo::bop_designation::create(&pool, &court.0, sentencing_uuid, body).await?;
    Ok((StatusCode::CREATED, Json(BopDesignationResponse::from(designation))))
}

/// GET /api/sentencing/rdap-eligible
/// List all BOP designations where the defendant is RDAP-eligible.
#[utoipa::path(
    get,
    path = "/api/sentencing/rdap-eligible",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "RDAP eligible designations", body = Vec<BopDesignationResponse>)),
    tag = "sentencing"
)]
pub async fn list_rdap_eligible(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<BopDesignationResponse>>, AppError> {
    let designations = crate::repo::bop_designation::list_rdap_eligible(&pool, &court.0).await?;
    let responses: Vec<BopDesignationResponse> = designations
        .into_iter()
        .map(BopDesignationResponse::from)
        .collect();
    Ok(Json(responses))
}

/// POST /api/sentencing/{id}/prior-sentences
/// Add a prior sentence to a sentencing record.
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/prior-sentences",
    request_body = CreatePriorSentenceRequest,
    params(
        ("id" = String, Path, description = "Sentencing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Prior sentence added", body = PriorSentenceResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "sentencing"
)]
pub async fn add_prior_sentence(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<CreatePriorSentenceRequest>,
) -> Result<(StatusCode, Json<PriorSentenceResponse>), AppError> {
    let sentencing_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if body.jurisdiction.trim().is_empty() {
        return Err(AppError::bad_request("jurisdiction must not be empty"));
    }
    if body.offense.trim().is_empty() {
        return Err(AppError::bad_request("offense must not be empty"));
    }

    let prior = crate::repo::prior_sentence::create(&pool, &court.0, sentencing_uuid, body).await?;
    Ok((StatusCode::CREATED, Json(PriorSentenceResponse::from(prior))))
}

/// GET /api/sentencing/upcoming
/// List sentencing records with upcoming sentencing dates (in the next 30 days).
#[utoipa::path(
    get,
    path = "/api/sentencing/upcoming",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Upcoming sentencings", body = Vec<SentencingResponse>)),
    tag = "sentencing"
)]
pub async fn list_upcoming(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<SentencingResponse>>, AppError> {
    let rows = sqlx::query_as!(
        shared_types::SentencingRecord,
        r#"
        SELECT id, court_id, case_id, defendant_id, judge_id,
               base_offense_level, specific_offense_level,
               adjusted_offense_level, total_offense_level,
               criminal_history_category, criminal_history_points,
               guidelines_range_low_months, guidelines_range_high_months,
               custody_months, probation_months,
               fine_amount::FLOAT8 as "fine_amount: f64",
               restitution_amount::FLOAT8 as "restitution_amount: f64",
               forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
               special_assessment::FLOAT8 as "special_assessment: f64",
               departure_type, departure_reason,
               variance_type, variance_justification,
               supervised_release_months, appeal_waiver,
               sentencing_date, judgment_date,
               created_at, updated_at
        FROM sentencing
        WHERE court_id = $1
          AND sentencing_date IS NOT NULL
          AND sentencing_date > NOW()
          AND sentencing_date <= NOW() + INTERVAL '30 days'
        ORDER BY sentencing_date ASC
        "#,
        &court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<SentencingResponse> = rows.into_iter().map(SentencingResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/sentencing/appeal-deadlines
/// List sentencing records where appeal deadline is approaching (14 days from judgment).
#[utoipa::path(
    get,
    path = "/api/sentencing/appeal-deadlines",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Sentencings with approaching appeal deadlines", body = Vec<SentencingResponse>)),
    tag = "sentencing"
)]
pub async fn list_appeal_deadlines(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<SentencingResponse>>, AppError> {
    let rows = sqlx::query_as!(
        shared_types::SentencingRecord,
        r#"
        SELECT id, court_id, case_id, defendant_id, judge_id,
               base_offense_level, specific_offense_level,
               adjusted_offense_level, total_offense_level,
               criminal_history_category, criminal_history_points,
               guidelines_range_low_months, guidelines_range_high_months,
               custody_months, probation_months,
               fine_amount::FLOAT8 as "fine_amount: f64",
               restitution_amount::FLOAT8 as "restitution_amount: f64",
               forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
               special_assessment::FLOAT8 as "special_assessment: f64",
               departure_type, departure_reason,
               variance_type, variance_justification,
               supervised_release_months, appeal_waiver,
               sentencing_date, judgment_date,
               created_at, updated_at
        FROM sentencing
        WHERE court_id = $1
          AND judgment_date IS NOT NULL
          AND appeal_waiver = false
          AND judgment_date + INTERVAL '14 days' > NOW()
          AND judgment_date + INTERVAL '14 days' <= NOW() + INTERVAL '14 days'
        ORDER BY judgment_date ASC
        "#,
        &court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<SentencingResponse> = rows.into_iter().map(SentencingResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/sentencing/date-range
/// List sentencing records within a date range.
#[utoipa::path(
    get,
    path = "/api/sentencing/date-range",
    params(
        ("from" = Option<String>, Query, description = "Start date (ISO 8601)"),
        ("to" = Option<String>, Query, description = "End date (ISO 8601)"),
        ("offset" = Option<i64>, Query, description = "Pagination offset"),
        ("limit" = Option<i64>, Query, description = "Pagination limit"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses((status = 200, description = "Sentencings in date range", body = Vec<SentencingResponse>)),
    tag = "sentencing"
)]
pub async fn list_by_date_range(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<DateRangeParams>,
) -> Result<Json<Vec<SentencingResponse>>, AppError> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let rows = sqlx::query_as!(
        shared_types::SentencingRecord,
        r#"
        SELECT id, court_id, case_id, defendant_id, judge_id,
               base_offense_level, specific_offense_level,
               adjusted_offense_level, total_offense_level,
               criminal_history_category, criminal_history_points,
               guidelines_range_low_months, guidelines_range_high_months,
               custody_months, probation_months,
               fine_amount::FLOAT8 as "fine_amount: f64",
               restitution_amount::FLOAT8 as "restitution_amount: f64",
               forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
               special_assessment::FLOAT8 as "special_assessment: f64",
               departure_type, departure_reason,
               variance_type, variance_justification,
               supervised_release_months, appeal_waiver,
               sentencing_date, judgment_date,
               created_at, updated_at
        FROM sentencing
        WHERE court_id = $1
          AND ($2::TIMESTAMPTZ IS NULL OR sentencing_date >= $2)
          AND ($3::TIMESTAMPTZ IS NULL OR sentencing_date <= $3)
        ORDER BY sentencing_date DESC NULLS LAST
        LIMIT $4 OFFSET $5
        "#,
        &court.0,
        params.from,
        params.to,
        limit,
        offset,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<SentencingResponse> = rows.into_iter().map(SentencingResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/sentencing/{id}/criminal-history-points
/// Calculate criminal-history points from prior sentences.
#[utoipa::path(
    get,
    path = "/api/sentencing/{id}/criminal-history-points",
    params(
        ("id" = String, Path, description = "Sentencing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Criminal history points", body = serde_json::Value)
    ),
    tag = "sentencing"
)]
pub async fn calc_history_points(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let points = crate::repo::prior_sentence::calc_points(&pool, &court.0, uuid).await?;

    // Map points to criminal-history category
    let category = match points {
        0..=1 => "I",
        2..=3 => "II",
        4..=6 => "III",
        7..=9 => "IV",
        10..=12 => "V",
        _ => "VI",
    };

    Ok(Json(serde_json::json!({
        "total_points": points,
        "criminal_history_category": category,
    })))
}

/// POST /api/sentencing/{id}/calculate-offense-level
/// Calculate the total offense level from a base level and adjustments.
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/calculate-offense-level",
    request_body = OffenseLevelRequest,
    params(
        ("id" = String, Path, description = "Sentencing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Offense level calculation", body = serde_json::Value)
    ),
    tag = "sentencing"
)]
pub async fn calc_offense_level(
    State(_pool): State<Pool<Postgres>>,
    _court: CourtId,
    Path(_id): Path<String>,
    Json(body): Json<OffenseLevelRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut total = body.base_level;

    for adj in &body.adjustments {
        if let Some(val) = adj.get("value").and_then(|v| v.as_i64()) {
            total += val as i32;
        }
    }

    // Clamp to valid range
    let total = total.max(1).min(43);

    Ok(Json(serde_json::json!({
        "base_level": body.base_level,
        "adjustments_applied": body.adjustments.len(),
        "total_offense_level": total,
    })))
}

/// POST /api/sentencing/{id}/lookup-guidelines-range
/// Look up sentencing guidelines range from offense level and history category.
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/lookup-guidelines-range",
    request_body = GuidelinesRequest,
    params(
        ("id" = String, Path, description = "Sentencing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Guidelines range lookup", body = GuidelinesResult),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "sentencing"
)]
pub async fn lookup_guidelines(
    State(_pool): State<Pool<Postgres>>,
    _court: CourtId,
    Path(_id): Path<String>,
    Json(body): Json<GuidelinesRequest>,
) -> Result<Json<GuidelinesResult>, AppError> {
    if !is_valid_criminal_history_category(&body.criminal_history_category) {
        return Err(AppError::bad_request(format!(
            "Invalid criminal_history_category: {}. Valid values: {}",
            body.criminal_history_category,
            CRIMINAL_HISTORY_CATEGORIES.join(", ")
        )));
    }

    let adjustments_sum: i32 = body.specific_offense_adjustments.iter().sum();
    let acceptance = body.acceptance_reduction.unwrap_or(0);
    let total_offense_level = (body.base_offense_level + adjustments_sum - acceptance).max(1).min(43);

    let category_index = match body.criminal_history_category.as_str() {
        "I" => 0, "II" => 1, "III" => 2, "IV" => 3, "V" => 4, "VI" => 5, _ => 0,
    };

    let base_low = match total_offense_level {
        1..=6 => 0,
        7..=10 => [2, 4, 6, 8, 9, 12][category_index],
        11..=15 => [8, 10, 12, 15, 18, 21][category_index],
        16..=20 => [21, 24, 27, 30, 33, 37][category_index],
        21..=25 => [37, 41, 46, 51, 57, 63][category_index],
        26..=30 => [63, 70, 78, 87, 97, 108][category_index],
        31..=35 => [108, 121, 135, 151, 168, 188][category_index],
        36..=40 => [188, 210, 235, 262, 292, 324][category_index],
        41..=43 => [360, 360, 360, 360, 360, 360][category_index],
        _ => 0,
    };
    let base_high = (base_low as f64 * 1.25).ceil() as i32;

    Ok(Json(GuidelinesResult {
        total_offense_level,
        criminal_history_category: body.criminal_history_category,
        range_low_months: base_low,
        range_high_months: base_high,
    }))
}

/// GET /api/sentencing/{id}/safety-valve-eligible
/// Check if a defendant qualifies for the safety-valve provision.
#[utoipa::path(
    get,
    path = "/api/sentencing/{id}/safety-valve-eligible",
    params(
        ("id" = String, Path, description = "Sentencing UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Safety valve eligibility", body = serde_json::Value),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "sentencing"
)]
pub async fn check_safety_valve(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let sentencing = crate::repo::sentencing::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Sentencing {} not found", id)))?;

    // Safety valve criteria (simplified per 18 USC 3553(f)):
    // 1. Criminal history category I (0 or 1 points)
    let points_ok = sentencing.criminal_history_points.unwrap_or(0) <= 1;
    // 2. No violence or firearms involved (checked by offense level <= 26 as proxy)
    let offense_ok = sentencing.total_offense_level.unwrap_or(99) <= 26;
    // 3. No death or serious injury (assumed from offense level)
    // 4. Not an organizer/leader (assumed)
    // 5. Defendant provided truthful information

    let eligible = points_ok && offense_ok;

    Ok(Json(serde_json::json!({
        "eligible": eligible,
        "criteria": {
            "criminal_history_points_ok": points_ok,
            "offense_level_ok": offense_ok,
            "criminal_history_points": sentencing.criminal_history_points,
            "total_offense_level": sentencing.total_offense_level,
        }
    })))
}
