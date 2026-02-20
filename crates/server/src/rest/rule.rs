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
        body.triggers.as_ref(),
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
        body.triggers.as_ref(),
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
    use crate::compliance::engine;
    use shared_types::compliance::{FilingContext, TriggerEvent};

    // Build FilingContext from request body
    let context_obj = match &body.context {
        serde_json::Value::Object(m) => m.clone(),
        _ => serde_json::Map::new(),
    };

    let filing_context = FilingContext {
        case_type: context_obj
            .get("case_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        document_type: context_obj
            .get("document_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        filer_role: context_obj
            .get("filer_role")
            .and_then(|v| v.as_str())
            .unwrap_or("attorney")
            .to_string(),
        jurisdiction_id: court.0.clone(),
        division: context_obj
            .get("division")
            .and_then(|v| v.as_str())
            .map(String::from),
        assigned_judge: context_obj
            .get("assigned_judge")
            .and_then(|v| v.as_str())
            .map(String::from),
        service_method: None,
        metadata: body.context.clone(),
    };

    // Parse trigger event from context (default to ManualEvaluation)
    let trigger = context_obj
        .get("trigger")
        .and_then(|v| v.as_str())
        .and_then(TriggerEvent::from_str_opt)
        .unwrap_or(TriggerEvent::ManualEvaluation);

    // Load all active rules for the court
    let all_rules = crate::repo::rule::list_active(&pool, &court.0, body.category.as_deref()).await?;

    // Stage 1: Select applicable rules by jurisdiction, trigger, and in-effect status
    let selected = engine::select_rules(&court.0, &trigger, &all_rules);

    // Stage 2: Resolve priority ordering (highest weight first)
    let prioritized = engine::resolve_priority(selected);

    // Stage 3-4: Evaluate conditions and process actions into compliance report
    let report = engine::evaluate(&filing_context, &prioritized);

    // Build backward-compatible response from the compliance report
    let matched_rules: Vec<RuleResponse> = prioritized
        .into_iter()
        .filter(|r| {
            report
                .results
                .iter()
                .any(|res| res.rule_id == r.id && res.matched)
        })
        .map(RuleResponse::from)
        .collect();

    let actions: Vec<serde_json::Value> = report
        .results
        .iter()
        .filter(|r| r.matched)
        .map(|r| {
            serde_json::json!({
                "rule_id": r.rule_id.to_string(),
                "action": r.action_taken,
                "message": r.message,
            })
        })
        .collect();

    Ok(Json(EvaluateRulesResponse {
        matched_rules,
        actions,
    }))
}
