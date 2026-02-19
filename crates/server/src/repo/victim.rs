use shared_types::{AppError, CreateVictimRequest, Victim};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new victim record.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateVictimRequest,
) -> Result<Victim, AppError> {
    let notification_mail = req.notification_mail.unwrap_or(false);

    let row = sqlx::query_as!(
        Victim,
        r#"
        INSERT INTO victims
            (court_id, case_id, name, victim_type, notification_email, notification_phone, notification_mail)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, court_id, case_id, name, victim_type,
                  notification_email, notification_mail, notification_phone,
                  created_at, updated_at
        "#,
        court_id,
        req.case_id,
        req.name,
        req.victim_type,
        req.notification_email,
        req.notification_phone,
        notification_mail,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a victim by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Victim>, AppError> {
    let row = sqlx::query_as!(
        Victim,
        r#"
        SELECT id, court_id, case_id, name, victim_type,
               notification_email, notification_mail, notification_phone,
               created_at, updated_at
        FROM victims
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

/// List all victims for a specific case.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<Victim>, AppError> {
    let rows = sqlx::query_as!(
        Victim,
        r#"
        SELECT id, court_id, case_id, name, victim_type,
               notification_email, notification_mail, notification_phone,
               created_at, updated_at
        FROM victims
        WHERE court_id = $1 AND case_id = $2
        ORDER BY created_at ASC
        "#,
        court_id,
        case_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all victims for a court (across all cases) with optional search and pagination.
pub async fn list_all(
    pool: &Pool<Postgres>,
    court_id: &str,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<Victim>, i64), AppError> {
    let search = q.map(|s| format!("%{}%", s.to_lowercase()));

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!" FROM victims
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
        Victim,
        r#"
        SELECT id, court_id, case_id, name, victim_type,
               notification_email, notification_mail, notification_phone,
               created_at, updated_at
        FROM victims
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

/// Delete a victim record. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM victims WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
