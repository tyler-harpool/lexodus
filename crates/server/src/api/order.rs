use dioxus::prelude::*;

// ── Order Server Functions ─────────────────────────────

/// List all orders for a court (across all cases) with optional search and pagination.
#[server]
pub async fn list_all_orders(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<shared_types::PaginatedResponse<shared_types::JudicialOrderResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = order::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::JudicialOrderResponse> =
        rows.into_iter().map(shared_types::JudicialOrderResponse::from).collect();

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

/// Sign an order, updating its status to "Signed".
#[server]
pub async fn sign_order_action(
    court_id: String,
    id: String,
    signed_by: String,
) -> Result<shared_types::JudicialOrderResponse, ServerFnError> {
    use crate::db::get_db;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let order = sqlx::query_as!(
        shared_types::JudicialOrder,
        r#"
        WITH upd AS (
            UPDATE judicial_orders SET
                status = 'Signed',
                signer_name = $3,
                signed_at = NOW(),
                signature_hash = md5($3 || id::text),
                updated_at = NOW()
            WHERE id = $1 AND court_id = $2
            RETURNING id, court_id, case_id, judge_id, order_type, title, content,
                      status, is_sealed, signer_name, signed_at, signature_hash,
                      issued_at, effective_date, expiration_date, related_motions,
                      created_at, updated_at
        )
        SELECT upd.id, upd.court_id, upd.case_id, upd.judge_id,
               j.name as judge_name,
               COALESCE(cc.case_number, cv.case_number) as "case_number?",
               upd.order_type, upd.title, upd.content,
               upd.status, upd.is_sealed, upd.signer_name, upd.signed_at, upd.signature_hash,
               upd.issued_at, upd.effective_date, upd.expiration_date, upd.related_motions,
               upd.created_at, upd.updated_at
        FROM upd
        LEFT JOIN judges j ON upd.judge_id = j.id AND j.court_id = upd.court_id
        LEFT JOIN criminal_cases cc ON upd.case_id = cc.id
        LEFT JOIN civil_cases cv ON upd.case_id = cv.id
        "#,
        uuid,
        &court_id,
        signed_by,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Order not found"))?;

    Ok(shared_types::JudicialOrderResponse::from(order))
}

/// Issue an order, updating its status to "Filed".
#[server]
pub async fn issue_order_action(
    court_id: String,
    id: String,
) -> Result<shared_types::JudicialOrderResponse, ServerFnError> {
    use crate::db::get_db;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let order = sqlx::query_as!(
        shared_types::JudicialOrder,
        r#"
        WITH upd AS (
            UPDATE judicial_orders SET
                status = 'Filed',
                issued_at = NOW(),
                updated_at = NOW()
            WHERE id = $1 AND court_id = $2
            RETURNING id, court_id, case_id, judge_id, order_type, title, content,
                      status, is_sealed, signer_name, signed_at, signature_hash,
                      issued_at, effective_date, expiration_date, related_motions,
                      created_at, updated_at
        )
        SELECT upd.id, upd.court_id, upd.case_id, upd.judge_id,
               j.name as judge_name,
               COALESCE(cc.case_number, cv.case_number) as "case_number?",
               upd.order_type, upd.title, upd.content,
               upd.status, upd.is_sealed, upd.signer_name, upd.signed_at, upd.signature_hash,
               upd.issued_at, upd.effective_date, upd.expiration_date, upd.related_motions,
               upd.created_at, upd.updated_at
        FROM upd
        LEFT JOIN judges j ON upd.judge_id = j.id AND j.court_id = upd.court_id
        LEFT JOIN criminal_cases cc ON upd.case_id = cc.id
        LEFT JOIN civil_cases cv ON upd.case_id = cv.id
        "#,
        uuid,
        &court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Order not found"))?;

    Ok(shared_types::JudicialOrderResponse::from(order))
}

#[server]
pub async fn list_orders_by_case(
    court_id: String,
    case_id: String,
) -> Result<Vec<shared_types::JudicialOrderResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = order::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::JudicialOrderResponse::from).collect())
}

#[server]
pub async fn list_orders_by_judge(
    court_id: String,
    judge_id: String,
) -> Result<Vec<shared_types::JudicialOrderResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;
    use uuid::Uuid;

    let pool = get_db().await;
    let judge_uuid =
        Uuid::parse_str(&judge_id).map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let rows = order::list_by_judge(pool, &court_id, judge_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::JudicialOrderResponse::from).collect())
}

#[server]
pub async fn get_order(
    court_id: String,
    id: String,
) -> Result<shared_types::JudicialOrderResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = order::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::JudicialOrderResponse::from(row))
}

#[server]
pub async fn create_order(
    court_id: String,
    body: shared_types::CreateJudicialOrderRequest,
) -> Result<shared_types::JudicialOrderResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;

    let pool = get_db().await;
    let row = order::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::JudicialOrderResponse::from(row))
}

#[server]
pub async fn update_order(
    court_id: String,
    id: String,
    body: shared_types::UpdateJudicialOrderRequest,
) -> Result<shared_types::JudicialOrderResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = order::update(pool, &court_id, uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::JudicialOrderResponse::from(row))
}

#[server]
pub async fn delete_order(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    order::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Order Template Server Functions ────────────────────

#[server]
pub async fn list_order_templates(
    court_id: String,
) -> Result<Vec<shared_types::OrderTemplateResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order_template;

    let pool = get_db().await;
    let rows = order_template::list_all(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::OrderTemplateResponse::from).collect())
}

#[server]
pub async fn list_active_order_templates(
    court_id: String,
) -> Result<Vec<shared_types::OrderTemplateResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order_template;

    let pool = get_db().await;
    let rows = order_template::list_active(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::OrderTemplateResponse::from).collect())
}

#[server]
pub async fn get_order_template(
    court_id: String,
    id: String,
) -> Result<shared_types::OrderTemplateResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order_template;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = order_template::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::OrderTemplateResponse::from(row))
}

#[server]
pub async fn create_order_template(
    court_id: String,
    body: shared_types::CreateOrderTemplateRequest,
) -> Result<shared_types::OrderTemplateResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order_template;

    let pool = get_db().await;
    let row = order_template::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::OrderTemplateResponse::from(row))
}

#[server]
pub async fn update_order_template(
    court_id: String,
    id: String,
    body: shared_types::UpdateOrderTemplateRequest,
) -> Result<shared_types::OrderTemplateResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order_template;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = order_template::update(pool, &court_id, uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::OrderTemplateResponse::from(row))
}

#[server]
pub async fn delete_order_template(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order_template;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    order_template::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}
