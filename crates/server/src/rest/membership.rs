use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AppError, CourtRoleRequestResponse, ReviewCourtRoleRequest, SetCourtRoleRequest,
};
use crate::auth::extractors::AuthRequired;
use crate::repo::court_role_request;

/// Extract the X-Court-District header value from request headers.
fn extract_court_header(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-court-district")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Require that the caller is either a platform admin or a clerk in the target court.
///
/// - **Admin** (`claims.role == "admin"`): always allowed, header optional.
/// - **Clerk**: must provide `X-Court-District` header (400 if missing),
///   must be clerk in that court, and `target_court_id` must match header (404 if cross-tenant).
/// - **Everyone else**: 403.
fn require_membership_access(
    claims: &crate::auth::jwt::Claims,
    header_court_id: Option<&str>,
    target_court_id: &str,
) -> Result<(), AppError> {
    // Platform admin: full access, no header required
    if claims.role == "admin" {
        return Ok(());
    }

    // Non-admin must provide the header
    let header = header_court_id
        .ok_or_else(|| AppError::bad_request("X-Court-District header required"))?;

    // Must be a clerk in the header court
    let is_clerk_in_header = claims
        .court_roles
        .get(header)
        .map(|r| r == "clerk")
        .unwrap_or(false);

    if !is_clerk_in_header {
        return Err(AppError::forbidden("Admin or clerk role required"));
    }

    // Target court must match header court — cross-tenant => 404
    if target_court_id != header {
        return Err(AppError::not_found("Resource not found"));
    }

    Ok(())
}

/// Require that the caller is admin or a clerk in at least one court.
/// Returns (is_admin, clerk_courts) for further filtering.
fn require_list_access(
    claims: &crate::auth::jwt::Claims,
) -> Result<(bool, Vec<String>), AppError> {
    if claims.role == "admin" {
        return Ok((true, vec![]));
    }

    let clerk_courts: Vec<String> = claims
        .court_roles
        .iter()
        .filter(|(_, role)| *role == "clerk")
        .map(|(court, _)| court.clone())
        .collect();

    if clerk_courts.is_empty() {
        return Err(AppError::forbidden("Admin or clerk role required"));
    }

    Ok((false, clerk_courts))
}

/// GET /api/admin/court-role-requests
///
/// List pending court role requests.
/// - Admin: all pending requests.
/// - Clerk: only requests for courts they manage.
#[utoipa::path(
    get,
    path = "/api/admin/court-role-requests",
    responses(
        (status = 200, description = "Pending court role requests", body = Vec<CourtRoleRequestResponse>),
        (status = 403, description = "Insufficient role", body = AppError)
    ),
    tag = "admin"
)]
pub async fn list_pending_requests(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
) -> Result<Json<Vec<CourtRoleRequestResponse>>, AppError> {
    let (is_admin, clerk_courts) = require_list_access(&auth.0)?;

    let requests = court_role_request::list_pending(&pool).await?;

    let responses: Vec<CourtRoleRequestResponse> = if is_admin {
        requests.iter().map(court_role_request::to_response).collect()
    } else {
        requests
            .iter()
            .filter(|r| clerk_courts.contains(&r.court_id))
            .map(court_role_request::to_response)
            .collect()
    };

    Ok(Json(responses))
}

/// POST /api/admin/court-role-requests/{id}/approve
///
/// Approve a pending court role request.
/// Requires admin or clerk in the request's court.
#[utoipa::path(
    post,
    path = "/api/admin/court-role-requests/{id}/approve",
    params(
        ("id" = String, Path, description = "Court role request UUID")
    ),
    responses(
        (status = 200, description = "Request approved", body = CourtRoleRequestResponse),
        (status = 403, description = "Insufficient role", body = AppError),
        (status = 404, description = "Request not found", body = AppError)
    ),
    tag = "admin"
)]
pub async fn approve_request(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<CourtRoleRequestResponse>, AppError> {
    let request_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid request ID"))?;

    // Load request first to check its court_id
    let pending = court_role_request::get_by_id(&pool, request_id)
        .await?
        .ok_or_else(|| AppError::not_found("Request not found or already reviewed"))?;

    let header_court = extract_court_header(&headers);
    require_membership_access(&auth.0, header_court.as_deref(), &pending.court_id)?;

    let request = court_role_request::approve(&pool, request_id, auth.0.sub).await?;
    Ok(Json(court_role_request::to_response(&request)))
}

/// POST /api/admin/court-role-requests/{id}/deny
///
/// Deny a pending court role request.
/// Requires admin or clerk in the request's court.
#[utoipa::path(
    post,
    path = "/api/admin/court-role-requests/{id}/deny",
    request_body = ReviewCourtRoleRequest,
    params(
        ("id" = String, Path, description = "Court role request UUID")
    ),
    responses(
        (status = 200, description = "Request denied", body = CourtRoleRequestResponse),
        (status = 403, description = "Insufficient role", body = AppError),
        (status = 404, description = "Request not found", body = AppError)
    ),
    tag = "admin"
)]
pub async fn deny_request(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<ReviewCourtRoleRequest>,
) -> Result<Json<CourtRoleRequestResponse>, AppError> {
    let request_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid request ID"))?;

    // Load request first to check its court_id
    let pending = court_role_request::get_by_id(&pool, request_id)
        .await?
        .ok_or_else(|| AppError::not_found("Request not found or already reviewed"))?;

    let header_court = extract_court_header(&headers);
    require_membership_access(&auth.0, header_court.as_deref(), &pending.court_id)?;

    let request = court_role_request::deny(
        &pool,
        request_id,
        auth.0.sub,
        body.notes.as_deref(),
    )
    .await?;

    Ok(Json(court_role_request::to_response(&request)))
}

/// GET /api/admin/court-memberships/user/{user_id}
///
/// List a user's court roles.
/// - Admin: returns all court roles.
/// - Clerk: returns only role for the header court (do not reveal other courts).
#[utoipa::path(
    get,
    path = "/api/admin/court-memberships/user/{user_id}",
    params(
        ("user_id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User court roles (court_id -> role)"),
        (status = 403, description = "Insufficient role", body = AppError)
    ),
    tag = "admin"
)]
pub async fn get_user_court_roles(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
) -> Result<Json<std::collections::HashMap<String, String>>, AppError> {
    let is_admin = auth.0.role == "admin";

    if !is_admin {
        let header = extract_court_header(&headers)
            .ok_or_else(|| AppError::bad_request("X-Court-District header required"))?;

        let is_clerk = auth.0
            .court_roles
            .get(&header)
            .map(|r| r == "clerk")
            .unwrap_or(false);

        if !is_clerk {
            return Err(AppError::forbidden("Admin or clerk role required"));
        }

        // Clerk: only reveal role for their header court
        let roles = court_role_request::get_user_court_roles(&pool, user_id).await?;
        let filtered: std::collections::HashMap<String, String> = roles
            .into_iter()
            .filter(|(court_id, _)| court_id == &header)
            .collect();
        return Ok(Json(filtered));
    }

    let roles = court_role_request::get_user_court_roles(&pool, user_id).await?;
    Ok(Json(roles))
}

/// DELETE /api/admin/court-memberships/{user_id}/{court_id}
///
/// Remove a court role from a user. Requires admin or clerk in target court.
#[utoipa::path(
    delete,
    path = "/api/admin/court-memberships/{user_id}/{court_id}",
    params(
        ("user_id" = i64, Path, description = "User ID"),
        ("court_id" = String, Path, description = "Court district ID")
    ),
    responses(
        (status = 204, description = "Court role removed"),
        (status = 403, description = "Insufficient role", body = AppError)
    ),
    tag = "admin"
)]
pub async fn remove_court_role(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
    headers: HeaderMap,
    Path((user_id, court_id)): Path<(i64, String)>,
) -> Result<StatusCode, AppError> {
    let header_court = extract_court_header(&headers);
    require_membership_access(&auth.0, header_court.as_deref(), &court_id)?;

    court_role_request::remove_court_role(&pool, user_id, &court_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// PUT /api/admin/court-memberships
///
/// Directly set a user's court role. Requires admin or clerk in target court.
#[utoipa::path(
    put,
    path = "/api/admin/court-memberships",
    request_body = SetCourtRoleRequest,
    responses(
        (status = 204, description = "Court role set"),
        (status = 400, description = "Invalid role", body = AppError),
        (status = 403, description = "Insufficient role", body = AppError)
    ),
    tag = "admin"
)]
pub async fn set_court_role(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
    headers: HeaderMap,
    Json(body): Json<SetCourtRoleRequest>,
) -> Result<StatusCode, AppError> {
    let header_court = extract_court_header(&headers);
    require_membership_access(&auth.0, header_court.as_deref(), &body.court_id)?;

    let valid_roles = ["attorney", "clerk", "judge"];
    if !valid_roles.contains(&body.role.as_str()) {
        return Err(AppError::bad_request(format!(
            "Invalid role '{}'. Must be one of: {}",
            body.role,
            valid_roles.join(", ")
        )));
    }

    court_role_request::set_court_role(&pool, body.user_id, &body.court_id, &body.role).await?;
    Ok(StatusCode::NO_CONTENT)
}
