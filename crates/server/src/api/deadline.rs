use dioxus::prelude::*;
use shared_types::{
    CreateDeadlineRequest, DeadlineResponse, DeadlineSearchResponse, UpdateDeadlineRequest,
};

// ═══════════════════════════════════════════════════════════════
// Deadline server functions
// ═══════════════════════════════════════════════════════════════

/// Search deadlines with filters.
#[server]
pub async fn search_deadlines(
    court_id: String,
    status: Option<String>,
    case_id: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<DeadlineSearchResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline;
    use chrono::DateTime;
    use uuid::Uuid;

    let pool = get_db().await;
    let offset = offset.unwrap_or(0).max(0);
    let limit = limit.unwrap_or(20).clamp(1, 100);

    let case_uuid = case_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;

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

    let (deadlines, total) = deadline::search(
        pool, &court_id,
        status.as_deref().filter(|s| !s.is_empty()),
        case_uuid,
        date_from_parsed, date_to_parsed,
        offset, limit,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(DeadlineSearchResponse {
        deadlines: deadlines.into_iter().map(DeadlineResponse::from).collect(),
        total,
    })
}

/// Get a single deadline by ID.
#[server]
pub async fn get_deadline(court_id: String, id: String) -> Result<DeadlineResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let dl = deadline::find_by_id(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Deadline not found"))?;

    Ok(DeadlineResponse::from(dl))
}

/// Create a new deadline.
#[server]
pub async fn create_deadline(court_id: String, body: CreateDeadlineRequest) -> Result<DeadlineResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline;

    let pool = get_db().await;

    let dl = deadline::create(pool, &court_id, body).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(DeadlineResponse::from(dl))
}

/// Update an existing deadline.
#[server]
pub async fn update_deadline(
    court_id: String,
    id: String,
    body: UpdateDeadlineRequest,
) -> Result<DeadlineResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let dl = deadline::update(pool, &court_id, uuid, body).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Deadline not found"))?;

    Ok(DeadlineResponse::from(dl))
}

/// Delete a deadline by ID.
#[server]
pub async fn delete_deadline(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let deleted = deadline::delete(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if deleted {
        Ok(())
    } else {
        Err(ServerFnError::new("Deadline not found"))
    }
}

// ── Deadline Reminder Server Functions ──────────────────

#[server]
pub async fn list_reminders_by_deadline(
    court_id: String,
    deadline_id: String,
) -> Result<Vec<shared_types::ReminderResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline_reminder;
    use shared_types::ReminderResponse;
    use uuid::Uuid;

    let pool = get_db().await;
    let dl_uuid = Uuid::parse_str(&deadline_id)
        .map_err(|_| ServerFnError::new("Invalid deadline_id UUID"))?;
    let rows = deadline_reminder::list_by_deadline(pool, &court_id, dl_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(ReminderResponse::from).collect())
}

#[server]
pub async fn list_pending_reminders(court_id: String) -> Result<Vec<shared_types::ReminderResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline_reminder;
    use shared_types::ReminderResponse;

    let pool = get_db().await;
    let rows = deadline_reminder::list_pending(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(ReminderResponse::from).collect())
}

#[server]
pub async fn send_reminder(
    court_id: String,
    deadline_id: String,
    recipient: String,
    reminder_type: String,
) -> Result<shared_types::ReminderResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline_reminder;
    use shared_types::ReminderResponse;
    use uuid::Uuid;

    let pool = get_db().await;
    let dl_uuid = Uuid::parse_str(&deadline_id)
        .map_err(|_| ServerFnError::new("Invalid deadline_id UUID"))?;
    let row = deadline_reminder::send(pool, &court_id, dl_uuid, &recipient, &reminder_type)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(ReminderResponse::from(row))
}

#[server]
pub async fn acknowledge_reminder(court_id: String, id: String) -> Result<shared_types::ReminderResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline_reminder;
    use shared_types::ReminderResponse;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = deadline_reminder::acknowledge(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(ReminderResponse::from(row))
}
