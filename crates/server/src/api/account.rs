use dioxus::prelude::*;
use shared_types::{
    AuthUser, CheckoutResponse, DeviceFlowInitResponse, DeviceFlowPollResponse, MessageResponse,
    SubscriptionStatus,
};

#[cfg(feature = "server")]
use crate::db::get_db;

#[cfg(feature = "server")]
use crate::error_convert::{AppErrorExt, SqlxErrorExt, ValidateRequest};

#[cfg(feature = "server")]
use shared_types::{DeviceAuthStatus, UserTier};

#[cfg(feature = "server")]
use super::auth::*;

/// Register a new user. Sets HTTP-only auth cookies on success.
#[cfg_attr(feature = "server", tracing::instrument(skip(password)))]
#[server]
pub async fn register(
    username: String,
    email: String,
    password: String,
    display_name: String,
) -> Result<AuthUser, ServerFnError> {
    use crate::auth::{cookies, jwt, password as pw};
    use shared_types::{AppError, RegisterRequest};

    let req = RegisterRequest {
        username: username.clone(),
        email: email.clone(),
        password: password.clone(),
        display_name: display_name.clone(),
    };
    req.validate_request()
        .map_err(|e| e.into_server_fn_error())?;

    let password_hash = pw::hash_password(&password)
        .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

    let db = get_db().await;
    let user = sqlx::query!(
        r#"INSERT INTO users (username, email, password_hash, display_name)
           VALUES ($1, $2, $3, $4)
           RETURNING id, username, display_name, email, role, tier, avatar_url,
                     email_verified, phone_number, phone_verified,
                     email_notifications_enabled, push_notifications_enabled, weekly_digest_enabled"#,
        username,
        email,
        password_hash,
        display_name
    )
    .fetch_one(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    let user_email = user.email.unwrap_or_default();
    let user_role = crate::auth::maybe_promote_admin(db, user.id, &user_email, user.role).await;
    let user_tier = UserTier::from_str_or_default(&user.tier);

    // Auto-populate court_roles from email domain for uscourts.gov staff
    let mut court_roles = std::collections::HashMap::<String, String>::new();
    if let Some(court_id) = parse_uscourts_court(&user_email) {
        court_roles.insert(court_id.clone(), "clerk".to_string());
        // Persist to users table
        let court_roles_json = serde_json::to_value(&court_roles).unwrap_or_default();
        let _ = sqlx::query!(
            "UPDATE users SET court_roles = $1 WHERE id = $2",
            court_roles_json,
            user.id,
        )
        .execute(db)
        .await;
        // Create an approved audit trail record
        let _ = sqlx::query!(
            r#"INSERT INTO court_role_requests (user_id, court_id, requested_role, status, reviewed_at, notes)
               VALUES ($1, $2, 'clerk', 'approved', NOW(), 'Auto-approved via uscourts.gov email domain')
               ON CONFLICT (user_id, court_id, status) DO NOTHING"#,
            user.id,
            court_id,
        )
        .execute(db)
        .await;
    }

    let access_token =
        jwt::create_access_token(user.id, &user_email, &user_role, user_tier.as_str(), &court_roles)
            .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

    let (refresh_token, expires_at) =
        jwt::create_refresh_token(user.id, &user_email, &user_role, user_tier.as_str(), &court_roles)
            .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

    // Store the hash of the refresh token — never persist raw JWTs
    let refresh_hash = jwt::hash_token(&refresh_token);
    sqlx::query!(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        user.id,
        refresh_hash,
        expires_at
    )
    .execute(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    // Schedule cookies to be set by the middleware
    cookies::schedule_auth_cookies(&access_token, &refresh_token);

    // Fire-and-forget: send welcome + verification emails (only if mailgun enabled)
    if crate::config::feature_flags().mailgun {
        let db_ref = db.clone();
        let email_clone = user_email.clone();
        let name_clone = user.display_name.clone();
        let uid = user.id;
        tokio::spawn(async move {
            crate::mailgun::send_welcome_email(&email_clone, &name_clone).await;
            if let Ok(token) = crate::mailgun::create_verification_token(&db_ref, uid).await {
                crate::mailgun::send_verification_email(&email_clone, &token).await;
            }
        });
    }

    Ok(AuthUser {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        email: user_email,
        role: user_role,
        tier: user_tier,
        avatar_url: user.avatar_url,
        email_verified: user.email_verified,
        phone_number: user.phone_number,
        phone_verified: user.phone_verified,
        email_notifications_enabled: user.email_notifications_enabled,
        push_notifications_enabled: user.push_notifications_enabled,
        weekly_digest_enabled: user.weekly_digest_enabled,
        has_password: true,
        court_roles,
        court_tiers: Default::default(),
        preferred_court_id: None,
        linked_judge_id: None,
        linked_attorney_id: None,
    })
}

/// Login with email and password. Sets HTTP-only auth cookies on success.
#[cfg_attr(feature = "server", tracing::instrument(skip(password)))]
#[server]
pub async fn login(email: String, password: String) -> Result<AuthUser, ServerFnError> {
    use crate::auth::{cookies, jwt, password as pw};
    use shared_types::{AppError, LoginRequest};

    let req = LoginRequest {
        email: email.clone(),
        password: password.clone(),
    };
    req.validate_request()
        .map_err(|e| e.into_server_fn_error())?;

    let db = get_db().await;
    let user = sqlx::query!(
        r#"SELECT id, username, display_name, email, password_hash, role, tier, avatar_url,
                  email_verified, phone_number, phone_verified, oauth_provider,
                  email_notifications_enabled, push_notifications_enabled, weekly_digest_enabled
           FROM users WHERE email = $1"#,
        email
    )
    .fetch_optional(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?
    .ok_or_else(|| AppError::unauthorized("Invalid email or password").into_server_fn_error())?;

    let password_hash = match user.password_hash {
        Some(ref hash) => hash.clone(),
        None => {
            let provider = user
                .oauth_provider
                .as_deref()
                .unwrap_or("a social provider");
            let msg = if crate::config::feature_flags().oauth {
                format!(
                    "This account uses {} sign-in. Please use that to log in.",
                    provider.to_uppercase()
                )
            } else {
                "NO_PASSWORD".to_string()
            };
            return Err(AppError::unauthorized(msg).into_server_fn_error());
        }
    };

    let valid = pw::verify_password(&password, &password_hash)
        .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

    if !valid {
        return Err(AppError::unauthorized("Invalid email or password").into_server_fn_error());
    }

    let user_email = user.email.unwrap_or_default();
    let user_role = crate::auth::maybe_promote_admin(db, user.id, &user_email, user.role).await;
    let user_tier = UserTier::from_str_or_default(&user.tier);

    // Load court_roles from DB
    let court_roles: std::collections::HashMap<String, String> = sqlx::query_scalar!(
        "SELECT court_roles FROM users WHERE id = $1",
        user.id
    )
    .fetch_optional(db)
    .await
    .ok()
    .flatten()
    .and_then(|v| serde_json::from_value(v).ok())
    .unwrap_or_default();

    let access_token =
        jwt::create_access_token(user.id, &user_email, &user_role, user_tier.as_str(), &court_roles)
            .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

    let (refresh_token, expires_at) =
        jwt::create_refresh_token(user.id, &user_email, &user_role, user_tier.as_str(), &court_roles)
            .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

    // Store the hash of the refresh token — never persist raw JWTs
    let refresh_hash = jwt::hash_token(&refresh_token);
    sqlx::query!(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        user.id,
        refresh_hash,
        expires_at
    )
    .execute(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    // Schedule cookies to be set by the middleware
    cookies::schedule_auth_cookies(&access_token, &refresh_token);

    // Fire-and-forget: security alert SMS on login if phone is verified
    if crate::config::feature_flags().twilio && user.phone_verified {
        let db_ref = db.clone();
        let uid = user.id;
        tokio::spawn(async move {
            crate::twilio::send_security_alert(&db_ref, uid, "New login detected on your account.")
                .await;
        });
    }

    Ok(AuthUser {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        email: user_email,
        role: user_role,
        tier: user_tier,
        avatar_url: user.avatar_url,
        email_verified: user.email_verified,
        phone_number: user.phone_number,
        phone_verified: user.phone_verified,
        email_notifications_enabled: user.email_notifications_enabled,
        push_notifications_enabled: user.push_notifications_enabled,
        weekly_digest_enabled: user.weekly_digest_enabled,
        has_password: true,
        court_roles,
        court_tiers: Default::default(),
        preferred_court_id: None,
        linked_judge_id: None,
        linked_attorney_id: None,
    })
}

/// Get the current authenticated user. Returns None if not authenticated.
///
/// First checks request extensions for `Claims` (set by auth_middleware which
/// already validated the token and handled transparent refresh). Falls back
/// to direct cookie parsing when extensions aren't available.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn get_current_user() -> Result<Option<AuthUser>, ServerFnError> {
    use crate::auth::{cookies, jwt};

    let ctx = match dioxus::fullstack::FullstackContext::current() {
        Some(c) => c,
        None => {
            eprintln!("[get_current_user] No FullstackContext available");
            return Ok(None);
        }
    };

    let parts = ctx.parts_mut();

    // Primary: read Claims from extensions (auth_middleware already validated)
    if let Some(claims) = parts.extensions.get::<jwt::Claims>() {
        eprintln!(
            "[get_current_user] Found Claims in extensions for user {}",
            claims.sub
        );
        return fetch_auth_user(claims.sub).await;
    }

    // Fallback: parse cookies directly (covers cases where middleware didn't run)
    let headers = parts.headers.clone();

    if let Some(token) = cookies::extract_access_token(&headers) {
        if let Ok(claims) = jwt::validate_access_token(&token) {
            eprintln!(
                "[get_current_user] Access token valid for user {}",
                claims.sub
            );
            return fetch_auth_user(claims.sub).await;
        }
        eprintln!("[get_current_user] Access token present but invalid/expired");
    }

    if let Some(refresh_token) = cookies::extract_refresh_token(&headers) {
        if let Ok(claims) = jwt::validate_refresh_token(&refresh_token) {
            let db = get_db().await;
            let token_hash = jwt::hash_token(&refresh_token);
            let stored = sqlx::query!(
                "SELECT id, revoked FROM refresh_tokens WHERE token_hash = $1 AND user_id = $2",
                token_hash,
                claims.sub
            )
            .fetch_optional(db)
            .await
            .map_err(|e| e.into_app_error().into_server_fn_error())?;

            if let Some(row) = stored {
                if !row.revoked {
                    eprintln!(
                        "[get_current_user] Refresh token valid for user {}",
                        claims.sub
                    );
                    return fetch_auth_user(claims.sub).await;
                }
            }
        }
    }

    eprintln!("[get_current_user] No valid auth found");
    Ok(None)
}

/// Logout by revoking all refresh tokens and clearing auth cookies.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use crate::auth::{cookies, jwt};

    if let Some(ctx) = dioxus::fullstack::FullstackContext::current() {
        let headers = ctx.parts_mut().headers.clone();
        if let Some(token) = cookies::extract_access_token(&headers) {
            if let Ok(claims) = jwt::validate_access_token(&token) {
                let db = get_db().await;
                let _ = sqlx::query!(
                    "UPDATE refresh_tokens SET revoked = TRUE WHERE user_id = $1 AND revoked = FALSE",
                    claims.sub
                )
                .execute(db)
                .await;
            }
        }
    }

    // Schedule cookie clearing via middleware
    cookies::schedule_clear_cookies();

    Ok(())
}

/// Request admission to a court with a specific role.
/// If the user's email matches *@{court_id}.uscourts.gov, auto-approve immediately.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn request_court_admission(court_id: String, role: String, notes: Option<String>) -> Result<String, ServerFnError> {
    use shared_types::AppError;

    let claims = require_auth()?;

    let valid_roles = ["attorney", "clerk", "judge"];
    if !valid_roles.contains(&role.as_str()) {
        return Err(AppError::bad_request(format!(
            "Invalid role '{}'. Must be one of: {}",
            role,
            valid_roles.join(", ")
        ))
        .into_server_fn_error());
    }

    let db = get_db().await;

    // Check if there's already a pending request
    let existing = sqlx::query_scalar!(
        "SELECT id FROM court_role_requests WHERE user_id = $1 AND court_id = $2 AND status = 'pending'",
        claims.sub,
        court_id
    )
    .fetch_optional(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    if existing.is_some() {
        return Err(AppError::bad_request("You already have a pending request for this court").into_server_fn_error());
    }

    // Check if user's email matches uscourts.gov domain for this court
    let auto_approve = parse_uscourts_court(&claims.email)
        .map(|court| court == court_id)
        .unwrap_or(false);

    if auto_approve {
        // Auto-approve: insert approved request + update user's court_roles
        sqlx::query!(
            r#"INSERT INTO court_role_requests (user_id, court_id, requested_role, status, reviewed_at, notes)
               VALUES ($1, $2, $3, 'approved', NOW(), 'Auto-approved via uscourts.gov email domain')
               ON CONFLICT (user_id, court_id, status) DO NOTHING"#,
            claims.sub,
            court_id,
            role,
        )
        .execute(db)
        .await
        .map_err(|e| e.into_app_error().into_server_fn_error())?;

        let role_obj = serde_json::json!({ &court_id: &role });
        sqlx::query!(
            "UPDATE users SET court_roles = court_roles || $1 WHERE id = $2",
            role_obj,
            claims.sub,
        )
        .execute(db)
        .await
        .map_err(|e| e.into_app_error().into_server_fn_error())?;

        Ok("approved".to_string())
    } else {
        // Create pending request
        sqlx::query!(
            r#"INSERT INTO court_role_requests (user_id, court_id, requested_role, status, notes)
               VALUES ($1, $2, $3, 'pending', $4)"#,
            claims.sub,
            court_id,
            role,
            notes.as_deref(),
        )
        .execute(db)
        .await
        .map_err(|e| e.into_app_error().into_server_fn_error())?;

        Ok("pending".to_string())
    }
}

/// List pending court role requests for a specific court.
/// Requires admin or clerk role in the target court.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn list_pending_court_requests(
    court_id: String,
) -> Result<Vec<shared_types::CourtRoleRequestResponse>, ServerFnError> {
    let claims = require_auth()?;
    let db = get_db().await;
    let header_court = extract_court_header_sfn();

    require_membership_access_sfn(&claims, header_court.as_deref(), &court_id)?;

    let rows = crate::repo::court_role_request::list_pending_for_court(db, &court_id)
        .await
        .map_err(|e| e.into_server_fn_error())?;

    Ok(rows
        .iter()
        .map(crate::repo::court_role_request::to_response_with_user)
        .collect())
}

/// Approve a pending court role request.
/// Requires admin or clerk role.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn approve_court_request(request_id: String) -> Result<(), ServerFnError> {
    use shared_types::AppError;

    let claims = require_auth()?;
    let db = get_db().await;
    let header_court = extract_court_header_sfn();

    let req_uuid = uuid::Uuid::parse_str(&request_id)
        .map_err(|_| AppError::bad_request("Invalid request ID").into_server_fn_error())?;

    // Fetch the request to determine its court_id for access control
    let request = crate::repo::court_role_request::get_by_id(db, req_uuid)
        .await
        .map_err(|e| e.into_server_fn_error())?
        .ok_or_else(|| AppError::not_found("Request not found").into_server_fn_error())?;

    require_membership_access_sfn(&claims, header_court.as_deref(), &request.court_id)?;

    crate::repo::court_role_request::approve(db, req_uuid, claims.sub)
        .await
        .map_err(|e| e.into_server_fn_error())?;

    Ok(())
}

/// Deny a pending court role request with optional notes.
/// Requires admin or clerk role.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn deny_court_request(
    request_id: String,
    notes: Option<String>,
) -> Result<(), ServerFnError> {
    use shared_types::AppError;

    let claims = require_auth()?;
    let db = get_db().await;
    let header_court = extract_court_header_sfn();

    let req_uuid = uuid::Uuid::parse_str(&request_id)
        .map_err(|_| AppError::bad_request("Invalid request ID").into_server_fn_error())?;

    // Fetch the request to determine its court_id for access control
    let request = crate::repo::court_role_request::get_by_id(db, req_uuid)
        .await
        .map_err(|e| e.into_server_fn_error())?
        .ok_or_else(|| AppError::not_found("Request not found").into_server_fn_error())?;

    require_membership_access_sfn(&claims, header_court.as_deref(), &request.court_id)?;

    crate::repo::court_role_request::deny(db, req_uuid, claims.sub, notes.as_deref())
        .await
        .map_err(|e| e.into_server_fn_error())?;

    Ok(())
}

/// Update the current user's profile (display name and email). Requires authentication.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn update_profile(
    display_name: String,
    email: String,
) -> Result<AuthUser, ServerFnError> {
    use shared_types::{AppError, UpdateProfileRequest};

    let claims = require_auth()?;

    let req = UpdateProfileRequest {
        display_name: display_name.clone(),
        email: email.clone(),
    };
    req.validate_request()
        .map_err(|e| e.into_server_fn_error())?;

    let db = get_db().await;
    let user = sqlx::query!(
        r#"UPDATE users SET display_name = $2, email = $3 WHERE id = $1
           RETURNING id, username, display_name, email, password_hash, role, tier, avatar_url,
                     email_verified, phone_number, phone_verified,
                     email_notifications_enabled, push_notifications_enabled, weekly_digest_enabled"#,
        claims.sub,
        display_name,
        email
    )
    .fetch_optional(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?
    .ok_or_else(|| AppError::not_found("User not found").into_server_fn_error())?;

    Ok(AuthUser {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        email: user.email.unwrap_or_default(),
        role: user.role,
        tier: UserTier::from_str_or_default(&user.tier),
        avatar_url: user.avatar_url,
        email_verified: user.email_verified,
        phone_number: user.phone_number,
        phone_verified: user.phone_verified,
        email_notifications_enabled: user.email_notifications_enabled,
        push_notifications_enabled: user.push_notifications_enabled,
        weekly_digest_enabled: user.weekly_digest_enabled,
        has_password: user.password_hash.is_some(),
        court_roles: claims.court_roles.clone(),
        court_tiers: Default::default(),
        preferred_court_id: None,
        linked_judge_id: None,
        linked_attorney_id: None,
    })
}

/// Upload a user avatar via base64-encoded file data. Requires authentication.
#[cfg_attr(feature = "server", tracing::instrument(skip(file_data)))]
#[server]
pub async fn upload_user_avatar(
    file_data: String,
    content_type: String,
) -> Result<AuthUser, ServerFnError> {
    use shared_types::AppError;

    if !crate::config::feature_flags().s3 {
        return Err(
            AppError::validation("File uploads are disabled", Default::default())
                .into_server_fn_error(),
        );
    }

    let claims = require_auth()?;

    let allowed = ["image/jpeg", "image/png", "image/webp"];
    if !allowed.contains(&content_type.as_str()) {
        return Err(AppError::validation(
            "Only JPEG, PNG, and WebP images are allowed",
            Default::default(),
        )
        .into_server_fn_error());
    }

    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &file_data)
        .map_err(|e| {
            AppError::validation(format!("Invalid file data: {}", e), Default::default())
                .into_server_fn_error()
        })?;

    if bytes.len() > 2 * 1024 * 1024 {
        return Err(
            AppError::validation("Avatar must be under 2 MB", Default::default())
                .into_server_fn_error(),
        );
    }

    let avatar_url = crate::s3::upload_avatar(claims.sub, &content_type, &bytes)
        .await
        .map_err(|e| AppError::internal(e).into_server_fn_error())?;

    let db = get_db().await;
    let user = sqlx::query!(
        r#"UPDATE users SET avatar_url = $2 WHERE id = $1
           RETURNING id, username, display_name, email, password_hash, role, tier, avatar_url,
                     email_verified, phone_number, phone_verified,
                     email_notifications_enabled, push_notifications_enabled, weekly_digest_enabled"#,
        claims.sub,
        avatar_url
    )
    .fetch_one(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    Ok(AuthUser {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        email: user.email.unwrap_or_default(),
        role: user.role,
        tier: UserTier::from_str_or_default(&user.tier),
        avatar_url: user.avatar_url,
        email_verified: user.email_verified,
        phone_number: user.phone_number,
        phone_verified: user.phone_verified,
        email_notifications_enabled: user.email_notifications_enabled,
        push_notifications_enabled: user.push_notifications_enabled,
        weekly_digest_enabled: user.weekly_digest_enabled,
        has_password: user.password_hash.is_some(),
        court_roles: claims.court_roles.clone(),
        court_tiers: Default::default(),
        preferred_court_id: None,
        linked_judge_id: None,
        linked_attorney_id: None,
    })
}

/// Get the OAuth authorization URL for a given provider.
/// `redirect_after` optionally specifies a path to return to after login (e.g. "/activate").
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn oauth_authorize_url(
    provider: String,
    redirect_after: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::auth::oauth;
    use shared_types::AppError;

    if !crate::config::feature_flags().oauth {
        return Err(
            AppError::validation("OAuth is disabled", Default::default()).into_server_fn_error(),
        );
    }

    let provider = shared_types::OAuthProvider::parse_provider(&provider).ok_or_else(|| {
        AppError::validation("Unsupported OAuth provider", Default::default())
            .into_server_fn_error()
    })?;

    let url = oauth::get_authorize_url(&provider, redirect_after.clone())
        .await
        .map_err(|e| AppError::internal(e).into_server_fn_error())?;

    // Schedule an HTTP-only cookie so the OAuth callback knows where to redirect.
    // This is a BFF-pattern fallback alongside the in-memory state store.
    if let Some(ref path) = redirect_after {
        crate::auth::cookies::schedule_redirect_cookie(path);
    }

    Ok(url)
}

// ── Billing Server Functions ─────────────────────────

/// Create a Stripe checkout session for a subscription or one-time payment.
/// For subscriptions, `court_id` identifies the court whose tier will be updated.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn create_billing_checkout(
    checkout_type: String,
    tier: Option<String>,
    price_cents: Option<i64>,
    product_name: Option<String>,
    product_description: Option<String>,
    court_id: Option<String>,
) -> Result<CheckoutResponse, ServerFnError> {
    use shared_types::AppError;

    if !crate::config::feature_flags().stripe {
        return Err(
            AppError::validation("Stripe billing is disabled", Default::default())
                .into_server_fn_error(),
        );
    }

    let claims = require_auth()?;
    let db = get_db().await;

    let url = match checkout_type.as_str() {
        "subscription" => {
            let tier = tier.ok_or_else(|| {
                AppError::validation("Tier is required for subscriptions", Default::default())
                    .into_server_fn_error()
            })?;
            let cid = court_id.as_deref().unwrap_or("");
            crate::stripe::checkout::create_subscription_checkout(
                db,
                claims.sub,
                &claims.email,
                &tier,
                cid,
            )
            .await
            .map_err(|e| AppError::internal(e).into_server_fn_error())?
        }
        "onetime" => {
            let cents = price_cents.ok_or_else(|| {
                AppError::validation("price_cents required for one-time", Default::default())
                    .into_server_fn_error()
            })?;
            let name = product_name.as_deref().unwrap_or("One-time purchase");
            let desc = product_description.as_deref().unwrap_or("");
            crate::stripe::checkout::create_onetime_checkout(
                db,
                claims.sub,
                &claims.email,
                cents,
                name,
                desc,
            )
            .await
            .map_err(|e| AppError::internal(e).into_server_fn_error())?
        }
        _ => {
            return Err(AppError::validation(
                "checkout_type must be 'subscription' or 'onetime'",
                Default::default(),
            )
            .into_server_fn_error());
        }
    };

    Ok(CheckoutResponse { url })
}

/// Create a Stripe Customer Portal session.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn create_billing_portal() -> Result<CheckoutResponse, ServerFnError> {
    use shared_types::AppError;

    if !crate::config::feature_flags().stripe {
        return Err(
            AppError::validation("Stripe billing is disabled", Default::default())
                .into_server_fn_error(),
        );
    }

    let claims = require_auth()?;
    let db = get_db().await;

    let url = crate::stripe::portal::create_portal_session(db, claims.sub)
        .await
        .map_err(|e| AppError::internal(e).into_server_fn_error())?;

    Ok(CheckoutResponse { url })
}

/// Get the current user's subscription status.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn get_subscription_status() -> Result<SubscriptionStatus, ServerFnError> {
    if !crate::config::feature_flags().stripe {
        return Ok(SubscriptionStatus {
            active: false,
            status: "disabled".to_string(),
            price_id: None,
            current_period_end: None,
            cancel_at_period_end: false,
        });
    }

    let claims = require_auth()?;
    let db = get_db().await;

    let sub = sqlx::query!(
        r#"SELECT status, stripe_price_id, current_period_end, cancel_at_period_end
           FROM subscriptions WHERE user_id = $1
           ORDER BY created_at DESC LIMIT 1"#,
        claims.sub
    )
    .fetch_optional(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    Ok(match sub {
        Some(s) => SubscriptionStatus {
            active: s.status == "active",
            status: s.status,
            price_id: Some(s.stripe_price_id),
            current_period_end: s.current_period_end.map(|t| t.to_rfc3339()),
            cancel_at_period_end: s.cancel_at_period_end,
        },
        None => SubscriptionStatus {
            active: false,
            status: "none".to_string(),
            price_id: None,
            current_period_end: None,
            cancel_at_period_end: false,
        },
    })
}

/// Cancel the current user's active subscription.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn cancel_subscription() -> Result<MessageResponse, ServerFnError> {
    use shared_types::AppError;

    if !crate::config::feature_flags().stripe {
        return Err(
            AppError::validation("Stripe billing is disabled", Default::default())
                .into_server_fn_error(),
        );
    }

    let claims = require_auth()?;
    let db = get_db().await;

    crate::stripe::checkout::cancel_subscription(db, claims.sub)
        .await
        .map_err(|e| AppError::internal(e).into_server_fn_error())?;

    Ok(MessageResponse {
        message: "Subscription canceled successfully".to_string(),
    })
}

// ── Billing Event Long-Poll ──────────────────────────

/// Long-poll for billing events targeting the current user.
///
/// Subscribes to the broadcast channel, skips events for other users,
/// and returns the first matching event. Times out after 25 seconds
/// with `Ok(None)` so the client can immediately retry (long-poll loop).
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn poll_billing_event() -> Result<Option<shared_types::BillingEvent>, ServerFnError> {
    use std::time::Duration;
    use tokio::time::timeout;

    if !crate::config::feature_flags().stripe {
        // Sleep briefly to avoid tight loop from client, then return None
        tokio::time::sleep(Duration::from_secs(25)).await;
        return Ok(None);
    }

    let claims = require_auth()?;
    let user_id = claims.sub;

    let mut rx = crate::stripe::client::billing_subscribe();

    let poll_timeout = Duration::from_secs(25);
    let result = timeout(poll_timeout, async {
        loop {
            match rx.recv().await {
                Ok((event_user_id, event)) if event_user_id == user_id => {
                    return Some(event);
                }
                Ok(_) => {
                    // Event for a different user — keep waiting
                    continue;
                }
                Err(_) => {
                    // Channel closed or lagged — bail out
                    return None;
                }
            }
        }
    })
    .await;

    match result {
        Ok(event) => Ok(event),
        Err(_) => Ok(None), // Timeout — client will retry
    }
}

// ── Email Server Functions ───────────────────────────

/// Initiate a password reset. Always returns success to prevent email enumeration.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn forgot_password(email: String) -> Result<MessageResponse, ServerFnError> {
    // Fire-and-forget: check if user exists and send email (only if mailgun enabled)
    if crate::config::feature_flags().mailgun {
        let db = get_db().await;
        let db_ref = db.clone();
        tokio::spawn(async move {
            let user_exists =
                sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE email = $1", email)
                    .fetch_one(&db_ref)
                    .await
                    .unwrap_or(Some(0))
                    .unwrap_or(0);

            if user_exists > 0 {
                if let Ok(token) =
                    crate::mailgun::create_password_reset_token(&db_ref, &email).await
                {
                    crate::mailgun::send_password_reset_email(&email, &token).await;
                }
            }
        });
    }

    Ok(MessageResponse {
        message: "If an account with that email exists, a password reset link has been sent."
            .to_string(),
    })
}

/// Reset a user's password using a token from the password reset email.
/// Validates the token, hashes the new password, revokes sessions, and sends
/// a security alert via SMS if the user has a verified phone number.
#[cfg_attr(feature = "server", tracing::instrument(skip(new_password)))]
#[server]
pub async fn reset_password(
    token: String,
    new_password: String,
) -> Result<MessageResponse, ServerFnError> {
    use crate::auth::password as pw;
    use shared_types::AppError;

    if new_password.len() < 8 {
        return Err(AppError::validation(
            "Password must be at least 8 characters",
            Default::default(),
        )
        .into_server_fn_error());
    }

    let db = get_db().await;

    let email = crate::mailgun::validate_password_reset_token(db, &token)
        .await
        .map_err(|e| AppError::validation(e, Default::default()).into_server_fn_error())?;

    let password_hash = pw::hash_password(&new_password)
        .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

    sqlx::query!(
        "UPDATE users SET password_hash = $2 WHERE email = $1",
        email,
        password_hash
    )
    .execute(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    // Revoke all refresh tokens for security
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked = TRUE WHERE user_id = (SELECT id FROM users WHERE email = $1) AND revoked = FALSE",
        email
    )
    .execute(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    // Fire-and-forget: send security alert if phone verified
    let db_clone = db.clone();
    let email_clone = email.clone();
    tokio::spawn(async move {
        let user_id = sqlx::query_scalar!("SELECT id FROM users WHERE email = $1", email_clone)
            .fetch_optional(&db_clone)
            .await
            .ok()
            .flatten();

        if let Some(uid) = user_id {
            crate::twilio::send_security_alert(
                &db_clone,
                uid,
                "Your password was recently changed.",
            )
            .await;
        }
    });

    Ok(MessageResponse {
        message: "Password reset successfully. Please log in with your new password.".to_string(),
    })
}

/// Resend email verification for the current user.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn resend_verification_email() -> Result<MessageResponse, ServerFnError> {
    use shared_types::AppError;

    if !crate::config::feature_flags().mailgun {
        return Err(
            AppError::validation("Email service is disabled", Default::default())
                .into_server_fn_error(),
        );
    }

    let claims = require_auth()?;
    let db = get_db().await;

    let token = crate::mailgun::create_verification_token(db, claims.sub)
        .await
        .map_err(|e| AppError::internal(e).into_server_fn_error())?;

    let email = claims.email.clone();
    tokio::spawn(async move {
        crate::mailgun::send_verification_email(&email, &token).await;
    });

    Ok(MessageResponse {
        message: "Verification email sent.".to_string(),
    })
}

// ── Phone Server Functions ───────────────────────────

/// Send a phone verification code via SMS.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn send_phone_verification(
    phone_number: String,
) -> Result<MessageResponse, ServerFnError> {
    if !crate::config::feature_flags().twilio {
        return Err(shared_types::AppError::validation(
            "Phone verification is disabled",
            Default::default(),
        )
        .into_server_fn_error());
    }

    let claims = require_auth()?;
    let db = get_db().await;

    crate::twilio::send_verification_code(db, claims.sub, &phone_number)
        .await
        .map_err(|e| e.into_server_fn_error())?;

    Ok(MessageResponse {
        message: "Verification code sent.".to_string(),
    })
}

/// Verify a phone number with a code.
#[cfg_attr(feature = "server", tracing::instrument(skip(code)))]
#[server]
pub async fn verify_phone(phone_number: String, code: String) -> Result<AuthUser, ServerFnError> {
    if !crate::config::feature_flags().twilio {
        return Err(shared_types::AppError::validation(
            "Phone verification is disabled",
            Default::default(),
        )
        .into_server_fn_error());
    }

    let claims = require_auth()?;
    let db = get_db().await;

    crate::twilio::verify_code(db, claims.sub, &phone_number, &code)
        .await
        .map_err(|e| e.into_server_fn_error())?;

    // Return updated user
    fetch_auth_user(claims.sub)
        .await?
        .ok_or_else(|| shared_types::AppError::not_found("User not found").into_server_fn_error())
}

// ── Notification Preferences ─────────────────────────

/// Update the current user's notification preferences. Requires authentication.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn update_notification_preferences(
    email_notifications: bool,
    push_notifications: bool,
    weekly_digest: bool,
) -> Result<AuthUser, ServerFnError> {
    let claims = require_auth()?;
    let db = get_db().await;

    sqlx::query!(
        r#"UPDATE users
           SET email_notifications_enabled = $2,
               push_notifications_enabled = $3,
               weekly_digest_enabled = $4
           WHERE id = $1"#,
        claims.sub,
        email_notifications,
        push_notifications,
        weekly_digest
    )
    .execute(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    fetch_auth_user(claims.sub)
        .await?
        .ok_or_else(|| shared_types::AppError::not_found("User not found").into_server_fn_error())
}

/// Set a password on an OAuth-only account (no existing password_hash).
/// Lets locked-out OAuth users regain access when OAuth is disabled.
#[cfg_attr(feature = "server", tracing::instrument(skip(new_password)))]
#[server]
pub async fn set_oauth_account_password(
    email: String,
    new_password: String,
) -> Result<MessageResponse, ServerFnError> {
    use crate::auth::password as pw;
    use shared_types::AppError;

    let db = get_db().await;

    let user = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE email = $1",
        email
    )
    .fetch_optional(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?
    .ok_or_else(|| AppError::unauthorized("Invalid email").into_server_fn_error())?;

    if user.password_hash.is_some() {
        return Err(
            AppError::validation("Account already has a password", Default::default())
                .into_server_fn_error(),
        );
    }

    if new_password.len() < 8 {
        return Err(AppError::validation(
            "Password must be at least 8 characters",
            Default::default(),
        )
        .into_server_fn_error());
    }

    let hash = pw::hash_password(&new_password)
        .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

    sqlx::query!(
        "UPDATE users SET password_hash = $2 WHERE id = $1",
        user.id,
        hash
    )
    .execute(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    Ok(MessageResponse {
        message: "Password set successfully. You can now sign in.".to_string(),
    })
}

/// Change password for the currently authenticated user.
/// Requires the current password for verification before setting the new one.
#[cfg_attr(
    feature = "server",
    tracing::instrument(skip(current_password, new_password))
)]
#[server]
pub async fn change_password(
    current_password: String,
    new_password: String,
) -> Result<MessageResponse, ServerFnError> {
    use crate::auth::password as pw;
    use shared_types::AppError;

    let claims = require_auth()?;
    let db = get_db().await;

    if new_password.len() < 8 {
        return Err(AppError::validation(
            "New password must be at least 8 characters",
            Default::default(),
        )
        .into_server_fn_error());
    }

    let user = sqlx::query!("SELECT password_hash FROM users WHERE id = $1", claims.sub)
        .fetch_optional(db)
        .await
        .map_err(|e| e.into_app_error().into_server_fn_error())?
        .ok_or_else(|| AppError::not_found("User not found").into_server_fn_error())?;

    let password_hash = user.password_hash.ok_or_else(|| {
        AppError::validation(
            "No password set on this account. Use the set-password option instead.",
            Default::default(),
        )
        .into_server_fn_error()
    })?;

    let valid = pw::verify_password(&current_password, &password_hash)
        .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

    if !valid {
        return Err(
            AppError::validation("Current password is incorrect", Default::default())
                .into_server_fn_error(),
        );
    }

    let new_hash = pw::hash_password(&new_password)
        .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

    sqlx::query!(
        "UPDATE users SET password_hash = $2 WHERE id = $1",
        claims.sub,
        new_hash
    )
    .execute(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    // Fire-and-forget: send security alert if phone verified
    let db_clone = db.clone();
    let user_id = claims.sub;
    tokio::spawn(async move {
        crate::twilio::send_security_alert(
            &db_clone,
            user_id,
            "Your password was recently changed.",
        )
        .await;
    });

    Ok(MessageResponse {
        message: "Password changed successfully.".to_string(),
    })
}

// ── Device Authorization Flow (RFC 8628) ─────────────

/// Initiate a device authorization flow. No auth required.
/// Returns a device code (secret) and user code (displayed to user).
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn initiate_device_auth(
    client_platform: Option<String>,
) -> Result<DeviceFlowInitResponse, ServerFnError> {
    use crate::auth::device_flow;
    use shared_types::AppError;

    let db = get_db().await;
    let base_url = crate::stripe::client::app_base_url();

    // Retry loop to handle user_code collisions (unique constraint)
    let mut attempts = 0;
    loop {
        let raw_device_code = device_flow::generate_device_code();
        let user_code = device_flow::generate_user_code();
        let device_code_hash = device_flow::hash_device_code(&raw_device_code);

        let expires_at =
            chrono::Utc::now() + chrono::Duration::seconds(device_flow::DEVICE_CODE_EXPIRY_SECONDS);

        let result = sqlx::query!(
            r#"INSERT INTO device_authorizations (device_code, user_code, client_info, expires_at)
               VALUES ($1, $2, $3, $4)"#,
            device_code_hash,
            user_code,
            client_platform,
            expires_at
        )
        .execute(db)
        .await;

        match result {
            Ok(_) => {
                let verification_uri = format!("{}/activate", base_url);
                let verification_uri_complete =
                    Some(format!("{}/activate?code={}", base_url, user_code));

                return Ok(DeviceFlowInitResponse {
                    device_code: raw_device_code,
                    user_code,
                    verification_uri,
                    verification_uri_complete,
                    expires_in: device_flow::DEVICE_CODE_EXPIRY_SECONDS,
                    interval: device_flow::DEVICE_POLL_INTERVAL_SECONDS,
                });
            }
            Err(e) => {
                // Retry on unique constraint violation (user_code collision)
                if e.as_database_error()
                    .is_some_and(|db_err| db_err.is_unique_violation())
                {
                    attempts += 1;
                    if attempts >= 5 {
                        return Err(AppError::internal(
                            "Failed to generate unique device code after retries",
                        )
                        .into_server_fn_error());
                    }
                    continue;
                }
                return Err(e.into_app_error().into_server_fn_error());
            }
        }
    }
}

/// Poll for device authorization approval. No auth required.
/// Uses semi-long-poll: sleeps ~5s server-side before returning "pending".
#[cfg_attr(feature = "server", tracing::instrument(skip(device_code)))]
#[server]
pub async fn poll_device_auth(
    device_code: String,
) -> Result<DeviceFlowPollResponse, ServerFnError> {
    use crate::auth::{cookies, device_flow, jwt};
    use shared_types::AppError;
    use std::time::Duration;

    let db = get_db().await;
    let code_hash = device_flow::hash_device_code(&device_code);

    let row = sqlx::query!(
        r#"SELECT id, user_id, status, expires_at
           FROM device_authorizations
           WHERE device_code = $1"#,
        code_hash
    )
    .fetch_optional(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?
    .ok_or_else(|| AppError::not_found("Invalid device code").into_server_fn_error())?;

    // Check if expired
    if row.expires_at < chrono::Utc::now() {
        // Mark as expired if still pending
        if row.status == "pending" {
            let _ = sqlx::query!(
                "UPDATE device_authorizations SET status = 'expired' WHERE id = $1",
                row.id
            )
            .execute(db)
            .await;
        }
        return Ok(DeviceFlowPollResponse {
            status: DeviceAuthStatus::Expired,
            user: None,
        });
    }

    match row.status.as_str() {
        "approved" => {
            let user_id = row.user_id.ok_or_else(|| {
                AppError::internal("Approved device auth missing user_id").into_server_fn_error()
            })?;

            // Fetch user data for token creation
            let user = sqlx::query!(
                r#"SELECT id, username, display_name, email, password_hash, role, tier, avatar_url,
                          email_verified, phone_number, phone_verified,
                          email_notifications_enabled, push_notifications_enabled, weekly_digest_enabled
                   FROM users WHERE id = $1"#,
                user_id
            )
            .fetch_optional(db)
            .await
            .map_err(|e| e.into_app_error().into_server_fn_error())?
            .ok_or_else(|| AppError::not_found("User not found").into_server_fn_error())?;

            let user_email = user.email.unwrap_or_default();
            let user_role = user.role;
            let user_tier = UserTier::from_str_or_default(&user.tier);

            // Load court_roles from DB
            let court_roles: std::collections::HashMap<String, String> = sqlx::query_scalar!(
                "SELECT court_roles FROM users WHERE id = $1",
                user.id
            )
            .fetch_optional(db)
            .await
            .ok()
            .flatten()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

            // Create JWTs
            let access_token =
                jwt::create_access_token(user.id, &user_email, &user_role, user_tier.as_str(), &court_roles)
                    .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

            let (refresh_token, refresh_expires_at) =
                jwt::create_refresh_token(user.id, &user_email, &user_role, user_tier.as_str(), &court_roles)
                    .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

            // Store refresh token hash
            let refresh_hash = jwt::hash_token(&refresh_token);
            sqlx::query!(
                "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
                user.id,
                refresh_hash,
                refresh_expires_at
            )
            .execute(db)
            .await
            .map_err(|e| e.into_app_error().into_server_fn_error())?;

            // Schedule cookies
            cookies::schedule_auth_cookies(&access_token, &refresh_token);

            // Delete the device auth row (one-time use)
            let _ = sqlx::query!("DELETE FROM device_authorizations WHERE id = $1", row.id)
                .execute(db)
                .await;

            let auth_user = AuthUser {
                id: user.id,
                username: user.username,
                display_name: user.display_name,
                email: user_email,
                role: user_role,
                tier: user_tier,
                avatar_url: user.avatar_url,
                email_verified: user.email_verified,
                phone_number: user.phone_number,
                phone_verified: user.phone_verified,
                email_notifications_enabled: user.email_notifications_enabled,
                push_notifications_enabled: user.push_notifications_enabled,
                weekly_digest_enabled: user.weekly_digest_enabled,
                has_password: user.password_hash.is_some(),
                court_roles,
                court_tiers: Default::default(),
                preferred_court_id: None,
                linked_judge_id: None,
                linked_attorney_id: None,
            };

            Ok(DeviceFlowPollResponse {
                status: DeviceAuthStatus::Approved,
                user: Some(auth_user),
            })
        }
        "pending" => {
            // Semi-long-poll: sleep server-side to avoid tight client loop
            tokio::time::sleep(Duration::from_secs(
                device_flow::DEVICE_POLL_INTERVAL_SECONDS as u64,
            ))
            .await;

            Ok(DeviceFlowPollResponse {
                status: DeviceAuthStatus::Pending,
                user: None,
            })
        }
        _ => {
            // expired or unknown status
            Ok(DeviceFlowPollResponse {
                status: DeviceAuthStatus::Expired,
                user: None,
            })
        }
    }
}

/// Approve a device authorization. Requires authentication.
/// Called by the user in a browser after they enter the user code.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn approve_device(user_code: String) -> Result<MessageResponse, ServerFnError> {
    use crate::auth::device_flow;
    use shared_types::AppError;

    let claims = require_auth()?;
    let db = get_db().await;

    let normalized = device_flow::normalize_user_code(&user_code).ok_or_else(|| {
        AppError::validation(
            "Invalid code format. Expected 8 characters like ABCD-EFGH.",
            Default::default(),
        )
        .into_server_fn_error()
    })?;

    let row = sqlx::query!(
        r#"SELECT id, status, expires_at
           FROM device_authorizations
           WHERE user_code = $1"#,
        normalized
    )
    .fetch_optional(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?
    .ok_or_else(|| {
        AppError::not_found("Code not found. It may have expired.").into_server_fn_error()
    })?;

    // Check expiry
    if row.expires_at < chrono::Utc::now() {
        let _ = sqlx::query!(
            "UPDATE device_authorizations SET status = 'expired' WHERE id = $1",
            row.id
        )
        .execute(db)
        .await;
        return Err(AppError::validation(
            "This code has expired. Please try again on your device.",
            Default::default(),
        )
        .into_server_fn_error());
    }

    // Check status
    if row.status != "pending" {
        return Err(AppError::validation(
            "This code has already been used or expired.",
            Default::default(),
        )
        .into_server_fn_error());
    }

    // Approve: set user_id, status, approved_at
    sqlx::query!(
        r#"UPDATE device_authorizations
           SET status = 'approved', user_id = $2, approved_at = NOW()
           WHERE id = $1"#,
        row.id,
        claims.sub
    )
    .execute(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    Ok(MessageResponse {
        message: "Device authorized successfully. You can close this page.".to_string(),
    })
}
