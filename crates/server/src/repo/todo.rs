use shared_types::{AppError, Todo};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new to-do item.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    user_id: i64,
    title: &str,
    description: Option<&str>,
) -> Result<Todo, AppError> {
    let row = sqlx::query_as!(
        Todo,
        r#"
        INSERT INTO todos (court_id, user_id, title, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id, court_id, user_id, title, description, completed, created_at, updated_at
        "#,
        court_id,
        user_id,
        title,
        description,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a to-do by ID within a court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Todo>, AppError> {
    let row = sqlx::query_as!(
        Todo,
        r#"
        SELECT id, court_id, user_id, title, description, completed, created_at, updated_at
        FROM todos
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

/// List all to-dos for a specific user within a court.
pub async fn list_by_user(
    pool: &Pool<Postgres>,
    court_id: &str,
    user_id: i64,
) -> Result<Vec<Todo>, AppError> {
    let rows = sqlx::query_as!(
        Todo,
        r#"
        SELECT id, court_id, user_id, title, description, completed, created_at, updated_at
        FROM todos
        WHERE court_id = $1 AND user_id = $2
        ORDER BY created_at DESC
        "#,
        court_id,
        user_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Delete a to-do. Returns true if a row was actually deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM todos WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// Toggle the completed status of a to-do. Returns the updated record or None.
pub async fn toggle(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Todo>, AppError> {
    let row = sqlx::query_as!(
        Todo,
        r#"
        UPDATE todos
        SET completed = NOT completed, updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, user_id, title, description, completed, created_at, updated_at
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}
