use dioxus::prelude::*;

// ── Document Action Server Functions ───────────────────────────

/// Seal a document. Requires clerk/judge role via REST handler delegation.
#[server]
pub async fn seal_document_action(
    court_id: String,
    document_id: String,
    sealing_level: String,
    reason_code: String,
    motion_id: Option<String>,
) -> Result<shared_types::DocumentResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{document, document_event};
    use shared_types::SealingLevel;
    use uuid::Uuid;

    let pool = get_db().await;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    let level = SealingLevel::from_db_str(&sealing_level);
    if !level.is_sealed() {
        return Err(ServerFnError::new(
            "sealing_level must be one of: SealedCourtOnly, SealedCaseParticipants, SealedAttorneysOnly",
        ));
    }

    let motion_uuid = motion_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid motion_id UUID"))?;

    let doc = document::seal(pool, &court_id, doc_uuid, &level, &reason_code, motion_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Audit event (fire-and-forget)
    let _ = document_event::create(
        pool,
        &court_id,
        doc_uuid,
        "sealed",
        "ui-user",
        serde_json::json!({
            "sealing_level": sealing_level,
            "reason_code": reason_code,
            "motion_id": motion_id,
        }),
    )
    .await;

    Ok(shared_types::DocumentResponse::from(doc))
}

/// Unseal a document.
#[server]
pub async fn unseal_document_action(
    court_id: String,
    document_id: String,
) -> Result<shared_types::DocumentResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{document, document_event};
    use uuid::Uuid;

    let pool = get_db().await;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    let doc = document::unseal(pool, &court_id, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let _ = document_event::create(
        pool, &court_id, doc_uuid, "unsealed", "ui-user", serde_json::json!({}),
    )
    .await;

    Ok(shared_types::DocumentResponse::from(doc))
}

/// Strike a document from the record.
#[server]
pub async fn strike_document_action(
    court_id: String,
    document_id: String,
) -> Result<shared_types::DocumentResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{document, document_event};
    use uuid::Uuid;

    let pool = get_db().await;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    let doc = document::strike(pool, &court_id, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let _ = document_event::create(
        pool, &court_id, doc_uuid, "stricken", "ui-user", serde_json::json!({}),
    )
    .await;

    Ok(shared_types::DocumentResponse::from(doc))
}

/// Replace a document file. Handles S3 upload server-side.
#[server]
pub async fn replace_document_file(
    court_id: String,
    document_id: String,
    file_name: String,
    content_type: String,
    file_size: i64,
    file_bytes: Vec<u8>,
    title: Option<String>,
) -> Result<shared_types::DocumentResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{document, document_event, filing};
    use crate::storage::{ObjectStore, S3ObjectStore};
    use uuid::Uuid;

    let pool = get_db().await;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    if file_name.trim().is_empty() {
        return Err(ServerFnError::new("file_name must not be empty"));
    }

    // Stage upload
    let file_uuid = Uuid::new_v4();
    let object_key = format!(
        "{}/documents/replace/{}/{}",
        court_id, file_uuid, file_name
    );

    let upload = filing::create_pending_upload(
        pool, &court_id, &file_name, file_size, &content_type, &object_key,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Upload to S3
    let store = S3ObjectStore::from_env();
    store
        .put(&object_key, &content_type, file_bytes)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    // Mark as finalized
    filing::mark_upload_finalized(pool, &court_id, upload.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Get original document to inherit title
    let original = document::find_by_id(pool, &court_id, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Document not found"))?;

    let doc_title = title.as_deref().unwrap_or(&original.title);

    let replacement = document::replace(
        pool,
        &court_id,
        doc_uuid,
        doc_title,
        &object_key,
        file_size,
        &content_type,
        "",
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let _ = document_event::create(
        pool,
        &court_id,
        doc_uuid,
        "replaced",
        "ui-user",
        serde_json::json!({
            "replacement_document_id": replacement.id.to_string(),
        }),
    )
    .await;

    Ok(shared_types::DocumentResponse::from(replacement))
}

/// List document audit events.
#[server]
pub async fn list_document_events_action(
    court_id: String,
    document_id: String,
) -> Result<Vec<shared_types::DocumentEventResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::document_event;
    use uuid::Uuid;

    let pool = get_db().await;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    let events = document_event::list_by_document(pool, &court_id, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(events.into_iter().map(Into::into).collect())
}

/// List all documents for a court with optional title search and pagination.
#[server]
pub async fn list_all_documents(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<shared_types::PaginatedResponse<shared_types::DocumentResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::document;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = document::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::DocumentResponse> =
        rows.into_iter().map(shared_types::DocumentResponse::from).collect();

    let total_pages = if per_page > 0 { (total + per_page - 1) / per_page } else { 0 };
    let meta = shared_types::PaginationMeta {
        total,
        page,
        limit: per_page,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    };

    Ok(shared_types::PaginatedResponse {
        data: responses,
        meta,
    })
}

/// Get a single document by ID.
#[server]
pub async fn get_document_by_id(
    court_id: String,
    id: String,
) -> Result<shared_types::DocumentResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::document;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let doc = document::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Document not found"))?;

    Ok(shared_types::DocumentResponse::from(doc))
}

// ── Unified Event Composer Server Functions ────────────────────

/// Submit a unified docket event (text entry, filing, or promote attachment).
/// Dispatches to the appropriate workflow based on event_kind.
#[server]
pub async fn submit_event(
    court_id: String,
    body: shared_types::SubmitEventRequest,
) -> Result<shared_types::SubmitEventResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::event;

    let pool = get_db().await;

    let response = event::submit_event(pool, &court_id, &body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(response)
}

/// Get the unified case timeline (docket entries + document events).
#[server]
pub async fn get_case_timeline(
    court_id: String,
    case_id: String,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<shared_types::TimelineResponse, ServerFnError> {
    use crate::db::get_db;
    use shared_types::TimelineEntry;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;

    let limit = limit.unwrap_or(50).min(200);
    let offset = offset.unwrap_or(0);

    // Fetch docket entries
    let (docket_entries, _) =
        crate::repo::docket::list_by_case(pool, &court_id, case_uuid, 0, 1000)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Fetch document events for documents in this case
    let doc_events =
        crate::repo::document_event::list_by_case(pool, &court_id, case_uuid)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Fetch NEFs for this case
    let nefs =
        crate::repo::nef::list_by_case(pool, &court_id, case_uuid)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Merge into unified timeline
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
    let paginated: Vec<TimelineEntry> = entries
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    Ok(shared_types::TimelineResponse {
        entries: paginated,
        total,
    })
}
