use shared_types::{AppError, CreateDocketEntryRequest, DocketEntry};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Generate the next sequential entry number for a case's docket.
/// Uses MAX + 1 with the UNIQUE(court_id, case_id, entry_number) constraint
/// providing concurrency safety (retried on conflict).
async fn next_entry_number(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<i32, AppError> {
    let max: Option<i32> = sqlx::query_scalar!(
        r#"SELECT MAX(entry_number) as "max" FROM docket_entries WHERE court_id = $1 AND case_id = $2"#,
        court_id,
        case_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(max.unwrap_or(0) + 1)
}

/// Insert a new docket entry with an auto-generated sequential entry number.
/// Retries up to 3 times on unique constraint violation for concurrency safety.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateDocketEntryRequest,
) -> Result<DocketEntry, AppError> {
    // Extract owned fields so they can be referenced across loop iterations.
    let case_id = req.case_id;
    let entry_type = req.entry_type;
    let description = req.description;
    let filed_by = req.filed_by;
    let document_id = req.document_id;
    let is_sealed = req.is_sealed;
    let is_ex_parte = req.is_ex_parte;
    let page_count = req.page_count;
    let related_entries = req.related_entries;
    let service_list = req.service_list;

    let mut attempts = 0;
    loop {
        let entry_number = next_entry_number(pool, court_id, case_id).await?;

        let result = sqlx::query_as!(
            DocketEntry,
            r#"
            WITH ins AS (
                INSERT INTO docket_entries
                    (court_id, case_id, entry_number, entry_type, description,
                     filed_by, document_id, is_sealed, is_ex_parte, page_count,
                     related_entries, service_list)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                RETURNING id, court_id, case_id, entry_number, date_filed, date_entered,
                          filed_by, entry_type, description, document_id,
                          is_sealed, is_ex_parte, page_count, related_entries, service_list
            )
            SELECT ins.id as "id!",
                   ins.court_id as "court_id!",
                   ins.case_id as "case_id!",
                   ins.entry_number as "entry_number!",
                   ins.date_filed as "date_filed!",
                   ins.date_entered as "date_entered!",
                   ins.filed_by,
                   ins.entry_type as "entry_type!",
                   ins.description as "description!",
                   ins.document_id,
                   ins.is_sealed as "is_sealed!",
                   ins.is_ex_parte as "is_ex_parte!",
                   ins.page_count,
                   ins.related_entries as "related_entries!",
                   ins.service_list as "service_list!",
                   COALESCE(cc.case_number, cv.case_number) as "case_number?"
            FROM ins
            LEFT JOIN criminal_cases cc ON ins.case_id = cc.id
            LEFT JOIN civil_cases cv ON ins.case_id = cv.id
            "#,
            court_id,
            case_id,
            entry_number,
            entry_type.as_str(),
            description.as_str(),
            filed_by.as_deref() as Option<&str>,
            document_id as Option<Uuid>,
            is_sealed,
            is_ex_parte,
            page_count as Option<i32>,
            &related_entries,
            &service_list,
        )
        .fetch_one(pool)
        .await;

        match result {
            Ok(row) => return Ok(row),
            Err(sqlx::Error::Database(ref db_err)) if db_err.code().as_deref() == Some("23505") => {
                attempts += 1;
                if attempts >= 3 {
                    return Err(result.map_err(SqlxErrorExt::into_app_error).unwrap_err());
                }
                continue;
            }
            Err(e) => return Err(e.into_app_error()),
        }
    }
}

/// Find a docket entry by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<DocketEntry>, AppError> {
    let row = sqlx::query_as!(
        DocketEntry,
        r#"
        SELECT de.id, de.court_id, de.case_id, de.entry_number, de.date_filed, de.date_entered,
               de.filed_by, de.entry_type, de.description, de.document_id,
               de.is_sealed, de.is_ex_parte, de.page_count, de.related_entries, de.service_list,
               COALESCE(cc.case_number, cv.case_number) as "case_number?"
        FROM docket_entries de
        LEFT JOIN criminal_cases cc ON de.case_id = cc.id
        LEFT JOIN civil_cases cv ON de.case_id = cv.id
        WHERE de.id = $1 AND de.court_id = $2
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete a docket entry. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM docket_entries WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// List all docket entries for a specific case, ordered by entry_number ascending.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
    offset: i64,
    limit: i64,
) -> Result<(Vec<DocketEntry>, i64), AppError> {
    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM docket_entries WHERE court_id = $1 AND case_id = $2"#,
        court_id,
        case_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        DocketEntry,
        r#"
        SELECT de.id, de.court_id, de.case_id, de.entry_number, de.date_filed, de.date_entered,
               de.filed_by, de.entry_type, de.description, de.document_id,
               de.is_sealed, de.is_ex_parte, de.page_count, de.related_entries, de.service_list,
               COALESCE(cc.case_number, cv.case_number) as "case_number?"
        FROM docket_entries de
        LEFT JOIN criminal_cases cc ON de.case_id = cc.id
        LEFT JOIN civil_cases cv ON de.case_id = cv.id
        WHERE de.court_id = $1 AND de.case_id = $2
        ORDER BY de.entry_number ASC
        LIMIT $3 OFFSET $4
        "#,
        court_id,
        case_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, total))
}

/// Link a document to a docket entry. Returns the updated entry.
pub async fn link_document(
    pool: &Pool<Postgres>,
    court_id: &str,
    entry_id: Uuid,
    document_id: Uuid,
) -> Result<DocketEntry, AppError> {
    sqlx::query_as!(
        DocketEntry,
        r#"
        WITH upd AS (
            UPDATE docket_entries
            SET document_id = $3
            WHERE id = $1 AND court_id = $2
            RETURNING id, court_id, case_id, entry_number, date_filed, date_entered,
                      filed_by, entry_type, description, document_id,
                      is_sealed, is_ex_parte, page_count, related_entries, service_list
        )
        SELECT upd.id as "id!",
               upd.court_id as "court_id!",
               upd.case_id as "case_id!",
               upd.entry_number as "entry_number!",
               upd.date_filed as "date_filed!",
               upd.date_entered as "date_entered!",
               upd.filed_by,
               upd.entry_type as "entry_type!",
               upd.description as "description!",
               upd.document_id,
               upd.is_sealed as "is_sealed!",
               upd.is_ex_parte as "is_ex_parte!",
               upd.page_count,
               upd.related_entries as "related_entries!",
               upd.service_list as "service_list!",
               COALESCE(cc.case_number, cv.case_number) as "case_number?"
        FROM upd
        LEFT JOIN criminal_cases cc ON upd.case_id = cc.id
        LEFT JOIN civil_cases cv ON upd.case_id = cv.id
        "#,
        entry_id,
        court_id,
        document_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found("Docket entry not found"))
}

/// Search docket entries with filters. Returns (entries, total_count).
pub async fn search(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Option<Uuid>,
    entry_type: Option<&str>,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<DocketEntry>, i64), AppError> {
    let search_pattern = q.map(|s| format!("%{}%", s));

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM docket_entries
        WHERE court_id = $1
          AND ($2::UUID IS NULL OR case_id = $2)
          AND ($3::TEXT IS NULL OR entry_type = $3)
          AND ($4::TEXT IS NULL OR description ILIKE $4)
        "#,
        court_id,
        case_id as Option<Uuid>,
        entry_type as Option<&str>,
        search_pattern.clone() as Option<String>,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        DocketEntry,
        r#"
        SELECT de.id, de.court_id, de.case_id, de.entry_number, de.date_filed, de.date_entered,
               de.filed_by, de.entry_type, de.description, de.document_id,
               de.is_sealed, de.is_ex_parte, de.page_count, de.related_entries, de.service_list,
               COALESCE(cc.case_number, cv.case_number) as "case_number?"
        FROM docket_entries de
        LEFT JOIN criminal_cases cc ON de.case_id = cc.id
        LEFT JOIN civil_cases cv ON de.case_id = cv.id
        WHERE de.court_id = $1
          AND ($2::UUID IS NULL OR de.case_id = $2)
          AND ($3::TEXT IS NULL OR de.entry_type = $3)
          AND ($4::TEXT IS NULL OR de.description ILIKE $4)
        ORDER BY de.date_filed DESC, de.entry_number DESC
        LIMIT $5 OFFSET $6
        "#,
        court_id,
        case_id as Option<Uuid>,
        entry_type as Option<&str>,
        search_pattern as Option<String>,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, total))
}
