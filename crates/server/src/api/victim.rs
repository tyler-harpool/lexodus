use dioxus::prelude::*;

// ── Victim Server Functions ────────────────────────────

#[server]
pub async fn list_victims_by_case(
    court_id: String,
    case_id: String,
) -> Result<Vec<shared_types::VictimResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::victim;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = victim::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::VictimResponse::from).collect())
}

#[server]
pub async fn create_victim(
    court_id: String,
    body: shared_types::CreateVictimRequest,
) -> Result<shared_types::VictimResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::victim;

    let pool = get_db().await;
    let row = victim::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::VictimResponse::from(row))
}

#[server]
pub async fn get_victim(
    court_id: String,
    id: String,
) -> Result<shared_types::VictimResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::victim;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = victim::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::VictimResponse::from(row))
}

#[server]
pub async fn delete_victim(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::victim;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    victim::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

/// List all victims for a court (across all cases) with optional search and pagination.
#[server]
pub async fn list_all_victims(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<shared_types::PaginatedResponse<shared_types::VictimResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::victim;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = victim::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::VictimResponse> =
        rows.into_iter().map(shared_types::VictimResponse::from).collect();

    let total_pages = if per_page > 0 { (total + per_page - 1) / per_page } else { 0 };
    let meta = shared_types::PaginationMeta {
        total,
        page,
        limit: per_page,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    };

    Ok(shared_types::PaginatedResponse { data: responses, meta })
}

/// List notifications for a specific victim.
#[server]
pub async fn list_victim_notifications(
    court_id: String,
    victim_id: String,
) -> Result<Vec<shared_types::VictimNotificationResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::victim_notification;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&victim_id).map_err(|_| ServerFnError::new("Invalid victim_id UUID"))?;
    let rows = victim_notification::list_by_victim(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let responses: Vec<shared_types::VictimNotificationResponse> =
        rows.into_iter().map(shared_types::VictimNotificationResponse::from).collect();
    Ok(responses)
}
