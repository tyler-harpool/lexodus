use shared_types::{AppError, CreateJudicialOrderRequest, JudicialOrder, UpdateJudicialOrderRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new judicial order.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateJudicialOrderRequest,
) -> Result<JudicialOrder, AppError> {
    let status = req.status.as_deref().unwrap_or("Draft");
    let is_sealed = req.is_sealed.unwrap_or(false);

    let row = sqlx::query_as!(
        JudicialOrder,
        r#"
        INSERT INTO judicial_orders
            (court_id, case_id, judge_id, order_type, title, content,
             status, is_sealed, effective_date, expiration_date, related_motions)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, court_id, case_id, judge_id, order_type, title, content,
                  status, is_sealed, signer_name, signed_at, signature_hash,
                  issued_at, effective_date, expiration_date, related_motions,
                  created_at, updated_at
        "#,
        court_id,
        req.case_id,
        req.judge_id,
        req.order_type,
        req.title,
        req.content,
        status,
        is_sealed,
        req.effective_date,
        req.expiration_date,
        &req.related_motions as &[Uuid],
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a judicial order by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<JudicialOrder>, AppError> {
    let row = sqlx::query_as!(
        JudicialOrder,
        r#"
        SELECT id, court_id, case_id, judge_id, order_type, title, content,
               status, is_sealed, signer_name, signed_at, signature_hash,
               issued_at, effective_date, expiration_date, related_motions,
               created_at, updated_at
        FROM judicial_orders
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

/// List all judicial orders for a given case within a court.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<JudicialOrder>, AppError> {
    let rows = sqlx::query_as!(
        JudicialOrder,
        r#"
        SELECT id, court_id, case_id, judge_id, order_type, title, content,
               status, is_sealed, signer_name, signed_at, signature_hash,
               issued_at, effective_date, expiration_date, related_motions,
               created_at, updated_at
        FROM judicial_orders
        WHERE case_id = $1 AND court_id = $2
        ORDER BY created_at DESC
        "#,
        case_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all judicial orders for a given judge within a court.
pub async fn list_by_judge(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Uuid,
) -> Result<Vec<JudicialOrder>, AppError> {
    let rows = sqlx::query_as!(
        JudicialOrder,
        r#"
        SELECT id, court_id, case_id, judge_id, order_type, title, content,
               status, is_sealed, signer_name, signed_at, signature_hash,
               issued_at, effective_date, expiration_date, related_motions,
               created_at, updated_at
        FROM judicial_orders
        WHERE judge_id = $1 AND court_id = $2
        ORDER BY created_at DESC
        "#,
        judge_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all judicial orders for a court (across all cases), ordered by creation date.
/// Supports optional search by title and pagination.
pub async fn list_all(
    pool: &Pool<Postgres>,
    court_id: &str,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<JudicialOrder>, i64), AppError> {
    let search = q.map(|s| format!("%{}%", s.to_lowercase()));

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!" FROM judicial_orders
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR LOWER(title) LIKE $2)
        "#,
        court_id,
        search.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        JudicialOrder,
        r#"
        SELECT id, court_id, case_id, judge_id, order_type, title, content,
               status, is_sealed, signer_name, signed_at, signature_hash,
               issued_at, effective_date, expiration_date, related_motions,
               created_at, updated_at
        FROM judicial_orders
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR LOWER(title) LIKE $2)
        ORDER BY created_at DESC
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

/// Update a judicial order with only the provided fields.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: UpdateJudicialOrderRequest,
) -> Result<Option<JudicialOrder>, AppError> {
    let row = sqlx::query_as!(
        JudicialOrder,
        r#"
        UPDATE judicial_orders SET
            title           = COALESCE($3, title),
            content         = COALESCE($4, content),
            status          = COALESCE($5, status),
            is_sealed       = COALESCE($6, is_sealed),
            effective_date  = COALESCE($7, effective_date),
            expiration_date = COALESCE($8, expiration_date),
            related_motions = COALESCE($9, related_motions),
            updated_at      = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, judge_id, order_type, title, content,
                  status, is_sealed, signer_name, signed_at, signature_hash,
                  issued_at, effective_date, expiration_date, related_motions,
                  created_at, updated_at
        "#,
        id,
        court_id,
        req.title.as_deref(),
        req.content.as_deref(),
        req.status.as_deref(),
        req.is_sealed,
        req.effective_date,
        req.expiration_date,
        req.related_motions.as_deref().map(|m| m as &[Uuid]),
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete a judicial order. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM judicial_orders WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
