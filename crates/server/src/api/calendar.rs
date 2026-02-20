use dioxus::prelude::*;
use shared_types::{CalendarEntryResponse, CalendarSearchResponse, ScheduleEventRequest};

/// Search calendar events with filters.
#[server]
pub async fn search_calendar_events(
    court_id: String,
    judge_id: Option<String>,
    courtroom: Option<String>,
    event_type: Option<String>,
    status: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<CalendarSearchResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::calendar;
    use chrono::DateTime;
    use uuid::Uuid;

    let pool = get_db().await;
    let offset = offset.unwrap_or(0).max(0);
    let limit = limit.unwrap_or(20).clamp(1, 100);

    let judge_uuid = judge_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;

    let date_from_parsed = date_from
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid date_from format"))?;

    let date_to_parsed = date_to
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid date_to format"))?;

    let (events, total) = calendar::search(
        pool, &court_id, judge_uuid,
        courtroom.as_deref().filter(|s| !s.is_empty()),
        event_type.as_deref().filter(|s| !s.is_empty()),
        status.as_deref().filter(|s| !s.is_empty()),
        date_from_parsed, date_to_parsed, offset, limit,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(CalendarSearchResponse {
        events: events.into_iter().map(CalendarEntryResponse::from).collect(),
        total,
    })
}

/// Get a single calendar event by ID.
#[server]
pub async fn get_calendar_event(court_id: String, id: String) -> Result<CalendarEntryResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::calendar;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let event = calendar::find_by_id(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Calendar event not found"))?;

    Ok(CalendarEntryResponse::from(event))
}

/// Schedule a new calendar event.
#[server]
pub async fn schedule_calendar_event(court_id: String, body: ScheduleEventRequest) -> Result<CalendarEntryResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::calendar;

    let pool = get_db().await;
    let event = calendar::create(pool, &court_id, body).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(CalendarEntryResponse::from(event))
}

/// Delete a calendar event by ID.
#[server]
pub async fn delete_calendar_event(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::calendar;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let deleted = calendar::delete(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if deleted {
        Ok(())
    } else {
        Err(ServerFnError::new("Calendar event not found"))
    }
}

/// List all calendar events for a specific case.
#[server]
pub async fn list_calendar_by_case(court_id: String, case_id: String) -> Result<Vec<CalendarEntryResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::calendar;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid = Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;

    let rows = calendar::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows.into_iter().map(CalendarEntryResponse::from).collect())
}
