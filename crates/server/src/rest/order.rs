use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError,
    CreateJudicialOrderRequest, JudicialOrderResponse, UpdateJudicialOrderRequest,
    OrderListParams,
    is_valid_order_type, is_valid_order_status, ORDER_TYPES, ORDER_STATUSES,
    CreateOrderTemplateRequest, OrderTemplateResponse, UpdateOrderTemplateRequest,
    SignOrderRequest, IssueOrderRequest, ServeOrderRequest, OrderStatistics,
    CreateFromTemplateRequest, GenerateContentRequest,
};
use crate::tenant::CourtId;

// ── Order handlers ──────────────────────────────────────────────

/// POST /api/orders
#[utoipa::path(
    post,
    path = "/api/orders",
    request_body = CreateJudicialOrderRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Order created", body = JudicialOrderResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "orders"
)]
pub async fn create_order(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateJudicialOrderRequest>,
) -> Result<(StatusCode, Json<JudicialOrderResponse>), AppError> {
    if !is_valid_order_type(&body.order_type) {
        return Err(AppError::bad_request(format!(
            "Invalid order_type: {}. Valid values: {}",
            body.order_type,
            ORDER_TYPES.join(", ")
        )));
    }

    if let Some(ref s) = body.status {
        if !is_valid_order_status(s) {
            return Err(AppError::bad_request(format!(
                "Invalid status: {}. Valid values: {}",
                s,
                ORDER_STATUSES.join(", ")
            )));
        }
    }

    let order = crate::repo::order::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(JudicialOrderResponse::from(order))))
}

/// GET /api/orders
#[utoipa::path(
    get,
    path = "/api/orders",
    params(
        OrderListParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "List of orders", body = Vec<JudicialOrderResponse>)
    ),
    tag = "orders"
)]
pub async fn list_orders(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<OrderListParams>,
) -> Result<Json<Vec<JudicialOrderResponse>>, AppError> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let case_uuid = params.case_id
        .as_deref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| AppError::bad_request("Invalid case_id UUID format"))?;
    let judge_uuid = params.judge_id
        .as_deref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| AppError::bad_request("Invalid judge_id UUID format"))?;

    let rows = sqlx::query_as!(
        shared_types::JudicialOrder,
        r#"
        SELECT id, court_id, case_id, judge_id, order_type, title, content,
               status, is_sealed, signer_name, signed_at, signature_hash,
               issued_at, effective_date, expiration_date, related_motions,
               created_at, updated_at
        FROM judicial_orders
        WHERE court_id = $1
          AND ($2::UUID IS NULL OR case_id = $2)
          AND ($3::UUID IS NULL OR judge_id = $3)
          AND ($4::TEXT IS NULL OR status = $4)
          AND ($5::BOOL IS NULL OR is_sealed = $5)
        ORDER BY created_at DESC
        LIMIT $6 OFFSET $7
        "#,
        &court.0,
        case_uuid,
        judge_uuid,
        params.status.as_deref(),
        params.is_sealed,
        limit,
        offset,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<JudicialOrderResponse> = rows.into_iter().map(JudicialOrderResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/orders/{order_id}
#[utoipa::path(
    get,
    path = "/api/orders/{order_id}",
    params(
        ("order_id" = String, Path, description = "Order UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Order found", body = JudicialOrderResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "orders"
)]
pub async fn get_order(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(order_id): Path<String>,
) -> Result<Json<JudicialOrderResponse>, AppError> {
    let uuid = Uuid::parse_str(&order_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let order = crate::repo::order::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Order {} not found", order_id)))?;

    Ok(Json(JudicialOrderResponse::from(order)))
}

/// PATCH /api/orders/{order_id}
#[utoipa::path(
    patch,
    path = "/api/orders/{order_id}",
    request_body = UpdateJudicialOrderRequest,
    params(
        ("order_id" = String, Path, description = "Order UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Order updated", body = JudicialOrderResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "orders"
)]
pub async fn update_order(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(order_id): Path<String>,
    Json(body): Json<UpdateJudicialOrderRequest>,
) -> Result<Json<JudicialOrderResponse>, AppError> {
    let uuid = Uuid::parse_str(&order_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref s) = body.status {
        if !is_valid_order_status(s) {
            return Err(AppError::bad_request(format!(
                "Invalid status: {}. Valid values: {}",
                s,
                ORDER_STATUSES.join(", ")
            )));
        }
    }

    let order = crate::repo::order::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Order {} not found", order_id)))?;

    Ok(Json(JudicialOrderResponse::from(order)))
}

/// DELETE /api/orders/{order_id}
#[utoipa::path(
    delete,
    path = "/api/orders/{order_id}",
    params(
        ("order_id" = String, Path, description = "Order UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Order deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "orders"
)]
pub async fn delete_order(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(order_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&order_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::order::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Order {} not found", order_id)))
    }
}

/// GET /api/cases/{case_id}/orders
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/orders",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Orders for case", body = Vec<JudicialOrderResponse>)
    ),
    tag = "orders"
)]
pub async fn list_orders_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<JudicialOrderResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let orders = crate::repo::order::list_by_case(&pool, &court.0, uuid).await?;
    let responses: Vec<JudicialOrderResponse> = orders.into_iter().map(JudicialOrderResponse::from).collect();

    Ok(Json(responses))
}

/// GET /api/judges/{judge_id}/orders
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/orders",
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Orders for judge", body = Vec<JudicialOrderResponse>)
    ),
    tag = "orders"
)]
pub async fn list_orders_by_judge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(judge_id): Path<String>,
) -> Result<Json<Vec<JudicialOrderResponse>>, AppError> {
    let uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let orders = crate::repo::order::list_by_judge(&pool, &court.0, uuid).await?;
    let responses: Vec<JudicialOrderResponse> = orders.into_iter().map(JudicialOrderResponse::from).collect();

    Ok(Json(responses))
}

// ── Order Template handlers ──────────────────────────────────────

/// POST /api/templates/orders
#[utoipa::path(
    post,
    path = "/api/templates/orders",
    request_body = CreateOrderTemplateRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Order template created", body = OrderTemplateResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "order-templates"
)]
pub async fn create_template(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateOrderTemplateRequest>,
) -> Result<(StatusCode, Json<OrderTemplateResponse>), AppError> {
    if !is_valid_order_type(&body.order_type) {
        return Err(AppError::bad_request(format!(
            "Invalid order_type: {}. Valid values: {}",
            body.order_type,
            ORDER_TYPES.join(", ")
        )));
    }

    let template = crate::repo::order_template::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(OrderTemplateResponse::from(template))))
}

/// GET /api/templates/orders
#[utoipa::path(
    get,
    path = "/api/templates/orders",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "All order templates", body = Vec<OrderTemplateResponse>)
    ),
    tag = "order-templates"
)]
pub async fn list_templates(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<OrderTemplateResponse>>, AppError> {
    let templates = crate::repo::order_template::list_all(&pool, &court.0).await?;
    let responses: Vec<OrderTemplateResponse> = templates.into_iter().map(OrderTemplateResponse::from).collect();

    Ok(Json(responses))
}

/// GET /api/templates/orders/active
#[utoipa::path(
    get,
    path = "/api/templates/orders/active",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Active order templates", body = Vec<OrderTemplateResponse>)
    ),
    tag = "order-templates"
)]
pub async fn list_active_templates(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<OrderTemplateResponse>>, AppError> {
    let templates = crate::repo::order_template::list_active(&pool, &court.0).await?;
    let responses: Vec<OrderTemplateResponse> = templates.into_iter().map(OrderTemplateResponse::from).collect();

    Ok(Json(responses))
}

/// GET /api/templates/orders/{template_id}
#[utoipa::path(
    get,
    path = "/api/templates/orders/{template_id}",
    params(
        ("template_id" = String, Path, description = "Order template UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Order template found", body = OrderTemplateResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "order-templates"
)]
pub async fn get_template(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(template_id): Path<String>,
) -> Result<Json<OrderTemplateResponse>, AppError> {
    let uuid = Uuid::parse_str(&template_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let template = crate::repo::order_template::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Order template {} not found", template_id)))?;

    Ok(Json(OrderTemplateResponse::from(template)))
}

/// PUT /api/templates/orders/{template_id}
#[utoipa::path(
    put,
    path = "/api/templates/orders/{template_id}",
    request_body = UpdateOrderTemplateRequest,
    params(
        ("template_id" = String, Path, description = "Order template UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Order template updated", body = OrderTemplateResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "order-templates"
)]
pub async fn update_template(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(template_id): Path<String>,
    Json(body): Json<UpdateOrderTemplateRequest>,
) -> Result<Json<OrderTemplateResponse>, AppError> {
    let uuid = Uuid::parse_str(&template_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if let Some(ref ot) = body.order_type {
        if !is_valid_order_type(ot) {
            return Err(AppError::bad_request(format!(
                "Invalid order_type: {}. Valid values: {}",
                ot,
                ORDER_TYPES.join(", ")
            )));
        }
    }

    let template = crate::repo::order_template::update(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Order template {} not found", template_id)))?;

    Ok(Json(OrderTemplateResponse::from(template)))
}

/// DELETE /api/templates/orders/{template_id}
#[utoipa::path(
    delete,
    path = "/api/templates/orders/{template_id}",
    params(
        ("template_id" = String, Path, description = "Order template UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Order template deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "order-templates"
)]
pub async fn delete_template(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(template_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&template_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::order_template::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Order template {} not found", template_id)))
    }
}

// ── Extended order handlers ─────────────────────────────────────────

/// POST /api/orders/{order_id}/sign
/// Sign an order, updating status to "Signed".
#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/sign",
    request_body = SignOrderRequest,
    params(
        ("order_id" = String, Path, description = "Order UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Order signed", body = JudicialOrderResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "orders"
)]
pub async fn sign_order(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(order_id): Path<String>,
    Json(body): Json<SignOrderRequest>,
) -> Result<Json<JudicialOrderResponse>, AppError> {
    let uuid = Uuid::parse_str(&order_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if body.signed_by.trim().is_empty() {
        return Err(AppError::bad_request("signed_by must not be empty"));
    }

    let order = sqlx::query_as!(
        shared_types::JudicialOrder,
        r#"
        UPDATE judicial_orders SET
            status = 'Signed',
            signer_name = $3,
            signed_at = NOW(),
            signature_hash = md5($3 || id::text),
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, judge_id, order_type, title, content,
                  status, is_sealed, signer_name, signed_at, signature_hash,
                  issued_at, effective_date, expiration_date, related_motions,
                  created_at, updated_at
        "#,
        uuid,
        &court.0,
        body.signed_by,
    )
    .fetch_optional(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found(format!("Order {} not found", order_id)))?;

    Ok(Json(JudicialOrderResponse::from(order)))
}

/// POST /api/orders/{order_id}/issue
/// Issue an order, updating status to "Filed".
#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/issue",
    request_body = IssueOrderRequest,
    params(
        ("order_id" = String, Path, description = "Order UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Order issued", body = JudicialOrderResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "orders"
)]
pub async fn issue_order(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(order_id): Path<String>,
    Json(body): Json<IssueOrderRequest>,
) -> Result<Json<JudicialOrderResponse>, AppError> {
    let uuid = Uuid::parse_str(&order_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if body.issued_by.trim().is_empty() {
        return Err(AppError::bad_request("issued_by must not be empty"));
    }

    let order = sqlx::query_as!(
        shared_types::JudicialOrder,
        r#"
        UPDATE judicial_orders SET
            status = 'Filed',
            issued_at = NOW(),
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, judge_id, order_type, title, content,
                  status, is_sealed, signer_name, signed_at, signature_hash,
                  issued_at, effective_date, expiration_date, related_motions,
                  created_at, updated_at
        "#,
        uuid,
        &court.0,
    )
    .fetch_optional(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found(format!("Order {} not found", order_id)))?;

    Ok(Json(JudicialOrderResponse::from(order)))
}

/// POST /api/orders/{order_id}/service
/// Record that an order was served on parties.
#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/service",
    request_body = ServeOrderRequest,
    params(
        ("order_id" = String, Path, description = "Order UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Service recorded", body = JudicialOrderResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "orders"
)]
pub async fn serve_order(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(order_id): Path<String>,
    Json(body): Json<ServeOrderRequest>,
) -> Result<Json<JudicialOrderResponse>, AppError> {
    let uuid = Uuid::parse_str(&order_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if body.served_to.is_empty() {
        return Err(AppError::bad_request("served_to must not be empty"));
    }

    // Verify order exists and return it (service details are recorded externally)
    let order = crate::repo::order::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Order {} not found", order_id)))?;

    Ok(Json(JudicialOrderResponse::from(order)))
}

/// GET /api/orders/expired
/// List expired orders (expiration_date < now).
#[utoipa::path(
    get,
    path = "/api/orders/expired",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Expired orders", body = Vec<JudicialOrderResponse>)),
    tag = "orders"
)]
pub async fn check_expired(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<JudicialOrderResponse>>, AppError> {
    let orders = sqlx::query_as!(
        shared_types::JudicialOrder,
        r#"
        SELECT id, court_id, case_id, judge_id, order_type, title, content,
               status, is_sealed, signer_name, signed_at, signature_hash,
               issued_at, effective_date, expiration_date, related_motions,
               created_at, updated_at
        FROM judicial_orders
        WHERE court_id = $1 AND expiration_date < NOW() AND status NOT IN ('Vacated', 'Superseded')
        ORDER BY expiration_date DESC
        "#,
        &court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<JudicialOrderResponse> = orders
        .into_iter()
        .map(JudicialOrderResponse::from)
        .collect();
    Ok(Json(responses))
}

/// GET /api/orders/requires-attention
/// List orders in "Draft" or "Pending Signature" status.
#[utoipa::path(
    get,
    path = "/api/orders/requires-attention",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Orders requiring attention", body = Vec<JudicialOrderResponse>)),
    tag = "orders"
)]
pub async fn check_requires_attention(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<JudicialOrderResponse>>, AppError> {
    let orders = sqlx::query_as!(
        shared_types::JudicialOrder,
        r#"
        SELECT id, court_id, case_id, judge_id, order_type, title, content,
               status, is_sealed, signer_name, signed_at, signature_hash,
               issued_at, effective_date, expiration_date, related_motions,
               created_at, updated_at
        FROM judicial_orders
        WHERE court_id = $1 AND status IN ('Draft', 'Pending Signature')
        ORDER BY created_at ASC
        "#,
        &court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<JudicialOrderResponse> = orders
        .into_iter()
        .map(JudicialOrderResponse::from)
        .collect();
    Ok(Json(responses))
}

/// GET /api/orders/pending-signatures
/// List orders awaiting signatures (status = "Pending Signature").
#[utoipa::path(
    get,
    path = "/api/orders/pending-signatures",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Orders pending signature", body = Vec<JudicialOrderResponse>)),
    tag = "orders"
)]
pub async fn list_pending_signatures(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<JudicialOrderResponse>>, AppError> {
    let orders = sqlx::query_as!(
        shared_types::JudicialOrder,
        r#"
        SELECT id, court_id, case_id, judge_id, order_type, title, content,
               status, is_sealed, signer_name, signed_at, signature_hash,
               issued_at, effective_date, expiration_date, related_motions,
               created_at, updated_at
        FROM judicial_orders
        WHERE court_id = $1 AND status = 'Pending Signature'
        ORDER BY created_at ASC
        "#,
        &court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<JudicialOrderResponse> = orders
        .into_iter()
        .map(JudicialOrderResponse::from)
        .collect();
    Ok(Json(responses))
}

/// Query parameters for expiring-soon search.
#[derive(Debug, serde::Deserialize)]
pub struct ExpiringQuery {
    pub days: Option<i64>,
}

/// GET /api/orders/expiring
/// List orders expiring within N days (default 30).
#[utoipa::path(
    get,
    path = "/api/orders/expiring",
    params(
        ("days" = Option<i64>, Query, description = "Number of days (default 30)"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses((status = 200, description = "Expiring orders", body = Vec<JudicialOrderResponse>)),
    tag = "orders"
)]
pub async fn list_expiring(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<ExpiringQuery>,
) -> Result<Json<Vec<JudicialOrderResponse>>, AppError> {
    let days = params.days.unwrap_or(30);

    let orders = sqlx::query_as!(
        shared_types::JudicialOrder,
        r#"
        SELECT id, court_id, case_id, judge_id, order_type, title, content,
               status, is_sealed, signer_name, signed_at, signature_hash,
               issued_at, effective_date, expiration_date, related_motions,
               created_at, updated_at
        FROM judicial_orders
        WHERE court_id = $1
          AND expiration_date IS NOT NULL
          AND expiration_date > NOW()
          AND expiration_date <= NOW() + ($2::INT || ' days')::INTERVAL
          AND status NOT IN ('Vacated', 'Superseded')
        ORDER BY expiration_date ASC
        "#,
        &court.0,
        days as i32,
    )
    .fetch_all(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let responses: Vec<JudicialOrderResponse> = orders
        .into_iter()
        .map(JudicialOrderResponse::from)
        .collect();
    Ok(Json(responses))
}

/// GET /api/orders/statistics
/// Get aggregate order statistics for a court.
#[utoipa::path(
    get,
    path = "/api/orders/statistics",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses((status = 200, description = "Order statistics", body = OrderStatistics)),
    tag = "orders"
)]
pub async fn order_statistics(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<OrderStatistics>, AppError> {
    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM judicial_orders WHERE court_id = $1"#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let by_type: serde_json::Value = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(json_object_agg(order_type, cnt), '{}')::TEXT as "json!"
        FROM (SELECT order_type, COUNT(*) as cnt FROM judicial_orders WHERE court_id = $1 GROUP BY order_type) sub
        "#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map(|s| serde_json::from_str(&s).unwrap_or(serde_json::Value::Object(Default::default())))
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let by_status: serde_json::Value = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(json_object_agg(status, cnt), '{}')::TEXT as "json!"
        FROM (SELECT status, COUNT(*) as cnt FROM judicial_orders WHERE court_id = $1 GROUP BY status) sub
        "#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map(|s| serde_json::from_str(&s).unwrap_or(serde_json::Value::Object(Default::default())))
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    let avg_days_to_sign: Option<f64> = sqlx::query_scalar!(
        r#"
        SELECT AVG(EXTRACT(EPOCH FROM (signed_at - created_at)) / 86400.0)::FLOAT8 as "avg: f64"
        FROM judicial_orders
        WHERE court_id = $1 AND signed_at IS NOT NULL
        "#,
        &court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    Ok(Json(OrderStatistics {
        total,
        by_type,
        by_status,
        avg_days_to_sign,
    }))
}

/// POST /api/orders/from-template
/// Create a new order from a template by substituting variables.
#[utoipa::path(
    post,
    path = "/api/orders/from-template",
    request_body = CreateFromTemplateRequest,
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    responses(
        (status = 201, description = "Order created from template", body = JudicialOrderResponse),
        (status = 404, description = "Template not found", body = AppError)
    ),
    tag = "orders"
)]
pub async fn create_from_template(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateFromTemplateRequest>,
) -> Result<(StatusCode, Json<JudicialOrderResponse>), AppError> {
    let template = crate::repo::order_template::find_by_id(&pool, &court.0, body.template_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Template {} not found", body.template_id)))?;

    // Substitute variables in the template content
    let mut content = template.content_template.clone();
    if let serde_json::Value::Object(vars) = &body.variables {
        for (key, value) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            content = content.replace(&placeholder, &replacement);
        }
    }

    let create_req = CreateJudicialOrderRequest {
        case_id: body.case_id,
        judge_id: Uuid::nil(), // Will be set by the caller or defaulted
        order_type: template.order_type,
        title: template.name,
        content,
        status: Some("Draft".to_string()),
        is_sealed: Some(false),
        effective_date: None,
        expiration_date: None,
        related_motions: vec![],
    };

    let order = crate::repo::order::create(&pool, &court.0, create_req).await?;
    Ok((StatusCode::CREATED, Json(JudicialOrderResponse::from(order))))
}

/// POST /api/templates/orders/{template_id}/generate
/// Generate content from a template by substituting variables.
#[utoipa::path(
    post,
    path = "/api/templates/orders/{template_id}/generate",
    request_body = GenerateContentRequest,
    params(
        ("template_id" = String, Path, description = "Template UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Generated content", body = String),
        (status = 404, description = "Template not found", body = AppError)
    ),
    tag = "order-templates"
)]
pub async fn generate_content(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(template_id): Path<String>,
    Json(body): Json<GenerateContentRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uuid = Uuid::parse_str(&template_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let template = crate::repo::order_template::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Template {} not found", template_id)))?;

    let mut content = template.content_template.clone();
    if let serde_json::Value::Object(vars) = &body.variables {
        for (key, value) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            content = content.replace(&placeholder, &replacement);
        }
    }

    Ok(Json(serde_json::json!({ "content": content })))
}
