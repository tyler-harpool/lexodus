use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, BulkCreateServiceRecordRequest, CreateServiceRecordRequest,
    ServiceMethod, ServiceRecordResponse, ServiceRecordSearchParams,
};
use crate::tenant::CourtId;

/// GET /api/service-records
#[utoipa::path(
    get,
    path = "/api/service-records",
    params(
        ServiceRecordSearchParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "List of service records", body = serde_json::Value),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "service-records"
)]
pub async fn list_service_records(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<ServiceRecordSearchParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let document_id = params
        .document_id
        .as_deref()
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::bad_request("Invalid document_id UUID"))?;

    let party_id = params
        .party_id
        .as_deref()
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::bad_request("Invalid party_id UUID"))?;

    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(50).min(100);

    let (records, total) =
        crate::repo::service_record::list(&pool, &court.0, document_id, party_id, offset, limit)
            .await?;

    let response_items: Vec<ServiceRecordResponse> =
        records.into_iter().map(ServiceRecordResponse::from).collect();

    Ok(Json(serde_json::json!({
        "records": response_items,
        "total": total,
    })))
}

/// POST /api/service-records
#[utoipa::path(
    post,
    path = "/api/service-records",
    request_body = CreateServiceRecordRequest,
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 201, description = "Service record created", body = ServiceRecordResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Document or party not found", body = AppError)
    ),
    tag = "service-records"
)]
pub async fn create_service_record(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateServiceRecordRequest>,
) -> Result<(StatusCode, Json<ServiceRecordResponse>), AppError> {
    // Validate service_method early
    ServiceMethod::try_from(body.service_method.as_str())
        .map_err(AppError::bad_request)?;

    let record = crate::repo::service_record::create(&pool, &court.0, body).await?;
    Ok((StatusCode::CREATED, Json(ServiceRecordResponse::from(record))))
}

/// GET /api/service-records/document/{document_id}
#[utoipa::path(
    get,
    path = "/api/service-records/document/{document_id}",
    params(
        ("document_id" = String, Path, description = "Document UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Service records for document", body = Vec<ServiceRecordResponse>),
        (status = 404, description = "Document not found", body = AppError)
    ),
    tag = "service-records"
)]
pub async fn list_by_document(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(document_id): Path<String>,
) -> Result<Json<Vec<ServiceRecordResponse>>, AppError> {
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| AppError::bad_request("Invalid document UUID"))?;

    // Verify the document belongs to this court — return 404 for cross-tenant
    if !crate::repo::service_record::document_in_court(&pool, &court.0, doc_uuid).await? {
        return Err(AppError::not_found("Document not found"));
    }

    let records =
        crate::repo::service_record::list_by_document(&pool, &court.0, doc_uuid).await?;

    let response: Vec<ServiceRecordResponse> =
        records.into_iter().map(Into::into).collect();

    Ok(Json(response))
}

/// POST /api/service-records/{id}/complete
#[utoipa::path(
    post,
    path = "/api/service-records/{id}/complete",
    params(
        ("id" = String, Path, description = "Service record UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Service record marked complete", body = ServiceRecordResponse),
        (status = 404, description = "Service record not found", body = AppError)
    ),
    tag = "service-records"
)]
pub async fn complete_service_record(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<ServiceRecordResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let record = crate::repo::service_record::complete(&pool, &court.0, uuid).await?;
    Ok(Json(ServiceRecordResponse::from(record)))
}

/// GET /api/service-records/party/{party_id}
#[utoipa::path(
    get,
    path = "/api/service-records/party/{party_id}",
    params(
        ("party_id" = String, Path, description = "Party UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Service records for party", body = Vec<ServiceRecordResponse>),
        (status = 400, description = "Invalid UUID", body = AppError)
    ),
    tag = "service-records"
)]
pub async fn list_by_party(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(party_id): Path<String>,
) -> Result<Json<Vec<ServiceRecordResponse>>, AppError> {
    let uuid = Uuid::parse_str(&party_id)
        .map_err(|_| AppError::bad_request("Invalid party_id UUID"))?;

    let records =
        crate::repo::service_record::list_by_party(&pool, &court.0, uuid).await?;

    let response: Vec<ServiceRecordResponse> =
        records.into_iter().map(Into::into).collect();

    Ok(Json(response))
}

/// POST /api/service-records/bulk/{document_id}
#[utoipa::path(
    post,
    path = "/api/service-records/bulk/{document_id}",
    params(
        ("document_id" = String, Path, description = "Document UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = BulkCreateServiceRecordRequest,
    responses(
        (status = 201, description = "Service records created", body = Vec<ServiceRecordResponse>),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Document or party not found", body = AppError)
    ),
    tag = "service-records"
)]
pub async fn bulk_create(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(document_id): Path<String>,
    Json(body): Json<BulkCreateServiceRecordRequest>,
) -> Result<(StatusCode, Json<Vec<ServiceRecordResponse>>), AppError> {
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| AppError::bad_request("Invalid document_id UUID"))?;

    // Validate service_method early
    ServiceMethod::try_from(body.service_method.as_str())
        .map_err(AppError::bad_request)?;

    let records =
        crate::repo::service_record::bulk_create(&pool, &court.0, doc_uuid, &body).await?;

    let response: Vec<ServiceRecordResponse> =
        records.into_iter().map(ServiceRecordResponse::from).collect();

    Ok((StatusCode::CREATED, Json(response)))
}
