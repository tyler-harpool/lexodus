use shared_types::{AppError, CreateJudgeRequest, Judge, UpdateJudgeRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Lightweight judge info for dropdowns.
#[derive(Debug)]
pub struct JudgeOption {
    pub id: Uuid,
    pub name: String,
    pub title: String,
    pub courtroom: Option<String>,
}

/// List active judges in a court (lightweight, for dropdowns).
pub async fn list_active_by_court(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<JudgeOption>, AppError> {
    sqlx::query_as!(
        JudgeOption,
        r#"
        SELECT id, name, title, courtroom
        FROM judges
        WHERE court_id = $1 AND status IN ('Active', 'Senior')
        ORDER BY name ASC
        "#,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Insert a new judge.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateJudgeRequest,
) -> Result<Judge, AppError> {
    let title = req.title.as_str();
    let status = req.status.as_deref().unwrap_or("Active");
    let max_caseload = req.max_caseload.unwrap_or(150);

    let row = sqlx::query_as!(
        Judge,
        r#"
        INSERT INTO judges
            (court_id, name, title, district, appointed_date, status,
             senior_status_date, courtroom, max_caseload, specializations)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, court_id, name, title, district,
                  appointed_date, status, senior_status_date, courtroom,
                  current_caseload, max_caseload, specializations,
                  created_at, updated_at
        "#,
        court_id,
        req.name,
        title,
        req.district,
        req.appointed_date,
        status,
        req.senior_status_date,
        req.courtroom.as_deref(),
        max_caseload,
        &req.specializations as &[String],
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a judge by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Judge>, AppError> {
    let row = sqlx::query_as!(
        Judge,
        r#"
        SELECT id, court_id, name, title, district,
               appointed_date, status, senior_status_date, courtroom,
               current_caseload, max_caseload, specializations,
               created_at, updated_at
        FROM judges
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

/// List all judges in a court.
pub async fn list_by_court(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<Judge>, AppError> {
    let rows = sqlx::query_as!(
        Judge,
        r#"
        SELECT id, court_id, name, title, district,
               appointed_date, status, senior_status_date, courtroom,
               current_caseload, max_caseload, specializations,
               created_at, updated_at
        FROM judges
        WHERE court_id = $1
        ORDER BY name ASC
        "#,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List judges by status within a court.
pub async fn list_by_status(
    pool: &Pool<Postgres>,
    court_id: &str,
    status: &str,
) -> Result<Vec<Judge>, AppError> {
    let rows = sqlx::query_as!(
        Judge,
        r#"
        SELECT id, court_id, name, title, district,
               appointed_date, status, senior_status_date, courtroom,
               current_caseload, max_caseload, specializations,
               created_at, updated_at
        FROM judges
        WHERE court_id = $1 AND status = $2
        ORDER BY name ASC
        "#,
        court_id,
        status,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Search judges by name or district (ILIKE).
pub async fn search(
    pool: &Pool<Postgres>,
    court_id: &str,
    query: &str,
) -> Result<Vec<Judge>, AppError> {
    let pattern = format!("%{}%", query);
    let rows = sqlx::query_as!(
        Judge,
        r#"
        SELECT id, court_id, name, title, district,
               appointed_date, status, senior_status_date, courtroom,
               current_caseload, max_caseload, specializations,
               created_at, updated_at
        FROM judges
        WHERE court_id = $1 AND (name ILIKE $2 OR district ILIKE $2)
        ORDER BY name ASC
        "#,
        court_id,
        pattern,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Update a judge with only the provided fields.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: UpdateJudgeRequest,
) -> Result<Option<Judge>, AppError> {
    let row = sqlx::query_as!(
        Judge,
        r#"
        UPDATE judges SET
            name               = COALESCE($3, name),
            title              = COALESCE($4, title),
            district           = COALESCE($5, district),
            appointed_date     = COALESCE($6, appointed_date),
            status             = COALESCE($7, status),
            senior_status_date = COALESCE($8, senior_status_date),
            courtroom          = COALESCE($9, courtroom),
            max_caseload       = COALESCE($10, max_caseload),
            specializations    = COALESCE($11, specializations),
            updated_at         = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, name, title, district,
                  appointed_date, status, senior_status_date, courtroom,
                  current_caseload, max_caseload, specializations,
                  created_at, updated_at
        "#,
        id,
        court_id,
        req.name.as_deref(),
        req.title.as_deref(),
        req.district.as_deref(),
        req.appointed_date,
        req.status.as_deref(),
        req.senior_status_date,
        req.courtroom.as_deref(),
        req.max_caseload,
        req.specializations.as_deref().map(|s| s as &[String]),
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Update only a judge's status.
pub async fn update_status(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    status: &str,
) -> Result<Option<Judge>, AppError> {
    let row = sqlx::query_as!(
        Judge,
        r#"
        UPDATE judges SET
            status     = $3,
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, name, title, district,
                  appointed_date, status, senior_status_date, courtroom,
                  current_caseload, max_caseload, specializations,
                  created_at, updated_at
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

/// Delete a judge. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM judges WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
