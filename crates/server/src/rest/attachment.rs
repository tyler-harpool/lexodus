use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CreateAttachmentRequest, CreateAttachmentResponse, DocketAttachmentResponse,
};
use crate::storage::{ObjectStore, S3ObjectStore};
use crate::tenant::CourtId;

/// Optional query param for finalize-on-create.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct CreateAttachmentParams {
    pub finalize: Option<bool>,
}

/// GET /api/docket/entries/{entry_id}/attachments
///
/// List uploaded attachments for a docket entry.
#[utoipa::path(
    get,
    path = "/api/docket/entries/{entry_id}/attachments",
    params(
        ("entry_id" = String, Path, description = "Docket entry UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attachment list", body = Vec<DocketAttachmentResponse>),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "docket"
)]
pub async fn list_entry_attachments(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(entry_id): Path<String>,
) -> Result<Json<Vec<DocketAttachmentResponse>>, AppError> {
    let entry_uuid = Uuid::parse_str(&entry_id)
        .map_err(|_| AppError::bad_request("Invalid entry_id UUID format"))?;

    // Verify the entry belongs to this tenant
    crate::repo::docket::find_by_id(&pool, &court.0, entry_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Docket entry {} not found", entry_id)))?;

    let attachments = crate::repo::attachment::list_by_entry(&pool, &court.0, entry_uuid).await?;

    let response: Vec<DocketAttachmentResponse> = attachments
        .into_iter()
        .map(DocketAttachmentResponse::from)
        .collect();

    Ok(Json(response))
}

/// POST /api/docket/entries/{entry_id}/attachments
///
/// Initiate a presigned upload for a new attachment.
#[utoipa::path(
    post,
    path = "/api/docket/entries/{entry_id}/attachments",
    request_body = CreateAttachmentRequest,
    params(
        ("entry_id" = String, Path, description = "Docket entry UUID"),
        ("finalize" = Option<bool>, Query, description = "If true, verify object and mark uploaded"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Presigned upload initiated", body = CreateAttachmentResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Entry not found", body = AppError)
    ),
    tag = "docket"
)]
pub async fn create_entry_attachment(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(entry_id): Path<String>,
    Query(params): Query<CreateAttachmentParams>,
    Json(body): Json<CreateAttachmentRequest>,
) -> Result<(StatusCode, Json<CreateAttachmentResponse>), AppError> {
    let entry_uuid = Uuid::parse_str(&entry_id)
        .map_err(|_| AppError::bad_request("Invalid entry_id UUID format"))?;

    if body.file_name.trim().is_empty() {
        return Err(AppError::bad_request("file_name must not be empty"));
    }

    // Verify the entry belongs to this tenant
    crate::repo::docket::find_by_id(&pool, &court.0, entry_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Docket entry {} not found", entry_id)))?;

    // Generate unique object key
    let file_uuid = Uuid::new_v4();
    let object_key = format!(
        "{}/docket/{}/{}/{}",
        court.0, entry_id, file_uuid, body.file_name
    );

    // Insert pending DB row (uploaded_at = NULL)
    let attachment = crate::repo::attachment::create_pending(
        &pool,
        &court.0,
        entry_uuid,
        &body.file_name,
        body.file_size,
        &body.content_type,
        &object_key,
    )
    .await?;

    // Generate presigned PUT URL with SSE enforcement
    let store = S3ObjectStore::from_env();
    let (presign_url, required_headers) = store
        .presign_put(&object_key, &body.content_type)
        .await
        .map_err(|e| AppError::internal(format!("Failed to generate presigned URL: {}", e)))?;

    // Optional finalize: if ?finalize=true, check HEAD and mark uploaded
    if params.finalize.unwrap_or(false) {
        let exists = store
            .head(&object_key)
            .await
            .map_err(|e| AppError::internal(format!("HEAD check failed: {}", e)))?;

        if exists {
            crate::repo::attachment::mark_uploaded(&pool, &court.0, attachment.id).await?;
        }
    }

    let response = CreateAttachmentResponse {
        attachment_id: attachment.id.to_string(),
        presign_url,
        object_key,
        required_headers,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// POST /api/docket/attachments/{attachment_id}/finalize
///
/// Finalize an attachment upload by checking HEAD and marking uploaded_at.
#[utoipa::path(
    post,
    path = "/api/docket/attachments/{attachment_id}/finalize",
    params(
        ("attachment_id" = String, Path, description = "Attachment UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Attachment finalized", body = DocketAttachmentResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "docket"
)]
pub async fn finalize_attachment(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(attachment_id): Path<String>,
) -> Result<Json<DocketAttachmentResponse>, AppError> {
    let att_uuid = Uuid::parse_str(&attachment_id)
        .map_err(|_| AppError::bad_request("Invalid attachment_id UUID format"))?;

    let attachment = crate::repo::attachment::find_by_id(&pool, &court.0, att_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attachment {} not found", attachment_id)))?;

    // Verify the object exists in S3
    let store = S3ObjectStore::from_env();
    let exists = store
        .head(&attachment.storage_key)
        .await
        .map_err(|e| AppError::internal(format!("HEAD check failed: {}", e)))?;

    if !exists {
        return Err(AppError::bad_request(
            "Object not yet uploaded to storage",
        ));
    }

    crate::repo::attachment::mark_uploaded(&pool, &court.0, att_uuid).await?;

    // Re-fetch to get updated timestamps
    let updated = crate::repo::attachment::find_by_id(&pool, &court.0, att_uuid)
        .await?
        .ok_or_else(|| AppError::not_found("Attachment not found after update"))?;

    Ok(Json(DocketAttachmentResponse::from(updated)))
}

/// GET /api/docket/attachments/{attachment_id}/download
///
/// Get a presigned download URL for an attachment.
#[utoipa::path(
    get,
    path = "/api/docket/attachments/{attachment_id}/download",
    params(
        ("attachment_id" = String, Path, description = "Attachment UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Download URL"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "docket"
)]
pub async fn download_attachment(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(attachment_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let att_uuid = Uuid::parse_str(&attachment_id)
        .map_err(|_| AppError::bad_request("Invalid attachment_id UUID format"))?;

    let attachment = crate::repo::attachment::find_by_id(&pool, &court.0, att_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attachment {} not found", attachment_id)))?;

    let store = S3ObjectStore::from_env();
    let url = store
        .presign_get(&attachment.storage_key)
        .await
        .map_err(|e| AppError::internal(format!("Failed to generate download URL: {}", e)))?;

    Ok(Json(serde_json::json!({
        "download_url": url,
        "filename": attachment.filename,
        "content_type": attachment.content_type,
    })))
}

/// GET /api/docket/attachments/{attachment_id}/file
///
/// Proxy download — streams the file bytes through the server with proper
/// Content-Type and Content-Disposition headers.  Works as a plain `<a href>`
/// on web, desktop, and mobile (no JS required).
#[utoipa::path(
    get,
    path = "/api/docket/attachments/{attachment_id}/file",
    params(
        ("attachment_id" = String, Path, description = "Attachment UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "File bytes"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "docket"
)]
pub async fn serve_attachment_file(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(attachment_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let att_uuid = Uuid::parse_str(&attachment_id)
        .map_err(|_| AppError::bad_request("Invalid attachment_id UUID format"))?;

    let attachment = crate::repo::attachment::find_by_id(&pool, &court.0, att_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Attachment {} not found", attachment_id)))?;

    let store = S3ObjectStore::from_env();
    let bytes = store
        .get(&attachment.storage_key)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("NoSuchKey") || msg.contains("not found") || msg.contains("NotFound") {
                AppError::not_found(format!("File not found in storage for attachment {}", attachment_id))
            } else {
                AppError::internal(format!("Failed to download file: {}", e))
            }
        })?;

    let content_disposition = format!(
        "attachment; filename=\"{}\"",
        attachment.filename.replace('"', "\\\"")
    );

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, attachment.content_type),
            (header::CONTENT_DISPOSITION, content_disposition),
        ],
        bytes,
    ))
}
