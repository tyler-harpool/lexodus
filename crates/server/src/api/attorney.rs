use dioxus::prelude::*;
use shared_types::{
    AttorneyResponse, CalendarEntryResponse, CaseResponse, CreateAttorneyRequest,
    DeadlineResponse, PaginatedResponse, UpdateAttorneyRequest,
};

/// Fetch attorneys for the selected court district.
#[server]
pub async fn list_attorneys(
    court_id: String,
    page: Option<i64>,
    limit: Option<i64>,
) -> Result<PaginatedResponse<AttorneyResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attorney;
    use shared_types::normalize_pagination;

    let pool = get_db().await;
    let (page, limit) = normalize_pagination(page, limit);
    let (attorneys, total) = attorney::list(pool, &court_id, page, limit).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(PaginatedResponse::new(
        attorneys.into_iter().map(AttorneyResponse::from).collect(),
        page, limit, total,
    ))
}

/// Get a single attorney by ID.
#[server]
pub async fn get_attorney(court_id: String, id: String) -> Result<AttorneyResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attorney;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let att = attorney::find_by_id(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attorney not found"))?;

    Ok(AttorneyResponse::from(att))
}

/// Create a new attorney.
#[server]
pub async fn create_attorney(court_id: String, body: CreateAttorneyRequest) -> Result<AttorneyResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attorney;

    let pool = get_db().await;
    let att = attorney::create(pool, &court_id, body).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(AttorneyResponse::from(att))
}

/// Update an existing attorney.
#[server]
pub async fn update_attorney(
    court_id: String,
    id: String,
    body: UpdateAttorneyRequest,
) -> Result<AttorneyResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attorney;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let att = attorney::update(pool, &court_id, uuid, body).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attorney not found"))?;

    Ok(AttorneyResponse::from(att))
}

/// Delete an attorney by ID.
#[server]
pub async fn delete_attorney(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attorney;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let deleted = attorney::delete(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if deleted {
        Ok(())
    } else {
        Err(ServerFnError::new("Attorney not found"))
    }
}

/// Search attorneys by query string.
#[server]
pub async fn search_attorneys(
    court_id: String,
    query: String,
    page: Option<i64>,
    limit: Option<i64>,
) -> Result<PaginatedResponse<AttorneyResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attorney;
    use shared_types::normalize_pagination;

    let pool = get_db().await;
    let (page, limit) = normalize_pagination(page, limit);
    let (attorneys, total) = attorney::search(pool, &court_id, &query, page, limit).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(PaginatedResponse::new(
        attorneys.into_iter().map(AttorneyResponse::from).collect(),
        page, limit, total,
    ))
}

/// List cases where the attorney has an active representation.
/// Joins through the representations table to find all case_ids,
/// then fetches case details from both criminal_cases and civil_cases.
#[server]
pub async fn list_cases_for_attorney(
    court_id: String,
    attorney_id: String,
) -> Result<Vec<CaseResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{case, civil_case, representation};
    use uuid::Uuid;

    let pool = get_db().await;
    let atty_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;

    // Get all active representations for this attorney
    let reps = representation::list_active_by_attorney(pool, &court_id, atty_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Collect unique case IDs
    let case_ids: Vec<Uuid> = reps
        .iter()
        .map(|r| r.case_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let mut cases = Vec::new();
    for cid in case_ids {
        // Try criminal first, then civil
        if let Some(c) = case::find_by_id(pool, &court_id, cid)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
        {
            cases.push(CaseResponse::from(c));
        } else if let Some(c) = civil_case::find_by_id(pool, &court_id, cid)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
        {
            cases.push(CaseResponse {
                id: c.id.to_string(),
                case_number: c.case_number,
                title: c.title,
                description: c.description,
                case_type: "civil".to_string(),
                crime_type: c.nature_of_suit,
                status: c.status,
                priority: c.priority,
                district_code: c.district_code,
                location: c.location,
                opened_at: c.opened_at.to_rfc3339(),
                updated_at: c.updated_at.to_rfc3339(),
                closed_at: c.closed_at.map(|d| d.to_rfc3339()),
                assigned_judge_id: c.assigned_judge_id.map(|u| u.to_string()),
                is_sealed: c.is_sealed,
                sealed_by: c.sealed_by,
                sealed_date: c.sealed_date.map(|d| d.to_rfc3339()),
                seal_reason: c.seal_reason,
                jurisdiction_basis: Some(c.jurisdiction_basis),
                jury_demand: Some(c.jury_demand),
                class_action: Some(c.class_action),
                amount_in_controversy: c.amount_in_controversy,
                consent_to_magistrate: Some(c.consent_to_magistrate),
                pro_se: Some(c.pro_se),
            });
        }
    }

    Ok(cases)
}

/// List calendar events for cases where the attorney has active representations.
/// JOINs calendar_events through representations and resolves case numbers from
/// both criminal_cases and civil_cases.
#[server]
pub async fn list_calendar_events_for_attorney(
    court_id: String,
    attorney_id: String,
    date_from: Option<String>,
) -> Result<Vec<CalendarEntryResponse>, ServerFnError> {
    use crate::db::get_db;
    use chrono::DateTime;
    use shared_types::CalendarEvent;
    use uuid::Uuid;

    let pool = get_db().await;
    let atty_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;

    let date_from_parsed = date_from
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid date_from format"))?;

    let events = sqlx::query_as!(
        CalendarEvent,
        r#"
        SELECT
            ce.id, ce.court_id, ce.case_id, ce.judge_id,
            ce.event_type, ce.scheduled_date, ce.duration_minutes,
            ce.courtroom, ce.description, ce.participants,
            ce.court_reporter, ce.is_public, ce.status, ce.notes,
            ce.actual_start, ce.actual_end, ce.call_time,
            ce.created_at, ce.updated_at,
            COALESCE(cc.case_number, cv.case_number) as case_number
        FROM calendar_events ce
        JOIN representations r
            ON ce.case_id = r.case_id
            AND r.attorney_id = $2
            AND r.status = 'Active'
        LEFT JOIN criminal_cases cc ON ce.case_id = cc.id AND cc.court_id = ce.court_id
        LEFT JOIN civil_cases cv ON ce.case_id = cv.id AND cv.court_id = ce.court_id
        WHERE ce.court_id = $1
          AND ($3::TIMESTAMPTZ IS NULL OR ce.scheduled_date >= $3)
        ORDER BY ce.scheduled_date ASC
        "#,
        court_id,
        atty_uuid,
        date_from_parsed,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to fetch calendar events: {e}")))?;

    Ok(events.into_iter().map(CalendarEntryResponse::from).collect())
}

/// List deadlines for cases where the attorney has active representations.
/// JOINs deadlines through representations with optional status and date filters.
#[server]
pub async fn list_deadlines_for_attorney(
    court_id: String,
    attorney_id: String,
    status: Option<String>,
    date_to: Option<String>,
) -> Result<Vec<DeadlineResponse>, ServerFnError> {
    use crate::db::get_db;
    use chrono::DateTime;
    use shared_types::Deadline;
    use uuid::Uuid;

    let pool = get_db().await;
    let atty_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;

    let date_to_parsed = date_to
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid date_to format"))?;

    let deadlines = sqlx::query_as!(
        Deadline,
        r#"
        SELECT
            d.id, d.court_id, d.case_id, d.title,
            d.rule_code, d.due_at, d.status, d.notes,
            d.created_at, d.updated_at
        FROM deadlines d
        JOIN representations r
            ON d.case_id = r.case_id
            AND r.attorney_id = $2
            AND r.status = 'Active'
        WHERE d.court_id = $1
          AND ($3::TEXT IS NULL OR d.status = $3)
          AND ($4::TIMESTAMPTZ IS NULL OR d.due_at <= $4)
        ORDER BY d.due_at ASC
        "#,
        court_id,
        atty_uuid,
        status.as_deref(),
        date_to_parsed,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to fetch deadlines: {e}")))?;

    Ok(deadlines.into_iter().map(DeadlineResponse::from).collect())
}
