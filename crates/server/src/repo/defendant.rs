use shared_types::{AppError, CreateDefendantRequest, Defendant};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new defendant.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateDefendantRequest,
) -> Result<Defendant, AppError> {
    let citizenship = req.citizenship_status.as_deref().unwrap_or("Unknown");
    let custody = req.custody_status.as_deref().unwrap_or("Released");

    let row = sqlx::query_as!(
        Defendant,
        r#"
        INSERT INTO defendants
            (court_id, case_id, name, aliases, usm_number, fbi_number,
             date_of_birth, citizenship_status, custody_status, bail_type,
             bail_amount, bond_conditions, bond_posted_date, surety_name)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11::FLOAT8, $12, $13, $14)
        RETURNING id, court_id, case_id, name, aliases, usm_number, fbi_number,
                  date_of_birth,
                  COALESCE(citizenship_status, 'Unknown') as "citizenship_status!",
                  custody_status, bail_type,
                  bail_amount as "bail_amount: f64",
                  bond_conditions, bond_posted_date, surety_name,
                  created_at, updated_at
        "#,
        court_id,
        req.case_id,
        req.name,
        &req.aliases as &[String],
        req.usm_number.as_deref(),
        req.fbi_number.as_deref(),
        req.date_of_birth,
        citizenship,
        custody,
        req.bail_type.as_deref(),
        req.bail_amount,
        &req.bond_conditions as &[String],
        req.bond_posted_date,
        req.surety_name.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a defendant by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Defendant>, AppError> {
    let row = sqlx::query_as!(
        Defendant,
        r#"
        SELECT id, court_id, case_id, name, aliases, usm_number, fbi_number,
               date_of_birth,
               COALESCE(citizenship_status, 'Unknown') as "citizenship_status!",
               custody_status, bail_type,
               bail_amount as "bail_amount: f64",
               bond_conditions, bond_posted_date, surety_name,
               created_at, updated_at
        FROM defendants
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

/// List all defendants for a given case within a court.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<Defendant>, AppError> {
    let rows = sqlx::query_as!(
        Defendant,
        r#"
        SELECT id, court_id, case_id, name, aliases, usm_number, fbi_number,
               date_of_birth,
               COALESCE(citizenship_status, 'Unknown') as "citizenship_status!",
               custody_status, bail_type,
               bail_amount as "bail_amount: f64",
               bond_conditions, bond_posted_date, surety_name,
               created_at, updated_at
        FROM defendants
        WHERE case_id = $1 AND court_id = $2
        ORDER BY created_at ASC
        "#,
        case_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all defendants for a court (across all cases), ordered by name.
pub async fn list_all(
    pool: &Pool<Postgres>,
    court_id: &str,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<Defendant>, i64), AppError> {
    let search = q.map(|s| format!("%{}%", s.to_lowercase()));

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!" FROM defendants
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR LOWER(name) LIKE $2)
        "#,
        court_id,
        search.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        Defendant,
        r#"
        SELECT id, court_id, case_id, name, aliases, usm_number, fbi_number,
               date_of_birth,
               COALESCE(citizenship_status, 'Unknown') as "citizenship_status!",
               custody_status, bail_type,
               bail_amount as "bail_amount: f64",
               bond_conditions, bond_posted_date, surety_name,
               created_at, updated_at
        FROM defendants
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR LOWER(name) LIKE $2)
        ORDER BY name ASC
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

/// Update a defendant with only the provided fields.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: shared_types::UpdateDefendantRequest,
) -> Result<Option<Defendant>, AppError> {
    let row = sqlx::query_as!(
        Defendant,
        r#"
        UPDATE defendants SET
            name               = COALESCE($3, name),
            aliases            = COALESCE($4, aliases),
            usm_number         = COALESCE($5, usm_number),
            fbi_number         = COALESCE($6, fbi_number),
            date_of_birth      = COALESCE($7, date_of_birth),
            citizenship_status = COALESCE($8, citizenship_status),
            custody_status     = COALESCE($9, custody_status),
            bail_type          = COALESCE($10, bail_type),
            bail_amount        = COALESCE($11::FLOAT8, bail_amount),
            bond_conditions    = COALESCE($12, bond_conditions),
            bond_posted_date   = COALESCE($13, bond_posted_date),
            surety_name        = COALESCE($14, surety_name),
            updated_at         = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, name, aliases, usm_number, fbi_number,
                  date_of_birth,
                  COALESCE(citizenship_status, 'Unknown') as "citizenship_status!",
                  custody_status, bail_type,
                  bail_amount as "bail_amount: f64",
                  bond_conditions, bond_posted_date, surety_name,
                  created_at, updated_at
        "#,
        id,
        court_id,
        req.name.as_deref(),
        req.aliases.as_deref().map(|a| a as &[String]),
        req.usm_number.as_deref(),
        req.fbi_number.as_deref(),
        req.date_of_birth,
        req.citizenship_status.as_deref(),
        req.custody_status.as_deref(),
        req.bail_type.as_deref(),
        req.bail_amount,
        req.bond_conditions.as_deref().map(|a| a as &[String]),
        req.bond_posted_date,
        req.surety_name.as_deref(),
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete a defendant. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM defendants WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
