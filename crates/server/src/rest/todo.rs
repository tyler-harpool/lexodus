use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{AppError, CreateTodoRequest, TodoResponse};
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct TodoListParams {
    pub user_id: i64,
}

// ---------------------------------------------------------------------------
// GET /api/todos
// ---------------------------------------------------------------------------

/// List to-do items for a specific user.
#[utoipa::path(
    get,
    path = "/api/todos",
    params(
        TodoListParams,
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "To-do list", body = Vec<TodoResponse>)
    ),
    tag = "todos"
)]
pub async fn list_todos(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<TodoListParams>,
) -> Result<Json<Vec<TodoResponse>>, AppError> {
    let todos = crate::repo::todo::list_by_user(&pool, &court.0, params.user_id).await?;
    let response: Vec<TodoResponse> = todos.into_iter().map(TodoResponse::from).collect();
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// POST /api/todos
// ---------------------------------------------------------------------------

/// Create a new to-do item.
#[utoipa::path(
    post,
    path = "/api/todos",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = CreateTodoRequest,
    responses(
        (status = 201, description = "To-do created", body = TodoResponse),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "todos"
)]
pub async fn create_todo(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateTodoRequest>,
) -> Result<(StatusCode, Json<TodoResponse>), AppError> {
    if body.title.trim().is_empty() {
        return Err(AppError::bad_request("Title cannot be empty"));
    }

    let todo = crate::repo::todo::create(
        &pool,
        &court.0,
        body.user_id,
        &body.title,
        body.description.as_deref(),
    )
    .await?;

    Ok((StatusCode::CREATED, Json(TodoResponse::from(todo))))
}

// ---------------------------------------------------------------------------
// GET /api/todos/{id}
// ---------------------------------------------------------------------------

/// Get a single to-do item by ID.
#[utoipa::path(
    get,
    path = "/api/todos/{id}",
    params(
        ("id" = String, Path, description = "Todo UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "To-do found", body = TodoResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "todos"
)]
pub async fn get_todo(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<TodoResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let todo = crate::repo::todo::find_by_id(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Todo {} not found", id)))?;

    Ok(Json(TodoResponse::from(todo)))
}

// ---------------------------------------------------------------------------
// DELETE /api/todos/{id}
// ---------------------------------------------------------------------------

/// Delete a to-do item.
#[utoipa::path(
    delete,
    path = "/api/todos/{id}",
    params(
        ("id" = String, Path, description = "Todo UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "To-do deleted"),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "todos"
)]
pub async fn delete_todo(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let deleted = crate::repo::todo::delete(&pool, &court.0, uuid).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("Todo {} not found", id)))
    }
}

// ---------------------------------------------------------------------------
// POST /api/todos/{id}/toggle
// ---------------------------------------------------------------------------

/// Toggle the completed status of a to-do item.
#[utoipa::path(
    post,
    path = "/api/todos/{id}/toggle",
    params(
        ("id" = String, Path, description = "Todo UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "To-do toggled", body = TodoResponse),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "todos"
)]
pub async fn toggle_todo(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
) -> Result<Json<TodoResponse>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    let todo = crate::repo::todo::toggle(&pool, &court.0, uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Todo {} not found", id)))?;

    Ok(Json(TodoResponse::from(todo)))
}
