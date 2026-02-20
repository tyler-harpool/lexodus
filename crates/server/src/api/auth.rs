// Server-only auth helpers for server functions.
// These are shared across all api/* modules.

use dioxus::prelude::*;
use shared_types::AuthUser;

use crate::db::get_db;
use crate::error_convert::{AppErrorExt, SqlxErrorExt};

/// Extract and validate the caller's identity from the current request.
/// Checks middleware-injected Claims first, falls back to cookie parsing.
/// Returns the validated Claims or an "Authentication required" error.
pub(crate) fn require_auth() -> Result<crate::auth::jwt::Claims, ServerFnError> {
    use crate::auth::{cookies, jwt};
    use shared_types::AppError;

    let ctx = dioxus::fullstack::FullstackContext::current()
        .ok_or_else(|| AppError::unauthorized("Authentication required").into_server_fn_error())?;

    let parts = ctx.parts_mut();

    // Primary: Claims already validated by auth middleware
    if let Some(claims) = parts.extensions.get::<jwt::Claims>() {
        return Ok(claims.clone());
    }

    // Fallback: parse access token from cookies/Bearer header
    let headers = parts.headers.clone();
    let token = cookies::extract_access_token(&headers)
        .ok_or_else(|| AppError::unauthorized("Authentication required").into_server_fn_error())?;

    jwt::validate_access_token(&token)
        .map_err(|_| AppError::unauthorized("Invalid or expired token").into_server_fn_error())
}

/// Require the caller to be authenticated with the "admin" role.
pub(crate) fn require_admin() -> Result<crate::auth::jwt::Claims, ServerFnError> {
    use shared_types::AppError;

    let claims = require_auth()?;
    if claims.role != "admin" {
        return Err(AppError::forbidden("Admin role required").into_server_fn_error());
    }
    Ok(claims)
}

/// Fetch a full AuthUser by user ID, including court roles and tiers.
/// Returns None and clears cookies if the user no longer exists.
pub(crate) async fn fetch_auth_user(user_id: i64) -> Result<Option<AuthUser>, ServerFnError> {
    use shared_types::UserTier;

    let db = get_db().await;
    let user = sqlx::query!(
        r#"SELECT id, username, display_name, email, password_hash, role, tier, avatar_url,
                  email_verified, phone_number, phone_verified,
                  email_notifications_enabled, push_notifications_enabled, weekly_digest_enabled,
                  court_roles, preferred_court_id
           FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_optional(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    match user {
        Some(u) => {
            let court_roles: std::collections::HashMap<String, String> =
                serde_json::from_value(u.court_roles).unwrap_or_default();

            // Load per-court tiers for all courts the user belongs to
            let court_tiers: std::collections::HashMap<String, String> = if !court_roles.is_empty()
            {
                let court_ids: Vec<&str> = court_roles.keys().map(|s| s.as_str()).collect();
                let rows = sqlx::query!(
                    "SELECT id, tier FROM courts WHERE id = ANY($1)",
                    &court_ids as &[&str]
                )
                .fetch_all(db)
                .await
                .unwrap_or_default();
                rows.into_iter().map(|r| (r.id, r.tier)).collect()
            } else {
                std::collections::HashMap::new()
            };

            // Resolve linked judge/attorney by email
            let email = u.email.clone().unwrap_or_default();
            let linked_judge_id: Option<String> = sqlx::query_scalar(
                "SELECT id::TEXT FROM judges WHERE email = $1 LIMIT 1",
            )
            .bind(&email)
            .fetch_optional(db)
            .await
            .ok()
            .flatten();

            let linked_attorney_id: Option<String> = sqlx::query_scalar(
                "SELECT id::TEXT FROM attorneys WHERE email = $1 LIMIT 1",
            )
            .bind(&email)
            .fetch_optional(db)
            .await
            .ok()
            .flatten();

            Ok(Some(AuthUser {
                id: u.id,
                username: u.username,
                display_name: u.display_name,
                email,
                role: u.role,
                tier: UserTier::from_str_or_default(&u.tier),
                avatar_url: u.avatar_url,
                email_verified: u.email_verified,
                phone_number: u.phone_number,
                phone_verified: u.phone_verified,
                email_notifications_enabled: u.email_notifications_enabled,
                push_notifications_enabled: u.push_notifications_enabled,
                weekly_digest_enabled: u.weekly_digest_enabled,
                has_password: u.password_hash.is_some(),
                court_roles,
                court_tiers,
                preferred_court_id: u.preferred_court_id,
                linked_judge_id,
                linked_attorney_id,
            }))
        }
        None => {
            // User no longer exists — clear stale auth cookies to prevent
            // the client from getting stuck in a broken authenticated state
            crate::auth::cookies::schedule_clear_cookies();
            tracing::warn!(
                user_id,
                "Auth token references non-existent user, clearing cookies"
            );
            Ok(None)
        }
    }
}

/// Parse a uscourts.gov email domain to extract the court identifier.
/// e.g., "user@arwd.uscourts.gov" → Some("arwd")
pub(crate) fn parse_uscourts_court(email: &str) -> Option<String> {
    let domain = email.rsplit('@').next()?;
    let parts: Vec<&str> = domain.split('.').collect();
    if parts.len() == 3 && parts[1] == "uscourts" && parts[2] == "gov" {
        Some(parts[0].to_lowercase())
    } else {
        None
    }
}

/// Extract the X-Court-District header from the current Dioxus server context.
pub(crate) fn extract_court_header_sfn() -> Option<String> {
    let ctx = dioxus::fullstack::FullstackContext::current()?;
    let parts = ctx.parts_mut();
    parts
        .headers
        .get("x-court-district")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Require membership management access for server functions.
/// - Admin: always OK, no header required.
/// - Clerk: header required, must be clerk in header court, target must match header.
/// - Others: forbidden.
pub(crate) fn require_membership_access_sfn(
    claims: &crate::auth::jwt::Claims,
    header_court: Option<&str>,
    target_court: &str,
) -> Result<(), ServerFnError> {
    use shared_types::AppError;

    if claims.role == "admin" {
        return Ok(());
    }

    let header = header_court
        .ok_or_else(|| AppError::bad_request("X-Court-District header required").into_server_fn_error())?;

    let is_clerk = claims
        .court_roles
        .get(header)
        .map(|r| r == "clerk")
        .unwrap_or(false);

    if !is_clerk {
        return Err(AppError::forbidden("Admin or clerk role required").into_server_fn_error());
    }

    if target_court != header {
        return Err(AppError::not_found("Resource not found").into_server_fn_error());
    }

    Ok(())
}
