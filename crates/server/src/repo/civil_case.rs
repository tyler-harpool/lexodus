use shared_types::{AppError, CivilCase, CreateCivilCaseRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Extract a division code from a court_id (e.g. "district9" → "9", "district12" → "12").
fn division_code(court_id: &str) -> &str {
    court_id.strip_prefix("district").unwrap_or(court_id)
}

/// Generate a CM/ECF case number for a civil case.
/// Format: `D:YY-cv-NNNNN` (e.g. `9:26-cv-00001`).
async fn generate_case_number(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<String, AppError> {
    let count: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM civil_cases WHERE court_id = $1"#,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let year = chrono::Utc::now().format("%y");
    let div = division_code(court_id);
    Ok(format!("{}:{}-cv-{:05}", div, year, count + 1))
}

/// Insert a new civil case with an auto-generated case number.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateCivilCaseRequest,
) -> Result<CivilCase, AppError> {
    let case_number = generate_case_number(pool, court_id).await?;
    let priority = req.priority.as_deref().unwrap_or("medium");
    let jury_demand = req.jury_demand.as_deref().unwrap_or("none");
    let class_action = req.class_action.unwrap_or(false);
    let consent_to_magistrate = req.consent_to_magistrate.unwrap_or(false);
    let pro_se = req.pro_se.unwrap_or(false);
    let description = req.description.as_deref().unwrap_or("");
    let cause_of_action = req.cause_of_action.as_deref().unwrap_or("");
    let district_code = req.district_code.as_deref().unwrap_or("");

    let assigned_judge_id: Option<Uuid> = req
        .assigned_judge_id
        .as_deref()
        .and_then(|s| s.parse().ok());

    // Convert f64 to string for safe NUMERIC binding (avoids binary encoding mismatch).
    let amount_str = req.amount_in_controversy.map(|v| v.to_string());

    let row = sqlx::query_as!(
        CivilCase,
        r#"
        INSERT INTO civil_cases
            (court_id, case_number, title, description, nature_of_suit,
             cause_of_action, jurisdiction_basis, jury_demand, class_action,
             amount_in_controversy, priority, assigned_judge_id, district_code,
             consent_to_magistrate, pro_se)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::NUMERIC, $11, $12, $13, $14, $15)
        RETURNING id, court_id, case_number, title, description, nature_of_suit,
                  cause_of_action, jurisdiction_basis, jury_demand, class_action,
                  amount_in_controversy::FLOAT8 as "amount_in_controversy: f64",
                  status, priority, assigned_judge_id, district_code, location,
                  is_sealed, sealed_date, sealed_by, seal_reason, related_case_id,
                  consent_to_magistrate, pro_se, opened_at, updated_at, closed_at
        "#,
        court_id,
        case_number,
        req.title,
        description,
        req.nature_of_suit,
        cause_of_action,
        req.jurisdiction_basis,
        jury_demand,
        class_action,
        amount_str as Option<String>,
        priority,
        assigned_judge_id,
        district_code,
        consent_to_magistrate,
        pro_se,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a civil case by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<CivilCase>, AppError> {
    let row = sqlx::query_as!(
        CivilCase,
        r#"
        SELECT id, court_id, case_number, title, description, nature_of_suit,
               cause_of_action, jurisdiction_basis, jury_demand, class_action,
               amount_in_controversy::FLOAT8 as "amount_in_controversy: f64",
               status, priority, assigned_judge_id, district_code, location,
               is_sealed, sealed_date, sealed_by, seal_reason, related_case_id,
               consent_to_magistrate, pro_se, opened_at, updated_at, closed_at
        FROM civil_cases
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

/// Delete a civil case. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM civil_cases WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// Update only the status of a civil case. Returns the updated row or None.
pub async fn update_status(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    status: &str,
) -> Result<Option<CivilCase>, AppError> {
    let row = sqlx::query_as!(
        CivilCase,
        r#"
        UPDATE civil_cases SET
            status = $3,
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_number, title, description, nature_of_suit,
                  cause_of_action, jurisdiction_basis, jury_demand, class_action,
                  amount_in_controversy::FLOAT8 as "amount_in_controversy: f64",
                  status, priority, assigned_judge_id, district_code, location,
                  is_sealed, sealed_date, sealed_by, seal_reason, related_case_id,
                  consent_to_magistrate, pro_se, opened_at, updated_at, closed_at
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

/// Search civil cases with filters. Returns (cases, total_count).
pub async fn search(
    pool: &Pool<Postgres>,
    court_id: &str,
    status: Option<&str>,
    nature_of_suit: Option<&str>,
    jurisdiction_basis: Option<&str>,
    class_action: Option<bool>,
    assigned_judge_id: Option<&str>,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<CivilCase>, i64), AppError> {
    let search_pattern = q.map(|s| format!("%{}%", s));
    let judge_uuid: Option<Uuid> = assigned_judge_id.and_then(|s| s.parse().ok());

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM civil_cases
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR status = $2)
          AND ($3::TEXT IS NULL OR nature_of_suit = $3)
          AND ($4::TEXT IS NULL OR jurisdiction_basis = $4)
          AND ($5::BOOL IS NULL OR class_action = $5)
          AND ($6::UUID IS NULL OR assigned_judge_id = $6)
          AND ($7::TEXT IS NULL OR title ILIKE $7 OR case_number ILIKE $7)
        "#,
        court_id,
        status as Option<&str>,
        nature_of_suit as Option<&str>,
        jurisdiction_basis as Option<&str>,
        class_action as Option<bool>,
        judge_uuid as Option<Uuid>,
        search_pattern.clone() as Option<String>,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        CivilCase,
        r#"
        SELECT id, court_id, case_number, title, description, nature_of_suit,
               cause_of_action, jurisdiction_basis, jury_demand, class_action,
               amount_in_controversy::FLOAT8 as "amount_in_controversy: f64",
               status, priority, assigned_judge_id, district_code, location,
               is_sealed, sealed_date, sealed_by, seal_reason, related_case_id,
               consent_to_magistrate, pro_se, opened_at, updated_at, closed_at
        FROM civil_cases
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR status = $2)
          AND ($3::TEXT IS NULL OR nature_of_suit = $3)
          AND ($4::TEXT IS NULL OR jurisdiction_basis = $4)
          AND ($5::BOOL IS NULL OR class_action = $5)
          AND ($6::UUID IS NULL OR assigned_judge_id = $6)
          AND ($7::TEXT IS NULL OR title ILIKE $7 OR case_number ILIKE $7)
        ORDER BY opened_at DESC
        LIMIT $8 OFFSET $9
        "#,
        court_id,
        status as Option<&str>,
        nature_of_suit as Option<&str>,
        jurisdiction_basis as Option<&str>,
        class_action as Option<bool>,
        judge_uuid as Option<Uuid>,
        search_pattern as Option<String>,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, total))
}
