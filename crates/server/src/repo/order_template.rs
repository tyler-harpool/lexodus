use shared_types::{AppError, CreateOrderTemplateRequest, OrderTemplate, UpdateOrderTemplateRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new order template.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateOrderTemplateRequest,
) -> Result<OrderTemplate, AppError> {
    let row = sqlx::query_as!(
        OrderTemplate,
        r#"
        INSERT INTO order_templates
            (court_id, order_type, name, description, content_template)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, court_id, order_type, name, description,
                  content_template, is_active, created_at, updated_at
        "#,
        court_id,
        req.order_type,
        req.name,
        req.description.as_deref(),
        req.content_template,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find an order template by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<OrderTemplate>, AppError> {
    let row = sqlx::query_as!(
        OrderTemplate,
        r#"
        SELECT id, court_id, order_type, name, description,
               content_template, is_active, created_at, updated_at
        FROM order_templates
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

/// List all order templates for a court.
pub async fn list_all(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<OrderTemplate>, AppError> {
    let rows = sqlx::query_as!(
        OrderTemplate,
        r#"
        SELECT id, court_id, order_type, name, description,
               content_template, is_active, created_at, updated_at
        FROM order_templates
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

/// List only active order templates for a court.
pub async fn list_active(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<OrderTemplate>, AppError> {
    let rows = sqlx::query_as!(
        OrderTemplate,
        r#"
        SELECT id, court_id, order_type, name, description,
               content_template, is_active, created_at, updated_at
        FROM order_templates
        WHERE court_id = $1 AND is_active = true
        ORDER BY name ASC
        "#,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Update an order template with only the provided fields.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: UpdateOrderTemplateRequest,
) -> Result<Option<OrderTemplate>, AppError> {
    let row = sqlx::query_as!(
        OrderTemplate,
        r#"
        UPDATE order_templates SET
            order_type       = COALESCE($3, order_type),
            name             = COALESCE($4, name),
            description      = COALESCE($5, description),
            content_template = COALESCE($6, content_template),
            is_active        = COALESCE($7, is_active),
            updated_at       = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, order_type, name, description,
                  content_template, is_active, created_at, updated_at
        "#,
        id,
        court_id,
        req.order_type.as_deref(),
        req.name.as_deref(),
        req.description.as_deref(),
        req.content_template.as_deref(),
        req.is_active,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete an order template. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM order_templates WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
