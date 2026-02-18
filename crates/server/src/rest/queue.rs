use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AdvanceQueueRequest, AppError, CreateQueueItemRequest, QueueItemResponse,
    QueueSearchParams, QueueSearchResponse, QueueStats, RejectQueueRequest,
};
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// GET /api/queue
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/api/queue",
    params(QueueSearchParams),
    responses(
        (status = 200, description = "Queue items", body = QueueSearchResponse)
    ),
    tag = "queue"
)]
pub async fn list_queue(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<QueueSearchParams>,
) -> Result<Json<QueueSearchResponse>, AppError> {
    let case_uuid = params.case_id.as_deref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| AppError::bad_request("invalid case_id UUID"))?;

    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);

    let (items, total) = crate::repo::queue::search(
        &pool,
        &court.0,
        params.status.as_deref(),
        params.queue_type.as_deref(),
        params.priority,
        params.assigned_to,
        case_uuid,
        offset,
        limit,
    )
    .await?;

    Ok(Json(QueueSearchResponse {
        items: items.into_iter().map(QueueItemResponse::from).collect(),
        total,
    }))
}

// ---------------------------------------------------------------------------
// GET /api/queue/stats
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct QueueStatsParams {
    pub user_id: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/queue/stats",
    params(QueueStatsParams),
    responses(
        (status = 200, description = "Queue statistics", body = QueueStats)
    ),
    tag = "queue"
)]
pub async fn queue_stats(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<QueueStatsParams>,
) -> Result<Json<QueueStats>, AppError> {
    let stats = crate::repo::queue::stats(&pool, &court.0, params.user_id).await?;
    Ok(Json(stats))
}

// ---------------------------------------------------------------------------
// GET /api/queue/{id}
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/api/queue/{id}",
    params(
        ("id" = String, Path, description = "Queue item UUID")
    ),
    responses(
        (status = 200, description = "Queue item", body = QueueItemResponse),
        (status = 404, description = "Not found")
    ),
    tag = "queue"
)]
pub async fn get_queue_item(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<QueueItemResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let item = crate::repo::queue::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found("Queue item not found"))?;
    Ok(Json(QueueItemResponse::from(item)))
}

// ---------------------------------------------------------------------------
// POST /api/queue
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/queue",
    request_body = CreateQueueItemRequest,
    responses(
        (status = 201, description = "Queue item created", body = QueueItemResponse)
    ),
    tag = "queue"
)]
pub async fn create_queue_item(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateQueueItemRequest>,
) -> Result<(StatusCode, Json<QueueItemResponse>), AppError> {
    if body.title.trim().is_empty() {
        return Err(AppError::bad_request("title must not be empty"));
    }
    if !shared_types::is_valid_queue_type(&body.queue_type) {
        return Err(AppError::bad_request(format!("invalid queue_type: {}", body.queue_type)));
    }
    if !shared_types::QUEUE_SOURCE_TYPES.contains(&body.source_type.as_str()) {
        return Err(AppError::bad_request(format!("invalid source_type: {}", body.source_type)));
    }

    let source_uuid = Uuid::parse_str(&body.source_id)
        .map_err(|_| AppError::bad_request("invalid source_id UUID"))?;
    let case_uuid = body.case_id.as_deref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| AppError::bad_request("invalid case_id UUID"))?;

    let steps = shared_types::pipeline_steps(&body.queue_type);
    let first_step = steps.first().copied().unwrap_or("review");

    let item = crate::repo::queue::create(
        &pool,
        &court.0,
        &body.queue_type,
        body.priority.unwrap_or(3),
        &body.title,
        body.description.as_deref(),
        &body.source_type,
        source_uuid,
        case_uuid,
        body.case_number.as_deref(),
        body.submitted_by,
        body.metadata.clone(),
        first_step,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(QueueItemResponse::from(item))))
}

// ---------------------------------------------------------------------------
// POST /api/queue/{id}/claim
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ClaimRequest {
    pub user_id: i64,
}

#[utoipa::path(
    post,
    path = "/api/queue/{id}/claim",
    request_body = ClaimRequest,
    responses(
        (status = 200, description = "Claimed", body = QueueItemResponse),
        (status = 409, description = "Already claimed")
    ),
    tag = "queue"
)]
pub async fn claim_queue_item(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<ClaimRequest>,
) -> Result<Json<QueueItemResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let item = crate::repo::queue::claim(&pool, &court.0, uuid, body.user_id)
        .await?
        .ok_or_else(|| AppError::conflict("Queue item is not available for claiming"))?;
    Ok(Json(QueueItemResponse::from(item)))
}

// ---------------------------------------------------------------------------
// POST /api/queue/{id}/release
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/queue/{id}/release",
    request_body = ClaimRequest,
    responses(
        (status = 200, description = "Released", body = QueueItemResponse),
        (status = 404, description = "Not found or not assigned to user")
    ),
    tag = "queue"
)]
pub async fn release_queue_item(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<ClaimRequest>,
) -> Result<Json<QueueItemResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let item = crate::repo::queue::release(&pool, &court.0, uuid, body.user_id)
        .await?
        .ok_or_else(|| AppError::not_found("Queue item not found or not assigned to this user"))?;
    Ok(Json(QueueItemResponse::from(item)))
}

// ---------------------------------------------------------------------------
// POST /api/queue/{id}/advance
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/queue/{id}/advance",
    request_body = AdvanceQueueRequest,
    responses(
        (status = 200, description = "Advanced", body = QueueItemResponse),
        (status = 404, description = "Not found")
    ),
    tag = "queue"
)]
pub async fn advance_queue_item(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(_body): Json<AdvanceQueueRequest>,
) -> Result<Json<QueueItemResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let current = crate::repo::queue::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found("Queue item not found"))?;

    match shared_types::next_step(&current.queue_type, &current.current_step) {
        Some(next) => {
            let (status, completed_at) = if shared_types::next_step(&current.queue_type, next).is_none() {
                ("completed", Some(Utc::now()))
            } else {
                ("processing", None)
            };
            let item = crate::repo::queue::advance(&pool, &court.0, uuid, next, status, completed_at)
                .await?
                .ok_or_else(|| AppError::not_found("Queue item not found"))?;
            Ok(Json(QueueItemResponse::from(item)))
        }
        None => {
            let item = crate::repo::queue::advance(&pool, &court.0, uuid, "completed", "completed", Some(Utc::now()))
                .await?
                .ok_or_else(|| AppError::not_found("Queue item not found"))?;
            Ok(Json(QueueItemResponse::from(item)))
        }
    }
}

// ---------------------------------------------------------------------------
// POST /api/queue/{id}/reject
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/queue/{id}/reject",
    request_body = RejectQueueRequest,
    responses(
        (status = 200, description = "Rejected", body = QueueItemResponse),
        (status = 404, description = "Not found")
    ),
    tag = "queue"
)]
pub async fn reject_queue_item(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<RejectQueueRequest>,
) -> Result<Json<QueueItemResponse>, AppError> {
    if body.reason.trim().is_empty() {
        return Err(AppError::bad_request("reason must not be empty"));
    }
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;
    let item = crate::repo::queue::reject(&pool, &court.0, uuid, &body.reason)
        .await?
        .ok_or_else(|| AppError::not_found("Queue item not found"))?;
    Ok(Json(QueueItemResponse::from(item)))
}
