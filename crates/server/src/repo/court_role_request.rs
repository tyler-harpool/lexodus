use shared_types::AppError;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

/// A court role request row from the database.
pub struct CourtRoleRequest {
    pub id: Uuid,
    pub user_id: i64,
    pub court_id: String,
    pub requested_role: String,
    pub status: String,
    pub requested_at: chrono::DateTime<chrono::Utc>,
    pub reviewed_by: Option<i64>,
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
}

/// A court role request row joined with user data (display_name, email).
pub struct CourtRoleRequestWithUser {
    pub id: Uuid,
    pub user_id: i64,
    pub court_id: String,
    pub requested_role: String,
    pub status: String,
    pub requested_at: chrono::DateTime<chrono::Utc>,
    pub reviewed_by: Option<i64>,
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
    pub user_display_name: Option<String>,
    pub user_email: Option<String>,
}

/// Create a new pending court role request.
pub async fn create_request(
    pool: &Pool<Postgres>,
    user_id: i64,
    court_id: &str,
    role: &str,
) -> Result<CourtRoleRequest, AppError> {
    let row = sqlx::query_as!(
        CourtRoleRequest,
        r#"INSERT INTO court_role_requests (user_id, court_id, requested_role)
           VALUES ($1, $2, $3)
           RETURNING id, user_id, court_id, requested_role, status,
                     requested_at, reviewed_by, reviewed_at, notes"#,
        user_id,
        court_id,
        role,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint().is_some() => {
            AppError::bad_request("A pending request already exists for this court")
        }
        _ => AppError::internal(e.to_string()),
    })?;

    Ok(row)
}

/// Approve a court role request: updates the request status AND the user's court_roles JSONB.
pub async fn approve(
    pool: &Pool<Postgres>,
    request_id: Uuid,
    reviewer_id: i64,
) -> Result<CourtRoleRequest, AppError> {
    let request = sqlx::query_as!(
        CourtRoleRequest,
        r#"UPDATE court_role_requests
           SET status = 'approved', reviewed_by = $2, reviewed_at = NOW()
           WHERE id = $1 AND status = 'pending'
           RETURNING id, user_id, court_id, requested_role, status,
                     requested_at, reviewed_by, reviewed_at, notes"#,
        request_id,
        reviewer_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?
    .ok_or_else(|| AppError::not_found("Request not found or already reviewed"))?;

    // Update the user's court_roles JSONB
    let role_obj = serde_json::json!({ &request.court_id: &request.requested_role });
    sqlx::query!(
        "UPDATE users SET court_roles = court_roles || $1 WHERE id = $2",
        role_obj,
        request.user_id,
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(request)
}

/// Deny a court role request.
pub async fn deny(
    pool: &Pool<Postgres>,
    request_id: Uuid,
    reviewer_id: i64,
    notes: Option<&str>,
) -> Result<CourtRoleRequest, AppError> {
    let request = sqlx::query_as!(
        CourtRoleRequest,
        r#"UPDATE court_role_requests
           SET status = 'denied', reviewed_by = $2, reviewed_at = NOW(), notes = $3
           WHERE id = $1 AND status = 'pending'
           RETURNING id, user_id, court_id, requested_role, status,
                     requested_at, reviewed_by, reviewed_at, notes"#,
        request_id,
        reviewer_id,
        notes,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?
    .ok_or_else(|| AppError::not_found("Request not found or already reviewed"))?;

    Ok(request)
}

/// Fetch a single court role request by ID.
pub async fn get_by_id(
    pool: &Pool<Postgres>,
    request_id: Uuid,
) -> Result<Option<CourtRoleRequest>, AppError> {
    let row = sqlx::query_as!(
        CourtRoleRequest,
        r#"SELECT id, user_id, court_id, requested_role, status,
                  requested_at, reviewed_by, reviewed_at, notes
           FROM court_role_requests
           WHERE id = $1"#,
        request_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(row)
}

/// List all pending court role requests with user data.
pub async fn list_pending(pool: &Pool<Postgres>) -> Result<Vec<CourtRoleRequestWithUser>, AppError> {
    let rows = sqlx::query_as!(
        CourtRoleRequestWithUser,
        r#"SELECT crr.id, crr.user_id, crr.court_id, crr.requested_role, crr.status,
                  crr.requested_at, crr.reviewed_by, crr.reviewed_at, crr.notes,
                  u.display_name AS user_display_name,
                  u.email AS user_email
           FROM court_role_requests crr
           LEFT JOIN users u ON u.id = crr.user_id
           WHERE crr.status = 'pending'
           ORDER BY crr.requested_at ASC"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(rows)
}

/// List pending court role requests for a specific court, with user data.
pub async fn list_pending_for_court(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<CourtRoleRequestWithUser>, AppError> {
    let rows = sqlx::query_as!(
        CourtRoleRequestWithUser,
        r#"SELECT crr.id, crr.user_id, crr.court_id, crr.requested_role, crr.status,
                  crr.requested_at, crr.reviewed_by, crr.reviewed_at, crr.notes,
                  u.display_name AS user_display_name,
                  u.email AS user_email
           FROM court_role_requests crr
           LEFT JOIN users u ON u.id = crr.user_id
           WHERE crr.court_id = $1 AND crr.status = 'pending'
           ORDER BY crr.requested_at ASC"#,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(rows)
}

/// List all court role requests for a specific user.
pub async fn list_by_user(
    pool: &Pool<Postgres>,
    user_id: i64,
) -> Result<Vec<CourtRoleRequest>, AppError> {
    let rows = sqlx::query_as!(
        CourtRoleRequest,
        r#"SELECT id, user_id, court_id, requested_role, status,
                  requested_at, reviewed_by, reviewed_at, notes
           FROM court_role_requests
           WHERE user_id = $1
           ORDER BY requested_at DESC"#,
        user_id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(rows)
}

/// Get a user's court_roles from the database.
pub async fn get_user_court_roles(
    pool: &Pool<Postgres>,
    user_id: i64,
) -> Result<std::collections::HashMap<String, String>, AppError> {
    let roles: std::collections::HashMap<String, String> = sqlx::query_scalar!(
        "SELECT court_roles FROM users WHERE id = $1",
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?
    .and_then(|v| serde_json::from_value(v).ok())
    .unwrap_or_default();

    Ok(roles)
}

/// Directly set a court role for a user (admin operation).
pub async fn set_court_role(
    pool: &Pool<Postgres>,
    user_id: i64,
    court_id: &str,
    role: &str,
) -> Result<(), AppError> {
    let role_obj = serde_json::json!({ court_id: role });
    sqlx::query!(
        "UPDATE users SET court_roles = court_roles || $1 WHERE id = $2",
        role_obj,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(())
}

/// Remove a court role from a user (admin operation).
pub async fn remove_court_role(
    pool: &Pool<Postgres>,
    user_id: i64,
    court_id: &str,
) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE users SET court_roles = court_roles - $1 WHERE id = $2",
        court_id,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(())
}

/// Convert a DB row to the API response type.
pub fn to_response(r: &CourtRoleRequest) -> shared_types::CourtRoleRequestResponse {
    shared_types::CourtRoleRequestResponse {
        id: r.id.to_string(),
        user_id: r.user_id,
        court_id: r.court_id.clone(),
        requested_role: r.requested_role.clone(),
        status: r.status.clone(),
        requested_at: r.requested_at.to_rfc3339(),
        reviewed_by: r.reviewed_by,
        reviewed_at: r.reviewed_at.map(|t| t.to_rfc3339()),
        notes: r.notes.clone(),
        user_display_name: None,
        user_email: None,
    }
}

/// Convert a joined DB row (with user data) to the API response type.
pub fn to_response_with_user(r: &CourtRoleRequestWithUser) -> shared_types::CourtRoleRequestResponse {
    shared_types::CourtRoleRequestResponse {
        id: r.id.to_string(),
        user_id: r.user_id,
        court_id: r.court_id.clone(),
        requested_role: r.requested_role.clone(),
        status: r.status.clone(),
        requested_at: r.requested_at.to_rfc3339(),
        reviewed_by: r.reviewed_by,
        reviewed_at: r.reviewed_at.map(|t| t.to_rfc3339()),
        notes: r.notes.clone(),
        user_display_name: r.user_display_name.clone(),
        user_email: r.user_email.clone(),
    }
}
