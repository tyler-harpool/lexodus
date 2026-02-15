use shared_types::{Attorney, CreateAttorneyRequest, UpdateAttorneyRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;
use shared_types::AppError;

/// Insert a new attorney row. Returns the created attorney.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateAttorneyRequest,
) -> Result<Attorney, AppError> {
    let row = sqlx::query_as!(
        Attorney,
        r#"
        INSERT INTO attorneys (
            court_id, bar_number, first_name, last_name, middle_name,
            firm_name, email, phone, fax,
            address_street1, address_street2, address_city,
            address_state, address_zip, address_country
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
        RETURNING
            id, court_id, bar_number, first_name, last_name, middle_name,
            firm_name, email, phone, fax,
            address_street1, address_street2, address_city,
            address_state, address_zip, address_country,
            status, cja_panel_member, cja_panel_districts,
            languages_spoken, cases_handled, win_rate_percentage,
            avg_case_duration_days, created_at, updated_at
        "#,
        court_id,
        req.bar_number,
        req.first_name,
        req.last_name,
        req.middle_name,
        req.firm_name,
        req.email,
        req.phone,
        req.fax,
        req.address.street1,
        req.address.street2,
        req.address.city,
        req.address.state,
        req.address.zip_code,
        req.address.country,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find an attorney by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Attorney>, AppError> {
    let row = sqlx::query_as!(
        Attorney,
        r#"
        SELECT
            id, court_id, bar_number, first_name, last_name, middle_name,
            firm_name, email, phone, fax,
            address_street1, address_street2, address_city,
            address_state, address_zip, address_country,
            status, cja_panel_member, cja_panel_districts,
            languages_spoken, cases_handled, win_rate_percentage,
            avg_case_duration_days, created_at, updated_at
        FROM attorneys
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

/// Find an attorney by bar number within a specific court.
pub async fn find_by_bar_number(
    pool: &Pool<Postgres>,
    court_id: &str,
    bar_number: &str,
) -> Result<Option<Attorney>, AppError> {
    let row = sqlx::query_as!(
        Attorney,
        r#"
        SELECT
            id, court_id, bar_number, first_name, last_name, middle_name,
            firm_name, email, phone, fax,
            address_street1, address_street2, address_city,
            address_state, address_zip, address_country,
            status, cja_panel_member, cja_panel_districts,
            languages_spoken, cases_handled, win_rate_percentage,
            avg_case_duration_days, created_at, updated_at
        FROM attorneys
        WHERE bar_number = $1 AND court_id = $2
        "#,
        bar_number,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List attorneys for a court with pagination. Returns (attorneys, total_count).
pub async fn list(
    pool: &Pool<Postgres>,
    court_id: &str,
    page: i64,
    limit: i64,
) -> Result<(Vec<Attorney>, i64), AppError> {
    let offset = (page - 1) * limit;

    let count_row = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM attorneys WHERE court_id = $1"#,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        Attorney,
        r#"
        SELECT
            id, court_id, bar_number, first_name, last_name, middle_name,
            firm_name, email, phone, fax,
            address_street1, address_street2, address_city,
            address_state, address_zip, address_country,
            status, cja_panel_member, cja_panel_districts,
            languages_spoken, cases_handled, win_rate_percentage,
            avg_case_duration_days, created_at, updated_at
        FROM attorneys
        WHERE court_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        court_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, count_row))
}

/// Search attorneys by query string (matches first_name, last_name, bar_number, email, firm_name).
pub async fn search(
    pool: &Pool<Postgres>,
    court_id: &str,
    query: &str,
    page: i64,
    limit: i64,
) -> Result<(Vec<Attorney>, i64), AppError> {
    let offset = (page - 1) * limit;
    let pattern = format!("%{}%", query.to_lowercase());

    let count_row = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!" FROM attorneys
        WHERE court_id = $1
          AND (
            lower(first_name) LIKE $2
            OR lower(last_name) LIKE $2
            OR lower(bar_number) LIKE $2
            OR lower(email) LIKE $2
            OR lower(COALESCE(firm_name, '')) LIKE $2
          )
        "#,
        court_id,
        pattern,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        Attorney,
        r#"
        SELECT
            id, court_id, bar_number, first_name, last_name, middle_name,
            firm_name, email, phone, fax,
            address_street1, address_street2, address_city,
            address_state, address_zip, address_country,
            status, cja_panel_member, cja_panel_districts,
            languages_spoken, cases_handled, win_rate_percentage,
            avg_case_duration_days, created_at, updated_at
        FROM attorneys
        WHERE court_id = $1
          AND (
            lower(first_name) LIKE $2
            OR lower(last_name) LIKE $2
            OR lower(bar_number) LIKE $2
            OR lower(email) LIKE $2
            OR lower(COALESCE(firm_name, '')) LIKE $2
          )
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
        court_id,
        pattern,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, count_row))
}

/// Update an attorney using read-modify-write pattern.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: UpdateAttorneyRequest,
) -> Result<Option<Attorney>, AppError> {
    // Fetch existing
    let existing = match find_by_id(pool, court_id, id).await? {
        Some(a) => a,
        None => return Ok(None),
    };

    // Merge fields
    let bar_number = req.bar_number.unwrap_or(existing.bar_number);
    let first_name = req.first_name.unwrap_or(existing.first_name);
    let last_name = req.last_name.unwrap_or(existing.last_name);
    let middle_name = req.middle_name.or(existing.middle_name);
    let firm_name = req.firm_name.or(existing.firm_name);
    let email = req.email.unwrap_or(existing.email);
    let phone = req.phone.unwrap_or(existing.phone);
    let fax = req.fax.or(existing.fax);
    let status = req.status.unwrap_or(existing.status);
    let cja_panel_member = req.cja_panel_member.unwrap_or(existing.cja_panel_member);
    let cja_panel_districts = req.cja_panel_districts.unwrap_or(existing.cja_panel_districts);
    let languages_spoken = req.languages_spoken.unwrap_or(existing.languages_spoken);
    let cases_handled = req.cases_handled.unwrap_or(existing.cases_handled);
    let win_rate_percentage = req.win_rate_percentage.or(existing.win_rate_percentage);
    let avg_case_duration_days = req.avg_case_duration_days.or(existing.avg_case_duration_days);

    let (street1, street2, city, state, zip, country) = if let Some(addr) = req.address {
        (addr.street1, addr.street2, addr.city, addr.state, addr.zip_code, addr.country)
    } else {
        (
            existing.address_street1,
            existing.address_street2,
            existing.address_city,
            existing.address_state,
            existing.address_zip,
            existing.address_country,
        )
    };

    let row = sqlx::query_as!(
        Attorney,
        r#"
        UPDATE attorneys SET
            bar_number = $3, first_name = $4, last_name = $5, middle_name = $6,
            firm_name = $7, email = $8, phone = $9, fax = $10,
            address_street1 = $11, address_street2 = $12, address_city = $13,
            address_state = $14, address_zip = $15, address_country = $16,
            status = $17, cja_panel_member = $18, cja_panel_districts = $19,
            languages_spoken = $20, cases_handled = $21, win_rate_percentage = $22,
            avg_case_duration_days = $23, updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING
            id, court_id, bar_number, first_name, last_name, middle_name,
            firm_name, email, phone, fax,
            address_street1, address_street2, address_city,
            address_state, address_zip, address_country,
            status, cja_panel_member, cja_panel_districts,
            languages_spoken, cases_handled, win_rate_percentage,
            avg_case_duration_days, created_at, updated_at
        "#,
        id,
        court_id,
        bar_number,
        first_name,
        last_name,
        middle_name,
        firm_name,
        email,
        phone,
        fax,
        street1,
        street2,
        city,
        state,
        zip,
        country,
        status,
        cja_panel_member,
        &cja_panel_districts,
        &languages_spoken,
        cases_handled,
        win_rate_percentage,
        avg_case_duration_days,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete an attorney. Returns true if a row was actually deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM attorneys WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// Bulk update status for multiple attorneys.
pub async fn bulk_update_status(
    pool: &Pool<Postgres>,
    court_id: &str,
    ids: &[Uuid],
    status: &str,
) -> Result<u64, AppError> {
    let result = sqlx::query!(
        r#"
        UPDATE attorneys
        SET status = $3, updated_at = NOW()
        WHERE id = ANY($1) AND court_id = $2
        "#,
        ids,
        court_id,
        status,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected())
}
