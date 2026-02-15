use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::DateTime;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CreateRuleRequest, EvaluateRulesRequest, EvaluateRulesResponse,
    RuleResponse, UpdateRuleRequest,
};
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// GET /api/rules
// ---------------------------------------------------------------------------

/// List all rules for the court.
#[utoipa::path(
    get,
    path = "/api/rules",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "All rules", body = Vec<RuleResponse>)
    ),
    tag = "rules"
)]
pub async fn list_rules(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<RuleResponse>>, AppError> {
    let rules = crate::repo::rule::list_all(&pool, &court.0).await?;
    let response: Vec<RuleResponse> = rules.into_iter().map(RuleResponse::from).collect();
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// POST /api/rules
// ---------------------------------------------------------------------------

/// Create a new rule.
#[utoipa::path(
    post,
    path = "/api/rules",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = CreateRuleRequest,
    responses(
        (status = 201, description = "Rule created", body = RuleResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "rules"
)]
pub async fn create_rule(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateRuleRequest>,
) -> Result<(StatusCode, Json<RuleResponse>), AppError> {
    let status = body.status.as_deref().unwrap_or("Active");
    let effective_date = body
        .effective_date
        .as_deref()
        .map(|s| DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|_| AppError::bad_request("Invalid effective_date format (expected RFC 3339)"))?;

    let conditions = body.conditions.unwrap_or(serde_json::json!({}));
    let actions = body.actions.unwrap_or(serde_json::json!({}));

    let rule = crate::repo::rule::create(
        &pool,
        &court.0,
        &body.name,
        &body.description,
        &body.source,
        &body.category,
        body.priority,
        status,
        body.jurisdiction.as_deref(),
        body.citation.as_deref(),
        effective_date,
        &conditions,
        &actions,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(RuleResponse::from(rule))))
}

// ---------------------------------------------------------------------------
// GET /api/rules/{id}
// ---------------------------------------------------------------------------

/// Get a rule by ID.
#[utoipa::path(
    get,
    path = "/api/rules/{id}",
    params(
        ("id" = String, Path, description = "Rule UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Rule found", body = RuleResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "rules"
)]
pub async fn get_rule(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<RuleResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let rule = crate::repo::rule::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Rule {} not found", id)))?;

    Ok(Json(RuleResponse::from(rule)))
}

// ---------------------------------------------------------------------------
// PUT /api/rules/{id}
// ---------------------------------------------------------------------------

/// Update an existing rule.
#[utoipa::path(
    put,
    path = "/api/rules/{id}",
    params(
        ("id" = String, Path, description = "Rule UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = UpdateRuleRequest,
    responses(
        (status = 200, description = "Rule updated", body = RuleResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "rules"
)]
pub async fn update_rule(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateRuleRequest>,
) -> Result<Json<RuleResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let effective_date = body
        .effective_date
        .as_deref()
        .map(|s| DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|_| AppError::bad_request("Invalid effective_date format (expected RFC 3339)"))?;

    let rule = crate::repo::rule::update(
        &pool,
        &court.0,
        uuid,
        body.name.as_deref(),
        body.description.as_deref(),
        body.source.as_deref(),
        body.category.as_deref(),
        body.priority,
        body.status.as_deref(),
        body.jurisdiction.as_deref(),
        body.citation.as_deref(),
        effective_date,
        body.conditions.as_ref(),
        body.actions.as_ref(),
    )
    .await?
    .ok_or_else(|| AppError::not_found(format!("Rule {} not found", id)))?;

    Ok(Json(RuleResponse::from(rule)))
}

// ---------------------------------------------------------------------------
// DELETE /api/rules/{id}
// ---------------------------------------------------------------------------

/// Delete a rule.
#[utoipa::path(
    delete,
    path = "/api/rules/{id}",
    params(
        ("id" = String, Path, description = "Rule UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Rule deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "rules"
)]
pub async fn delete_rule(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::rule::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Rule {} not found", id)))
    }
}

// ---------------------------------------------------------------------------
// GET /api/rules/category/{category}
// ---------------------------------------------------------------------------

/// List rules filtered by category.
#[utoipa::path(
    get,
    path = "/api/rules/category/{category}",
    params(
        ("category" = String, Path, description = "Rule category"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Rules by category", body = Vec<RuleResponse>)
    ),
    tag = "rules"
)]
pub async fn list_by_category(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(category): Path<String>,
) -> Result<Json<Vec<RuleResponse>>, AppError> {
    let rules = crate::repo::rule::list_by_category(&pool, &court.0, &category).await?;
    let response: Vec<RuleResponse> = rules.into_iter().map(RuleResponse::from).collect();
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// GET /api/rules/trigger/{trigger}
// ---------------------------------------------------------------------------

/// List rules filtered by source/trigger type.
#[utoipa::path(
    get,
    path = "/api/rules/trigger/{trigger}",
    params(
        ("trigger" = String, Path, description = "Rule source/trigger"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Rules by trigger", body = Vec<RuleResponse>)
    ),
    tag = "rules"
)]
pub async fn list_by_trigger(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(trigger): Path<String>,
) -> Result<Json<Vec<RuleResponse>>, AppError> {
    let rules = crate::repo::rule::list_by_trigger(&pool, &court.0, &trigger).await?;
    let response: Vec<RuleResponse> = rules.into_iter().map(RuleResponse::from).collect();
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// GET /api/rules/jurisdiction/{jurisdiction}
// ---------------------------------------------------------------------------

/// List rules filtered by jurisdiction.
#[utoipa::path(
    get,
    path = "/api/rules/jurisdiction/{jurisdiction}",
    params(
        ("jurisdiction" = String, Path, description = "Jurisdiction"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Rules by jurisdiction", body = Vec<RuleResponse>)
    ),
    tag = "rules"
)]
pub async fn list_by_jurisdiction(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(jurisdiction): Path<String>,
) -> Result<Json<Vec<RuleResponse>>, AppError> {
    let rules = crate::repo::rule::list_by_jurisdiction(&pool, &court.0, &jurisdiction).await?;
    let response: Vec<RuleResponse> = rules.into_iter().map(RuleResponse::from).collect();
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// POST /api/rules/evaluate
// ---------------------------------------------------------------------------

/// Evaluate active rules against a provided context.
/// Returns all matching rules (whose conditions overlap with the context)
/// and the collected actions.
#[utoipa::path(
    post,
    path = "/api/rules/evaluate",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = EvaluateRulesRequest,
    responses(
        (status = 200, description = "Evaluation result", body = EvaluateRulesResponse)
    ),
    tag = "rules"
)]
pub async fn evaluate_rules(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<EvaluateRulesRequest>,
) -> Result<Json<EvaluateRulesResponse>, AppError> {
    let rules = crate::repo::rule::list_active(&pool, &court.0, body.category.as_deref()).await?;

    let context_map = match &body.context {
        serde_json::Value::Object(m) => m.clone(),
        _ => serde_json::Map::new(),
    };

    let mut matched = Vec::new();
    let mut actions = Vec::new();

    for rule in rules {
        // A rule matches if every key in its conditions object exists in the context
        // and the values are equal, or if the conditions object is empty.
        let rule_conditions = match &rule.conditions {
            serde_json::Value::Object(m) => m.clone(),
            _ => serde_json::Map::new(),
        };

        let matches = rule_conditions.is_empty()
            || rule_conditions.iter().all(|(k, v)| {
                context_map.get(k).map_or(false, |ctx_v| ctx_v == v)
            });

        if matches {
            // Collect actions from this rule
            match &rule.actions {
                serde_json::Value::Array(arr) => {
                    actions.extend(arr.clone());
                }
                serde_json::Value::Object(_) => {
                    actions.push(rule.actions.clone());
                }
                _ => {}
            }
            matched.push(RuleResponse::from(rule));
        }
    }

    Ok(Json(EvaluateRulesResponse {
        matched_rules: matched,
        actions,
    }))
}
