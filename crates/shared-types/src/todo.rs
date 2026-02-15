use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Domain Struct
// ---------------------------------------------------------------------------

/// A to-do item associated with a user within a court.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Todo {
    pub id: Uuid,
    pub court_id: String,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Request/Response DTOs
// ---------------------------------------------------------------------------

/// API response for a to-do item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TodoResponse {
    pub id: String,
    pub user_id: i64,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Todo> for TodoResponse {
    fn from(t: Todo) -> Self {
        Self {
            id: t.id.to_string(),
            user_id: t.user_id,
            title: t.title,
            description: t.description,
            completed: t.completed,
            created_at: t.created_at.to_rfc3339(),
            updated_at: t.updated_at.to_rfc3339(),
        }
    }
}

/// Request body for creating a new to-do item.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateTodoRequest {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub user_id: i64,
}
