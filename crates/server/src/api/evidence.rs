use dioxus::prelude::*;

// ── Evidence Server Functions ──────────────────────────

/// List all evidence for a court (across all cases) with optional search and pagination.
#[server]
pub async fn list_all_evidence(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<shared_types::PaginatedResponse<shared_types::EvidenceResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::evidence;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = evidence::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::EvidenceResponse> =
        rows.into_iter().map(shared_types::EvidenceResponse::from).collect();

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

#[server]
pub async fn list_evidence_by_case(
    court_id: String,
    case_id: String,
) -> Result<Vec<shared_types::EvidenceResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::evidence;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = evidence::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::EvidenceResponse::from).collect())
}

#[server]
pub async fn get_evidence(
    court_id: String,
    id: String,
) -> Result<shared_types::EvidenceResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::evidence;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = evidence::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::EvidenceResponse::from(row))
}

#[server]
pub async fn create_evidence(
    court_id: String,
    body: shared_types::CreateEvidenceRequest,
) -> Result<shared_types::EvidenceResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::evidence;

    let pool = get_db().await;
    let row = evidence::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::EvidenceResponse::from(row))
}

#[server]
pub async fn update_evidence(
    court_id: String,
    id: String,
    body: shared_types::UpdateEvidenceRequest,
) -> Result<shared_types::EvidenceResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::evidence;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = evidence::update(pool, &court_id, uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::EvidenceResponse::from(row))
}

#[server]
pub async fn delete_evidence(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::evidence;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    evidence::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Custody Transfer Server Functions ──────────────────

#[server]
pub async fn list_custody_transfers(
    court_id: String,
    evidence_id: String,
) -> Result<Vec<shared_types::CustodyTransferResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::custody_transfer;
    use uuid::Uuid;

    let pool = get_db().await;
    let ev_uuid =
        Uuid::parse_str(&evidence_id).map_err(|_| ServerFnError::new("Invalid evidence_id UUID"))?;
    let rows = custody_transfer::list_by_evidence(pool, &court_id, ev_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::CustodyTransferResponse::from).collect())
}

#[server]
pub async fn create_custody_transfer(
    court_id: String,
    body: shared_types::CreateCustodyTransferRequest,
) -> Result<shared_types::CustodyTransferResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::custody_transfer;

    let pool = get_db().await;
    let row = custody_transfer::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::CustodyTransferResponse::from(row))
}
