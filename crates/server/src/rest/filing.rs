use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, FilingResponse, FinalizeFilingUploadResponse, InitFilingUploadRequest,
    InitFilingUploadResponse, JurisdictionInfo, NefSummary, SubmitFilingRequest,
    ValidateFilingRequest, ValidateFilingResponse,
};
use crate::error_convert::SqlxErrorExt;
use crate::storage::{ObjectStore, S3ObjectStore};
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// POST /api/filings/validate
// ---------------------------------------------------------------------------

/// POST /api/filings/validate
///
/// Validate a filing request without submitting it.
/// Always returns 200 — the response body indicates whether the filing is valid.
#[utoipa::path(
    post,
    path = "/api/filings/validate",
    request_body = ValidateFilingRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Validation result", body = ValidateFilingResponse),
        (status = 400, description = "Bad request", body = AppError)
    ),
    tag = "filings"
)]
pub async fn validate_filing(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<ValidateFilingRequest>,
) -> Result<Json<ValidateFilingResponse>, AppError> {
    let response = crate::repo::filing::validate(&pool, &court.0, &body).await?;
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// POST /api/filings
// ---------------------------------------------------------------------------

/// POST /api/filings
///
/// Submit an electronic filing. Atomically creates a Document, DocketEntry,
/// and Filing record. Returns 400 with validation errors if invalid.
#[utoipa::path(
    post,
    path = "/api/filings",
    request_body = SubmitFilingRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Filing submitted", body = FilingResponse),
        (status = 400, description = "Validation failed", body = ValidateFilingResponse)
    ),
    tag = "filings"
)]
pub async fn submit_filing(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<SubmitFilingRequest>,
) -> Result<(StatusCode, Json<FilingResponse>), AppError> {
    let (filing, _doc, docket_entry, case_number, _nef) =
        crate::repo::filing::submit(&pool, &court.0, &body).await?;

    let response = FilingResponse {
        filing_id: filing.id.to_string(),
        document_id: filing.document_id.map(|u| u.to_string()).unwrap_or_default(),
        docket_entry_id: filing.docket_entry_id.map(|u| u.to_string()).unwrap_or_default(),
        case_id: filing.case_id.to_string(),
        status: filing.status,
        filed_date: filing.filed_date.to_rfc3339(),
        nef: NefSummary {
            case_number,
            document_title: body.title.clone(),
            filed_by: body.filed_by.clone(),
            filed_date: filing.filed_date.to_rfc3339(),
            docket_number: docket_entry.entry_number,
        },
    };

    Ok((StatusCode::CREATED, Json(response)))
}

// ---------------------------------------------------------------------------
// GET /api/filings/jurisdictions
// ---------------------------------------------------------------------------

/// GET /api/filings/jurisdictions
///
/// Return jurisdiction info for the requesting court.
#[utoipa::path(
    get,
    path = "/api/filings/jurisdictions",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Jurisdiction info", body = Vec<JurisdictionInfo>)
    ),
    tag = "filings"
)]
pub async fn list_jurisdictions(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<JurisdictionInfo>>, AppError> {
    let rows = sqlx::query_as!(
        JurisdictionInfo,
        r#"
        SELECT id as court_id, name, court_type
        FROM courts
        WHERE id = $1
        "#,
        court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// POST /api/filings/upload/init
// ---------------------------------------------------------------------------

/// POST /api/filings/upload/init
///
/// Initiate a staged file upload for a filing. Returns a presigned PUT URL.
/// Client uploads directly to S3/RustFS, then calls finalize.
#[utoipa::path(
    post,
    path = "/api/filings/upload/init",
    request_body = InitFilingUploadRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Upload initiated", body = InitFilingUploadResponse),
        (status = 400, description = "Bad request", body = AppError)
    ),
    tag = "filings"
)]
pub async fn init_filing_upload(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<InitFilingUploadRequest>,
) -> Result<(StatusCode, Json<InitFilingUploadResponse>), AppError> {
    if body.filename.trim().is_empty() {
        return Err(AppError::bad_request("filename must not be empty"));
    }
    if body.file_size <= 0 {
        return Err(AppError::bad_request("file_size must be positive"));
    }

    let file_uuid = Uuid::new_v4();
    let object_key = format!(
        "{}/filings/staging/{}/{}",
        court.0, file_uuid, body.filename
    );

    let upload = crate::repo::filing::create_pending_upload(
        &pool,
        &court.0,
        &body.filename,
        body.file_size,
        &body.content_type,
        &object_key,
    )
    .await?;

    let store = S3ObjectStore::from_env();
    let (presign_url, required_headers) = store
        .presign_put(&object_key, &body.content_type)
        .await
        .map_err(|e| AppError::internal(format!("Failed to generate presigned URL: {}", e)))?;

    let response = InitFilingUploadResponse {
        upload_id: upload.id.to_string(),
        presign_url,
        object_key,
        required_headers,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

// ---------------------------------------------------------------------------
// POST /api/filings/upload/{id}/finalize
// ---------------------------------------------------------------------------

/// POST /api/filings/upload/{id}/finalize
///
/// Finalize a staged filing upload by verifying the object exists in S3
/// and marking it as uploaded.
#[utoipa::path(
    post,
    path = "/api/filings/upload/{id}/finalize",
    params(
        ("id" = String, Path, description = "Upload UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Upload finalized", body = FinalizeFilingUploadResponse),
        (status = 400, description = "Not yet uploaded", body = AppError),
        (status = 404, description = "Upload not found", body = AppError)
    ),
    tag = "filings"
)]
pub async fn finalize_filing_upload(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<FinalizeFilingUploadResponse>, AppError> {
    let upload_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid upload UUID format"))?;

    let upload = crate::repo::filing::find_upload_by_id(&pool, &court.0, upload_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Filing upload {} not found", id)))?;

    if upload.uploaded_at.is_some() {
        return Err(AppError::bad_request("Upload already finalized"));
    }

    // Verify object exists in S3
    let store = S3ObjectStore::from_env();
    let exists = store
        .head(&upload.storage_key)
        .await
        .map_err(|e| AppError::internal(format!("HEAD check failed: {}", e)))?;

    if !exists {
        return Err(AppError::bad_request("Object not yet uploaded to storage"));
    }

    crate::repo::filing::mark_upload_finalized(&pool, &court.0, upload_uuid).await?;

    // Re-fetch to get updated timestamps
    let updated = crate::repo::filing::find_upload_by_id(&pool, &court.0, upload_uuid)
        .await?
        .ok_or_else(|| AppError::not_found("Upload not found after update"))?;

    Ok(Json(FinalizeFilingUploadResponse {
        upload_id: updated.id.to_string(),
        filename: updated.filename,
        file_size: updated.file_size,
        content_type: updated.content_type,
        uploaded_at: updated.uploaded_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
    }))
}
