use dioxus::prelude::*;
use shared_types::{CourtMembership, UserWithMembership};

#[cfg(feature = "server")]
use crate::db::get_db;

#[cfg(feature = "server")]
use crate::error_convert::{AppErrorExt, SqlxErrorExt};

#[cfg(feature = "server")]
use super::auth::*;

// ── Court Membership Server Functions ──────────────────

/// A DB row shape for the list_users_with_memberships query.
/// We can't use `query_as!` with `UserWithMembership` directly because
/// the court_role column comes from a JSONB extraction expression.
#[cfg(feature = "server")]
struct UserRow {
    id: i64,
    username: String,
    display_name: String,
    role: String,
    tier: String,
    email: Option<String>,
    phone_number: Option<String>,
    court_roles: serde_json::Value,
}

/// List all users with their role in a specific court.
/// - Admin: all users.
/// - Clerk: all users (membership column still scoped to the requested court).
#[server]
pub async fn list_users_with_memberships(
    court_id: String,
) -> Result<Vec<UserWithMembership>, ServerFnError> {
    use shared_types::AppError;

    let claims = require_auth()?;
    let db = get_db().await;
    let header_court = extract_court_header_sfn();

    let is_admin = claims.role == "admin";

    if !is_admin {
        let header = header_court.as_deref()
            .ok_or_else(|| AppError::bad_request("X-Court-District header required").into_server_fn_error())?;

        let is_clerk = claims
            .court_roles
            .get(header)
            .map(|r| r == "clerk")
            .unwrap_or(false);

        if !is_clerk {
            return Err(AppError::forbidden("Admin or clerk role required").into_server_fn_error());
        }

        // Clerk can only list for their own court
        if court_id != header {
            return Err(AppError::not_found("Resource not found").into_server_fn_error());
        }
    }

    let rows = sqlx::query_as!(
        UserRow,
        "SELECT id, username, display_name, role, tier, email, phone_number, court_roles FROM users"
    )
    .fetch_all(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    let users: Vec<UserWithMembership> = rows
        .into_iter()
        .map(|row| {
            let court_role = row.court_roles
                .get(&court_id)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Parse full court_roles JSONB into HashMap
            let all_court_roles: std::collections::HashMap<String, String> =
                serde_json::from_value(row.court_roles).unwrap_or_default();

            UserWithMembership {
                id: row.id,
                username: row.username,
                display_name: row.display_name,
                role: row.role,
                tier: row.tier,
                email: row.email.unwrap_or_default(),
                phone_number: row.phone_number,
                court_role,
                all_court_roles,
            }
        })
        .collect();

    Ok(users)
}

/// Get a user's court memberships.
/// - Admin: returns all memberships.
/// - Clerk: returns only the membership for the header court.
#[server]
pub async fn get_user_memberships(user_id: i64) -> Result<Vec<CourtMembership>, ServerFnError> {
    use shared_types::AppError;

    let claims = require_auth()?;
    let db = get_db().await;
    let header_court = extract_court_header_sfn();

    let is_admin = claims.role == "admin";

    if !is_admin {
        let header = header_court.as_deref()
            .ok_or_else(|| AppError::bad_request("X-Court-District header required").into_server_fn_error())?;

        let is_clerk = claims
            .court_roles
            .get(header)
            .map(|r| r == "clerk")
            .unwrap_or(false);

        if !is_clerk {
            return Err(AppError::forbidden("Admin or clerk role required").into_server_fn_error());
        }
    }

    let roles = crate::repo::court_role_request::get_user_court_roles(db, user_id)
        .await
        .map_err(|e| e.into_server_fn_error())?;

    let memberships: Vec<CourtMembership> = if is_admin {
        roles
            .into_iter()
            .map(|(court_id, role)| CourtMembership { court_id, role })
            .collect()
    } else {
        // Clerk: only reveal their header court
        let header = header_court.unwrap_or_default();
        roles
            .into_iter()
            .filter(|(court_id, _)| court_id == &header)
            .map(|(court_id, role)| CourtMembership { court_id, role })
            .collect()
    };

    Ok(memberships)
}

/// Set a user's role in a specific court.
/// Admin: any court. Clerk: only header court.
#[server]
pub async fn set_user_court_role(
    user_id: i64,
    court_id: String,
    role: String,
) -> Result<(), ServerFnError> {
    use shared_types::AppError;

    let claims = require_auth()?;
    let db = get_db().await;
    let header_court = extract_court_header_sfn();

    require_membership_access_sfn(&claims, header_court.as_deref(), &court_id)?;

    let valid_roles = ["attorney", "clerk", "judge"];
    if !valid_roles.contains(&role.as_str()) {
        return Err(AppError::bad_request(format!(
            "Invalid role '{}'. Must be one of: {}",
            role,
            valid_roles.join(", ")
        ))
        .into_server_fn_error());
    }

    crate::repo::court_role_request::set_court_role(db, user_id, &court_id, &role)
        .await
        .map_err(|e| e.into_server_fn_error())?;

    Ok(())
}

/// Remove a user's role in a specific court.
/// Admin: any court. Clerk: only header court.
#[server]
pub async fn remove_user_court_role(
    user_id: i64,
    court_id: String,
) -> Result<(), ServerFnError> {
    let claims = require_auth()?;
    let db = get_db().await;
    let header_court = extract_court_header_sfn();

    require_membership_access_sfn(&claims, header_court.as_deref(), &court_id)?;

    crate::repo::court_role_request::remove_court_role(db, user_id, &court_id)
        .await
        .map_err(|e| e.into_server_fn_error())?;

    Ok(())
}
