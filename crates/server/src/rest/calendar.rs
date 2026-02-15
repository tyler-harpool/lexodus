use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Datelike;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, AvailableSlot, CalendarEntryResponse, CalendarEvent, CalendarEventType,
    CalendarSearchResponse, CourtUtilization, EventStatus,
    ScheduleEventRequest, UpdateEventStatusRequest, CalendarSearchParams,
};
use crate::error_convert::SqlxErrorExt;
use crate::tenant::CourtId;

/// POST /api/calendar/events
#[utoipa::path(
    post,
    path = "/api/calendar/events",
    request_body = ScheduleEventRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Event scheduled", body = CalendarEntryResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "calendar"
)]
pub async fn schedule_event(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<ScheduleEventRequest>,
) -> Result<(StatusCode, Json<CalendarEntryResponse>), AppError> {
    // Validate event_type
    if CalendarEventType::from_str_opt(&body.event_type).is_none() {
        return Err(AppError::bad_request(format!(
            "Invalid event_type: {}",
            body.event_type
        )));
    }

    // Validate duration
    if body.duration_minutes <= 0 {
        return Err(AppError::bad_request("duration_minutes must be positive"));
    }

    let event = crate::repo::calendar::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(CalendarEntryResponse::from(event))))
}

/// PATCH /api/calendar/events/{event_id}/status
#[utoipa::path(
    patch,
    path = "/api/calendar/events/{event_id}/status",
    params(
        ("event_id" = String, Path, description = "Calendar event UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = UpdateEventStatusRequest,
    responses(
        (status = 200, description = "Status updated", body = CalendarEntryResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "calendar"
)]
pub async fn update_event_status(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(event_id): Path<String>,
    Json(body): Json<UpdateEventStatusRequest>,
) -> Result<Json<CalendarEntryResponse>, AppError> {
    let uuid = Uuid::parse_str(&event_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    // Validate status
    if EventStatus::from_str_opt(&body.status).is_none() {
        return Err(AppError::bad_request(format!(
            "Invalid status: {}",
            body.status
        )));
    }

    let event = crate::repo::calendar::update_status(&pool, &court.0, uuid, body)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Calendar event {} not found", event_id)))?;

    Ok(Json(CalendarEntryResponse::from(event)))
}

/// DELETE /api/calendar/events/{id}
#[utoipa::path(
    delete,
    path = "/api/calendar/events/{id}",
    params(
        ("id" = String, Path, description = "Calendar event UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Event deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "calendar"
)]
pub async fn delete_event(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::calendar::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Calendar event {} not found", id)))
    }
}

/// GET /api/calendar/search
#[utoipa::path(
    get,
    path = "/api/calendar/search",
    params(
        CalendarSearchParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Search results", body = CalendarSearchResponse)
    ),
    tag = "calendar"
)]
pub async fn search_calendar(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<CalendarSearchParams>,
) -> Result<Json<CalendarSearchResponse>, AppError> {
    let offset = params.offset.unwrap_or(0).max(0);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);

    // Validate optional filters
    if let Some(ref et) = params.event_type {
        if CalendarEventType::from_str_opt(et).is_none() {
            return Err(AppError::bad_request(format!("Invalid event_type: {}", et)));
        }
    }
    if let Some(ref s) = params.status {
        if EventStatus::from_str_opt(s).is_none() {
            return Err(AppError::bad_request(format!("Invalid status: {}", s)));
        }
    }

    let (events, total) = crate::repo::calendar::search(
        &pool,
        &court.0,
        params.judge_id,
        params.courtroom.as_deref(),
        params.event_type.as_deref(),
        params.status.as_deref(),
        params.date_from,
        params.date_to,
        offset,
        limit,
    )
    .await?;

    let response = CalendarSearchResponse {
        events: events.into_iter().map(CalendarEntryResponse::from).collect(),
        total,
    };

    Ok(Json(response))
}

/// GET /api/cases/{case_id}/calendar
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/calendar",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Case calendar events", body = Vec<CalendarEntryResponse>)
    ),
    tag = "calendar"
)]
pub async fn get_case_calendar(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<CalendarEntryResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let events = crate::repo::calendar::list_by_case(&pool, &court.0, uuid).await?;
    let response: Vec<CalendarEntryResponse> =
        events.into_iter().map(CalendarEntryResponse::from).collect();

    Ok(Json(response))
}

/// GET /api/calendar/case/{case_id}
#[utoipa::path(
    get,
    path = "/api/calendar/case/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Calendar events for case", body = Vec<CalendarEntryResponse>)
    ),
    tag = "calendar"
)]
pub async fn list_calendar_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<CalendarEntryResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    let events = crate::repo::calendar::list_by_case(&pool, &court.0, uuid).await?;
    let response: Vec<CalendarEntryResponse> =
        events.into_iter().map(CalendarEntryResponse::from).collect();

    Ok(Json(response))
}

/// GET /api/judges/{judge_id}/schedule
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/schedule",
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Calendar events for judge", body = Vec<CalendarEntryResponse>)
    ),
    tag = "calendar"
)]
pub async fn list_by_judge(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(judge_id): Path<String>,
) -> Result<Json<Vec<CalendarEntryResponse>>, AppError> {
    let uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| AppError::bad_request("Invalid judge UUID format"))?;

    let rows = sqlx::query_as!(
        CalendarEvent,
        r#"
        SELECT
            id, court_id, case_id, judge_id, event_type, scheduled_date,
            duration_minutes, courtroom, description, participants,
            court_reporter, is_public, status, notes,
            actual_start, actual_end, call_time,
            created_at, updated_at
        FROM calendar_events
        WHERE court_id = $1 AND judge_id = $2
        ORDER BY scheduled_date ASC
        "#,
        court.0,
        uuid,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let response: Vec<CalendarEntryResponse> =
        rows.into_iter().map(CalendarEntryResponse::from).collect();

    Ok(Json(response))
}

/// GET /api/judges/{judge_id}/available-slot
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/available-slot",
    params(
        ("judge_id" = String, Path, description = "Judge UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Available slots for judge", body = Vec<AvailableSlot>)
    ),
    tag = "calendar"
)]
pub async fn find_available_slot(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(judge_id): Path<String>,
) -> Result<Json<Vec<AvailableSlot>>, AppError> {
    let uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| AppError::bad_request("Invalid judge UUID format"))?;

    // Get the judge's booked time slots for the next 14 days
    let booked = sqlx::query_as!(
        CalendarEvent,
        r#"
        SELECT
            id, court_id, case_id, judge_id, event_type, scheduled_date,
            duration_minutes, courtroom, description, participants,
            court_reporter, is_public, status, notes,
            actual_start, actual_end, call_time,
            created_at, updated_at
        FROM calendar_events
        WHERE court_id = $1
          AND judge_id = $2
          AND status NOT IN ('cancelled', 'completed')
          AND scheduled_date >= NOW()
          AND scheduled_date <= NOW() + INTERVAL '14 days'
        ORDER BY scheduled_date ASC
        "#,
        court.0,
        uuid,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    // Generate available slots for next 14 business days
    // Standard court hours: 9 AM to 5 PM, 1-hour slots
    let mut slots = Vec::new();
    let now = chrono::Utc::now();

    for day_offset in 1..=14i64 {
        let date = now + chrono::Duration::days(day_offset);
        let weekday = date.weekday();

        // Skip weekends
        if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
            continue;
        }

        let date_str = date.format("%Y-%m-%d").to_string();

        // Check each hour from 9 AM to 4 PM (last slot starts at 4 PM)
        for hour in 9..17 {
            let start = format!("{:02}:00", hour);
            let end = format!("{:02}:00", hour + 1);

            // Check if this slot conflicts with any booked event
            let slot_start = date.date_naive().and_hms_opt(hour, 0, 0);
            let is_available = slot_start.map(|ss| {
                !booked.iter().any(|b| {
                    let booked_start = b.scheduled_date.naive_utc();
                    let booked_end = booked_start + chrono::Duration::minutes(b.duration_minutes as i64);
                    let slot_end = ss + chrono::Duration::hours(1);
                    ss < booked_end && slot_end > booked_start
                })
            }).unwrap_or(true);

            if is_available {
                slots.push(AvailableSlot {
                    judge_id: judge_id.clone(),
                    date: date_str.clone(),
                    start_time: start,
                    end_time: end,
                    courtroom: None,
                });
            }
        }

        // Limit to first 5 days with availability
        if slots.len() >= 40 {
            break;
        }
    }

    Ok(Json(slots))
}

/// GET /api/courtrooms/utilization
#[utoipa::path(
    get,
    path = "/api/courtrooms/utilization",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Court utilization metrics", body = CourtUtilization)
    ),
    tag = "calendar"
)]
pub async fn get_utilization(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<CourtUtilization>, AppError> {
    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM calendar_events WHERE court_id = $1"#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let by_courtroom_raw = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(
            json_object_agg(courtroom, cnt),
            '{}'::json
        )::TEXT as "json!"
        FROM (
            SELECT courtroom, COUNT(*) as cnt
            FROM calendar_events
            WHERE court_id = $1
            GROUP BY courtroom
        ) sub
        "#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let by_judge_raw = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(
            json_object_agg(judge_id, cnt),
            '{}'::json
        )::TEXT as "json!"
        FROM (
            SELECT judge_id::TEXT, COUNT(*) as cnt
            FROM calendar_events
            WHERE court_id = $1
            GROUP BY judge_id
        ) sub
        "#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let completed = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM calendar_events WHERE court_id = $1 AND status = 'completed'"#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let utilization_rate = if total > 0 {
        (completed as f64) / (total as f64) * 100.0
    } else {
        0.0
    };

    let by_courtroom: serde_json::Value = serde_json::from_str(&by_courtroom_raw)
        .unwrap_or(serde_json::json!({}));
    let by_judge: serde_json::Value = serde_json::from_str(&by_judge_raw)
        .unwrap_or(serde_json::json!({}));

    Ok(Json(CourtUtilization {
        total_events: total,
        by_courtroom,
        by_judge,
        utilization_rate,
    }))
}

/// GET /api/courtrooms/{courtroom}/events
#[utoipa::path(
    get,
    path = "/api/courtrooms/{courtroom}/events",
    params(
        ("courtroom" = String, Path, description = "Courtroom name"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Events for courtroom", body = Vec<CalendarEntryResponse>)
    ),
    tag = "calendar"
)]
pub async fn list_by_courtroom(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(courtroom): Path<String>,
) -> Result<Json<Vec<CalendarEntryResponse>>, AppError> {
    let rows = sqlx::query_as!(
        CalendarEvent,
        r#"
        SELECT
            id, court_id, case_id, judge_id, event_type, scheduled_date,
            duration_minutes, courtroom, description, participants,
            court_reporter, is_public, status, notes,
            actual_start, actual_end, call_time,
            created_at, updated_at
        FROM calendar_events
        WHERE court_id = $1 AND courtroom = $2
        ORDER BY scheduled_date ASC
        "#,
        court.0,
        courtroom,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let response: Vec<CalendarEntryResponse> =
        rows.into_iter().map(CalendarEntryResponse::from).collect();

    Ok(Json(response))
}
