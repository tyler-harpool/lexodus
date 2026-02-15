use shared_types::{
    AppError, CreateDocketEntryRequest, EventKind,
    SubmitEventRequest, SubmitEventResponse, ValidateFilingRequest,
    VALID_DOCUMENT_TYPES,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

/// Submit a unified docket event. Dispatches to the appropriate workflow based
/// on `event_kind`, creating the necessary records atomically.
pub async fn submit_event(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: &SubmitEventRequest,
) -> Result<SubmitEventResponse, AppError> {
    let kind = EventKind::from_str(&req.event_kind)
        .ok_or_else(|| AppError::bad_request(format!("Unknown event_kind '{}'", req.event_kind)))?;

    match kind {
        EventKind::TextEntry => submit_text_entry(pool, court_id, req).await,
        EventKind::Filing => submit_filing_event(pool, court_id, req).await,
        EventKind::PromoteAttachment => submit_promote(pool, court_id, req).await,
    }
}

// ---------------------------------------------------------------------------
// TextEntry: creates only a DocketEntry (no document, no filing, no NEF)
// ---------------------------------------------------------------------------

async fn submit_text_entry(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: &SubmitEventRequest,
) -> Result<SubmitEventResponse, AppError> {
    let case_uuid = parse_case_id(&req.case_id)?;
    let entry_type = req
        .entry_type
        .as_deref()
        .ok_or_else(|| AppError::bad_request("entry_type is required for text_entry"))?;
    let description = req
        .description
        .as_deref()
        .ok_or_else(|| AppError::bad_request("description is required for text_entry"))?;

    if description.trim().is_empty() {
        return Err(AppError::bad_request("description must not be empty"));
    }

    let docket_req = CreateDocketEntryRequest {
        case_id: case_uuid,
        entry_type: entry_type.to_string(),
        description: description.to_string(),
        filed_by: req.filed_by.clone(),
        document_id: None,
        is_sealed: false,
        is_ex_parte: false,
        page_count: None,
        related_entries: vec![],
        service_list: vec![],
    };

    let entry = crate::repo::docket::create(pool, court_id, docket_req).await?;

    Ok(SubmitEventResponse {
        event_kind: EventKind::TextEntry.as_str().to_string(),
        docket_entry_id: entry.id.to_string(),
        entry_number: entry.entry_number,
        document_id: None,
        filing_id: None,
        nef_id: None,
    })
}

// ---------------------------------------------------------------------------
// Filing: delegates to the existing filing::submit which atomically creates
// Document + DocketEntry + Filing + service_records + NEF
// ---------------------------------------------------------------------------

async fn submit_filing_event(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: &SubmitEventRequest,
) -> Result<SubmitEventResponse, AppError> {
    let document_type = req
        .document_type
        .as_deref()
        .ok_or_else(|| AppError::bad_request("document_type is required for filing"))?;
    let title = req
        .title
        .as_deref()
        .ok_or_else(|| AppError::bad_request("title is required for filing"))?;
    let filed_by = req
        .filed_by
        .as_deref()
        .ok_or_else(|| AppError::bad_request("filed_by is required for filing"))?;

    let filing_req = ValidateFilingRequest {
        case_id: req.case_id.clone(),
        document_type: document_type.to_string(),
        title: title.to_string(),
        filed_by: filed_by.to_string(),
        upload_id: req.upload_id.clone(),
        is_sealed: req.is_sealed,
        sealing_level: req.sealing_level.clone(),
        reason_code: req.reason_code.clone(),
    };

    let (filing, doc, docket_entry, _case_number, nef) =
        crate::repo::filing::submit(pool, court_id, &filing_req).await?;

    Ok(SubmitEventResponse {
        event_kind: EventKind::Filing.as_str().to_string(),
        docket_entry_id: docket_entry.id.to_string(),
        entry_number: docket_entry.entry_number,
        document_id: Some(doc.id.to_string()),
        filing_id: Some(filing.id.to_string()),
        nef_id: Some(nef.id.to_string()),
    })
}

// ---------------------------------------------------------------------------
// PromoteAttachment: promotes a docket attachment to a canonical document
// ---------------------------------------------------------------------------

async fn submit_promote(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: &SubmitEventRequest,
) -> Result<SubmitEventResponse, AppError> {
    let attachment_id_str = req
        .attachment_id
        .as_deref()
        .ok_or_else(|| AppError::bad_request("attachment_id is required for promote_attachment"))?;
    let att_uuid = Uuid::parse_str(attachment_id_str)
        .map_err(|_| AppError::bad_request("Invalid attachment_id UUID format"))?;

    // Validate document_type if provided
    let doc_type = req.promote_document_type.as_deref().unwrap_or("Other");
    if !VALID_DOCUMENT_TYPES.contains(&doc_type) {
        return Err(AppError::bad_request(format!(
            "Invalid document_type '{}'. Valid values: {}",
            doc_type,
            VALID_DOCUMENT_TYPES.join(", ")
        )));
    }

    // Look up the attachment
    let attachment = crate::repo::attachment::find_by_id(pool, court_id, att_uuid)
        .await?
        .ok_or_else(|| AppError::not_found("Attachment not found"))?;

    if attachment.uploaded_at.is_none() {
        return Err(AppError::bad_request(
            "Attachment has not been uploaded yet. Finalize the upload first.",
        ));
    }

    // Check for existing document (idempotency)
    if let Some(existing) =
        crate::repo::document::find_by_source_attachment(pool, court_id, att_uuid).await?
    {
        let entry = crate::repo::docket::find_by_id(pool, court_id, attachment.docket_entry_id)
            .await?;
        return Ok(SubmitEventResponse {
            event_kind: EventKind::PromoteAttachment.as_str().to_string(),
            docket_entry_id: attachment.docket_entry_id.to_string(),
            entry_number: entry.map(|e| e.entry_number).unwrap_or(0),
            document_id: Some(existing.id.to_string()),
            filing_id: None,
            nef_id: None,
        });
    }

    // Resolve the case_id from the docket entry
    let entry = crate::repo::docket::find_by_id(pool, court_id, attachment.docket_entry_id)
        .await?
        .ok_or_else(|| {
            AppError::internal("Attachment's docket entry not found — data integrity issue")
        })?;

    let title = req
        .promote_title
        .as_deref()
        .unwrap_or(&attachment.filename);
    let checksum = attachment.sha256.clone().unwrap_or_default();

    let doc = crate::repo::document::promote_attachment(
        pool,
        court_id,
        att_uuid,
        entry.case_id,
        title,
        doc_type,
        &attachment.storage_key,
        attachment.file_size,
        &attachment.content_type,
        &checksum,
    )
    .await?;

    // Auto-link the new document to the owning docket entry
    let _ = crate::repo::docket::link_document(
        pool,
        court_id,
        attachment.docket_entry_id,
        doc.id,
    )
    .await;

    Ok(SubmitEventResponse {
        event_kind: EventKind::PromoteAttachment.as_str().to_string(),
        docket_entry_id: attachment.docket_entry_id.to_string(),
        entry_number: entry.entry_number,
        document_id: Some(doc.id.to_string()),
        filing_id: None,
        nef_id: None,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_case_id(s: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(s).map_err(|_| AppError::bad_request("Invalid case_id UUID format"))
}
