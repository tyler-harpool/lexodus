use dioxus::prelude::*;

// ── Defendant Server Functions ─────────────────────────

#[server]
pub async fn list_defendants(
    court_id: String,
    case_id: String,
) -> Result<Vec<shared_types::DefendantResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = defendant::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::DefendantResponse::from).collect())
}

/// List all defendants for a court (across all cases) with optional search and pagination.
#[server]
pub async fn list_all_defendants(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<shared_types::PaginatedResponse<shared_types::DefendantResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = defendant::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::DefendantResponse> =
        rows.into_iter().map(shared_types::DefendantResponse::from).collect();

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
pub async fn get_defendant(
    court_id: String,
    id: String,
) -> Result<shared_types::DefendantResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = defendant::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::DefendantResponse::from(row))
}

#[server]
pub async fn create_defendant(
    court_id: String,
    body: shared_types::CreateDefendantRequest,
) -> Result<shared_types::DefendantResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;

    let pool = get_db().await;
    let row = defendant::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::DefendantResponse::from(row))
}

#[server]
pub async fn update_defendant(
    court_id: String,
    id: String,
    body: shared_types::UpdateDefendantRequest,
) -> Result<shared_types::DefendantResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = defendant::update(pool, &court_id, uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::DefendantResponse::from(row))
}

#[server]
pub async fn delete_defendant(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    defendant::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Party Server Functions ─────────────────────────────

#[server]
pub async fn create_party(
    court_id: String,
    body: shared_types::CreatePartyRequest,
) -> Result<shared_types::PartyResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;

    let pool = get_db().await;
    let row = party::create(pool, &court_id, &body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::PartyResponse::from(row))
}

#[server]
pub async fn get_party(
    court_id: String,
    id: String,
) -> Result<shared_types::PartyResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = party::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::PartyResponse::from(row))
}

#[server]
pub async fn update_party(
    court_id: String,
    id: String,
    body: shared_types::UpdatePartyRequest,
) -> Result<shared_types::PartyResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = party::update(pool, &court_id, uuid, &body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::PartyResponse::from(row))
}

#[server]
pub async fn delete_party(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    party::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server]
pub async fn list_parties_by_case(
    court_id: String,
    case_id: String,
) -> Result<Vec<shared_types::PartyResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = party::list_full_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::PartyResponse::from).collect())
}

#[server]
pub async fn list_unrepresented_parties(
    court_id: String,
) -> Result<Vec<shared_types::PartyResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;

    let pool = get_db().await;
    let rows = party::list_unrepresented(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::PartyResponse::from).collect())
}

#[server]
pub async fn list_parties_by_attorney(
    court_id: String,
    attorney_id: String,
) -> Result<Vec<shared_types::PartyResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid =
        Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = party::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::PartyResponse::from).collect())
}

/// List all parties for a court (across all cases) with optional search and pagination.
#[server]
pub async fn list_all_parties(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<shared_types::PaginatedResponse<shared_types::PartyResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = party::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::PartyResponse> =
        rows.into_iter().map(shared_types::PartyResponse::from).collect();

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

/// List all representations for a specific party.
#[server]
pub async fn list_representations_by_party(
    court_id: String,
    party_id: String,
) -> Result<Vec<shared_types::RepresentationResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::representation;
    use uuid::Uuid;

    let pool = get_db().await;
    let party_uuid =
        Uuid::parse_str(&party_id).map_err(|_| ServerFnError::new("Invalid party_id UUID"))?;
    let rows = representation::list_by_party(pool, &court_id, party_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let responses: Vec<shared_types::RepresentationResponse> =
        rows.into_iter().map(shared_types::RepresentationResponse::from).collect();
    Ok(responses)
}
