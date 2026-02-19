use shared_types::{AppError, CreateCaseRequest, CriminalCase};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Extract a division code from a court_id (e.g. "district9" → "9", "district12" → "12").
fn division_code(court_id: &str) -> &str {
    court_id.strip_prefix("district").unwrap_or(court_id)
}

/// Generate a CM/ECF case number for a criminal case.
/// Format: `D:YY-cr-NNNNN` (e.g. `9:26-cr-00001`).
async fn generate_case_number(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<String, AppError> {
    let count: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM criminal_cases WHERE court_id = $1"#,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let year = chrono::Utc::now().format("%y");
    let div = division_code(court_id);
    Ok(format!("{}:{}-cr-{:05}", div, year, count + 1))
}

/// Insert a new criminal case with an auto-generated case number.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateCaseRequest,
) -> Result<CriminalCase, AppError> {
    let priority = req.priority.as_deref().unwrap_or("medium");
    let case_number = generate_case_number(pool, court_id).await?;

    let row = sqlx::query_as!(
        CriminalCase,
        r#"
        INSERT INTO criminal_cases
            (court_id, case_number, title, description, crime_type, priority,
             district_code, location, assigned_judge_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, court_id, case_number, title, description, crime_type,
                  status, priority, assigned_judge_id, district_code, location,
                  is_sealed, sealed_date, sealed_by, seal_reason,
                  opened_at, updated_at, closed_at
        "#,
        court_id,
        case_number,
        req.title,
        req.description,
        req.crime_type,
        priority,
        req.district_code,
        req.location,
        req.assigned_judge_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a case by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<CriminalCase>, AppError> {
    let row = sqlx::query_as!(
        CriminalCase,
        r#"
        SELECT id, court_id, case_number, title, description, crime_type,
               status, priority, assigned_judge_id, district_code, location,
               is_sealed, sealed_date, sealed_by, seal_reason,
               opened_at, updated_at, closed_at
        FROM criminal_cases
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

/// Delete a case. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM criminal_cases WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// Update only the status of a case. Returns the updated row or None.
pub async fn update_status(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    status: &str,
) -> Result<Option<CriminalCase>, AppError> {
    let row = sqlx::query_as!(
        CriminalCase,
        r#"
        UPDATE criminal_cases SET
            status = $3,
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_number, title, description, crime_type,
                  status, priority, assigned_judge_id, district_code, location,
                  is_sealed, sealed_date, sealed_by, seal_reason,
                  opened_at, updated_at, closed_at
        "#,
        id,
        court_id,
        status,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Update a case with only the provided fields. Returns the updated row or None.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: shared_types::UpdateCaseRequest,
) -> Result<Option<CriminalCase>, AppError> {
    let row = sqlx::query_as!(
        CriminalCase,
        r#"
        UPDATE criminal_cases SET
            title         = COALESCE($3, title),
            description   = COALESCE($4, description),
            crime_type    = COALESCE($5, crime_type),
            status        = COALESCE($6, status),
            priority      = COALESCE($7, priority),
            location      = COALESCE($8, location),
            district_code = COALESCE($9, district_code),
            updated_at    = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_number, title, description, crime_type,
                  status, priority, assigned_judge_id, district_code, location,
                  is_sealed, sealed_date, sealed_by, seal_reason,
                  opened_at, updated_at, closed_at
        "#,
        id,
        court_id,
        req.title.as_deref(),
        req.description.as_deref(),
        req.crime_type.as_deref(),
        req.status.as_deref(),
        req.priority.as_deref(),
        req.location.as_deref(),
        req.district_code.as_deref(),
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Search cases with filters. Returns (cases, total_count).
pub async fn search(
    pool: &Pool<Postgres>,
    court_id: &str,
    status: Option<&str>,
    crime_type: Option<&str>,
    priority: Option<&str>,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<CriminalCase>, i64), AppError> {
    let search_pattern = q.map(|s| format!("%{}%", s));

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM criminal_cases
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR status = $2)
          AND ($3::TEXT IS NULL OR crime_type = $3)
          AND ($4::TEXT IS NULL OR priority = $4)
          AND ($5::TEXT IS NULL OR title ILIKE $5 OR case_number ILIKE $5)
        "#,
        court_id,
        status as Option<&str>,
        crime_type as Option<&str>,
        priority as Option<&str>,
        search_pattern.clone() as Option<String>,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        CriminalCase,
        r#"
        SELECT id, court_id, case_number, title, description, crime_type,
               status, priority, assigned_judge_id, district_code, location,
               is_sealed, sealed_date, sealed_by, seal_reason,
               opened_at, updated_at, closed_at
        FROM criminal_cases
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR status = $2)
          AND ($3::TEXT IS NULL OR crime_type = $3)
          AND ($4::TEXT IS NULL OR priority = $4)
          AND ($5::TEXT IS NULL OR title ILIKE $5 OR case_number ILIKE $5)
        ORDER BY opened_at DESC
        LIMIT $6 OFFSET $7
        "#,
        court_id,
        status as Option<&str>,
        crime_type as Option<&str>,
        priority as Option<&str>,
        search_pattern as Option<String>,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, total))
}
