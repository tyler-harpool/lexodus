use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CreateDocketEntryRequest, DocketEntry, DocketEntryResponse, DocketSearchParams,
    DocketSearchResponse, DocketSheet, DocketStatistics, LinkDocumentRequest, ServiceCheckResponse,
    UserRole, is_valid_entry_type, DOCKET_ENTRY_TYPES,
};
use crate::auth::extractors::AuthRequired;
use crate::error_convert::SqlxErrorExt;
use crate::tenant::CourtId;

/// POST /api/docket/entries
#[utoipa::path(
    post,
    path = "/api/docket/entries",
    request_body = CreateDocketEntryRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Docket entry created", body = DocketEntryResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "docket"
)]
pub async fn create_docket_entry(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    auth: AuthRequired,
    Json(body): Json<CreateDocketEntryRequest>,
) -> Result<(StatusCode, Json<DocketEntryResponse>), AppError> {
    // Require clerk, judge, or admin for this court — attorneys file via /api/filings
    let role = crate::auth::court_role::resolve_court_role(&auth.0, &court.0);
    if !matches!(role, UserRole::Clerk | UserRole::Judge | UserRole::Admin) {
        return Err(AppError::forbidden("clerk or judge role required for this court"));
    }
    if body.description.trim().is_empty() {
        return Err(AppError::bad_request("description must not be empty"));
    }

    if !is_valid_entry_type(&body.entry_type) {
        return Err(AppError::bad_request(format!(
            "Invalid entry_type: {}. Valid values: {}",
            body.entry_type,
            DOCKET_ENTRY_TYPES.join(", ")
        )));
    }

    // Verify the case exists in this court
    let case_exists = crate::repo::case::find_by_id(&pool, &court.0, body.case_id).await?;
    if case_exists.is_none() {
        return Err(AppError::bad_request(format!(
            "Case {} not found in this court",
            body.case_id
        )));
    }

    let entry = crate::repo::docket::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(DocketEntryResponse::from(entry))))
}

/// GET /api/docket/entries/{id}
#[utoipa::path(
    get,
    path = "/api/docket/entries/{id}",
    params(
        ("id" = String, Path, description = "Docket entry UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Docket entry found", body = DocketEntryResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "docket"
)]
pub async fn get_docket_entry(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<DocketEntryResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let entry = crate::repo::docket::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Docket entry {} not found", id)))?;

    Ok(Json(DocketEntryResponse::from(entry)))
}

/// DELETE /api/docket/entries/{id}
#[utoipa::path(
    delete,
    path = "/api/docket/entries/{id}",
    params(
        ("id" = String, Path, description = "Docket entry UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Docket entry deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "docket"
)]
pub async fn delete_docket_entry(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::docket::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Docket entry {} not found", id)))
    }
}

/// GET /api/cases/{case_id}/docket
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/docket",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("offset" = Option<i64>, Query, description = "Pagination offset"),
        ("limit" = Option<i64>, Query, description = "Pagination limit"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Case docket entries", body = DocketSearchResponse),
        (status = 400, description = "Invalid case ID", body = AppError)
    ),
    tag = "docket"
)]
pub async fn get_case_docket(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<DocketSearchResponse>, AppError> {
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    let offset = params.offset.unwrap_or(0).max(0);
    let limit = params.limit.unwrap_or(50).clamp(1, 100);

    let (entries, total) = crate::repo::docket::list_by_case(
        &pool, &court.0, case_uuid, offset, limit,
    )
    .await?;

    let response = DocketSearchResponse {
        entries: entries.into_iter().map(DocketEntryResponse::from).collect(),
        total,
    };

    Ok(Json(response))
}

/// GET /api/docket/search
#[utoipa::path(
    get,
    path = "/api/docket/search",
    params(
        DocketSearchParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Search results", body = DocketSearchResponse)
    ),
    tag = "docket"
)]
pub async fn search_docket_entries(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<DocketSearchParams>,
) -> Result<Json<DocketSearchResponse>, AppError> {
    let offset = params.offset.unwrap_or(0).max(0);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);

    let case_id = params
        .case_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| AppError::bad_request("Invalid case_id UUID format"))?;

    if let Some(ref et) = params.entry_type {
        if !is_valid_entry_type(et) {
            return Err(AppError::bad_request(format!("Invalid entry_type: {}", et)));
        }
    }

    let (entries, total) = crate::repo::docket::search(
        &pool,
        &court.0,
        case_id,
        params.entry_type.as_deref(),
        params.q.as_deref(),
        offset,
        limit,
    )
    .await?;

    let response = DocketSearchResponse {
        entries: entries.into_iter().map(DocketEntryResponse::from).collect(),
        total,
    };

    Ok(Json(response))
}

/// POST /api/docket/entries/{entry_id}/link-document
///
/// Link an existing document to a docket entry. Both the entry and document
/// must belong to the same court tenant.
#[utoipa::path(
    post,
    path = "/api/docket/entries/{entry_id}/link-document",
    request_body = LinkDocumentRequest,
    params(
        ("entry_id" = String, Path, description = "Docket entry UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Document linked to docket entry", body = DocketEntryResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Entry or document not found", body = AppError)
    ),
    tag = "docket"
)]
pub async fn link_document(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(entry_id): Path<String>,
    Json(body): Json<LinkDocumentRequest>,
) -> Result<Json<DocketEntryResponse>, AppError> {
    let entry_uuid = Uuid::parse_str(&entry_id)
        .map_err(|_| AppError::bad_request("Invalid entry_id UUID format"))?;
    let doc_uuid = Uuid::parse_str(&body.document_id)
        .map_err(|_| AppError::bad_request("Invalid document_id UUID format"))?;

    // Verify entry exists in this court
    crate::repo::docket::find_by_id(&pool, &court.0, entry_uuid)
        .await?
        .ok_or_else(|| AppError::not_found("Docket entry not found"))?;

    // Verify document exists in this court
    crate::repo::document::find_by_id(&pool, &court.0, doc_uuid)
        .await?
        .ok_or_else(|| AppError::not_found("Document not found"))?;

    let updated = crate::repo::docket::link_document(&pool, &court.0, entry_uuid, doc_uuid).await?;
    Ok(Json(DocketEntryResponse::from(updated)))
}

/// Simple pagination params for the case docket endpoint.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct PaginationParams {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

/// GET /api/docket/case/{case_id}
#[utoipa::path(
    get,
    path = "/api/docket/case/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Docket entries for case", body = Vec<DocketEntryResponse>)
    ),
    tag = "docket"
)]
pub async fn list_docket_by_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<DocketEntryResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    let rows = sqlx::query_as!(
        DocketEntry,
        r#"
        SELECT id, court_id, case_id, entry_number, date_filed, date_entered,
               filed_by, entry_type, description, document_id,
               is_sealed, is_ex_parte, page_count, related_entries, service_list
        FROM docket_entries
        WHERE court_id = $1 AND case_id = $2
        ORDER BY entry_number ASC
        "#,
        court.0,
        uuid,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let response: Vec<DocketEntryResponse> =
        rows.into_iter().map(DocketEntryResponse::from).collect();

    Ok(Json(response))
}

/// GET /api/cases/{case_id}/docket-sheet
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/docket-sheet",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Full docket sheet", body = DocketSheet),
        (status = 404, description = "Case not found", body = AppError)
    ),
    tag = "docket"
)]
pub async fn get_docket_sheet(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<DocketSheet>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    // Get case info
    let case = crate::repo::case::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Case {} not found", case_id)))?;

    // Get all entries for this case (no pagination for a sheet)
    let rows = sqlx::query_as!(
        DocketEntry,
        r#"
        SELECT id, court_id, case_id, entry_number, date_filed, date_entered,
               filed_by, entry_type, description, document_id,
               is_sealed, is_ex_parte, page_count, related_entries, service_list
        FROM docket_entries
        WHERE court_id = $1 AND case_id = $2
        ORDER BY entry_number ASC
        "#,
        court.0,
        uuid,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let total = rows.len() as i64;
    let entries: Vec<DocketEntryResponse> =
        rows.into_iter().map(DocketEntryResponse::from).collect();

    Ok(Json(DocketSheet {
        case_id: case.id.to_string(),
        case_number: case.case_number,
        entries,
        total,
    }))
}

/// GET /api/cases/{case_id}/docket/type/{entry_type}
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/docket/type/{entry_type}",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("entry_type" = String, Path, description = "Docket entry type"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Docket entries by type", body = Vec<DocketEntryResponse>)
    ),
    tag = "docket"
)]
pub async fn list_by_type(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((case_id, entry_type)): Path<(String, String)>,
) -> Result<Json<Vec<DocketEntryResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    if !is_valid_entry_type(&entry_type) {
        return Err(AppError::bad_request(format!(
            "Invalid entry_type: {}",
            entry_type
        )));
    }

    let rows = sqlx::query_as!(
        DocketEntry,
        r#"
        SELECT id, court_id, case_id, entry_number, date_filed, date_entered,
               filed_by, entry_type, description, document_id,
               is_sealed, is_ex_parte, page_count, related_entries, service_list
        FROM docket_entries
        WHERE court_id = $1 AND case_id = $2 AND entry_type = $3
        ORDER BY entry_number ASC
        "#,
        court.0,
        uuid,
        entry_type,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let response: Vec<DocketEntryResponse> =
        rows.into_iter().map(DocketEntryResponse::from).collect();

    Ok(Json(response))
}

/// GET /api/cases/{case_id}/docket/sealed
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/docket/sealed",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Sealed docket entries", body = Vec<DocketEntryResponse>)
    ),
    tag = "docket"
)]
pub async fn list_sealed(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<Vec<DocketEntryResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    let rows = sqlx::query_as!(
        DocketEntry,
        r#"
        SELECT id, court_id, case_id, entry_number, date_filed, date_entered,
               filed_by, entry_type, description, document_id,
               is_sealed, is_ex_parte, page_count, related_entries, service_list
        FROM docket_entries
        WHERE court_id = $1 AND case_id = $2 AND is_sealed = true
        ORDER BY entry_number ASC
        "#,
        court.0,
        uuid,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let response: Vec<DocketEntryResponse> =
        rows.into_iter().map(DocketEntryResponse::from).collect();

    Ok(Json(response))
}

/// GET /api/cases/{case_id}/docket/search/{text}
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/docket/search/{text}",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("text" = String, Path, description = "Search text"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Search results within case docket", body = Vec<DocketEntryResponse>)
    ),
    tag = "docket"
)]
pub async fn search_in_case(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path((case_id, text)): Path<(String, String)>,
) -> Result<Json<Vec<DocketEntryResponse>>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    let search_pattern = format!("%{}%", text);

    let rows = sqlx::query_as!(
        DocketEntry,
        r#"
        SELECT id, court_id, case_id, entry_number, date_filed, date_entered,
               filed_by, entry_type, description, document_id,
               is_sealed, is_ex_parte, page_count, related_entries, service_list
        FROM docket_entries
        WHERE court_id = $1 AND case_id = $2 AND description ILIKE $3
        ORDER BY entry_number ASC
        "#,
        court.0,
        uuid,
        search_pattern,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let response: Vec<DocketEntryResponse> =
        rows.into_iter().map(DocketEntryResponse::from).collect();

    Ok(Json(response))
}

/// GET /api/cases/{case_id}/docket/statistics
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/docket/statistics",
    params(
        ("case_id" = String, Path, description = "Case UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Docket statistics", body = DocketStatistics)
    ),
    tag = "docket"
)]
pub async fn docket_statistics(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(case_id): Path<String>,
) -> Result<Json<DocketStatistics>, AppError> {
    let uuid = Uuid::parse_str(&case_id)
        .map_err(|_| AppError::bad_request("Invalid case UUID format"))?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM docket_entries WHERE court_id = $1 AND case_id = $2"#,
        court.0,
        uuid,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let sealed_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM docket_entries WHERE court_id = $1 AND case_id = $2 AND is_sealed = true"#,
        court.0,
        uuid,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let by_type_raw = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(
            json_object_agg(entry_type, cnt),
            '{}'::json
        )::TEXT as "json!"
        FROM (
            SELECT entry_type, COUNT(*) as cnt
            FROM docket_entries
            WHERE court_id = $1 AND case_id = $2
            GROUP BY entry_type
        ) sub
        "#,
        court.0,
        uuid,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let by_type: serde_json::Value = serde_json::from_str(&by_type_raw)
        .unwrap_or(serde_json::json!({}));

    Ok(Json(DocketStatistics {
        case_id: case_id.clone(),
        total_entries: total,
        by_type,
        sealed_count,
    }))
}

/// GET /api/docket/service-check/{entry_type}
#[utoipa::path(
    get,
    path = "/api/docket/service-check/{entry_type}",
    params(
        ("entry_type" = String, Path, description = "Docket entry type"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Service requirements for entry type", body = ServiceCheckResponse),
        (status = 400, description = "Invalid entry type", body = AppError)
    ),
    tag = "docket"
)]
pub async fn service_check(
    State(_pool): State<Pool<Postgres>>,
    _court: CourtId,
    Path(entry_type): Path<String>,
) -> Result<Json<ServiceCheckResponse>, AppError> {
    if !is_valid_entry_type(&entry_type) {
        return Err(AppError::bad_request(format!(
            "Invalid entry_type: {}. Valid values: {}",
            entry_type,
            DOCKET_ENTRY_TYPES.join(", ")
        )));
    }

    let (requires_service, service_method, service_deadline_days) = match entry_type.as_str() {
        "motion" | "answer" | "response" | "reply" | "appeal_brief" => (true, "Electronic", 3),
        "order" | "judgment" | "verdict" | "sentence" | "minute_order"
        | "scheduling_order" | "protective_order" | "sealing_order"
        | "appellate_order" => (true, "Electronic", 1),
        "summons" | "subpoena" => (true, "Personal", 7),
        "notice" | "hearing_notice" | "notice_of_appeal" => (true, "Electronic", 3),
        "complaint" | "indictment" | "information" | "criminal_complaint" => (true, "Personal", 21),
        _ => (false, "None", 0),
    };

    Ok(Json(ServiceCheckResponse {
        entry_type,
        requires_service,
        service_method: service_method.to_string(),
        service_deadline_days,
    }))
}
