use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CreateVictimRequest, SendVictimNotificationRequest,
    VictimNotificationResponse, VictimResponse,
    is_valid_victim_type, is_valid_notification_type, is_valid_notification_method,
    VICTIM_TYPES, NOTIFICATION_TYPES, NOTIFICATION_METHODS,
};
use crate::tenant::CourtId;

/// GET /api/cases/{id}/victims
#[utoipa::path(
    get,
    path = "/api/cases/{id}/victims",
    params(
        ("id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Victims for case", body = Vec<VictimResponse>)
    ),
    tag = "victims"
)]
pub async fn list_victims(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<VictimResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    let victims = crate::repo::victim::list_by_case(&pool, &court.0, uuid).await?;
    let response: Vec<VictimResponse> =
        victims.into_iter().map(VictimResponse::from).collect();

    Ok(Json(response))
}

/// POST /api/cases/{id}/victims
#[utoipa::path(
    post,
    path = "/api/cases/{id}/victims",
    request_body = CreateVictimRequest,
    params(
        ("id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Victim created", body = VictimResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "victims"
)]
pub async fn add_victim(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
    Json(mut body): Json<CreateVictimRequest>,
) -> Result<(StatusCode, Json<VictimResponse>), AppError> {
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    // Override case_id from path
    body.case_id = case_uuid;

    if body.name.trim().is_empty() {
        return Err(AppError::bad_request("name must not be empty"));
    }

    if !is_valid_victim_type(&body.victim_type) {
        return Err(AppError::bad_request(format!(
            "Invalid victim_type: {}. Valid values: {}",
            body.victim_type,
            VICTIM_TYPES.join(", ")
        )));
    }

    // Verify the case exists in this court
    crate::repo::case::find_by_id(&pool, &court.0, case_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Case {} not found", case_id)))?;

    let victim = crate::repo::victim::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(VictimResponse::from(victim))))
}

/// POST /api/cases/{id}/victims/{victim_id}/notifications
#[utoipa::path(
    post,
    path = "/api/cases/{id}/victims/{victim_id}/notifications",
    request_body = SendVictimNotificationRequest,
    params(
        ("id" = String, Path, description = "Case UUID"),
        ("victim_id" = String, Path, description = "Victim UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Notification sent", body = VictimNotificationResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Victim not found", body = AppError)
    ),
    tag = "victims"
)]
pub async fn send_notification(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((_case_id, victim_id)): Path<(String, String)>,
    Json(body): Json<SendVictimNotificationRequest>,
) -> Result<(StatusCode, Json<VictimNotificationResponse>), AppError> {
    let victim_uuid = Uuid::parse_str(&victim_id)
        .map_err(|_| AppError::bad_request("Invalid victim UUID format"))?;

    if !is_valid_notification_type(&body.notification_type) {
        return Err(AppError::bad_request(format!(
            "Invalid notification_type: {}. Valid values: {}",
            body.notification_type,
            NOTIFICATION_TYPES.join(", ")
        )));
    }

    if !is_valid_notification_method(&body.method) {
        return Err(AppError::bad_request(format!(
            "Invalid method: {}. Valid values: {}",
            body.method,
            NOTIFICATION_METHODS.join(", ")
        )));
    }

    if body.content_summary.trim().is_empty() {
        return Err(AppError::bad_request("content_summary must not be empty"));
    }

    // Verify the victim exists in this court
    crate::repo::victim::find_by_id(&pool, &court.0, victim_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Victim {} not found", victim_id)))?;

    let notification = crate::repo::victim_notification::send(
        &pool,
        &court.0,
        victim_uuid,
        &body.notification_type,
        &body.method,
        &body.content_summary,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(VictimNotificationResponse::from(notification)),
    ))
}
