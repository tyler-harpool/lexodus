use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, ReminderResponse, SendReminderRequest,
    is_valid_reminder_type, REMINDER_TYPES,
};
use crate::tenant::CourtId;

/// GET /api/deadlines/reminders/pending
#[utoipa::path(
    get,
    path = "/api/deadlines/reminders/pending",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Pending reminders", body = Vec<ReminderResponse>)
    ),
    tag = "reminders"
)]
pub async fn list_pending_reminders(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<ReminderResponse>>, AppError> {
    let reminders = crate::repo::deadline_reminder::list_pending(&pool, &court.0).await?;
    let response: Vec<ReminderResponse> =
        reminders.into_iter().map(ReminderResponse::from).collect();

    Ok(Json(response))
}

/// POST /api/deadlines/reminders/send
#[utoipa::path(
    post,
    path = "/api/deadlines/reminders/send",
    request_body = SendReminderRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Reminder sent", body = ReminderResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "reminders"
)]
pub async fn send_reminder(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<SendReminderRequest>,
) -> Result<(StatusCode, Json<ReminderResponse>), AppError> {
    if body.recipient.trim().is_empty() {
        return Err(AppError::bad_request("recipient must not be empty"));
    }

    if !is_valid_reminder_type(&body.reminder_type) {
        return Err(AppError::bad_request(format!(
            "Invalid reminder_type: {}. Valid values: {}",
            body.reminder_type,
            REMINDER_TYPES.join(", ")
        )));
    }

    // Verify the deadline exists in this court
    crate::repo::deadline::find_by_id(&pool, &court.0, body.deadline_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Deadline {} not found", body.deadline_id)))?;

    let reminder = crate::repo::deadline_reminder::send(
        &pool,
        &court.0,
        body.deadline_id,
        &body.recipient,
        &body.reminder_type,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(ReminderResponse::from(reminder))))
}

/// GET /api/deadlines/{deadline_id}/reminders
#[utoipa::path(
    get,
    path = "/api/deadlines/{deadline_id}/reminders",
    params(
        ("deadline_id" = String, Path, description = "Deadline UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Reminders for deadline", body = Vec<ReminderResponse>)
    ),
    tag = "reminders"
)]
pub async fn list_reminders_by_deadline(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(deadline_id): Path<String>,
) -> Result<Json<Vec<ReminderResponse>>, AppError> {
    let uuid = Uuid::parse_str(&deadline_id)
        .map_err(|_| AppError::bad_request("Invalid deadline UUID format"))?;

    let reminders = crate::repo::deadline_reminder::list_by_deadline(&pool, &court.0, uuid).await?;
    let response: Vec<ReminderResponse> =
        reminders.into_iter().map(ReminderResponse::from).collect();

    Ok(Json(response))
}

/// GET /api/deadlines/reminders/recipient/{recipient}
#[utoipa::path(
    get,
    path = "/api/deadlines/reminders/recipient/{recipient}",
    params(
        ("recipient" = String, Path, description = "Recipient identifier"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Reminders for recipient", body = Vec<ReminderResponse>)
    ),
    tag = "reminders"
)]
pub async fn list_by_recipient(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(recipient): Path<String>,
) -> Result<Json<Vec<ReminderResponse>>, AppError> {
    let reminders = crate::repo::deadline_reminder::list_by_recipient(
        &pool, &court.0, &recipient,
    )
    .await?;
    let response: Vec<ReminderResponse> =
        reminders.into_iter().map(ReminderResponse::from).collect();

    Ok(Json(response))
}

/// PATCH /api/deadlines/reminders/{reminder_id}/acknowledge
#[utoipa::path(
    patch,
    path = "/api/deadlines/reminders/{reminder_id}/acknowledge",
    params(
        ("reminder_id" = String, Path, description = "Reminder UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Reminder acknowledged", body = ReminderResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "reminders"
)]
pub async fn acknowledge_reminder(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(reminder_id): Path<String>,
) -> Result<Json<ReminderResponse>, AppError> {
    let uuid = Uuid::parse_str(&reminder_id)
        .map_err(|_| AppError::bad_request("Invalid reminder UUID format"))?;

    let reminder = crate::repo::deadline_reminder::acknowledge(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Reminder {} not found", reminder_id)))?;

    Ok(Json(ReminderResponse::from(reminder)))
}
