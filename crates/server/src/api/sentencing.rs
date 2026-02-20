use dioxus::prelude::*;

// ── Sentencing Server Functions ────────────────────────

#[server]
pub async fn list_sentencing_by_case(
    court_id: String,
    case_id: String,
) -> Result<Vec<shared_types::SentencingResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = sentencing::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::SentencingResponse::from).collect())
}

#[server]
pub async fn list_sentencing_by_defendant(
    court_id: String,
    defendant_id: String,
) -> Result<Vec<shared_types::SentencingResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing;
    use uuid::Uuid;

    let pool = get_db().await;
    let def_uuid = Uuid::parse_str(&defendant_id)
        .map_err(|_| ServerFnError::new("Invalid defendant_id UUID"))?;
    let rows = sentencing::list_by_defendant(pool, &court_id, def_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::SentencingResponse::from).collect())
}

#[server]
pub async fn get_sentencing(
    court_id: String,
    id: String,
) -> Result<shared_types::SentencingResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = sentencing::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::SentencingResponse::from(row))
}

#[server]
pub async fn create_sentencing(
    court_id: String,
    body: shared_types::CreateSentencingRequest,
) -> Result<shared_types::SentencingResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing;

    let pool = get_db().await;
    let row = sentencing::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::SentencingResponse::from(row))
}

#[server]
pub async fn update_sentencing(
    court_id: String,
    id: String,
    body: shared_types::UpdateSentencingRequest,
) -> Result<shared_types::SentencingResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = sentencing::update(pool, &court_id, uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::SentencingResponse::from(row))
}

#[server]
pub async fn delete_sentencing(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    sentencing::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Sentencing Condition Server Functions ───────────────

#[server]
pub async fn list_sentencing_conditions(
    court_id: String,
    sentencing_id: String,
) -> Result<Vec<shared_types::SpecialConditionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing_condition;
    use uuid::Uuid;

    let pool = get_db().await;
    let s_uuid = Uuid::parse_str(&sentencing_id)
        .map_err(|_| ServerFnError::new("Invalid sentencing_id UUID"))?;
    let rows = sentencing_condition::list_by_sentencing(pool, &court_id, s_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::SpecialConditionResponse::from).collect())
}

#[server]
pub async fn create_sentencing_condition(
    court_id: String,
    sentencing_id: String,
    body: shared_types::CreateSpecialConditionRequest,
) -> Result<shared_types::SpecialConditionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing_condition;
    use uuid::Uuid;

    let pool = get_db().await;
    let s_uuid = Uuid::parse_str(&sentencing_id)
        .map_err(|_| ServerFnError::new("Invalid sentencing_id UUID"))?;
    let row = sentencing_condition::create(pool, &court_id, s_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::SpecialConditionResponse::from(row))
}

// ── Sentencing List-All + BOP + Prior Sentence Server Functions ──

#[server]
pub async fn list_all_sentencing(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<shared_types::PaginatedResponse<shared_types::SentencingResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = sentencing::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::SentencingResponse> =
        rows.into_iter().map(shared_types::SentencingResponse::from).collect();

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
pub async fn list_bop_designations(
    court_id: String,
    sentencing_id: String,
) -> Result<Vec<shared_types::BopDesignationResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::bop_designation;
    use uuid::Uuid;

    let pool = get_db().await;
    let s_uuid = Uuid::parse_str(&sentencing_id)
        .map_err(|_| ServerFnError::new("Invalid sentencing_id UUID"))?;
    let rows = bop_designation::list_by_sentencing(pool, &court_id, s_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::BopDesignationResponse::from).collect())
}

#[server]
pub async fn list_prior_sentences(
    court_id: String,
    sentencing_id: String,
) -> Result<Vec<shared_types::PriorSentenceResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::prior_sentence;
    use uuid::Uuid;

    let pool = get_db().await;
    let s_uuid = Uuid::parse_str(&sentencing_id)
        .map_err(|_| ServerFnError::new("Invalid sentencing_id UUID"))?;
    let rows = prior_sentence::list_by_sentencing(pool, &court_id, s_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::PriorSentenceResponse::from).collect())
}
