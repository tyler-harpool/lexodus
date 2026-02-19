use shared_types::{
    AppError, CreateDocketEntryRequest, DocketEntry, Document, Filing, FilingUpload,
    FilingValidationError, Nef, ValidateFilingRequest, ValidateFilingResponse,
    VALID_DOCUMENT_TYPES,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

// ---------------------------------------------------------------------------
// Filing queries (list + detail)
// ---------------------------------------------------------------------------

/// Find a single filing by its ID within a court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Filing>, AppError> {
    sqlx::query_as!(
        Filing,
        r#"
        SELECT id, court_id, case_id, filing_type, filed_by, filed_date,
               status, validation_errors, document_id, docket_entry_id, created_at
        FROM filings
        WHERE id = $1 AND court_id = $2
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// List all filings for a court with optional search and pagination.
/// Search matches against filing_type or filed_by (case-insensitive).
pub async fn list_all(
    pool: &Pool<Postgres>,
    court_id: &str,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<Filing>, i64), AppError> {
    let search = q.map(|s| format!("%{}%", s.to_lowercase()));

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!" FROM filings
        WHERE court_id = $1
          AND ($2::TEXT IS NULL
               OR LOWER(filing_type) LIKE $2
               OR LOWER(filed_by) LIKE $2)
        "#,
        court_id,
        search.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        Filing,
        r#"
        SELECT id, court_id, case_id, filing_type, filed_by, filed_date,
               status, validation_errors, document_id, docket_entry_id, created_at
        FROM filings
        WHERE court_id = $1
          AND ($2::TEXT IS NULL
               OR LOWER(filing_type) LIKE $2
               OR LOWER(filed_by) LIKE $2)
        ORDER BY filed_date DESC
        LIMIT $3 OFFSET $4
        "#,
        court_id,
        search.as_deref(),
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, total))
}

// ---------------------------------------------------------------------------
// Document-type → docket entry-type mapping
// ---------------------------------------------------------------------------

/// Map a document type (e.g. "Motion") to its docket entry type (e.g. "motion").
fn document_type_to_entry_type(doc_type: &str) -> &'static str {
    match doc_type {
        "Motion" => "motion",
        "Order" => "order",
        "Brief" | "Memorandum" => "motion",
        "Declaration" | "Affidavit" | "Exhibit" => "exhibit",
        "Transcript" => "transcript",
        "Notice" => "notice",
        "Subpoena" => "subpoena",
        "Indictment" => "indictment",
        "Judgment" => "judgment",
        "Verdict" => "verdict",
        _ => "other",
    }
}

/// Map a document type to a filing_type matching the filings DB CHECK constraint.
/// Valid filing_types: Initial, Response, Reply, Motion, Notice, Stipulation,
///                     Supplement, Amendment, Exhibit, Certificate, Other
fn document_type_to_filing_type(doc_type: &str) -> &'static str {
    match doc_type {
        "Motion" | "Brief" | "Memorandum" => "Motion",
        "Order" | "Judgment" | "Verdict" => "Other",
        "Declaration" | "Affidavit" => "Certificate",
        "Exhibit" => "Exhibit",
        "Notice" => "Notice",
        "Subpoena" | "Warrant" => "Other",
        "Indictment" | "Plea Agreement" => "Initial",
        "Transcript" => "Supplement",
        _ => "Other",
    }
}

// ---------------------------------------------------------------------------
// Filing upload staging
// ---------------------------------------------------------------------------

/// Create a pending filing upload row (uploaded_at = NULL).
pub async fn create_pending_upload(
    pool: &Pool<Postgres>,
    court_id: &str,
    filename: &str,
    file_size: i64,
    content_type: &str,
    storage_key: &str,
) -> Result<FilingUpload, AppError> {
    sqlx::query_as!(
        FilingUpload,
        r#"
        INSERT INTO filing_uploads (court_id, filename, file_size, content_type, storage_key)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, court_id, filename, file_size, content_type, storage_key,
                  sha256, uploaded_at, created_at
        "#,
        court_id,
        filename,
        file_size,
        content_type,
        storage_key,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Mark a filing upload as uploaded (sets uploaded_at = NOW()).
pub async fn mark_upload_finalized(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        r#"
        UPDATE filing_uploads
        SET uploaded_at = NOW()
        WHERE id = $1 AND court_id = $2 AND uploaded_at IS NULL
        "#,
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// Find a filing upload by ID within a court.
pub async fn find_upload_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<FilingUpload>, AppError> {
    sqlx::query_as!(
        FilingUpload,
        r#"
        SELECT id, court_id, filename, file_size, content_type, storage_key,
               sha256, uploaded_at, created_at
        FROM filing_uploads
        WHERE id = $1 AND court_id = $2
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

// ---------------------------------------------------------------------------
// Filing validation
// ---------------------------------------------------------------------------

/// Validate a filing request. Always returns a response (never errors).
pub async fn validate(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: &ValidateFilingRequest,
) -> Result<ValidateFilingResponse, AppError> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Title required
    if req.title.trim().is_empty() {
        errors.push(FilingValidationError {
            field: "title".to_string(),
            message: "Title is required".to_string(),
            severity: "error".to_string(),
        });
    }

    // Filed-by required
    if req.filed_by.trim().is_empty() {
        errors.push(FilingValidationError {
            field: "filed_by".to_string(),
            message: "Filed-by is required".to_string(),
            severity: "error".to_string(),
        });
    }

    // Document type must be valid
    if !VALID_DOCUMENT_TYPES.contains(&req.document_type.as_str()) {
        errors.push(FilingValidationError {
            field: "document_type".to_string(),
            message: format!(
                "Invalid document_type '{}'. Valid values: {}",
                req.document_type,
                VALID_DOCUMENT_TYPES.join(", ")
            ),
            severity: "error".to_string(),
        });
    }

    // Case must exist in this tenant
    let case_uuid = match Uuid::parse_str(&req.case_id) {
        Ok(u) => u,
        Err(_) => {
            errors.push(FilingValidationError {
                field: "case_id".to_string(),
                message: "Invalid case_id UUID format".to_string(),
                severity: "error".to_string(),
            });
            return Ok(ValidateFilingResponse {
                valid: false,
                errors,
                warnings,
            });
        }
    };

    let case = crate::repo::case::find_by_id(pool, court_id, case_uuid).await?;
    if case.is_none() {
        errors.push(FilingValidationError {
            field: "case_id".to_string(),
            message: "Case not found".to_string(),
            severity: "error".to_string(),
        });
    }

    // Upload must exist and be finalized if provided
    if let Some(ref upload_id_str) = req.upload_id {
        match Uuid::parse_str(upload_id_str) {
            Ok(upload_uuid) => {
                let upload = find_upload_by_id(pool, court_id, upload_uuid).await?;
                match upload {
                    None => {
                        errors.push(FilingValidationError {
                            field: "upload_id".to_string(),
                            message: "Upload not found".to_string(),
                            severity: "error".to_string(),
                        });
                    }
                    Some(u) if u.uploaded_at.is_none() => {
                        errors.push(FilingValidationError {
                            field: "upload_id".to_string(),
                            message: "Upload has not been finalized".to_string(),
                            severity: "error".to_string(),
                        });
                    }
                    _ => {}
                }
            }
            Err(_) => {
                errors.push(FilingValidationError {
                    field: "upload_id".to_string(),
                    message: "Invalid upload_id UUID format".to_string(),
                    severity: "error".to_string(),
                });
            }
        }
    } else {
        warnings.push(FilingValidationError {
            field: "upload_id".to_string(),
            message: "No file attached to filing".to_string(),
            severity: "warning".to_string(),
        });
    }

    let valid = errors.is_empty();
    Ok(ValidateFilingResponse {
        valid,
        errors,
        warnings,
    })
}

// ---------------------------------------------------------------------------
// Filing submission
// ---------------------------------------------------------------------------

/// Submit an electronic filing. Creates Document + DocketEntry + Filing,
/// then auto-seeds service records for all active parties and persists a NEF.
///
/// Returns (Filing, Document, DocketEntry, case_number, Nef) on success.
pub async fn submit(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: &ValidateFilingRequest,
) -> Result<(Filing, Document, DocketEntry, String, Nef), AppError> {
    // 1. Validate first
    let validation = validate(pool, court_id, req).await?;
    if !validation.valid {
        return Err(AppError::bad_request(
            serde_json::to_string(&validation).unwrap_or_else(|_| "Validation failed".to_string()),
        ));
    }

    let case_uuid = Uuid::parse_str(&req.case_id)
        .map_err(|_| AppError::bad_request("Invalid case_id UUID format"))?;

    // 2. Look up case for case_number in NEF
    let case = crate::repo::case::find_by_id(pool, court_id, case_uuid)
        .await?
        .ok_or_else(|| AppError::not_found("Case not found"))?;

    // 3. Resolve upload metadata if provided
    let (storage_key, file_size, content_type, checksum) = if let Some(ref upload_id_str) = req.upload_id {
        let upload_uuid = Uuid::parse_str(upload_id_str)
            .map_err(|_| AppError::bad_request("Invalid upload_id UUID format"))?;
        let upload = find_upload_by_id(pool, court_id, upload_uuid)
            .await?
            .ok_or_else(|| AppError::bad_request("Upload not found"))?;
        (
            upload.storage_key,
            upload.file_size,
            upload.content_type,
            upload.sha256.unwrap_or_default(),
        )
    } else {
        (
            format!("{}/filings/{}/placeholder", court_id, Uuid::new_v4()),
            0_i64,
            "application/octet-stream".to_string(),
            String::new(),
        )
    };

    let is_sealed = req.is_sealed.unwrap_or(false);
    let sealing_level = req
        .sealing_level
        .as_deref()
        .unwrap_or(if is_sealed { "SealedCourtOnly" } else { "Public" });
    let seal_reason = if is_sealed {
        req.reason_code.as_deref()
    } else {
        None
    };

    // 4. INSERT Document
    let doc = sqlx::query_as!(
        Document,
        r#"
        INSERT INTO documents
            (court_id, case_id, title, document_type, storage_key,
             checksum, file_size, content_type, is_sealed, uploaded_by,
             sealing_level, seal_reason_code)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, court_id, case_id, title, document_type, storage_key,
                  checksum, file_size, content_type, is_sealed, uploaded_by,
                  source_attachment_id, created_at,
                  sealing_level, seal_reason_code, seal_motion_id,
                  replaced_by_document_id, is_stricken
        "#,
        court_id,
        case_uuid,
        req.title.as_str(),
        req.document_type.as_str(),
        storage_key.as_str(),
        checksum.as_str(),
        file_size,
        content_type.as_str(),
        is_sealed,
        req.filed_by.as_str(),
        sealing_level,
        seal_reason as Option<&str>,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    // 5. Create DocketEntry via repo (handles entry_number auto-increment)
    let entry_type = document_type_to_entry_type(&req.document_type);
    let docket_req = CreateDocketEntryRequest {
        case_id: case_uuid,
        entry_type: entry_type.to_string(),
        description: format!("Filing: {}", req.title),
        filed_by: Some(req.filed_by.clone()),
        document_id: Some(doc.id),
        is_sealed,
        is_ex_parte: false,
        page_count: None,
        related_entries: vec![],
        service_list: vec![],
    };
    let docket_entry = crate::repo::docket::create(pool, court_id, docket_req).await?;

    // 6. INSERT Filing
    let filing_type = document_type_to_filing_type(&req.document_type);
    let filing = sqlx::query_as!(
        Filing,
        r#"
        INSERT INTO filings
            (court_id, case_id, filing_type, filed_by, status,
             document_id, docket_entry_id, validation_errors)
        VALUES ($1, $2, $3, $4, 'Filed', $5, $6, '[]'::jsonb)
        RETURNING id, court_id, case_id, filing_type, filed_by, filed_date,
                  status, validation_errors, document_id, docket_entry_id, created_at
        "#,
        court_id,
        case_uuid,
        filing_type,
        req.filed_by.as_str(),
        doc.id,
        docket_entry.id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    // 7. Auto-seed service records for all active parties in the case.
    //    Electronic service is auto-successful with proof filed; non-electronic needs manual completion.
    let parties =
        crate::repo::party::list_service_info_by_case(pool, court_id, case_uuid).await?;

    for party in &parties {
        let method = party.service_method.as_deref().unwrap_or("Electronic");
        let electronic = method == "Electronic";
        sqlx::query!(
            r#"
            INSERT INTO service_records
                (court_id, document_id, party_id, service_method, served_by,
                 successful, proof_of_service_filed, attempts)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 1)
            "#,
            court_id,
            doc.id,
            party.id,
            method,
            req.filed_by.as_str(),
            electronic,
            electronic,
        )
        .execute(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?;
    }

    // 8. Persist NEF record
    let nef = crate::repo::nef::create(
        pool,
        court_id,
        &filing,
        &doc,
        &docket_entry,
        &case.case_number,
    )
    .await?;

    // 9. Fire-and-forget NEF delivery (email + SMS)
    {
        let pool = pool.clone();
        let court_id = court_id.to_string();
        let nef_clone = nef.clone();
        let doc_title = req.title.clone();
        let case_num = case.case_number.clone();
        let parties_clone = parties.clone();
        tokio::spawn(async move {
            crate::nef_delivery::deliver_nef(
                &pool,
                &court_id,
                &nef_clone,
                &doc_title,
                &case_num,
                &parties_clone,
            )
            .await;
        });
    }

    Ok((filing, doc, docket_entry, case.case_number, nef))
}
