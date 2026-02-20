use dioxus::prelude::*;
use shared_types::{CivilCaseResponse, CivilCaseSearchResponse, CreateCivilCaseRequest};

/// Search civil cases with filters.
#[server]
pub async fn search_civil_cases(
    court_id: String,
    status: Option<String>,
    nature_of_suit: Option<String>,
    jurisdiction_basis: Option<String>,
    class_action: Option<bool>,
    assigned_judge_id: Option<String>,
    q: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<CivilCaseSearchResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::civil_case;

    let pool = get_db().await;
    let offset = offset.unwrap_or(0).max(0);
    let limit = limit.unwrap_or(20).clamp(1, 100);

    let (cases, total) = civil_case::search(
        pool, &court_id,
        status.as_deref().filter(|s| !s.is_empty()),
        nature_of_suit.as_deref().filter(|s| !s.is_empty()),
        jurisdiction_basis.as_deref().filter(|s| !s.is_empty()),
        class_action,
        assigned_judge_id.as_deref().filter(|s| !s.is_empty()),
        q.as_deref().filter(|s| !s.is_empty()),
        offset, limit,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(CivilCaseSearchResponse {
        cases: cases.into_iter().map(CivilCaseResponse::from).collect(),
        total,
    })
}

/// Get a single civil case by ID.
#[server]
pub async fn get_civil_case(court_id: String, id: String) -> Result<CivilCaseResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::civil_case;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let c = civil_case::find_by_id(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Civil case not found"))?;

    Ok(CivilCaseResponse::from(c))
}

/// Create a new civil case.
#[server]
pub async fn create_civil_case(court_id: String, body: CreateCivilCaseRequest) -> Result<CivilCaseResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::civil_case;

    let pool = get_db().await;

    let c = civil_case::create(pool, &court_id, body).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(CivilCaseResponse::from(c))
}

/// Update the status of a civil case.
#[server]
pub async fn update_civil_case_status(
    court_id: String,
    id: String,
    status: String,
) -> Result<CivilCaseResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::civil_case;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let c = civil_case::update_status(pool, &court_id, uuid, &status).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Civil case not found"))?;

    Ok(CivilCaseResponse::from(c))
}
