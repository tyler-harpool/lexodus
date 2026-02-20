use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CreateMotionRequest, MotionResponse, UpdateMotionRequest,
    is_valid_motion_type, is_valid_motion_status,
    MOTION_TYPES, MOTION_STATUSES,
    is_valid_ruling_disposition, RuleMotionRequest,
    CreateJudicialOrderRequest, JudicialOrderResponse, RULING_DISPOSITIONS,
};
use crate::tenant::CourtId;

/// POST /api/motions
#[utoipa::path(
    post,
    path = "/api/motions",
    request_body = CreateMotionRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Motion created", body = MotionResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "motions"
)]
pub async fn create_motion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateMotionRequest>,
) -> Result<(StatusCode, Json<MotionResponse>), AppError> {
    if !is_valid_motion_type(&body.motion_type) {
        return Err(AppError::bad_request(format!(
            "Invalid motion_type: {}. Valid values: {}",
            body.motion_type,
            MOTION_TYPES.join(", ")
        )));
    }

    if body.filed_by.trim().is_empty() {
        return Err(AppError::bad_request("filed_by must not be empty"));
    }

    if body.description.trim().is_empty() {
        return Err(AppError::bad_request("description must not be empty"));
    }

    if let Some(ref s) = body.status {
        if !is_valid_motion_status(s) {
            return Err(AppError::bad_request(format!(
                "Invalid status: {}. Valid values: {}",
                s,
                MOTION_STATUSES.join(", ")
            )));
        }
    }

    let motion = crate::repo::motion::create(&pool, &court.0, body).await?;

    // Auto-create queue item for clerk processing (motions are higher priority)
    let _ = crate::repo::queue::create(
        &pool,
        &court.0,
        "motion",
        2,
        &format!("{} - {}", motion.motion_type, motion.description),
        Some("Motion requires clerk review"),
        "motion",
        motion.id,
        Some(motion.case_id),
        None,
        None,
        None,
        shared_types::pipeline_steps("motion").first().copied().unwrap_or("review"),
    )
    .await;

    Ok((StatusCode::CREATED, Json(MotionResponse::from(motion))))
}

/// GET /api/motions/{id}
#[utoipa::path(
    get,
    path = "/api/motions/{id}",
    params(
        ("id" = String, Path, description = "Motion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Motion found", body = MotionResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "motions"
)]
pub async fn get_motion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<MotionResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let motion = crate::repo::motion::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Motion {} not found", id)))?;

    Ok(Json(MotionResponse::from(motion)))
}

/// GET /api/motions/case/{case_id}
#[utoipa::path(
    get,
    path = "/api/motions/case/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Motions for case", body = Vec<MotionResponse>)
    ),
    tag = "motions"
)]
pub async fn list_motions_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<MotionResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let motions = crate::repo::motion::list_by_case(&pool, &court.0, uuid).await?;
    let responses: Vec<MotionResponse> = motions.into_iter().map(MotionResponse::from).collect();

    Ok(Json(responses))
}

/// PUT /api/motions/{id}
#[utoipa::path(
    put,
    path = "/api/motions/{id}",
    request_body = UpdateMotionRequest,
    params(
        ("id" = String, Path, description = "Motion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Motion updated", body = MotionResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "motions"
)]
pub async fn update_motion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<UpdateMotionRequest>,
) -> Result<Json<MotionResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref mt) = body.motion_type {
        if !is_valid_motion_type(mt) {
            return Err(AppError::bad_request(format!(
                "Invalid motion_type: {}. Valid values: {}",
                mt,
                MOTION_TYPES.join(", ")
            )));
        }
    }

    if let Some(ref s) = body.status {
        if !is_valid_motion_status(s) {
            return Err(AppError::bad_request(format!(
                "Invalid status: {}. Valid values: {}",
                s,
                MOTION_STATUSES.join(", ")
            )));
        }
    }

    if let Some(ref fb) = body.filed_by {
        if fb.trim().is_empty() {
            return Err(AppError::bad_request("filed_by must not be empty"));
        }
    }

    if let Some(ref d) = body.description {
        if d.trim().is_empty() {
            return Err(AppError::bad_request("description must not be empty"));
        }
    }

    let motion = crate::repo::motion::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Motion {} not found", id)))?;

    Ok(Json(MotionResponse::from(motion)))
}

/// DELETE /api/motions/{id}
#[utoipa::path(
    delete,
    path = "/api/motions/{id}",
    params(
        ("id" = String, Path, description = "Motion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Motion deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "motions"
)]
pub async fn delete_motion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::motion::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Motion {} not found", id)))
    }
}

// ── Rule on a motion (inner logic) ────────────────────────────────

/// Core logic for ruling on a motion. Shared by the REST handler and server function.
pub async fn rule_motion_inner(
    pool: &Pool<Postgres>,
    court_id: &str,
    motion_id: &str,
    body: RuleMotionRequest,
) -> Result<JudicialOrderResponse, AppError> {
    let motion_uuid = Uuid::parse_str(motion_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !is_valid_ruling_disposition(&body.disposition) {
        return Err(AppError::bad_request(format!(
            "Invalid disposition: {}. Valid values: {}",
            body.disposition,
            RULING_DISPOSITIONS.join(", ")
        )));
    }

    let judge_uuid = Uuid::parse_str(&body.judge_id)
        .map_err(|_| AppError::bad_request("Invalid judge_id UUID format"))?;

    // 1. Fetch the motion
    let motion = crate::repo::motion::find_by_id(pool, court_id, motion_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Motion {} not found", motion_id)))?;

    if motion.status != "Pending" {
        return Err(AppError::bad_request(format!(
            "Motion is not pending (current status: {})", motion.status
        )));
    }

    // 2. Map disposition to motion status
    let new_status = match body.disposition.as_str() {
        "Granted" | "Granted in Part" => "Granted",
        "Denied" => "Denied",
        "Moot" => "Moot",
        "Taken Under Advisement" => "Deferred",
        "Set for Hearing" => "Pending",
        _ => "Pending",
    };

    // 3. Update motion with ruling
    let update_req = UpdateMotionRequest {
        motion_type: None,
        filed_by: None,
        description: None,
        status: Some(new_status.to_string()),
        ruling_date: Some(chrono::Utc::now()),
        ruling_text: body.ruling_text.clone(),
    };
    crate::repo::motion::update(pool, court_id, motion_uuid, update_req).await?;

    // 4. Determine order type from motion type
    let order_type = match motion.motion_type.as_str() {
        "Dismiss" => "Dismissal",
        "Suppress" | "Limine" => "Procedural",
        "Compel" | "Discovery" => "Discovery",
        _ => "Procedural",
    };

    // 5. Generate order content from ruling
    let ruling_text = body.ruling_text.clone().unwrap_or_else(|| {
        format!(
            "The Court, having considered the {} filed by {}, and for good cause shown, hereby {} the motion.",
            motion.motion_type, motion.filed_by,
            body.disposition.to_lowercase()
        )
    });

    let order_title = format!(
        "Order on {} ({})",
        motion.motion_type, body.disposition
    );

    // 6. Create judicial order
    let create_order = CreateJudicialOrderRequest {
        case_id: motion.case_id,
        judge_id: judge_uuid,
        order_type: order_type.to_string(),
        title: order_title,
        content: ruling_text,
        status: Some("Pending Signature".to_string()),
        is_sealed: Some(false),
        effective_date: None,
        expiration_date: None,
        related_motions: vec![motion_uuid],
    };

    let order = crate::repo::order::create(pool, court_id, create_order).await?;

    // 7. Create clerk queue item for the new order
    let _ = crate::repo::queue::create(
        pool,
        court_id,
        "order",
        2,
        &format!("Order on {} - pending judge signature", motion.motion_type),
        Some("Auto-generated from motion ruling"),
        "order",
        order.id,
        Some(motion.case_id),
        None,
        None,
        None,
        "route_judge",
    )
    .await;

    // 8. Fire compliance engine with MotionDenied trigger (closest match for ruling)
    let trigger = shared_types::compliance::TriggerEvent::MotionDenied;
    let context = shared_types::compliance::FilingContext {
        case_type: "criminal".to_string(),
        document_type: format!("motion_ruling_{}", body.disposition.to_lowercase().replace(' ', "_")),
        filer_role: "judge".to_string(),
        jurisdiction_id: court_id.to_string(),
        division: None,
        assigned_judge: Some(body.judge_name.clone()),
        service_method: None,
        metadata: serde_json::json!({
            "case_id": motion.case_id.to_string(),
            "motion_id": motion.id.to_string(),
            "motion_type": motion.motion_type,
            "disposition": body.disposition,
            "order_id": order.id.to_string(),
        }),
    };

    let all_rules = crate::repo::rule::list_active(pool, court_id, None)
        .await
        .unwrap_or_default();
    let selected = crate::compliance::engine::select_rules(court_id, &trigger, &all_rules);
    let sorted = crate::compliance::engine::resolve_priority(selected);
    let report = crate::compliance::engine::evaluate(&context, &sorted);

    // Apply deadlines from engine
    for deadline in &report.deadlines {
        let due_at = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            deadline.due_date.and_hms_opt(17, 0, 0).unwrap(),
            chrono::Utc,
        );
        let _ = crate::repo::deadline::create(
            pool,
            court_id,
            shared_types::CreateDeadlineRequest {
                title: deadline.description.clone(),
                case_id: Some(motion.case_id),
                rule_code: Some(deadline.rule_citation.clone()),
                due_at,
                notes: Some(deadline.computation_notes.clone()),
            },
        )
        .await;
    }

    // Advance case status if dispositive
    for sc in &report.status_changes {
        tracing::info!(
            case_id = %motion.case_id,
            new_status = %sc.new_status,
            rule = %sc.rule_name,
            "Compliance engine advancing case status from motion ruling"
        );
        let _ = crate::repo::case::update_status(pool, court_id, motion.case_id, &sc.new_status).await;
    }

    // Log case event
    let report_json = serde_json::to_value(&report).ok();
    let _ = crate::repo::case_event::insert(
        pool,
        court_id,
        motion.case_id,
        "criminal",
        "motion_ruled",
        None,
        &serde_json::json!({
            "motion_id": motion.id.to_string(),
            "motion_type": motion.motion_type,
            "disposition": body.disposition,
            "order_id": order.id.to_string(),
        }),
        report_json.as_ref(),
    )
    .await;

    Ok(JudicialOrderResponse::from(order))
}

/// POST /api/motions/{id}/rule
/// Judge rules on a pending motion: updates status, creates order, fires compliance engine.
#[utoipa::path(
    post,
    path = "/api/motions/{id}/rule",
    request_body = RuleMotionRequest,
    params(
        ("id" = String, Path, description = "Motion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Motion ruled, order created", body = JudicialOrderResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "motions"
)]
pub async fn rule_motion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<RuleMotionRequest>,
) -> Result<Json<JudicialOrderResponse>, AppError> {
    let result = rule_motion_inner(&pool, &court.0, &id, body).await?;
    Ok(Json(result))
}
