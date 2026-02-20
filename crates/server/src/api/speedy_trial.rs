use dioxus::prelude::*;

// ── Speedy Trial Server Functions ──────────────────────

#[server]
pub async fn get_speedy_trial(
    court_id: String,
    case_id: String,
) -> Result<shared_types::SpeedyTrialResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::speedy_trial;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let row = speedy_trial::find_by_case_id(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No speedy trial clock found"))?;
    Ok(shared_types::SpeedyTrialResponse::from(row))
}

#[server]
pub async fn start_speedy_trial(
    court_id: String,
    body: shared_types::StartSpeedyTrialRequest,
) -> Result<shared_types::SpeedyTrialResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::speedy_trial;

    let pool = get_db().await;
    let row = speedy_trial::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::SpeedyTrialResponse::from(row))
}

#[server]
pub async fn update_speedy_trial(
    court_id: String,
    case_id: String,
    body: shared_types::UpdateSpeedyTrialClockRequest,
) -> Result<shared_types::SpeedyTrialResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::speedy_trial;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let row = speedy_trial::update(pool, &court_id, case_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::SpeedyTrialResponse::from(row))
}

#[server]
pub async fn list_speedy_trial_delays(
    court_id: String,
    case_id: String,
) -> Result<Vec<shared_types::ExcludableDelayResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::speedy_trial;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = speedy_trial::list_delays_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::ExcludableDelayResponse::from).collect())
}

#[server]
pub async fn create_speedy_trial_delay(
    court_id: String,
    case_id: String,
    body: shared_types::CreateExcludableDelayRequest,
) -> Result<shared_types::ExcludableDelayResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::speedy_trial;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let row = speedy_trial::create_delay(pool, &court_id, case_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::ExcludableDelayResponse::from(row))
}

#[server]
pub async fn delete_speedy_trial_delay(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::speedy_trial;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    speedy_trial::delete_delay(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Extension Request Server Functions ─────────────────

#[server]
pub async fn list_extensions_by_deadline(
    court_id: String,
    deadline_id: String,
) -> Result<Vec<shared_types::ExtensionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::extension_request;
    use uuid::Uuid;

    let pool = get_db().await;
    let dl_uuid = Uuid::parse_str(&deadline_id)
        .map_err(|_| ServerFnError::new("Invalid deadline_id UUID"))?;
    let rows = extension_request::list_by_deadline(pool, &court_id, dl_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::ExtensionResponse::from).collect())
}

#[server]
pub async fn get_extension(
    court_id: String,
    id: String,
) -> Result<shared_types::ExtensionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::extension_request;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = extension_request::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::ExtensionResponse::from(row))
}

#[server]
pub async fn create_extension_request_fn(
    court_id: String,
    deadline_id: String,
    body: shared_types::CreateExtensionRequest,
) -> Result<shared_types::ExtensionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::extension_request;
    use uuid::Uuid;

    let pool = get_db().await;
    let dl_uuid = Uuid::parse_str(&deadline_id)
        .map_err(|_| ServerFnError::new("Invalid deadline_id UUID"))?;
    let row = extension_request::create(pool, &court_id, dl_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::ExtensionResponse::from(row))
}

#[server]
pub async fn list_pending_extensions(
    court_id: String,
) -> Result<Vec<shared_types::ExtensionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::extension_request;

    let pool = get_db().await;
    let rows = extension_request::list_pending(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::ExtensionResponse::from).collect())
}

#[server]
pub async fn rule_on_extension(
    court_id: String,
    id: String,
    status: String,
    ruling_by: String,
    new_deadline_date: Option<String>,
) -> Result<shared_types::ExtensionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::extension_request;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let new_date = new_deadline_date
        .map(|d| {
            chrono::DateTime::parse_from_rfc3339(&d)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|_| ServerFnError::new("Invalid date format"))
        })
        .transpose()?;
    let row = extension_request::update_ruling(pool, &court_id, uuid, &status, &ruling_by, new_date)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::ExtensionResponse::from(row))
}
