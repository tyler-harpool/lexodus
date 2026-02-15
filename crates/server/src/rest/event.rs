use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, EventKind, SubmitEventRequest, SubmitEventResponse,
    TimelineEntry, TimelineResponse, UserRole, get_event_config,
};
use crate::auth::court_role::resolve_court_role;
use crate::auth::extractors::AuthRequired;
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// POST /api/events — unified event submission
// ---------------------------------------------------------------------------

/// POST /api/events
///
/// Submit a unified docket event (text entry, filing, or promote attachment).
/// Role requirements vary by event kind.
#[utoipa::path(
    post,
    path = "/api/events",
    request_body = SubmitEventRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Event created", body = SubmitEventResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 403, description = "Insufficient role", body = AppError),
    ),
    tag = "events"
)]
pub async fn submit_event(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    auth: AuthRequired,
    Json(body): Json<SubmitEventRequest>,
) -> Result<(StatusCode, Json<SubmitEventResponse>), AppError> {
    // Resolve event kind and validate role
    let kind = EventKind::from_str(&body.event_kind)
        .ok_or_else(|| AppError::bad_request(format!("Unknown event_kind '{}'", body.event_kind)))?;
    let config = get_event_config(kind);

    let role = resolve_court_role(&auth.0, &court.0);
    let has_permission = match config.min_role {
        "attorney" => matches!(role, UserRole::Attorney | UserRole::Clerk | UserRole::Judge | UserRole::Admin),
        "clerk" => matches!(role, UserRole::Clerk | UserRole::Judge | UserRole::Admin),
        "judge" => matches!(role, UserRole::Judge | UserRole::Admin),
        "admin" => matches!(role, UserRole::Admin),
        _ => matches!(role, UserRole::Clerk | UserRole::Judge | UserRole::Admin),
    };
    if !has_permission {
        return Err(AppError::forbidden(format!(
            "{} role or higher required for {} events",
            config.min_role,
            kind.label()
        )));
    }

    let response = crate::repo::event::submit_event(&pool, &court.0, &body).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

// ---------------------------------------------------------------------------
// GET /api/cases/{case_id}/timeline — unified case timeline
// ---------------------------------------------------------------------------

/// Query parameters for the timeline endpoint.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct TimelineParams {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

/// GET /api/cases/{case_id}/timeline
///
/// Returns a unified chronological timeline of all docket entries and document
/// events for a case, newest first.
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/timeline",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID"),
        ("offset" = Option<i64>, Query, description = "Pagination offset"),
        ("limit" = Option<i64>, Query, description = "Pagination limit"),
    ),
    responses(
        (status = 200, description = "Timeline entries", body = TimelineResponse),
        (status = 400, description = "Invalid case_id", body = AppError),
    ),
    tag = "events"
)]
pub async fn get_case_timeline(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
    Query(params): Query<TimelineParams>,
) -> Result<Json<TimelineResponse>, AppError> {
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case_id UUID format"))?;

    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);

    // Fetch docket entries for this case
    let (docket_entries, _docket_total) =
        crate::repo::docket::list_by_case(&pool, &court.0, case_uuid, 0, 1000).await?;

    // Fetch document events for all documents in this case
    let doc_events =
        crate::repo::document_event::list_by_case(&pool, &court.0, case_uuid).await?;

    // Fetch NEFs for this case
    let nefs =
        crate::repo::nef::list_by_case(&pool, &court.0, case_uuid).await?;

    // Merge into a unified timeline
    let mut entries: Vec<TimelineEntry> = Vec::new();

    for de in &docket_entries {
        entries.push(TimelineEntry {
            id: de.id.to_string(),
            source: "docket_entry".to_string(),
            timestamp: de.date_filed.to_rfc3339(),
            summary: de.description.clone(),
            actor: de.filed_by.clone(),
            entry_type: de.entry_type.clone(),
            is_sealed: de.is_sealed,
            document_id: de.document_id.map(|u| u.to_string()),
            entry_number: Some(de.entry_number),
            detail: serde_json::json!({}),
        });
    }

    for evt in &doc_events {
        entries.push(TimelineEntry {
            id: evt.id.to_string(),
            source: "document_event".to_string(),
            timestamp: evt.created_at.to_rfc3339(),
            summary: format!("Document {}", evt.event_type),
            actor: Some(evt.actor.clone()),
            entry_type: evt.event_type.clone(),
            is_sealed: false,
            document_id: Some(evt.document_id.to_string()),
            entry_number: None,
            detail: evt.detail.clone(),
        });
    }

    for nef in &nefs {
        entries.push(TimelineEntry {
            id: nef.id.to_string(),
            source: "nef".to_string(),
            timestamp: nef.created_at.to_rfc3339(),
            summary: "Notice of Electronic Filing issued".to_string(),
            actor: None,
            entry_type: "nef".to_string(),
            is_sealed: false,
            document_id: Some(nef.document_id.to_string()),
            entry_number: None,
            detail: serde_json::json!({
                "nef_id": nef.id.to_string(),
                "filing_id": nef.filing_id.to_string(),
                "docket_entry_id": nef.docket_entry_id.to_string(),
            }),
        });
    }

    // Sort newest first
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let total = entries.len() as i64;

    // Apply pagination
    let paginated: Vec<TimelineEntry> = entries
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    Ok(Json(TimelineResponse {
        entries: paginated,
        total,
    }))
}
