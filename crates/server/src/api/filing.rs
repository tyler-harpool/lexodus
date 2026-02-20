use dioxus::prelude::*;

// ── Service Record server functions ────────────────────────────

/// List service records for a document.
#[server]
pub async fn list_document_service_records(
    court_id: String,
    document_id: String,
) -> Result<Vec<shared_types::ServiceRecordResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::service_record;
    use uuid::Uuid;

    let pool = get_db().await;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    let records = service_record::list_by_document(pool, &court_id, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(records.into_iter().map(Into::into).collect())
}

/// Create a new service record.
#[server]
pub async fn create_service_record(
    court_id: String,
    body: shared_types::CreateServiceRecordRequest,
) -> Result<shared_types::ServiceRecordResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::service_record;

    let pool = get_db().await;
    let record = service_record::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(shared_types::ServiceRecordResponse::from(record))
}

/// List all service records for a court with optional search and pagination.
#[server]
pub async fn list_all_service_records(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<shared_types::PaginatedResponse<shared_types::ServiceRecordResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::service_record;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = service_record::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::ServiceRecordResponse> =
        rows.into_iter().map(Into::into).collect();

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

/// Get a single service record by ID.
#[server]
pub async fn get_service_record(
    court_id: String,
    id: String,
) -> Result<shared_types::ServiceRecordResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::service_record;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let record = service_record::find_by_id_with_party(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Service record not found"))?;

    Ok(shared_types::ServiceRecordResponse::from(record))
}

/// Link an existing document to a docket entry.
#[server]
pub async fn link_document_to_entry(
    court_id: String,
    entry_id: String,
    document_id: String,
) -> Result<shared_types::DocketEntryResponse, ServerFnError> {
    use crate::db::get_db;
    use uuid::Uuid;

    let pool = get_db().await;
    let entry_uuid = Uuid::parse_str(&entry_id)
        .map_err(|_| ServerFnError::new("Invalid entry UUID"))?;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    // Verify both exist in this court
    crate::repo::docket::find_by_id(pool, &court_id, entry_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Docket entry not found"))?;

    crate::repo::document::find_by_id(pool, &court_id, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Document not found"))?;

    let updated = crate::repo::docket::link_document(pool, &court_id, entry_uuid, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(shared_types::DocketEntryResponse::from(updated))
}

/// Promote a docket attachment into a canonical document.
#[server]
pub async fn promote_attachment_to_document(
    court_id: String,
    docket_attachment_id: String,
    title: Option<String>,
    document_type: Option<String>,
) -> Result<shared_types::DocumentResponse, ServerFnError> {
    use crate::db::get_db;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&docket_attachment_id)
        .map_err(|_| ServerFnError::new("Invalid attachment UUID"))?;

    // Look up attachment — must belong to this court
    let attachment = crate::repo::attachment::find_by_id(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attachment not found"))?;

    if attachment.uploaded_at.is_none() {
        return Err(ServerFnError::new("Attachment not uploaded yet"));
    }

    // Check for existing document (idempotency)
    if let Some(existing) =
        crate::repo::document::find_by_source_attachment(pool, &court_id, att_uuid)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        return Ok(shared_types::DocumentResponse::from(existing));
    }

    // Resolve case_id from docket entry
    let entry = crate::repo::docket::find_by_id(pool, &court_id, attachment.docket_entry_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Docket entry not found"))?;

    let doc_title = title.unwrap_or_else(|| attachment.filename.clone());
    let doc_type = document_type.as_deref().unwrap_or("Other");
    let checksum = attachment.sha256.clone().unwrap_or_default();

    let document = crate::repo::document::promote_attachment(
        pool,
        &court_id,
        att_uuid,
        entry.case_id,
        &doc_title,
        doc_type,
        &attachment.storage_key,
        attachment.file_size,
        &attachment.content_type,
        &checksum,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Auto-link the new document to the owning docket entry
    let _ = crate::repo::docket::link_document(
        pool,
        &court_id,
        attachment.docket_entry_id,
        document.id,
    )
    .await;

    Ok(shared_types::DocumentResponse::from(document))
}

/// List parties for a case (lightweight, for dropdowns).
#[server]
pub async fn list_case_parties(
    court_id: String,
    case_id: String,
) -> Result<Vec<shared_types::PartyListItem>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| ServerFnError::new("Invalid case UUID"))?;

    let parties = party::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(parties
        .iter()
        .map(|p| shared_types::PartyListItem {
            id: p.id.to_string(),
            name: p.name.clone(),
            party_type: p.party_type.clone(),
        })
        .collect())
}

/// Mark a service record as complete.
#[server]
pub async fn complete_service_record(
    court_id: String,
    record_id: String,
) -> Result<shared_types::ServiceRecordResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::service_record;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&record_id)
        .map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let record = service_record::complete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(shared_types::ServiceRecordResponse::from(record))
}

// ── Filing server functions ────────────────────────────────────

/// Upload a file for a filing. Handles S3 upload server-side and returns
/// the staged upload_id that can be referenced in the filing submission.
#[server]
pub async fn upload_filing_document(
    court_id: String,
    file_name: String,
    content_type: String,
    file_size: i64,
    file_bytes: Vec<u8>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::filing;
    use crate::storage::{ObjectStore, S3ObjectStore};
    use uuid::Uuid;

    let pool = get_db().await;

    if file_name.trim().is_empty() {
        return Err(ServerFnError::new("file_name must not be empty"));
    }

    let file_uuid = Uuid::new_v4();
    let object_key = format!(
        "{}/filings/staging/{}/{}",
        court_id, file_uuid, file_name
    );

    // Create pending upload row
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

    // Return the upload_id for use in filing submission
    Ok(upload.id.to_string())
}

/// Submit an electronic filing. Validates, then atomically creates
/// Document + DocketEntry + Filing. Returns FilingResponse.
#[server]
pub async fn submit_filing(
    court_id: String,
    body: shared_types::ValidateFilingRequest,
) -> Result<shared_types::FilingResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::filing;
    use shared_types::NefSummary;

    let pool = get_db().await;

    let (f, _doc, docket_entry, case_number, _nef) = filing::submit(pool, &court_id, &body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(shared_types::FilingResponse {
        filing_id: f.id.to_string(),
        document_id: f.document_id.map(|u| u.to_string()).unwrap_or_default(),
        docket_entry_id: f.docket_entry_id.map(|u| u.to_string()).unwrap_or_default(),
        case_id: f.case_id.to_string(),
        status: f.status,
        filed_date: f.filed_date.to_rfc3339(),
        nef: NefSummary {
            case_number,
            document_title: body.title.clone(),
            filed_by: body.filed_by.clone(),
            filed_date: f.filed_date.to_rfc3339(),
            docket_number: docket_entry.entry_number,
        },
    })
}

/// Validate a filing request without submitting.
/// Returns ValidateFilingResponse.
#[server]
pub async fn validate_filing_request(
    court_id: String,
    body: shared_types::ValidateFilingRequest,
) -> Result<shared_types::ValidateFilingResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::filing;

    let pool = get_db().await;

    let response = filing::validate(pool, &court_id, &body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(response)
}

/// Retrieve the Notice of Electronic Filing for a given filing.
#[server]
pub async fn get_nef(
    court_id: String,
    filing_id: String,
) -> Result<shared_types::NefResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::nef;
    use uuid::Uuid;

    let pool = get_db().await;
    let filing_uuid = Uuid::parse_str(&filing_id)
        .map_err(|_| ServerFnError::new("Invalid filing UUID"))?;

    let n = nef::find_by_filing(pool, &court_id, filing_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("NEF not found"))?;

    Ok(shared_types::NefResponse::from(n))
}

/// Retrieve a NEF by its primary ID.
#[server]
pub async fn get_nef_by_id(
    court_id: String,
    nef_id: String,
) -> Result<Option<shared_types::NefResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::nef;
    use uuid::Uuid;

    let pool = get_db().await;
    let nef_uuid = Uuid::parse_str(&nef_id)
        .map_err(|_| ServerFnError::new("Invalid NEF UUID"))?;

    let maybe_nef = nef::find_by_id(pool, &court_id, nef_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(maybe_nef.map(shared_types::NefResponse::from))
}

/// Retrieve the NEF for a docket entry (if one exists).
#[server]
pub async fn get_nef_by_docket_entry(
    court_id: String,
    docket_entry_id: String,
) -> Result<Option<shared_types::NefResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::nef;
    use uuid::Uuid;

    let pool = get_db().await;
    let entry_uuid = Uuid::parse_str(&docket_entry_id)
        .map_err(|_| ServerFnError::new("Invalid docket entry UUID"))?;

    let maybe_nef = nef::find_by_docket_entry(pool, &court_id, entry_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(maybe_nef.map(shared_types::NefResponse::from))
}

// ── Filing list / detail server functions ─────────────

/// List all filings for a court with optional search and pagination.
#[server]
pub async fn list_all_filings(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<shared_types::PaginatedResponse<shared_types::FilingListItem>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::filing;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = filing::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::FilingListItem> =
        rows.into_iter().map(shared_types::FilingListItem::from).collect();

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

/// Get a single filing by ID.
#[server]
pub async fn get_filing_by_id(court_id: String, id: String) -> Result<shared_types::FilingListItem, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::filing;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = filing::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Filing not found"))?;

    Ok(shared_types::FilingListItem::from(row))
}
