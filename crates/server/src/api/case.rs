use dioxus::prelude::*;
use shared_types::{CaseResponse, CaseSearchResponse, CreateCaseRequest, UpdateCaseRequest};

/// Search cases with filters.
#[server]
pub async fn search_cases(
    court_id: String,
    status: Option<String>,
    crime_type: Option<String>,
    priority: Option<String>,
    q: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<CaseSearchResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case;

    let pool = get_db().await;
    let offset = offset.unwrap_or(0).max(0);
    let limit = limit.unwrap_or(20).clamp(1, 100);

    let (cases, total) = case::search(
        pool, &court_id,
        status.as_deref().filter(|s| !s.is_empty()),
        crime_type.as_deref().filter(|s| !s.is_empty()),
        priority.as_deref().filter(|s| !s.is_empty()),
        q.as_deref().filter(|s| !s.is_empty()),
        offset, limit,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(CaseSearchResponse {
        cases: cases.into_iter().map(CaseResponse::from).collect(),
        total,
    })
}

/// Get a single case by ID. Checks criminal_cases first, then civil_cases.
#[server]
pub async fn get_case(court_id: String, id: String) -> Result<CaseResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{case, civil_case};
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    // Try criminal first
    if let Some(c) = case::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        return Ok(CaseResponse::from(c));
    }

    // Fall back to civil
    let c = civil_case::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Case not found"))?;

    Ok(CaseResponse {
        id: c.id.to_string(),
        case_number: c.case_number,
        title: c.title,
        description: c.description,
        case_type: "civil".to_string(),
        crime_type: c.nature_of_suit,
        status: c.status,
        priority: c.priority,
        district_code: c.district_code,
        location: c.location,
        opened_at: c.opened_at.to_rfc3339(),
        updated_at: c.updated_at.to_rfc3339(),
        closed_at: c.closed_at.map(|d| d.to_rfc3339()),
        assigned_judge_id: c.assigned_judge_id.map(|u| u.to_string()),
        is_sealed: c.is_sealed,
        sealed_by: c.sealed_by,
        sealed_date: c.sealed_date.map(|d| d.to_rfc3339()),
        seal_reason: c.seal_reason,
        jurisdiction_basis: Some(c.jurisdiction_basis),
        jury_demand: Some(c.jury_demand),
        class_action: Some(c.class_action),
        amount_in_controversy: c.amount_in_controversy,
        consent_to_magistrate: Some(c.consent_to_magistrate),
        pro_se: Some(c.pro_se),
    })
}

/// Create a new criminal case.
#[server]
pub async fn create_case(court_id: String, body: CreateCaseRequest) -> Result<CaseResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case;

    let pool = get_db().await;

    let c = case::create(pool, &court_id, body).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(CaseResponse::from(c))
}

/// Delete a case by ID.
#[server]
pub async fn delete_case(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let deleted = case::delete(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if deleted {
        Ok(())
    } else {
        Err(ServerFnError::new("Case not found"))
    }
}

/// Update the status of a case.
#[server]
pub async fn update_case_status(
    court_id: String,
    id: String,
    status: String,
) -> Result<CaseResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let c = case::update_status(pool, &court_id, uuid, &status).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Case not found"))?;

    Ok(CaseResponse::from(c))
}

/// Update a case (partial update -- only provided fields are changed).
#[server]
pub async fn update_case(
    court_id: String,
    id: String,
    body: UpdateCaseRequest,
) -> Result<CaseResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let c = case::update(pool, &court_id, uuid, body).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Case not found"))?;

    Ok(CaseResponse::from(c))
}
