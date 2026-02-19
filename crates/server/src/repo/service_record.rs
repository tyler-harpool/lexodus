use chrono::{DateTime, Utc};
use shared_types::{AppError, BulkCreateServiceRecordRequest, CreateServiceRecordRequest, ServiceMethod, ServiceRecord, ServiceRecordResponse};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Row struct for service records JOINed with party info.
/// Used only for the `list_by_document` query result.
#[derive(Debug, sqlx::FromRow)]
pub struct ServiceRecordWithParty {
    pub id: Uuid,
    pub court_id: String,
    pub document_id: Uuid,
    pub party_id: Uuid,
    pub party_name: String,
    pub party_type: String,
    pub service_date: DateTime<Utc>,
    pub service_method: String,
    pub served_by: String,
    pub proof_of_service_filed: bool,
    pub successful: bool,
    pub attempts: i32,
    pub notes: Option<String>,
}

impl From<ServiceRecordWithParty> for ServiceRecordResponse {
    fn from(r: ServiceRecordWithParty) -> Self {
        Self {
            id: r.id.to_string(),
            court_id: r.court_id,
            document_id: r.document_id.to_string(),
            party_id: r.party_id.to_string(),
            party_name: r.party_name,
            party_type: r.party_type,
            service_date: r.service_date.to_rfc3339(),
            service_method: r.service_method,
            served_by: r.served_by,
            proof_of_service_filed: r.proof_of_service_filed,
            successful: r.successful,
            attempts: r.attempts,
            notes: r.notes,
            certificate_of_service: None,
        }
    }
}

/// Insert a new service record. Validates document and party belong to the court.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateServiceRecordRequest,
) -> Result<ServiceRecordWithParty, AppError> {
    let document_id = Uuid::parse_str(&req.document_id)
        .map_err(|_| AppError::bad_request("Invalid document_id UUID"))?;
    let party_id = Uuid::parse_str(&req.party_id)
        .map_err(|_| AppError::bad_request("Invalid party_id UUID"))?;

    if req.served_by.trim().is_empty() {
        return Err(AppError::bad_request("served_by must not be empty"));
    }

    let method = ServiceMethod::try_from(req.service_method.as_str())
        .map_err(AppError::bad_request)?;

    // Validate document belongs to this court
    let doc_exists = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM documents WHERE id = $1 AND court_id = $2"#,
        document_id,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    if doc_exists == 0 {
        return Err(AppError::not_found("Document not found in this court"));
    }

    // Validate party belongs to this court
    let party_exists = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM parties WHERE id = $1 AND court_id = $2"#,
        party_id,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    if party_exists == 0 {
        return Err(AppError::not_found("Party not found in this court"));
    }

    // Parse optional service_date or default to now
    let service_date = if let Some(ref date_str) = req.service_date {
        chrono::DateTime::parse_from_rfc3339(date_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|_| AppError::bad_request("Invalid service_date format (expected RFC 3339)"))?
    } else {
        chrono::Utc::now()
    };

    let row = sqlx::query_as!(
        ServiceRecordWithParty,
        r#"
        WITH inserted AS (
            INSERT INTO service_records (
                court_id, document_id, party_id, service_date,
                service_method, served_by, proof_of_service_filed, successful, attempts, notes,
                certificate_of_service
            )
            VALUES ($1, $2, $3, $4, $5, $6, false, true, 1, $7, $8)
            RETURNING *
        )
        SELECT
            i.id AS "id!", i.court_id AS "court_id!", i.document_id AS "document_id!", i.party_id AS "party_id!",
            COALESCE(p.name, 'Unknown') AS "party_name!",
            COALESCE(p.party_type, 'Unknown') AS "party_type!",
            i.service_date AS "service_date!", i.service_method AS "service_method!", i.served_by AS "served_by!",
            i.proof_of_service_filed AS "proof_of_service_filed!", i.successful AS "successful!", i.attempts AS "attempts!", i.notes
        FROM inserted i
        LEFT JOIN parties p ON p.id = i.party_id AND p.court_id = i.court_id
        "#,
        court_id,
        document_id,
        party_id,
        service_date,
        method.as_db_str(),
        req.served_by.trim(),
        req.notes.as_deref(),
        req.certificate_of_service.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List service records for a court with optional filters.
/// Returns (records, total_count).
pub async fn list(
    pool: &Pool<Postgres>,
    court_id: &str,
    document_id: Option<Uuid>,
    party_id: Option<Uuid>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<ServiceRecord>, i64), AppError> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM service_records
        WHERE court_id = $1
          AND ($2::uuid IS NULL OR document_id = $2)
          AND ($3::uuid IS NULL OR party_id = $3)
        "#,
        court_id,
        document_id as Option<Uuid>,
        party_id as Option<Uuid>,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        ServiceRecord,
        r#"
        SELECT
            id, court_id, document_id, party_id, service_date,
            service_method, served_by, proof_of_service_filed, successful, attempts, notes,
            certificate_of_service
        FROM service_records
        WHERE court_id = $1
          AND ($2::uuid IS NULL OR document_id = $2)
          AND ($3::uuid IS NULL OR party_id = $3)
        ORDER BY service_date DESC
        LIMIT $4 OFFSET $5
        "#,
        court_id,
        document_id as Option<Uuid>,
        party_id as Option<Uuid>,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, count))
}

/// List service records for a specific document within a court, with party names.
/// The caller must verify the document belongs to the court first.
pub async fn list_by_document(
    pool: &Pool<Postgres>,
    court_id: &str,
    document_id: Uuid,
) -> Result<Vec<ServiceRecordWithParty>, AppError> {
    let rows = sqlx::query_as!(
        ServiceRecordWithParty,
        r#"
        SELECT
            sr.id, sr.court_id, sr.document_id, sr.party_id,
            p.name AS party_name, p.party_type,
            sr.service_date, sr.service_method, sr.served_by,
            sr.proof_of_service_filed, sr.successful, sr.attempts, sr.notes
        FROM service_records sr
        JOIN parties p ON p.id = sr.party_id
        WHERE sr.court_id = $1 AND sr.document_id = $2
        ORDER BY p.name ASC
        "#,
        court_id,
        document_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Find a single service record by ID within a court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<ServiceRecord>, AppError> {
    let row = sqlx::query_as!(
        ServiceRecord,
        r#"
        SELECT
            id, court_id, document_id, party_id, service_date,
            service_method, served_by, proof_of_service_filed, successful, attempts, notes,
            certificate_of_service
        FROM service_records
        WHERE id = $1 AND court_id = $2
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a single service record by ID with joined party info.
pub async fn find_by_id_with_party(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<ServiceRecordWithParty>, AppError> {
    let row = sqlx::query_as!(
        ServiceRecordWithParty,
        r#"
        SELECT
            sr.id, sr.court_id, sr.document_id, sr.party_id,
            COALESCE(p.name, 'Unknown') AS "party_name!",
            COALESCE(p.party_type, 'Unknown') AS "party_type!",
            sr.service_date, sr.service_method, sr.served_by,
            sr.proof_of_service_filed, sr.successful, sr.attempts, sr.notes
        FROM service_records sr
        LEFT JOIN parties p ON p.id = sr.party_id AND p.court_id = sr.court_id
        WHERE sr.id = $1 AND sr.court_id = $2
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all service records for a court with optional search and pagination.
/// Searches across party name, served_by, and service_method.
/// Returns (records_with_party, total_count).
pub async fn list_all(
    pool: &Pool<Postgres>,
    court_id: &str,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<ServiceRecordWithParty>, i64), AppError> {
    let search = q.map(|s| format!("%{}%", s.to_lowercase()));

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM service_records sr
        LEFT JOIN parties p ON p.id = sr.party_id AND p.court_id = sr.court_id
        WHERE sr.court_id = $1
          AND ($2::TEXT IS NULL
               OR LOWER(COALESCE(p.name, '')) LIKE $2
               OR LOWER(sr.served_by) LIKE $2
               OR LOWER(sr.service_method) LIKE $2)
        "#,
        court_id,
        search.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        ServiceRecordWithParty,
        r#"
        SELECT
            sr.id, sr.court_id, sr.document_id, sr.party_id,
            COALESCE(p.name, 'Unknown') AS "party_name!",
            COALESCE(p.party_type, 'Unknown') AS "party_type!",
            sr.service_date, sr.service_method, sr.served_by,
            sr.proof_of_service_filed, sr.successful, sr.attempts, sr.notes
        FROM service_records sr
        LEFT JOIN parties p ON p.id = sr.party_id AND p.court_id = sr.court_id
        WHERE sr.court_id = $1
          AND ($2::TEXT IS NULL
               OR LOWER(COALESCE(p.name, '')) LIKE $2
               OR LOWER(sr.served_by) LIKE $2
               OR LOWER(sr.service_method) LIKE $2)
        ORDER BY sr.service_date DESC
        LIMIT $3 OFFSET $4
        "#,
        court_id,
        search.as_deref(),
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, total))
}

/// Mark a service record as complete (successful + proof filed). Idempotent.
pub async fn complete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<ServiceRecordWithParty, AppError> {
    let row = sqlx::query_as!(
        ServiceRecordWithParty,
        r#"
        WITH updated AS (
            UPDATE service_records
            SET successful = true, proof_of_service_filed = true
            WHERE id = $1 AND court_id = $2
            RETURNING *
        )
        SELECT
            u.id AS "id!", u.court_id AS "court_id!", u.document_id AS "document_id!", u.party_id AS "party_id!",
            COALESCE(p.name, 'Unknown') AS "party_name!",
            COALESCE(p.party_type, 'Unknown') AS "party_type!",
            u.service_date AS "service_date!", u.service_method AS "service_method!", u.served_by AS "served_by!",
            u.proof_of_service_filed AS "proof_of_service_filed!", u.successful AS "successful!", u.attempts AS "attempts!", u.notes
        FROM updated u
        LEFT JOIN parties p ON p.id = u.party_id AND p.court_id = u.court_id
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found("Service record not found"))?;

    Ok(row)
}

/// Check if a document belongs to a specific court.
pub async fn document_in_court(
    pool: &Pool<Postgres>,
    court_id: &str,
    document_id: Uuid,
) -> Result<bool, AppError> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM documents WHERE id = $1 AND court_id = $2"#,
        document_id,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(count > 0)
}

/// List service records for a specific party within a court, with party names.
pub async fn list_by_party(
    pool: &Pool<Postgres>,
    court_id: &str,
    party_id: Uuid,
) -> Result<Vec<ServiceRecordWithParty>, AppError> {
    let rows = sqlx::query_as!(
        ServiceRecordWithParty,
        r#"
        SELECT
            sr.id, sr.court_id, sr.document_id, sr.party_id,
            p.name AS party_name, p.party_type,
            sr.service_date, sr.service_method, sr.served_by,
            sr.proof_of_service_filed, sr.successful, sr.attempts, sr.notes
        FROM service_records sr
        JOIN parties p ON p.id = sr.party_id
        WHERE sr.court_id = $1 AND sr.party_id = $2
        ORDER BY sr.service_date DESC
        "#,
        court_id,
        party_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Bulk-create service records for a document (one per party).
pub async fn bulk_create(
    pool: &Pool<Postgres>,
    court_id: &str,
    document_id: Uuid,
    req: &BulkCreateServiceRecordRequest,
) -> Result<Vec<ServiceRecordWithParty>, AppError> {
    let method = ServiceMethod::try_from(req.service_method.as_str())
        .map_err(AppError::bad_request)?;

    if req.served_by.trim().is_empty() {
        return Err(AppError::bad_request("served_by must not be empty"));
    }

    // Validate document belongs to this court
    if !document_in_court(pool, court_id, document_id).await? {
        return Err(AppError::not_found("Document not found in this court"));
    }

    let service_date = if let Some(ref date_str) = req.service_date {
        chrono::DateTime::parse_from_rfc3339(date_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|_| AppError::bad_request("Invalid service_date format (expected RFC 3339)"))?
    } else {
        chrono::Utc::now()
    };

    let mut records: Vec<ServiceRecordWithParty> = Vec::new();

    for pid_str in &req.party_ids {
        let party_id = Uuid::parse_str(pid_str)
            .map_err(|_| AppError::bad_request(format!("Invalid party_id UUID: {}", pid_str)))?;

        // Validate party belongs to court
        let party_exists = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM parties WHERE id = $1 AND court_id = $2"#,
            party_id,
            court_id,
        )
        .fetch_one(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?;

        if party_exists == 0 {
            return Err(AppError::not_found(format!(
                "Party {} not found in this court",
                pid_str
            )));
        }

        let row = sqlx::query_as!(
            ServiceRecordWithParty,
            r#"
            WITH inserted AS (
                INSERT INTO service_records (
                    court_id, document_id, party_id, service_date,
                    service_method, served_by, proof_of_service_filed, successful, attempts,
                    certificate_of_service
                )
                VALUES ($1, $2, $3, $4, $5, $6, false, true, 1, $7)
                RETURNING *
            )
            SELECT
                i.id AS "id!", i.court_id AS "court_id!", i.document_id AS "document_id!", i.party_id AS "party_id!",
                COALESCE(p.name, 'Unknown') AS "party_name!",
                COALESCE(p.party_type, 'Unknown') AS "party_type!",
                i.service_date AS "service_date!", i.service_method AS "service_method!", i.served_by AS "served_by!",
                i.proof_of_service_filed AS "proof_of_service_filed!", i.successful AS "successful!", i.attempts AS "attempts!", i.notes
            FROM inserted i
            LEFT JOIN parties p ON p.id = i.party_id AND p.court_id = i.court_id
            "#,
            court_id,
            document_id,
            party_id,
            service_date,
            method.as_db_str(),
            req.served_by.trim(),
            req.certificate_of_service.as_deref(),
        )
        .fetch_one(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?;

        records.push(row);
    }

    Ok(records)
}
