use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, DocumentEventResponse, DocumentResponse, PromoteAttachmentRequest,
    ReplaceDocumentRequest, SealDocumentRequest, SealingLevel, UserRole, VALID_DOCUMENT_TYPES,
};
use crate::auth::extractors::AuthRequired;
use crate::tenant::CourtId;

/// Require clerk or judge role for a specific court.
fn require_clerk_or_judge(claims: &crate::auth::jwt::Claims, court_id: &str) -> Result<(), AppError> {
    let role = crate::auth::court_role::resolve_court_role(claims, court_id);
    match role {
        UserRole::Clerk | UserRole::Judge | UserRole::Admin => Ok(()),
        _ => Err(AppError::forbidden("clerk or judge role required for this court")),
    }
}

/// POST /api/documents/from-attachment
///
/// Promote a docket attachment into a canonical document entry.
/// The attachment must belong to the tenant and must have been uploaded
/// (uploaded_at IS NOT NULL). If a document already exists for this
/// attachment, the existing document is returned (idempotent).
#[utoipa::path(
    post,
    path = "/api/documents/from-attachment",
    request_body = PromoteAttachmentRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Document created from attachment", body = DocumentResponse),
        (status = 200, description = "Document already exists for this attachment", body = DocumentResponse),
        (status = 400, description = "Invalid request or attachment not uploaded", body = AppError),
        (status = 404, description = "Attachment not found", body = AppError)
    ),
    tag = "documents"
)]
pub async fn promote_attachment(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PromoteAttachmentRequest>,
) -> Result<(StatusCode, Json<DocumentResponse>), AppError> {
    let att_uuid = Uuid::parse_str(&body.docket_attachment_id)
        .map_err(|_| AppError::bad_request("Invalid docket_attachment_id UUID format"))?;

    // Validate document_type if provided
    let doc_type = body.document_type.as_deref().unwrap_or("Other");
    if !VALID_DOCUMENT_TYPES.contains(&doc_type) {
        return Err(AppError::bad_request(format!(
            "Invalid document_type '{}'. Valid values: {}",
            doc_type,
            VALID_DOCUMENT_TYPES.join(", ")
        )));
    }

    // Look up the attachment — must belong to this court
    let attachment = crate::repo::attachment::find_by_id(&pool, &court.0, att_uuid)
        .await?
        .ok_or_else(|| AppError::not_found("Attachment not found"))?;

    // Must be uploaded
    if attachment.uploaded_at.is_none() {
        return Err(AppError::bad_request(
            "Attachment has not been uploaded yet. Finalize the upload first.",
        ));
    }

    // Check for existing document (idempotency before the INSERT)
    if let Some(existing) =
        crate::repo::document::find_by_source_attachment(&pool, &court.0, att_uuid).await?
    {
        return Ok((StatusCode::OK, Json(DocumentResponse::from(existing))));
    }

    // Resolve the case_id from the docket entry
    let entry = crate::repo::docket::find_by_id(&pool, &court.0, attachment.docket_entry_id)
        .await?
        .ok_or_else(|| {
            AppError::internal("Attachment's docket entry not found — data integrity issue")
        })?;

    let title = body
        .title
        .unwrap_or_else(|| attachment.filename.clone());

    let checksum = attachment.sha256.clone().unwrap_or_default();

    let document = crate::repo::document::promote_attachment(
        &pool,
        &court.0,
        att_uuid,
        entry.case_id,
        &title,
        doc_type,
        &attachment.storage_key,
        attachment.file_size,
        &attachment.content_type,
        &checksum,
    )
    .await?;

    // Auto-link the new document to the owning docket entry
    let _ = crate::repo::docket::link_document(
        &pool,
        &court.0,
        attachment.docket_entry_id,
        document.id,
    )
    .await;

    Ok((StatusCode::CREATED, Json(DocumentResponse::from(document))))
}

/// POST /api/documents/{id}/seal
///
/// Seal a document. Requires clerk or judge role.
#[utoipa::path(
    post,
    path = "/api/documents/{id}/seal",
    request_body = SealDocumentRequest,
    params(
        ("id" = String, Path, description = "Document UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Document sealed", body = DocumentResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 403, description = "Insufficient role", body = AppError),
        (status = 404, description = "Document not found", body = AppError)
    ),
    tag = "documents"
)]
pub async fn seal_document(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    auth: AuthRequired,
    Path(id): Path<String>,
    Json(body): Json<SealDocumentRequest>,
) -> Result<Json<DocumentResponse>, AppError> {
    require_clerk_or_judge(&auth.0, &court.0)?;
    let doc_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid document UUID"))?;

    let level = SealingLevel::from_db_str(&body.sealing_level);
    if !level.is_sealed() {
        return Err(AppError::bad_request(
            "sealing_level must be one of: SealedCourtOnly, SealedCaseParticipants, SealedAttorneysOnly",
        ));
    }

    let motion_id = body
        .motion_id
        .as_deref()
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::bad_request("Invalid motion_id UUID"))?;

    let doc = crate::repo::document::seal(
        &pool, &court.0, doc_uuid, &level, &body.reason_code, motion_id,
    )
    .await?;

    // Audit event
    let _ = crate::repo::document_event::create(
        &pool,
        &court.0,
        doc_uuid,
        "sealed",
        &auth.0.email,
        serde_json::json!({
            "sealing_level": body.sealing_level,
            "reason_code": body.reason_code,
            "motion_id": body.motion_id,
        }),
    )
    .await;

    Ok(Json(DocumentResponse::from(doc)))
}

/// POST /api/documents/{id}/unseal
///
/// Unseal a document. Requires clerk or judge role.
#[utoipa::path(
    post,
    path = "/api/documents/{id}/unseal",
    params(
        ("id" = String, Path, description = "Document UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Document unsealed", body = DocumentResponse),
        (status = 403, description = "Insufficient role", body = AppError),
        (status = 404, description = "Document not found", body = AppError)
    ),
    tag = "documents"
)]
pub async fn unseal_document(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    auth: AuthRequired,
    Path(id): Path<String>,
) -> Result<Json<DocumentResponse>, AppError> {
    require_clerk_or_judge(&auth.0, &court.0)?;
    let doc_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid document UUID"))?;

    let doc = crate::repo::document::unseal(&pool, &court.0, doc_uuid).await?;

    // Audit event
    let _ = crate::repo::document_event::create(
        &pool,
        &court.0,
        doc_uuid,
        "unsealed",
        &auth.0.email,
        serde_json::json!({}),
    )
    .await;

    Ok(Json(DocumentResponse::from(doc)))
}

/// POST /api/documents/{id}/replace
///
/// Replace a document with a corrected version. The original is stricken.
/// Requires clerk role.
#[utoipa::path(
    post,
    path = "/api/documents/{id}/replace",
    request_body = ReplaceDocumentRequest,
    params(
        ("id" = String, Path, description = "Document UUID to replace"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Replacement document created", body = DocumentResponse),
        (status = 400, description = "Invalid request or already replaced", body = AppError),
        (status = 403, description = "Insufficient role", body = AppError),
        (status = 404, description = "Document or upload not found", body = AppError)
    ),
    tag = "documents"
)]
pub async fn replace_document(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    auth: AuthRequired,
    Path(id): Path<String>,
    Json(body): Json<ReplaceDocumentRequest>,
) -> Result<(StatusCode, Json<DocumentResponse>), AppError> {
    require_clerk_or_judge(&auth.0, &court.0)?;
    let doc_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid document UUID"))?;

    let upload_uuid = Uuid::parse_str(&body.upload_id)
        .map_err(|_| AppError::bad_request("Invalid upload_id UUID"))?;

    // Resolve the upload metadata
    let upload = crate::repo::filing::find_upload_by_id(&pool, &court.0, upload_uuid)
        .await?
        .ok_or_else(|| AppError::not_found("Upload not found"))?;

    if upload.uploaded_at.is_none() {
        return Err(AppError::bad_request("Upload has not been finalized"));
    }

    // Get original document to inherit title if not specified
    let original = crate::repo::document::find_by_id(&pool, &court.0, doc_uuid)
        .await?
        .ok_or_else(|| AppError::not_found("Document not found"))?;

    let title = body.title.as_deref().unwrap_or(&original.title);

    let replacement = crate::repo::document::replace(
        &pool,
        &court.0,
        doc_uuid,
        title,
        &upload.storage_key,
        upload.file_size,
        &upload.content_type,
        &upload.sha256.unwrap_or_default(),
    )
    .await?;

    // Audit event
    let _ = crate::repo::document_event::create(
        &pool,
        &court.0,
        doc_uuid,
        "replaced",
        &auth.0.email,
        serde_json::json!({
            "replacement_document_id": replacement.id.to_string(),
        }),
    )
    .await;

    Ok((StatusCode::CREATED, Json(DocumentResponse::from(replacement))))
}

/// POST /api/documents/{id}/strike
///
/// Strike a document from the record without replacement. Requires clerk role.
#[utoipa::path(
    post,
    path = "/api/documents/{id}/strike",
    params(
        ("id" = String, Path, description = "Document UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Document stricken", body = DocumentResponse),
        (status = 403, description = "Insufficient role", body = AppError),
        (status = 404, description = "Document not found", body = AppError)
    ),
    tag = "documents"
)]
pub async fn strike_document(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    auth: AuthRequired,
    Path(id): Path<String>,
) -> Result<Json<DocumentResponse>, AppError> {
    require_clerk_or_judge(&auth.0, &court.0)?;
    let doc_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid document UUID"))?;

    let doc = crate::repo::document::strike(&pool, &court.0, doc_uuid).await?;

    // Audit event
    let _ = crate::repo::document_event::create(
        &pool,
        &court.0,
        doc_uuid,
        "stricken",
        &auth.0.email,
        serde_json::json!({}),
    )
    .await;

    Ok(Json(DocumentResponse::from(doc)))
}

/// GET /api/documents/{id}/events
///
/// List audit events for a document. Requires clerk or judge role.
#[utoipa::path(
    get,
    path = "/api/documents/{id}/events",
    params(
        ("id" = String, Path, description = "Document UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Document events", body = Vec<DocumentEventResponse>),
        (status = 403, description = "Insufficient role", body = AppError),
    ),
    tag = "documents"
)]
pub async fn list_document_events(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    auth: AuthRequired,
    Path(id): Path<String>,
) -> Result<Json<Vec<DocumentEventResponse>>, AppError> {
    require_clerk_or_judge(&auth.0, &court.0)?;
    let doc_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid document UUID"))?;

    let events = crate::repo::document_event::list_by_document(&pool, &court.0, doc_uuid).await?;
    let response: Vec<DocumentEventResponse> = events.into_iter().map(Into::into).collect();
    Ok(Json(response))
}
