use dioxus::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Docket server functions
// ═══════════════════════════════════════════════════════════════

/// Search docket entries with filters.
#[server]
pub async fn search_docket_entries(
    court_id: String,
    case_id: Option<String>,
    entry_type: Option<String>,
    q: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<shared_types::DocketSearchResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::docket;
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

    let (entries, total) = docket::search(
        pool, &court_id, case_uuid,
        entry_type.as_deref().filter(|s| !s.is_empty()),
        q.as_deref().filter(|s| !s.is_empty()),
        offset, limit,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(shared_types::DocketSearchResponse {
        entries: entries.into_iter().map(shared_types::DocketEntryResponse::from).collect(),
        total,
    })
}

/// Get a single docket entry by ID.
#[server]
pub async fn get_docket_entry(court_id: String, id: String) -> Result<shared_types::DocketEntryResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::docket;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let entry = docket::find_by_id(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Docket entry not found"))?;

    Ok(shared_types::DocketEntryResponse::from(entry))
}

/// Create a new docket entry.
#[server]
pub async fn create_docket_entry(court_id: String, body: shared_types::CreateDocketEntryRequest) -> Result<shared_types::DocketEntryResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::docket;

    let pool = get_db().await;
    let entry = docket::create(pool, &court_id, body).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(shared_types::DocketEntryResponse::from(entry))
}

/// Delete a docket entry by ID.
#[server]
pub async fn delete_docket_entry(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::docket;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let deleted = docket::delete(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if deleted {
        Ok(())
    } else {
        Err(ServerFnError::new("Docket entry not found"))
    }
}

/// List docket entries for a specific case.
#[server]
pub async fn get_case_docket(
    court_id: String,
    case_id: String,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<shared_types::DocketSearchResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::docket;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| ServerFnError::new("Invalid case UUID"))?;
    let offset = offset.unwrap_or(0).max(0);
    let limit = limit.unwrap_or(50).clamp(1, 100);

    let (entries, total) = docket::list_by_case(pool, &court_id, case_uuid, offset, limit).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(shared_types::DocketSearchResponse {
        entries: entries.into_iter().map(shared_types::DocketEntryResponse::from).collect(),
        total,
    })
}

// ── Docket Attachment server functions ────────────────────────────

/// List uploaded attachments for a docket entry.
#[server]
pub async fn list_entry_attachments(
    court_id: String,
    entry_id: String,
) -> Result<Vec<shared_types::DocketAttachmentResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attachment;
    use uuid::Uuid;

    let pool = get_db().await;
    let entry_uuid = Uuid::parse_str(&entry_id)
        .map_err(|_| ServerFnError::new("Invalid entry UUID"))?;

    let attachments = attachment::list_by_entry(pool, &court_id, entry_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(attachments
        .into_iter()
        .map(shared_types::DocketAttachmentResponse::from)
        .collect())
}

/// Initiate a presigned upload for a new attachment.
#[server]
pub async fn create_entry_attachment(
    court_id: String,
    entry_id: String,
    body: shared_types::CreateAttachmentRequest,
) -> Result<shared_types::CreateAttachmentResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{attachment, docket};
    use crate::storage::{ObjectStore, S3ObjectStore};
    use uuid::Uuid;

    let pool = get_db().await;
    let entry_uuid = Uuid::parse_str(&entry_id)
        .map_err(|_| ServerFnError::new("Invalid entry UUID"))?;

    // Verify entry exists in tenant
    docket::find_by_id(pool, &court_id, entry_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Docket entry not found"))?;

    if body.file_name.trim().is_empty() {
        return Err(ServerFnError::new("file_name must not be empty"));
    }

    let file_uuid = Uuid::new_v4();
    let object_key = format!(
        "{}/docket/{}/{}/{}",
        court_id, entry_id, file_uuid, body.file_name
    );

    let att = attachment::create_pending(
        pool,
        &court_id,
        entry_uuid,
        &body.file_name,
        body.file_size,
        &body.content_type,
        &object_key,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let store = S3ObjectStore::from_env();
    let (presign_url, required_headers) = store
        .presign_put(&object_key, &body.content_type)
        .await
        .map_err(|e| ServerFnError::new(format!("Presign failed: {}", e)))?;

    Ok(shared_types::CreateAttachmentResponse {
        attachment_id: att.id.to_string(),
        presign_url,
        object_key,
        required_headers,
    })
}

/// Finalize an attachment upload (verify in S3, mark uploaded_at).
#[server]
pub async fn finalize_attachment(
    court_id: String,
    attachment_id: String,
) -> Result<shared_types::DocketAttachmentResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attachment;
    use crate::storage::{ObjectStore, S3ObjectStore};
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attachment_id)
        .map_err(|_| ServerFnError::new("Invalid attachment UUID"))?;

    let att = attachment::find_by_id(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attachment not found"))?;

    let store = S3ObjectStore::from_env();
    let exists = store
        .head(&att.storage_key)
        .await
        .map_err(|e| ServerFnError::new(format!("HEAD failed: {}", e)))?;

    if !exists {
        return Err(ServerFnError::new("Object not yet uploaded to storage"));
    }

    attachment::mark_uploaded(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let updated = attachment::find_by_id(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attachment not found after update"))?;

    Ok(shared_types::DocketAttachmentResponse::from(updated))
}

/// Get a presigned download URL for an attachment.
#[server]
pub async fn get_attachment_download_url(
    court_id: String,
    attachment_id: String,
) -> Result<shared_types::AttachmentDownloadUrlResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attachment;
    use crate::storage::{ObjectStore, S3ObjectStore};
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attachment_id)
        .map_err(|_| ServerFnError::new("Invalid attachment UUID"))?;

    let att = attachment::find_by_id(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attachment not found"))?;

    let store = S3ObjectStore::from_env();
    let url = store
        .presign_get(&att.storage_key)
        .await
        .map_err(|e| ServerFnError::new(format!("Presign GET failed: {}", e)))?;

    Ok(shared_types::AttachmentDownloadUrlResponse {
        download_url: url,
        filename: att.filename,
        content_type: att.content_type,
    })
}

/// Cross-platform upload: receives file bytes, uploads to S3 server-side, and finalizes.
/// This avoids requiring client-side JS fetch for presigned URL PUT.
#[server]
pub async fn upload_docket_attachment(
    court_id: String,
    entry_id: String,
    file_name: String,
    content_type: String,
    file_size: i64,
    file_bytes: Vec<u8>,
) -> Result<shared_types::DocketAttachmentResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{attachment, docket};
    use crate::storage::{ObjectStore, S3ObjectStore};
    use uuid::Uuid;

    let pool = get_db().await;
    let entry_uuid = Uuid::parse_str(&entry_id)
        .map_err(|_| ServerFnError::new("Invalid entry UUID"))?;

    // Verify entry exists in tenant
    docket::find_by_id(pool, &court_id, entry_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Docket entry not found"))?;

    if file_name.trim().is_empty() {
        return Err(ServerFnError::new("file_name must not be empty"));
    }

    let file_uuid = Uuid::new_v4();
    let object_key = format!(
        "{}/docket/{}/{}/{}",
        court_id, entry_id, file_uuid, file_name
    );

    // Insert pending row
    let att = attachment::create_pending(
        pool,
        &court_id,
        entry_uuid,
        &file_name,
        file_size,
        &content_type,
        &object_key,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Upload to S3 with SSE-S3 (AES256) encryption
    let store = S3ObjectStore::from_env();
    store
        .put(&object_key, &content_type, file_bytes)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    // Mark as uploaded
    attachment::mark_uploaded(pool, &court_id, att.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let updated = attachment::find_by_id(pool, &court_id, att.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attachment not found after upload"))?;

    Ok(shared_types::DocketAttachmentResponse::from(updated))
}
