use dioxus::prelude::*;
use shared_types::{
    AuthUser, CheckoutResponse, CourtMembership, DashboardStats, DeviceFlowInitResponse,
    DeviceFlowPollResponse, FeatureFlags, MessageResponse, Product, SubscriptionStatus, User,
    UserWithMembership,
};

#[cfg(feature = "server")]
use crate::db::get_db;

#[cfg(feature = "server")]
use crate::error_convert::{AppErrorExt, SqlxErrorExt, ValidateRequest};

#[cfg(feature = "server")]
use shared_types::{
    CreateProductRequest, CreateUserRequest, DeviceAuthStatus, UpdateProductRequest,
    UpdateUserRequest, UserTier,
};

// ── Auth helpers for server functions ──────────────────

/// Extract and validate the caller's identity from the current request.
/// Checks middleware-injected Claims first, falls back to cookie parsing.
/// Returns the validated Claims or an "Authentication required" error.
#[cfg(feature = "server")]
fn require_auth() -> Result<crate::auth::jwt::Claims, ServerFnError> {
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
#[cfg(feature = "server")]
fn require_admin() -> Result<crate::auth::jwt::Claims, ServerFnError> {
    use shared_types::AppError;

    let claims = require_auth()?;
    if claims.role != "admin" {
        return Err(AppError::forbidden("Admin role required").into_server_fn_error());
    }
    Ok(claims)
}

/// Get the current feature flags. No auth required — flags are not sensitive.
#[server]
pub async fn get_feature_flags() -> Result<FeatureFlags, ServerFnError> {
    Ok(crate::config::feature_flags().clone())
}

/// Get premium analytics data. Requires Pro tier or above on any court the user belongs to.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn get_premium_analytics() -> Result<shared_types::PremiumAnalytics, ServerFnError> {
    use shared_types::AppError;

    let claims = require_auth()?;
    let db = get_db().await;

    // Check court tier — user has access if ANY of their courts is Pro+
    let court_ids: Vec<&str> = claims.court_roles.keys().map(|s| s.as_str()).collect();
    let has_pro_court = if !court_ids.is_empty() {
        let tiers: Vec<String> = sqlx::query_scalar!(
            "SELECT tier FROM courts WHERE id = ANY($1)",
            &court_ids as &[&str]
        )
        .fetch_all(db)
        .await
        .map_err(|e| e.into_app_error().into_server_fn_error())?;
        tiers
            .iter()
            .any(|t| UserTier::from_str_or_default(t).has_access(&UserTier::Pro))
    } else {
        false
    };

    if !has_pro_court {
        return Err(AppError::forbidden("Pro tier required for analytics").into_server_fn_error());
    }

    let total_revenue = sqlx::query_scalar!(
        "SELECT COALESCE(SUM(price), 0.0) FROM products WHERE status = 'active'"
    )
    .fetch_one(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?
    .unwrap_or(0.0);

    let avg_product_price = sqlx::query_scalar!("SELECT COALESCE(AVG(price), 0.0) FROM products")
        .fetch_one(db)
        .await
        .map_err(|e| e.into_app_error().into_server_fn_error())?
        .unwrap_or(0.0);

    let category_rows = sqlx::query!(
        "SELECT category, COUNT(*) as count FROM products GROUP BY category ORDER BY count DESC"
    )
    .fetch_all(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    let products_by_category: Vec<shared_types::CategoryCount> = category_rows
        .into_iter()
        .map(|r| shared_types::CategoryCount {
            category: r.category,
            count: r.count.unwrap_or(0),
        })
        .collect();

    let users_last_30_days = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE created_at >= NOW() - INTERVAL '30 days'"
    )
    .fetch_one(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?
    .unwrap_or(0);

    Ok(shared_types::PremiumAnalytics {
        total_revenue,
        avg_product_price,
        products_by_category,
        users_last_30_days,
    })
}

/// Get a user by ID.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn get_user(user_id: i64) -> Result<User, ServerFnError> {
    let db = get_db().await;
    let user = sqlx::query_as!(
        User,
        "SELECT id, username, display_name, role, tier FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?
    .ok_or_else(|| {
        shared_types::AppError::not_found(format!("User with id {} not found", user_id))
            .into_server_fn_error()
    })?;
    Ok(user)
}

/// List all users.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn list_users() -> Result<Vec<User>, ServerFnError> {
    let db = get_db().await;
    let users = sqlx::query_as!(
        User,
        "SELECT id, username, display_name, role, tier FROM users"
    )
    .fetch_all(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;
    Ok(users)
}

/// Create a new user. Requires admin role.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn create_user(username: String, display_name: String) -> Result<User, ServerFnError> {
    require_admin()?;

    let req = CreateUserRequest {
        username,
        display_name,
    };
    req.validate_request()
        .map_err(|e| e.into_server_fn_error())?;

    let db = get_db().await;
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username, display_name) VALUES ($1, $2) RETURNING id, username, display_name, role, tier",
        req.username,
        req.display_name
    )
    .fetch_one(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    Ok(user)
}

/// Update an existing user. Requires admin role.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn update_user(
    user_id: i64,
    username: String,
    display_name: String,
) -> Result<User, ServerFnError> {
    require_admin()?;

    let req = UpdateUserRequest {
        username,
        display_name,
    };
    req.validate_request()
        .map_err(|e| e.into_server_fn_error())?;

    let db = get_db().await;
    let user = sqlx::query_as!(
        User,
        "UPDATE users SET username = $2, display_name = $3 WHERE id = $1 RETURNING id, username, display_name, role, tier",
        user_id,
        req.username,
        req.display_name
    )
    .fetch_one(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;
    Ok(user)
}

/// Delete a user by ID. Requires admin role.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn delete_user(user_id: i64) -> Result<(), ServerFnError> {
    require_admin()?;

    let db = get_db().await;
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(db)
        .await
        .map_err(|e| e.into_app_error().into_server_fn_error())?;
    Ok(())
}

/// Update a user's tier. Requires admin role.
/// NOTE: Vestigial — tier is now per-court. Prefer `update_court_tier`.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn update_user_tier(user_id: i64, tier: String) -> Result<User, ServerFnError> {
    use shared_types::AppError;

    require_admin()?;

    let valid_tiers = ["free", "pro", "enterprise"];
    let tier_lower = tier.to_lowercase();
    if !valid_tiers.contains(&tier_lower.as_str()) {
        return Err(
            AppError::validation("Invalid tier value", Default::default()).into_server_fn_error(),
        );
    }

    let db = get_db().await;
    let user = sqlx::query_as!(
        User,
        "UPDATE users SET tier = $2 WHERE id = $1 RETURNING id, username, display_name, role, tier",
        user_id,
        tier_lower
    )
    .fetch_one(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    Ok(user)
}

/// Update a court's subscription tier. Requires admin or clerk of that court.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn update_court_tier(court_id: String, tier: String) -> Result<MessageResponse, ServerFnError> {
    use shared_types::AppError;

    let claims = require_auth()?;

    // Check admin or clerk of the target court
    let is_admin = claims.role == "admin";
    let is_clerk = claims.court_roles.get(&court_id).map(|r| r.as_str()) == Some("clerk");
    if !is_admin && !is_clerk {
        return Err(
            AppError::forbidden("Admin or court clerk required").into_server_fn_error(),
        );
    }

    let valid_tiers = ["free", "pro", "enterprise"];
    let tier_lower = tier.to_lowercase();
    if !valid_tiers.contains(&tier_lower.as_str()) {
        return Err(
            AppError::validation("Invalid tier value", Default::default()).into_server_fn_error(),
        );
    }

    let db = get_db().await;
    sqlx::query!(
        "UPDATE courts SET tier = $2 WHERE id = $1",
        court_id,
        tier_lower
    )
    .execute(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    Ok(MessageResponse {
        message: format!("Court tier updated to {}", tier_lower),
    })
}

/// List all products.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn list_products() -> Result<Vec<Product>, ServerFnError> {
    let db = get_db().await;
    let rows = sqlx::query!(
        "SELECT id, name, description, price, category, status, created_at FROM products ORDER BY id DESC"
    )
    .fetch_all(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    let products = rows
        .into_iter()
        .map(|r| Product {
            id: r.id,
            name: r.name,
            description: r.description,
            price: r.price,
            category: r.category,
            status: r.status,
            created_at: r.created_at.to_string(),
        })
        .collect();
    Ok(products)
}

/// Create a new product. Requires authentication.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn create_product(
    name: String,
    description: String,
    price: f64,
    category: String,
    status: String,
) -> Result<Product, ServerFnError> {
    require_auth()?;

    let req = CreateProductRequest {
        name,
        description,
        price,
        category,
        status,
    };
    req.validate_request()
        .map_err(|e| e.into_server_fn_error())?;

    let db = get_db().await;
    let row = sqlx::query!(
        "INSERT INTO products (name, description, price, category, status) VALUES ($1, $2, $3, $4, $5) RETURNING id, name, description, price, category, status, created_at",
        req.name,
        req.description,
        req.price,
        req.category,
        req.status
    )
    .fetch_one(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    Ok(Product {
        id: row.id,
        name: row.name,
        description: row.description,
        price: row.price,
        category: row.category,
        status: row.status,
        created_at: row.created_at.to_string(),
    })
}

/// Update an existing product. Requires authentication.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn update_product(
    product_id: i64,
    name: String,
    description: String,
    price: f64,
    category: String,
    status: String,
) -> Result<Product, ServerFnError> {
    require_auth()?;

    let req = UpdateProductRequest {
        name,
        description,
        price,
        category,
        status,
    };
    req.validate_request()
        .map_err(|e| e.into_server_fn_error())?;

    let db = get_db().await;
    let row = sqlx::query!(
        "UPDATE products SET name = $2, description = $3, price = $4, category = $5, status = $6 WHERE id = $1 RETURNING id, name, description, price, category, status, created_at",
        product_id,
        req.name,
        req.description,
        req.price,
        req.category,
        req.status
    )
    .fetch_one(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    Ok(Product {
        id: row.id,
        name: row.name,
        description: row.description,
        price: row.price,
        category: row.category,
        status: row.status,
        created_at: row.created_at.to_string(),
    })
}

/// Delete a product by ID. Requires authentication.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn delete_product(product_id: i64) -> Result<(), ServerFnError> {
    require_auth()?;

    let db = get_db().await;
    sqlx::query!("DELETE FROM products WHERE id = $1", product_id)
        .execute(db)
        .await
        .map_err(|e| e.into_app_error().into_server_fn_error())?;
    Ok(())
}

/// Get dashboard statistics.
#[cfg_attr(feature = "server", tracing::instrument)]
#[server]
pub async fn get_dashboard_stats() -> Result<DashboardStats, ServerFnError> {
    let db = get_db().await;

    let user_count = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(db)
        .await
        .map_err(|e| e.into_app_error().into_server_fn_error())?
        .unwrap_or(0);

    let product_count = sqlx::query_scalar!("SELECT COUNT(*) FROM products")
        .fetch_one(db)
        .await
        .map_err(|e| e.into_app_error().into_server_fn_error())?
        .unwrap_or(0);

    let active_count = sqlx::query_scalar!("SELECT COUNT(*) FROM products WHERE status = 'active'")
        .fetch_one(db)
        .await
        .map_err(|e| e.into_app_error().into_server_fn_error())?
        .unwrap_or(0);

    let recent_users = sqlx::query_as!(
        User,
        "SELECT id, username, display_name, role, tier FROM users ORDER BY id DESC LIMIT 5"
    )
    .fetch_all(db)
    .await
    .map_err(|e| e.into_app_error().into_server_fn_error())?;

    Ok(DashboardStats {
        total_users: user_count,
        total_products: product_count,
        active_products: active_count,
        recent_users,
    })
}

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

/// Fetch an AuthUser from the database by user ID.
#[cfg(feature = "server")]
async fn fetch_auth_user(user_id: i64) -> Result<Option<AuthUser>, ServerFnError> {
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

            Ok(Some(AuthUser {
                id: u.id,
                username: u.username,
                display_name: u.display_name,
                email: u.email.unwrap_or_default(),
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

/// Parse a uscourts.gov email domain to extract the court identifier.
/// e.g., "user@arwd.uscourts.gov" → Some("arwd")
#[cfg(feature = "server")]
fn parse_uscourts_court(email: &str) -> Option<String> {
    let domain = email.rsplit('@').next()?;
    let parts: Vec<&str> = domain.split('.').collect();
    if parts.len() == 3 && parts[1] == "uscourts" && parts[2] == "gov" {
        Some(parts[0].to_lowercase())
    } else {
        None
    }
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

// ═══════════════════════════════════════════════════════════════
// Court domain server functions (called from frontend via RPC)
// ═══════════════════════════════════════════════════════════════

/// Fetch attorneys for the selected court district.
#[server]
pub async fn list_attorneys(
    court_id: String,
    page: Option<i64>,
    limit: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attorney;
    use shared_types::normalize_pagination;

    let pool = get_db().await;
    let (page, limit) = normalize_pagination(page, limit);
    let (attorneys, total) = attorney::list(pool, &court_id, page, limit).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response = shared_types::PaginatedResponse::new(
        attorneys.into_iter().map(shared_types::AttorneyResponse::from).collect(),
        page, limit, total,
    );
    Ok(serde_json::to_string(&response).unwrap_or_default())
}

/// Get a single attorney by ID.
#[server]
pub async fn get_attorney(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attorney;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let att = attorney::find_by_id(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attorney not found"))?;

    Ok(serde_json::to_string(&shared_types::AttorneyResponse::from(att)).unwrap_or_default())
}

/// Create a new attorney.
#[server]
pub async fn create_attorney(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attorney;
    use shared_types::CreateAttorneyRequest;

    let pool = get_db().await;
    let req: CreateAttorneyRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;

    let att = attorney::create(pool, &court_id, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(serde_json::to_string(&shared_types::AttorneyResponse::from(att)).unwrap_or_default())
}

/// Update an existing attorney.
#[server]
pub async fn update_attorney(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attorney;
    use shared_types::UpdateAttorneyRequest;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: UpdateAttorneyRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;

    let att = attorney::update(pool, &court_id, uuid, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attorney not found"))?;

    Ok(serde_json::to_string(&shared_types::AttorneyResponse::from(att)).unwrap_or_default())
}

/// Delete an attorney by ID.
#[server]
pub async fn delete_attorney(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attorney;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let deleted = attorney::delete(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if deleted {
        Ok(())
    } else {
        Err(ServerFnError::new("Attorney not found"))
    }
}

/// Search attorneys by query string.
#[server]
pub async fn search_attorneys(
    court_id: String,
    query: String,
    page: Option<i64>,
    limit: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attorney;
    use shared_types::normalize_pagination;

    let pool = get_db().await;
    let (page, limit) = normalize_pagination(page, limit);
    let (attorneys, total) = attorney::search(pool, &court_id, &query, page, limit).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response = shared_types::PaginatedResponse::new(
        attorneys.into_iter().map(shared_types::AttorneyResponse::from).collect(),
        page, limit, total,
    );
    Ok(serde_json::to_string(&response).unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// Calendar server functions
// ═══════════════════════════════════════════════════════════════

/// Search calendar events with filters.
#[server]
pub async fn search_calendar_events(
    court_id: String,
    judge_id: Option<String>,
    courtroom: Option<String>,
    event_type: Option<String>,
    status: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::calendar;
    use chrono::DateTime;
    use uuid::Uuid;

    let pool = get_db().await;
    let offset = offset.unwrap_or(0).max(0);
    let limit = limit.unwrap_or(20).clamp(1, 100);

    let judge_uuid = judge_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;

    let date_from_parsed = date_from
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid date_from format"))?;

    let date_to_parsed = date_to
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid date_to format"))?;

    let (events, total) = calendar::search(
        pool, &court_id, judge_uuid,
        courtroom.as_deref().filter(|s| !s.is_empty()),
        event_type.as_deref().filter(|s| !s.is_empty()),
        status.as_deref().filter(|s| !s.is_empty()),
        date_from_parsed, date_to_parsed, offset, limit,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response = shared_types::CalendarSearchResponse {
        events: events.into_iter().map(shared_types::CalendarEntryResponse::from).collect(),
        total,
    };
    Ok(serde_json::to_string(&response).unwrap_or_default())
}

/// Get a single calendar event by ID.
#[server]
pub async fn get_calendar_event(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::calendar;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let event = calendar::find_by_id(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Calendar event not found"))?;

    Ok(serde_json::to_string(&shared_types::CalendarEntryResponse::from(event)).unwrap_or_default())
}

/// Schedule a new calendar event.
#[server]
pub async fn schedule_calendar_event(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::calendar;
    use shared_types::ScheduleEventRequest;

    let pool = get_db().await;
    let req: ScheduleEventRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;

    let event = calendar::create(pool, &court_id, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(serde_json::to_string(&shared_types::CalendarEntryResponse::from(event)).unwrap_or_default())
}

/// Delete a calendar event by ID.
#[server]
pub async fn delete_calendar_event(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::calendar;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let deleted = calendar::delete(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if deleted {
        Ok(())
    } else {
        Err(ServerFnError::new("Calendar event not found"))
    }
}

/// List all calendar events for a specific case.
#[server]
pub async fn list_calendar_by_case(court_id: String, case_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::calendar;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid = Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;

    let rows = calendar::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response: Vec<shared_types::CalendarEntryResponse> =
        rows.into_iter().map(shared_types::CalendarEntryResponse::from).collect();
    Ok(serde_json::to_string(&response).unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// Deadline server functions
// ═══════════════════════════════════════════════════════════════

/// Search deadlines with filters.
#[server]
pub async fn search_deadlines(
    court_id: String,
    status: Option<String>,
    case_id: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline;
    use chrono::DateTime;
    use uuid::Uuid;

    let pool = get_db().await;
    let offset = offset.unwrap_or(0).max(0);
    let limit = limit.unwrap_or(20).clamp(1, 100);

    let case_uuid = case_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;

    let date_from_parsed = date_from
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid date_from format"))?;

    let date_to_parsed = date_to
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid date_to format"))?;

    let (deadlines, total) = deadline::search(
        pool, &court_id,
        status.as_deref().filter(|s| !s.is_empty()),
        case_uuid,
        date_from_parsed, date_to_parsed,
        offset, limit,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response = shared_types::DeadlineSearchResponse {
        deadlines: deadlines.into_iter().map(shared_types::DeadlineResponse::from).collect(),
        total,
    };
    Ok(serde_json::to_string(&response).unwrap_or_default())
}

/// Get a single deadline by ID.
#[server]
pub async fn get_deadline(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let dl = deadline::find_by_id(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Deadline not found"))?;

    Ok(serde_json::to_string(&shared_types::DeadlineResponse::from(dl)).unwrap_or_default())
}

/// Create a new deadline.
#[server]
pub async fn create_deadline(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline;
    use shared_types::CreateDeadlineRequest;

    let pool = get_db().await;
    let req: CreateDeadlineRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;

    let dl = deadline::create(pool, &court_id, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(serde_json::to_string(&shared_types::DeadlineResponse::from(dl)).unwrap_or_default())
}

/// Update an existing deadline.
#[server]
pub async fn update_deadline(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline;
    use shared_types::UpdateDeadlineRequest;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: UpdateDeadlineRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;

    let dl = deadline::update(pool, &court_id, uuid, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Deadline not found"))?;

    Ok(serde_json::to_string(&shared_types::DeadlineResponse::from(dl)).unwrap_or_default())
}

/// Delete a deadline by ID.
#[server]
pub async fn delete_deadline(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let deleted = deadline::delete(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if deleted {
        Ok(())
    } else {
        Err(ServerFnError::new("Deadline not found"))
    }
}

// ═══════════════════════════════════════════════════════════════
// Case server functions
// ═══════════════════════════════════════════════════════════════

/// Search cases with filters.
#[server]
pub async fn search_cases(
    court_id: String,
    status: Option<String>,
    crime_type: Option<String>,
    priority: Option<String>,
    q: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case;

    let pool = get_db().await;
    let offset = offset.unwrap_or(0).max(0);
    let limit = limit.unwrap_or(20).clamp(1, 100);

    let (cases, total) = case::search(
        pool, &court_id,
        status.as_deref().filter(|s| !s.is_empty()),
        crime_type.as_deref().filter(|s| !s.is_empty()),
        priority.as_deref().filter(|s| !s.is_empty()),
        q.as_deref().filter(|s| !s.is_empty()),
        offset, limit,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response = shared_types::CaseSearchResponse {
        cases: cases.into_iter().map(shared_types::CaseResponse::from).collect(),
        total,
    };
    Ok(serde_json::to_string(&response).unwrap_or_default())
}

/// Get a single case by ID.
#[server]
pub async fn get_case(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let c = case::find_by_id(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Case not found"))?;

    Ok(serde_json::to_string(&shared_types::CaseResponse::from(c)).unwrap_or_default())
}

/// Create a new criminal case.
#[server]
pub async fn create_case(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case;
    use shared_types::CreateCaseRequest;

    let pool = get_db().await;
    let req: CreateCaseRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;

    let c = case::create(pool, &court_id, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(serde_json::to_string(&shared_types::CaseResponse::from(c)).unwrap_or_default())
}

/// Delete a case by ID.
#[server]
pub async fn delete_case(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let deleted = case::delete(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if deleted {
        Ok(())
    } else {
        Err(ServerFnError::new("Case not found"))
    }
}

/// Update the status of a case.
#[server]
pub async fn update_case_status(
    court_id: String,
    id: String,
    status: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let c = case::update_status(pool, &court_id, uuid, &status).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Case not found"))?;

    Ok(serde_json::to_string(&shared_types::CaseResponse::from(c)).unwrap_or_default())
}

/// Update a case (partial update — only provided fields are changed).
#[server]
pub async fn update_case(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case;
    use shared_types::UpdateCaseRequest;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: UpdateCaseRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;

    let c = case::update(pool, &court_id, uuid, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Case not found"))?;

    Ok(serde_json::to_string(&shared_types::CaseResponse::from(c)).unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// Docket server functions
// ═══════════════════════════════════════════════════════════════

/// Search docket entries with filters.
#[server]
pub async fn search_docket_entries(
    court_id: String,
    case_id: Option<String>,
    entry_type: Option<String>,
    q: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::docket;
    use uuid::Uuid;

    let pool = get_db().await;
    let offset = offset.unwrap_or(0).max(0);
    let limit = limit.unwrap_or(20).clamp(1, 100);

    let case_uuid = case_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;

    let (entries, total) = docket::search(
        pool, &court_id, case_uuid,
        entry_type.as_deref().filter(|s| !s.is_empty()),
        q.as_deref().filter(|s| !s.is_empty()),
        offset, limit,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response = shared_types::DocketSearchResponse {
        entries: entries.into_iter().map(shared_types::DocketEntryResponse::from).collect(),
        total,
    };
    Ok(serde_json::to_string(&response).unwrap_or_default())
}

/// Get a single docket entry by ID.
#[server]
pub async fn get_docket_entry(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::docket;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let entry = docket::find_by_id(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Docket entry not found"))?;

    Ok(serde_json::to_string(&shared_types::DocketEntryResponse::from(entry)).unwrap_or_default())
}

/// Create a new docket entry.
#[server]
pub async fn create_docket_entry(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::docket;
    use shared_types::CreateDocketEntryRequest;

    let pool = get_db().await;
    let req: CreateDocketEntryRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;

    let entry = docket::create(pool, &court_id, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(serde_json::to_string(&shared_types::DocketEntryResponse::from(entry)).unwrap_or_default())
}

/// Delete a docket entry by ID.
#[server]
pub async fn delete_docket_entry(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::docket;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let deleted = docket::delete(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if deleted {
        Ok(())
    } else {
        Err(ServerFnError::new("Docket entry not found"))
    }
}

/// List docket entries for a specific case.
#[server]
pub async fn get_case_docket(
    court_id: String,
    case_id: String,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::docket;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| ServerFnError::new("Invalid case UUID"))?;
    let offset = offset.unwrap_or(0).max(0);
    let limit = limit.unwrap_or(50).clamp(1, 100);

    let (entries, total) = docket::list_by_case(pool, &court_id, case_uuid, offset, limit).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response = shared_types::DocketSearchResponse {
        entries: entries.into_iter().map(shared_types::DocketEntryResponse::from).collect(),
        total,
    };
    Ok(serde_json::to_string(&response).unwrap_or_default())
}

// ── Docket Attachment server functions ────────────────────────────

/// List uploaded attachments for a docket entry.
#[server]
pub async fn list_entry_attachments(
    court_id: String,
    entry_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attachment;
    use uuid::Uuid;

    let pool = get_db().await;
    let entry_uuid = Uuid::parse_str(&entry_id)
        .map_err(|_| ServerFnError::new("Invalid entry UUID"))?;

    let attachments = attachment::list_by_entry(pool, &court_id, entry_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response: Vec<shared_types::DocketAttachmentResponse> = attachments
        .into_iter()
        .map(shared_types::DocketAttachmentResponse::from)
        .collect();

    Ok(serde_json::to_string(&response).unwrap_or_default())
}

/// Initiate a presigned upload for a new attachment.
#[server]
pub async fn create_entry_attachment(
    court_id: String,
    entry_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{attachment, docket};
    use crate::storage::{ObjectStore, S3ObjectStore};
    use shared_types::CreateAttachmentRequest;
    use uuid::Uuid;

    let pool = get_db().await;
    let entry_uuid = Uuid::parse_str(&entry_id)
        .map_err(|_| ServerFnError::new("Invalid entry UUID"))?;

    let req: CreateAttachmentRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;

    // Verify entry exists in tenant
    docket::find_by_id(pool, &court_id, entry_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Docket entry not found"))?;

    if req.file_name.trim().is_empty() {
        return Err(ServerFnError::new("file_name must not be empty"));
    }

    let file_uuid = Uuid::new_v4();
    let object_key = format!(
        "{}/docket/{}/{}/{}",
        court_id, entry_id, file_uuid, req.file_name
    );

    let att = attachment::create_pending(
        pool,
        &court_id,
        entry_uuid,
        &req.file_name,
        req.file_size,
        &req.content_type,
        &object_key,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let store = S3ObjectStore::from_env();
    let (presign_url, required_headers) = store
        .presign_put(&object_key, &req.content_type)
        .await
        .map_err(|e| ServerFnError::new(format!("Presign failed: {}", e)))?;

    let response = shared_types::CreateAttachmentResponse {
        attachment_id: att.id.to_string(),
        presign_url,
        object_key,
        required_headers,
    };

    Ok(serde_json::to_string(&response).unwrap_or_default())
}

/// Finalize an attachment upload (verify in S3, mark uploaded_at).
#[server]
pub async fn finalize_attachment(
    court_id: String,
    attachment_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attachment;
    use crate::storage::{ObjectStore, S3ObjectStore};
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attachment_id)
        .map_err(|_| ServerFnError::new("Invalid attachment UUID"))?;

    let att = attachment::find_by_id(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attachment not found"))?;

    let store = S3ObjectStore::from_env();
    let exists = store
        .head(&att.storage_key)
        .await
        .map_err(|e| ServerFnError::new(format!("HEAD failed: {}", e)))?;

    if !exists {
        return Err(ServerFnError::new("Object not yet uploaded to storage"));
    }

    attachment::mark_uploaded(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let updated = attachment::find_by_id(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attachment not found after update"))?;

    Ok(serde_json::to_string(&shared_types::DocketAttachmentResponse::from(updated))
        .unwrap_or_default())
}

/// Get a presigned download URL for an attachment.
#[server]
pub async fn get_attachment_download_url(
    court_id: String,
    attachment_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::attachment;
    use crate::storage::{ObjectStore, S3ObjectStore};
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attachment_id)
        .map_err(|_| ServerFnError::new("Invalid attachment UUID"))?;

    let att = attachment::find_by_id(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attachment not found"))?;

    let store = S3ObjectStore::from_env();
    let url = store
        .presign_get(&att.storage_key)
        .await
        .map_err(|e| ServerFnError::new(format!("Presign GET failed: {}", e)))?;

    Ok(serde_json::to_string(&serde_json::json!({
        "download_url": url,
        "filename": att.filename,
        "content_type": att.content_type,
    }))
    .unwrap_or_default())
}

/// Cross-platform upload: receives file bytes, uploads to S3 server-side, and finalizes.
/// This avoids requiring client-side JS fetch for presigned URL PUT.
#[server]
pub async fn upload_docket_attachment(
    court_id: String,
    entry_id: String,
    file_name: String,
    content_type: String,
    file_size: i64,
    file_bytes: Vec<u8>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{attachment, docket};
    use crate::storage::{ObjectStore, S3ObjectStore};
    use uuid::Uuid;

    let pool = get_db().await;
    let entry_uuid = Uuid::parse_str(&entry_id)
        .map_err(|_| ServerFnError::new("Invalid entry UUID"))?;

    // Verify entry exists in tenant
    docket::find_by_id(pool, &court_id, entry_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Docket entry not found"))?;

    if file_name.trim().is_empty() {
        return Err(ServerFnError::new("file_name must not be empty"));
    }

    let file_uuid = Uuid::new_v4();
    let object_key = format!(
        "{}/docket/{}/{}/{}",
        court_id, entry_id, file_uuid, file_name
    );

    // Insert pending row
    let att = attachment::create_pending(
        pool,
        &court_id,
        entry_uuid,
        &file_name,
        file_size,
        &content_type,
        &object_key,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Upload to S3 with SSE-S3 (AES256) encryption
    let store = S3ObjectStore::from_env();
    store
        .put(&object_key, &content_type, file_bytes)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    // Mark as uploaded
    attachment::mark_uploaded(pool, &court_id, att.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let updated = attachment::find_by_id(pool, &court_id, att.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attachment not found after upload"))?;

    Ok(serde_json::to_string(&shared_types::DocketAttachmentResponse::from(updated))
        .unwrap_or_default())
}

// ── Service Record server functions ────────────────────────────

/// List service records for a document.
#[server]
pub async fn list_document_service_records(
    court_id: String,
    document_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::service_record;
    use uuid::Uuid;

    let pool = get_db().await;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    let records = service_record::list_by_document(pool, &court_id, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response: Vec<shared_types::ServiceRecordResponse> = records
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(serde_json::to_string(&response).unwrap_or_default())
}

/// Create a new service record.
#[server]
pub async fn create_service_record(
    court_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::service_record;
    use shared_types::CreateServiceRecordRequest;

    let pool = get_db().await;
    let req: CreateServiceRecordRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;

    let record = service_record::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(serde_json::to_string(&shared_types::ServiceRecordResponse::from(record))
        .unwrap_or_default())
}

/// Link an existing document to a docket entry.
#[server]
pub async fn link_document_to_entry(
    court_id: String,
    entry_id: String,
    document_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use uuid::Uuid;

    let pool = get_db().await;
    let entry_uuid = Uuid::parse_str(&entry_id)
        .map_err(|_| ServerFnError::new("Invalid entry UUID"))?;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    // Verify both exist in this court
    crate::repo::docket::find_by_id(pool, &court_id, entry_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Docket entry not found"))?;

    crate::repo::document::find_by_id(pool, &court_id, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Document not found"))?;

    let updated = crate::repo::docket::link_document(pool, &court_id, entry_uuid, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(serde_json::to_string(&shared_types::DocketEntryResponse::from(updated))
        .unwrap_or_default())
}

/// Promote a docket attachment into a canonical document.
#[server]
pub async fn promote_attachment_to_document(
    court_id: String,
    docket_attachment_id: String,
    title: Option<String>,
    document_type: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&docket_attachment_id)
        .map_err(|_| ServerFnError::new("Invalid attachment UUID"))?;

    // Look up attachment — must belong to this court
    let attachment = crate::repo::attachment::find_by_id(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Attachment not found"))?;

    if attachment.uploaded_at.is_none() {
        return Err(ServerFnError::new("Attachment not uploaded yet"));
    }

    // Check for existing document (idempotency)
    if let Some(existing) =
        crate::repo::document::find_by_source_attachment(pool, &court_id, att_uuid)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        return Ok(serde_json::to_string(&shared_types::DocumentResponse::from(existing))
            .unwrap_or_default());
    }

    // Resolve case_id from docket entry
    let entry = crate::repo::docket::find_by_id(pool, &court_id, attachment.docket_entry_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Docket entry not found"))?;

    let doc_title = title.unwrap_or_else(|| attachment.filename.clone());
    let doc_type = document_type.as_deref().unwrap_or("Other");
    let checksum = attachment.sha256.clone().unwrap_or_default();

    let document = crate::repo::document::promote_attachment(
        pool,
        &court_id,
        att_uuid,
        entry.case_id,
        &doc_title,
        doc_type,
        &attachment.storage_key,
        attachment.file_size,
        &attachment.content_type,
        &checksum,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Auto-link the new document to the owning docket entry
    let _ = crate::repo::docket::link_document(
        pool,
        &court_id,
        attachment.docket_entry_id,
        document.id,
    )
    .await;

    Ok(serde_json::to_string(&shared_types::DocumentResponse::from(document))
        .unwrap_or_default())
}

/// List parties for a case (lightweight, for dropdowns).
#[server]
pub async fn list_case_parties(
    court_id: String,
    case_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| ServerFnError::new("Invalid case UUID"))?;

    let parties = party::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Return lightweight JSON: just id, name, party_type for the dropdown
    let items: Vec<serde_json::Value> = parties
        .iter()
        .map(|p| serde_json::json!({
            "id": p.id.to_string(),
            "name": p.name,
            "party_type": p.party_type,
        }))
        .collect();

    Ok(serde_json::to_string(&items).unwrap_or_default())
}

/// Mark a service record as complete.
#[server]
pub async fn complete_service_record(
    court_id: String,
    record_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::service_record;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&record_id)
        .map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let record = service_record::complete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(serde_json::to_string(&shared_types::ServiceRecordResponse::from(record))
        .unwrap_or_default())
}

// ── Filing server functions ────────────────────────────────────

/// Upload a file for a filing. Handles S3 upload server-side and returns
/// the staged upload_id that can be referenced in the filing submission.
#[server]
pub async fn upload_filing_document(
    court_id: String,
    file_name: String,
    content_type: String,
    file_size: i64,
    file_bytes: Vec<u8>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::filing;
    use crate::storage::{ObjectStore, S3ObjectStore};
    use uuid::Uuid;

    let pool = get_db().await;

    if file_name.trim().is_empty() {
        return Err(ServerFnError::new("file_name must not be empty"));
    }

    let file_uuid = Uuid::new_v4();
    let object_key = format!(
        "{}/filings/staging/{}/{}",
        court_id, file_uuid, file_name
    );

    // Create pending upload row
    let upload = filing::create_pending_upload(
        pool, &court_id, &file_name, file_size, &content_type, &object_key,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Upload to S3
    let store = S3ObjectStore::from_env();
    store
        .put(&object_key, &content_type, file_bytes)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    // Mark as finalized
    filing::mark_upload_finalized(pool, &court_id, upload.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Return the upload_id for use in filing submission
    Ok(upload.id.to_string())
}

/// Submit an electronic filing. Validates, then atomically creates
/// Document + DocketEntry + Filing. Returns JSON FilingResponse.
#[server]
pub async fn submit_filing(
    court_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::filing;
    use shared_types::{FilingResponse, NefSummary, ValidateFilingRequest};

    let pool = get_db().await;
    let req: ValidateFilingRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;

    let (f, _doc, docket_entry, case_number, _nef) = filing::submit(pool, &court_id, &req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response = FilingResponse {
        filing_id: f.id.to_string(),
        document_id: f.document_id.map(|u| u.to_string()).unwrap_or_default(),
        docket_entry_id: f.docket_entry_id.map(|u| u.to_string()).unwrap_or_default(),
        case_id: f.case_id.to_string(),
        status: f.status,
        filed_date: f.filed_date.to_rfc3339(),
        nef: NefSummary {
            case_number,
            document_title: req.title.clone(),
            filed_by: req.filed_by.clone(),
            filed_date: f.filed_date.to_rfc3339(),
            docket_number: docket_entry.entry_number,
        },
    };

    Ok(serde_json::to_string(&response).unwrap_or_default())
}

/// Validate a filing request without submitting.
/// Returns JSON ValidateFilingResponse.
#[server]
pub async fn validate_filing_request(
    court_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::filing;
    use shared_types::ValidateFilingRequest;

    let pool = get_db().await;
    let req: ValidateFilingRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;

    let response = filing::validate(pool, &court_id, &req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(serde_json::to_string(&response).unwrap_or_default())
}

/// Retrieve the Notice of Electronic Filing for a given filing.
#[server]
pub async fn get_nef(
    court_id: String,
    filing_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::nef;
    use uuid::Uuid;

    let pool = get_db().await;
    let filing_uuid = Uuid::parse_str(&filing_id)
        .map_err(|_| ServerFnError::new("Invalid filing UUID"))?;

    let n = nef::find_by_filing(pool, &court_id, filing_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("NEF not found"))?;

    Ok(serde_json::to_string(&shared_types::NefResponse::from(n)).unwrap_or_default())
}

/// Retrieve a NEF by its primary ID.
#[server]
pub async fn get_nef_by_id(
    court_id: String,
    nef_id: String,
) -> Result<Option<String>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::nef;
    use uuid::Uuid;

    let pool = get_db().await;
    let nef_uuid = Uuid::parse_str(&nef_id)
        .map_err(|_| ServerFnError::new("Invalid NEF UUID"))?;

    let maybe_nef = nef::find_by_id(pool, &court_id, nef_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(maybe_nef.map(|n| serde_json::to_string(&shared_types::NefResponse::from(n)).unwrap_or_default()))
}

/// Retrieve the NEF for a docket entry (if one exists).
#[server]
pub async fn get_nef_by_docket_entry(
    court_id: String,
    docket_entry_id: String,
) -> Result<Option<String>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::nef;
    use uuid::Uuid;

    let pool = get_db().await;
    let entry_uuid = Uuid::parse_str(&docket_entry_id)
        .map_err(|_| ServerFnError::new("Invalid docket entry UUID"))?;

    let maybe_nef = nef::find_by_docket_entry(pool, &court_id, entry_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(maybe_nef.map(|n| serde_json::to_string(&shared_types::NefResponse::from(n)).unwrap_or_default()))
}

// ── Filing list / detail server functions ─────────────

/// List all filings for a court with optional search and pagination.
#[server]
pub async fn list_all_filings(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::filing;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = filing::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::FilingListItem> =
        rows.into_iter().map(shared_types::FilingListItem::from).collect();

    let total_pages = if per_page > 0 { (total + per_page - 1) / per_page } else { 0 };
    let meta = shared_types::PaginationMeta {
        total,
        page,
        limit: per_page,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    };

    let resp = shared_types::PaginatedResponse {
        data: responses,
        meta,
    };

    Ok(serde_json::to_string(&resp).unwrap_or_default())
}

/// Get a single filing by ID.
#[server]
pub async fn get_filing_by_id(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::filing;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = filing::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Filing not found"))?;

    Ok(serde_json::to_string(&shared_types::FilingListItem::from(row)).unwrap_or_default())
}

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

/// Extract the X-Court-District header from the current Dioxus server context.
#[cfg(feature = "server")]
fn extract_court_header_sfn() -> Option<String> {
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
#[cfg(feature = "server")]
fn require_membership_access_sfn(
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

// ── Document Action Server Functions ───────────────────────────

/// Seal a document. Requires clerk/judge role via REST handler delegation.
#[server]
pub async fn seal_document_action(
    court_id: String,
    document_id: String,
    sealing_level: String,
    reason_code: String,
    motion_id: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{document, document_event};
    use shared_types::{DocumentResponse, SealingLevel};
    use uuid::Uuid;

    let pool = get_db().await;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    let level = SealingLevel::from_db_str(&sealing_level);
    if !level.is_sealed() {
        return Err(ServerFnError::new(
            "sealing_level must be one of: SealedCourtOnly, SealedCaseParticipants, SealedAttorneysOnly",
        ));
    }

    let motion_uuid = motion_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| ServerFnError::new("Invalid motion_id UUID"))?;

    let doc = document::seal(pool, &court_id, doc_uuid, &level, &reason_code, motion_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Audit event (fire-and-forget)
    let _ = document_event::create(
        pool,
        &court_id,
        doc_uuid,
        "sealed",
        "ui-user",
        serde_json::json!({
            "sealing_level": sealing_level,
            "reason_code": reason_code,
            "motion_id": motion_id,
        }),
    )
    .await;

    Ok(serde_json::to_string(&DocumentResponse::from(doc)).unwrap_or_default())
}

/// Unseal a document.
#[server]
pub async fn unseal_document_action(
    court_id: String,
    document_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{document, document_event};
    use shared_types::DocumentResponse;
    use uuid::Uuid;

    let pool = get_db().await;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    let doc = document::unseal(pool, &court_id, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let _ = document_event::create(
        pool, &court_id, doc_uuid, "unsealed", "ui-user", serde_json::json!({}),
    )
    .await;

    Ok(serde_json::to_string(&DocumentResponse::from(doc)).unwrap_or_default())
}

/// Strike a document from the record.
#[server]
pub async fn strike_document_action(
    court_id: String,
    document_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{document, document_event};
    use shared_types::DocumentResponse;
    use uuid::Uuid;

    let pool = get_db().await;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    let doc = document::strike(pool, &court_id, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let _ = document_event::create(
        pool, &court_id, doc_uuid, "stricken", "ui-user", serde_json::json!({}),
    )
    .await;

    Ok(serde_json::to_string(&DocumentResponse::from(doc)).unwrap_or_default())
}

/// Replace a document file. Handles S3 upload server-side.
#[server]
pub async fn replace_document_file(
    court_id: String,
    document_id: String,
    file_name: String,
    content_type: String,
    file_size: i64,
    file_bytes: Vec<u8>,
    title: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::{document, document_event, filing};
    use crate::storage::{ObjectStore, S3ObjectStore};
    use shared_types::DocumentResponse;
    use uuid::Uuid;

    let pool = get_db().await;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    if file_name.trim().is_empty() {
        return Err(ServerFnError::new("file_name must not be empty"));
    }

    // Stage upload
    let file_uuid = Uuid::new_v4();
    let object_key = format!(
        "{}/documents/replace/{}/{}",
        court_id, file_uuid, file_name
    );

    let upload = filing::create_pending_upload(
        pool, &court_id, &file_name, file_size, &content_type, &object_key,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Upload to S3
    let store = S3ObjectStore::from_env();
    store
        .put(&object_key, &content_type, file_bytes)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    // Mark as finalized
    filing::mark_upload_finalized(pool, &court_id, upload.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Get original document to inherit title
    let original = document::find_by_id(pool, &court_id, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Document not found"))?;

    let doc_title = title.as_deref().unwrap_or(&original.title);

    let replacement = document::replace(
        pool,
        &court_id,
        doc_uuid,
        doc_title,
        &object_key,
        file_size,
        &content_type,
        "",
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let _ = document_event::create(
        pool,
        &court_id,
        doc_uuid,
        "replaced",
        "ui-user",
        serde_json::json!({
            "replacement_document_id": replacement.id.to_string(),
        }),
    )
    .await;

    Ok(serde_json::to_string(&DocumentResponse::from(replacement)).unwrap_or_default())
}

/// List document audit events.
#[server]
pub async fn list_document_events_action(
    court_id: String,
    document_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::document_event;
    use shared_types::DocumentEventResponse;
    use uuid::Uuid;

    let pool = get_db().await;
    let doc_uuid = Uuid::parse_str(&document_id)
        .map_err(|_| ServerFnError::new("Invalid document UUID"))?;

    let events = document_event::list_by_document(pool, &court_id, doc_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response: Vec<DocumentEventResponse> = events.into_iter().map(Into::into).collect();
    Ok(serde_json::to_string(&response).unwrap_or_default())
}

/// List all documents for a court with optional title search and pagination.
#[server]
pub async fn list_all_documents(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::document;
    use shared_types::DocumentResponse;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = document::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<DocumentResponse> =
        rows.into_iter().map(DocumentResponse::from).collect();

    let total_pages = if per_page > 0 { (total + per_page - 1) / per_page } else { 0 };
    let meta = shared_types::PaginationMeta {
        total,
        page,
        limit: per_page,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    };

    let resp = shared_types::PaginatedResponse {
        data: responses,
        meta,
    };

    Ok(serde_json::to_string(&resp).unwrap_or_default())
}

/// Get a single document by ID.
#[server]
pub async fn get_document_by_id(
    court_id: String,
    id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::document;
    use shared_types::DocumentResponse;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let doc = document::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Document not found"))?;
    Ok(serde_json::to_string(&DocumentResponse::from(doc)).unwrap_or_default())
}

// ── Unified Event Composer Server Functions ────────────────────

/// Submit a unified docket event (text entry, filing, or promote attachment).
/// Dispatches to the appropriate workflow based on event_kind.
#[server]
pub async fn submit_event(
    court_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::event;
    use shared_types::SubmitEventRequest;

    let pool = get_db().await;
    let req: SubmitEventRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;

    let response = event::submit_event(pool, &court_id, &req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(serde_json::to_string(&response).unwrap_or_default())
}

/// Get the unified case timeline (docket entries + document events).
#[server]
pub async fn get_case_timeline(
    court_id: String,
    case_id: String,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use shared_types::{TimelineEntry, TimelineResponse};
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;

    let limit = limit.unwrap_or(50).min(200);
    let offset = offset.unwrap_or(0);

    // Fetch docket entries
    let (docket_entries, _) =
        crate::repo::docket::list_by_case(pool, &court_id, case_uuid, 0, 1000)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Fetch document events for documents in this case
    let doc_events =
        crate::repo::document_event::list_by_case(pool, &court_id, case_uuid)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Fetch NEFs for this case
    let nefs =
        crate::repo::nef::list_by_case(pool, &court_id, case_uuid)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Merge into unified timeline
    let mut entries: Vec<TimelineEntry> = Vec::new();

    for de in &docket_entries {
        entries.push(TimelineEntry {
            id: de.id.to_string(),
            source: "docket_entry".to_string(),
            timestamp: de.date_filed.to_rfc3339(),
            summary: de.description.clone(),
            actor: de.filed_by.clone(),
            entry_type: de.entry_type.clone(),
            is_sealed: de.is_sealed,
            document_id: de.document_id.map(|u| u.to_string()),
            entry_number: Some(de.entry_number),
            detail: serde_json::json!({}),
        });
    }

    for evt in &doc_events {
        entries.push(TimelineEntry {
            id: evt.id.to_string(),
            source: "document_event".to_string(),
            timestamp: evt.created_at.to_rfc3339(),
            summary: format!("Document {}", evt.event_type),
            actor: Some(evt.actor.clone()),
            entry_type: evt.event_type.clone(),
            is_sealed: false,
            document_id: Some(evt.document_id.to_string()),
            entry_number: None,
            detail: evt.detail.clone(),
        });
    }

    for nef in &nefs {
        entries.push(TimelineEntry {
            id: nef.id.to_string(),
            source: "nef".to_string(),
            timestamp: nef.created_at.to_rfc3339(),
            summary: "Notice of Electronic Filing issued".to_string(),
            actor: None,
            entry_type: "nef".to_string(),
            is_sealed: false,
            document_id: Some(nef.document_id.to_string()),
            entry_number: None,
            detail: serde_json::json!({
                "nef_id": nef.id.to_string(),
                "filing_id": nef.filing_id.to_string(),
                "docket_entry_id": nef.docket_entry_id.to_string(),
            }),
        });
    }

    // Sort newest first
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let total = entries.len() as i64;
    let paginated: Vec<TimelineEntry> = entries
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    Ok(serde_json::to_string(&TimelineResponse {
        entries: paginated,
        total,
    })
    .unwrap_or_default())
}

// ── Defendant Server Functions ─────────────────────────

#[server]
pub async fn list_defendants(court_id: String, case_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = defendant::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

/// List all defendants for a court (across all cases) with optional search and pagination.
#[server]
pub async fn list_all_defendants(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = defendant::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::DefendantResponse> =
        rows.into_iter().map(shared_types::DefendantResponse::from).collect();

    let total_pages = if per_page > 0 { (total + per_page - 1) / per_page } else { 0 };
    let meta = shared_types::PaginationMeta {
        total,
        page,
        limit: per_page,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    };

    let resp = shared_types::PaginatedResponse {
        data: responses,
        meta,
    };

    Ok(serde_json::to_string(&resp).unwrap_or_default())
}

#[server]
pub async fn get_defendant(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = defendant::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn create_defendant(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;

    let pool = get_db().await;
    let req: shared_types::CreateDefendantRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = defendant::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn update_defendant(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: shared_types::UpdateDefendantRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = defendant::update(pool, &court_id, uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_defendant(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    defendant::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Party Server Functions ─────────────────────────────

#[server]
pub async fn create_party(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;

    let pool = get_db().await;
    let req: shared_types::CreatePartyRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = party::create(pool, &court_id, &req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn get_party(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = party::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn update_party(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: shared_types::UpdatePartyRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = party::update(pool, &court_id, uuid, &req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_party(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    party::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server]
pub async fn list_parties_by_case(
    court_id: String,
    case_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = party::list_full_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn list_unrepresented_parties(court_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;

    let pool = get_db().await;
    let rows = party::list_unrepresented(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn list_parties_by_attorney(
    court_id: String,
    attorney_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid =
        Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = party::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

/// List all parties for a court (across all cases) with optional search and pagination.
#[server]
pub async fn list_all_parties(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::party;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = party::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::PartyResponse> =
        rows.into_iter().map(shared_types::PartyResponse::from).collect();

    let total_pages = if per_page > 0 { (total + per_page - 1) / per_page } else { 0 };
    let meta = shared_types::PaginationMeta {
        total,
        page,
        limit: per_page,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    };

    let resp = shared_types::PaginatedResponse {
        data: responses,
        meta,
    };

    Ok(serde_json::to_string(&resp).unwrap_or_default())
}

/// List all representations for a specific party.
#[server]
pub async fn list_representations_by_party(
    court_id: String,
    party_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::representation;
    use uuid::Uuid;

    let pool = get_db().await;
    let party_uuid =
        Uuid::parse_str(&party_id).map_err(|_| ServerFnError::new("Invalid party_id UUID"))?;
    let rows = representation::list_by_party(pool, &court_id, party_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let responses: Vec<shared_types::RepresentationResponse> =
        rows.into_iter().map(shared_types::RepresentationResponse::from).collect();
    Ok(serde_json::to_string(&responses).unwrap_or_default())
}

// ── Evidence Server Functions ──────────────────────────

/// List all evidence for a court (across all cases) with optional search and pagination.
#[server]
pub async fn list_all_evidence(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::evidence;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = evidence::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::EvidenceResponse> =
        rows.into_iter().map(shared_types::EvidenceResponse::from).collect();

    let total_pages = if per_page > 0 { (total + per_page - 1) / per_page } else { 0 };
    let meta = shared_types::PaginationMeta {
        total,
        page,
        limit: per_page,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    };

    let resp = shared_types::PaginatedResponse {
        data: responses,
        meta,
    };

    Ok(serde_json::to_string(&resp).unwrap_or_default())
}

#[server]
pub async fn list_evidence_by_case(
    court_id: String,
    case_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::evidence;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = evidence::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn get_evidence(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::evidence;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = evidence::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn create_evidence(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::evidence;

    let pool = get_db().await;
    let req: shared_types::CreateEvidenceRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = evidence::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn update_evidence(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::evidence;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: shared_types::UpdateEvidenceRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = evidence::update(pool, &court_id, uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_evidence(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::evidence;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    evidence::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Custody Transfer Server Functions ──────────────────

#[server]
pub async fn list_custody_transfers(
    court_id: String,
    evidence_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::custody_transfer;
    use uuid::Uuid;

    let pool = get_db().await;
    let ev_uuid =
        Uuid::parse_str(&evidence_id).map_err(|_| ServerFnError::new("Invalid evidence_id UUID"))?;
    let rows = custody_transfer::list_by_evidence(pool, &court_id, ev_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_custody_transfer(
    court_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::custody_transfer;

    let pool = get_db().await;
    let req: shared_types::CreateCustodyTransferRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = custody_transfer::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

// ── Order Server Functions ─────────────────────────────

/// List all orders for a court (across all cases) with optional search and pagination.
#[server]
pub async fn list_all_orders(
    court_id: String,
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;

    let pool = get_db().await;
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let page = page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let (rows, total) = order::list_all(
        pool,
        &court_id,
        q.as_deref().filter(|s| !s.is_empty()),
        offset,
        per_page,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let responses: Vec<shared_types::JudicialOrderResponse> =
        rows.into_iter().map(shared_types::JudicialOrderResponse::from).collect();

    let total_pages = if per_page > 0 { (total + per_page - 1) / per_page } else { 0 };
    let meta = shared_types::PaginationMeta {
        total,
        page,
        limit: per_page,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    };

    let resp = shared_types::PaginatedResponse {
        data: responses,
        meta,
    };

    Ok(serde_json::to_string(&resp).unwrap_or_default())
}

/// Sign an order, updating its status to "Signed".
#[server]
pub async fn sign_order_action(
    court_id: String,
    id: String,
    signed_by: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let order = sqlx::query_as!(
        shared_types::JudicialOrder,
        r#"
        UPDATE judicial_orders SET
            status = 'Signed',
            signer_name = $3,
            signed_at = NOW(),
            signature_hash = md5($3 || id::text),
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, judge_id, order_type, title, content,
                  status, is_sealed, signer_name, signed_at, signature_hash,
                  issued_at, effective_date, expiration_date, related_motions,
                  created_at, updated_at
        "#,
        uuid,
        &court_id,
        signed_by,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Order not found"))?;

    Ok(serde_json::to_string(&shared_types::JudicialOrderResponse::from(order)).unwrap_or_default())
}

/// Issue an order, updating its status to "Filed".
#[server]
pub async fn issue_order_action(
    court_id: String,
    id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let order = sqlx::query_as!(
        shared_types::JudicialOrder,
        r#"
        UPDATE judicial_orders SET
            status = 'Filed',
            issued_at = NOW(),
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, judge_id, order_type, title, content,
                  status, is_sealed, signer_name, signed_at, signature_hash,
                  issued_at, effective_date, expiration_date, related_motions,
                  created_at, updated_at
        "#,
        uuid,
        &court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Order not found"))?;

    Ok(serde_json::to_string(&shared_types::JudicialOrderResponse::from(order)).unwrap_or_default())
}

#[server]
pub async fn list_orders_by_case(
    court_id: String,
    case_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = order::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn list_orders_by_judge(
    court_id: String,
    judge_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;
    use uuid::Uuid;

    let pool = get_db().await;
    let judge_uuid =
        Uuid::parse_str(&judge_id).map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let rows = order::list_by_judge(pool, &court_id, judge_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn get_order(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = order::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn create_order(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;

    let pool = get_db().await;
    let req: shared_types::CreateJudicialOrderRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = order::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn update_order(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: shared_types::UpdateJudicialOrderRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = order::update(pool, &court_id, uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_order(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    order::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Order Template Server Functions ────────────────────

#[server]
pub async fn list_order_templates(court_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order_template;

    let pool = get_db().await;
    let rows = order_template::list_all(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn list_active_order_templates(court_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order_template;

    let pool = get_db().await;
    let rows = order_template::list_active(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn get_order_template(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order_template;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = order_template::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn create_order_template(
    court_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order_template;

    let pool = get_db().await;
    let req: shared_types::CreateOrderTemplateRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = order_template::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn update_order_template(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order_template;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: shared_types::UpdateOrderTemplateRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = order_template::update(pool, &court_id, uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_order_template(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::order_template;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    order_template::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Sentencing Server Functions ────────────────────────

#[server]
pub async fn list_sentencing_by_case(
    court_id: String,
    case_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = sentencing::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn list_sentencing_by_defendant(
    court_id: String,
    defendant_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing;
    use uuid::Uuid;

    let pool = get_db().await;
    let def_uuid = Uuid::parse_str(&defendant_id)
        .map_err(|_| ServerFnError::new("Invalid defendant_id UUID"))?;
    let rows = sentencing::list_by_defendant(pool, &court_id, def_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn get_sentencing(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = sentencing::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn create_sentencing(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing;

    let pool = get_db().await;
    let req: shared_types::CreateSentencingRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = sentencing::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn update_sentencing(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: shared_types::UpdateSentencingRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = sentencing::update(pool, &court_id, uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_sentencing(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    sentencing::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Sentencing Condition Server Functions ───────────────

#[server]
pub async fn list_sentencing_conditions(
    court_id: String,
    sentencing_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing_condition;
    use uuid::Uuid;

    let pool = get_db().await;
    let s_uuid = Uuid::parse_str(&sentencing_id)
        .map_err(|_| ServerFnError::new("Invalid sentencing_id UUID"))?;
    let rows = sentencing_condition::list_by_sentencing(pool, &court_id, s_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_sentencing_condition(
    court_id: String,
    sentencing_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::sentencing_condition;
    use uuid::Uuid;

    let pool = get_db().await;
    let s_uuid = Uuid::parse_str(&sentencing_id)
        .map_err(|_| ServerFnError::new("Invalid sentencing_id UUID"))?;
    let req: shared_types::CreateSpecialConditionRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = sentencing_condition::create(pool, &court_id, s_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

// ── Speedy Trial Server Functions ──────────────────────

#[server]
pub async fn get_speedy_trial(
    court_id: String,
    case_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::speedy_trial;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let row = speedy_trial::find_by_case_id(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No speedy trial clock found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn start_speedy_trial(
    court_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::speedy_trial;

    let pool = get_db().await;
    let req: shared_types::StartSpeedyTrialRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = speedy_trial::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn update_speedy_trial(
    court_id: String,
    case_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::speedy_trial;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let req: shared_types::UpdateSpeedyTrialClockRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = speedy_trial::update(pool, &court_id, case_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn list_speedy_trial_delays(
    court_id: String,
    case_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::speedy_trial;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = speedy_trial::list_delays_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_speedy_trial_delay(
    court_id: String,
    case_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::speedy_trial;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let req: shared_types::CreateExcludableDelayRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = speedy_trial::create_delay(pool, &court_id, case_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_speedy_trial_delay(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::speedy_trial;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    speedy_trial::delete_delay(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Extension Request Server Functions ─────────────────

#[server]
pub async fn list_extensions_by_deadline(
    court_id: String,
    deadline_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::extension_request;
    use uuid::Uuid;

    let pool = get_db().await;
    let dl_uuid = Uuid::parse_str(&deadline_id)
        .map_err(|_| ServerFnError::new("Invalid deadline_id UUID"))?;
    let rows = extension_request::list_by_deadline(pool, &court_id, dl_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn get_extension(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::extension_request;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = extension_request::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn create_extension_request_fn(
    court_id: String,
    deadline_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::extension_request;
    use uuid::Uuid;

    let pool = get_db().await;
    let dl_uuid = Uuid::parse_str(&deadline_id)
        .map_err(|_| ServerFnError::new("Invalid deadline_id UUID"))?;
    let req: shared_types::CreateExtensionRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = extension_request::create(pool, &court_id, dl_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn list_pending_extensions(court_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::extension_request;

    let pool = get_db().await;
    let rows = extension_request::list_pending(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn rule_on_extension(
    court_id: String,
    id: String,
    status: String,
    ruling_by: String,
    new_deadline_date: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::extension_request;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let new_date = new_deadline_date
        .map(|d| {
            chrono::DateTime::parse_from_rfc3339(&d)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|_| ServerFnError::new("Invalid date format"))
        })
        .transpose()?;
    let row = extension_request::update_ruling(pool, &court_id, uuid, &status, &ruling_by, new_date)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

// ── Deadline Reminder Server Functions ──────────────────

#[server]
pub async fn list_reminders_by_deadline(
    court_id: String,
    deadline_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline_reminder;
    use uuid::Uuid;

    let pool = get_db().await;
    let dl_uuid = Uuid::parse_str(&deadline_id)
        .map_err(|_| ServerFnError::new("Invalid deadline_id UUID"))?;
    let rows = deadline_reminder::list_by_deadline(pool, &court_id, dl_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn list_pending_reminders(court_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline_reminder;

    let pool = get_db().await;
    let rows = deadline_reminder::list_pending(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn send_reminder(
    court_id: String,
    deadline_id: String,
    recipient: String,
    reminder_type: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline_reminder;
    use uuid::Uuid;

    let pool = get_db().await;
    let dl_uuid = Uuid::parse_str(&deadline_id)
        .map_err(|_| ServerFnError::new("Invalid deadline_id UUID"))?;
    let row = deadline_reminder::send(pool, &court_id, dl_uuid, &recipient, &reminder_type)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn acknowledge_reminder(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::deadline_reminder;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = deadline_reminder::acknowledge(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

// ── Judge Server Functions ─────────────────────────────

#[server]
pub async fn list_judges(court_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge;

    let pool = get_db().await;
    let rows = judge::list_by_court(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn search_judges(court_id: String, query: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge;

    let pool = get_db().await;
    let rows = judge::search(pool, &court_id, &query)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn get_judge(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = judge::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn create_judge(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge;

    let pool = get_db().await;
    let req: shared_types::CreateJudgeRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = judge::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn update_judge(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: shared_types::UpdateJudgeRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = judge::update(pool, &court_id, uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_judge(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    judge::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Judge Conflict Server Functions ────────────────────

#[server]
pub async fn list_judge_conflicts(
    court_id: String,
    judge_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge_conflict;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid =
        Uuid::parse_str(&judge_id).map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let rows = judge_conflict::list_by_judge(pool, &court_id, j_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_judge_conflict(
    court_id: String,
    judge_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge_conflict;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid =
        Uuid::parse_str(&judge_id).map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let req: shared_types::CreateJudgeConflictRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = judge_conflict::create(pool, &court_id, j_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_judge_conflict(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge_conflict;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    judge_conflict::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Case Assignment Server Functions ───────────────────

#[server]
pub async fn list_case_assignments(
    court_id: String,
    case_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_assignment;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = case_assignment::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_case_assignment(
    court_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_assignment;

    let pool = get_db().await;
    let req: shared_types::CreateCaseAssignmentRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = case_assignment::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_case_assignment(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_assignment;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    case_assignment::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Recusal Motion Server Functions ────────────────────

#[server]
pub async fn create_recusal(
    court_id: String,
    judge_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::recusal_motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid =
        Uuid::parse_str(&judge_id).map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let req: shared_types::CreateRecusalMotionRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = recusal_motion::create(pool, &court_id, j_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn list_pending_recusals(court_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::recusal_motion;

    let pool = get_db().await;
    let rows = recusal_motion::list_pending(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn rule_on_recusal(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::recusal_motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: shared_types::UpdateRecusalRulingRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = recusal_motion::update_ruling(pool, &court_id, uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

// ── Opinion Server Functions ───────────────────────────

#[server]
pub async fn list_opinions_by_case(
    court_id: String,
    case_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = opinion::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn list_opinions_by_judge(
    court_id: String,
    judge_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid =
        Uuid::parse_str(&judge_id).map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let rows = opinion::list_by_judge(pool, &court_id, j_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn search_opinions(
    court_id: String,
    query: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;

    let pool = get_db().await;
    let rows = opinion::search(pool, &court_id, &query)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn get_opinion(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = opinion::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn create_opinion(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;

    let pool = get_db().await;
    let req: shared_types::CreateJudicialOpinionRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = opinion::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn update_opinion(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: shared_types::UpdateJudicialOpinionRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = opinion::update(pool, &court_id, uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_opinion(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    opinion::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Opinion Draft Server Functions ─────────────────────

#[server]
pub async fn list_opinion_drafts(
    court_id: String,
    opinion_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion_draft;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let rows = opinion_draft::list_by_opinion(pool, &court_id, op_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_opinion_draft(
    court_id: String,
    opinion_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion_draft;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let req: shared_types::CreateOpinionDraftRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = opinion_draft::create(pool, &court_id, op_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn get_current_opinion_draft(
    court_id: String,
    opinion_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion_draft;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let row = opinion_draft::find_current(pool, &court_id, op_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No current draft found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

// ── Opinion Vote Server Functions ──────────────────────

#[server]
pub async fn list_opinion_votes(
    court_id: String,
    opinion_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion_vote;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let rows = opinion_vote::list_by_opinion(pool, &court_id, op_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_opinion_vote(
    court_id: String,
    opinion_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::opinion_vote;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let req: shared_types::CreateOpinionVoteRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = opinion_vote::create(pool, &court_id, op_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

// ── Headnote Server Functions ──────────────────────────

#[server]
pub async fn list_headnotes(
    court_id: String,
    opinion_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::headnote;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let rows = headnote::list_by_opinion(pool, &court_id, op_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_headnote(
    court_id: String,
    opinion_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::headnote;
    use uuid::Uuid;

    let pool = get_db().await;
    let op_uuid =
        Uuid::parse_str(&opinion_id).map_err(|_| ServerFnError::new("Invalid opinion_id UUID"))?;
    let req: shared_types::CreateHeadnoteRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = headnote::create(pool, &court_id, op_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

// ── Victim Server Functions ────────────────────────────

#[server]
pub async fn list_victims_by_case(
    court_id: String,
    case_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::victim;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = victim::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_victim(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::victim;

    let pool = get_db().await;
    let req: shared_types::CreateVictimRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = victim::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn get_victim(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::victim;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = victim::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_victim(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::victim;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    victim::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Representation Server Functions ────────────────────

#[server]
pub async fn list_representations_by_case(
    court_id: String,
    case_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::representation;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = representation::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn list_active_representations(
    court_id: String,
    attorney_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::representation;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = representation::list_active_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn get_representation(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::representation;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = representation::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn create_representation(
    court_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::representation;

    let pool = get_db().await;
    let req: shared_types::CreateRepresentationRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = representation::create(pool, &court_id, &req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn end_representation(
    court_id: String,
    id: String,
    reason: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::representation;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = representation::end_representation(pool, &court_id, uuid, reason.as_deref())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

// ── Charge Server Functions ────────────────────────────

#[server]
pub async fn list_charges_by_defendant(
    court_id: String,
    defendant_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::charge;
    use uuid::Uuid;

    let pool = get_db().await;
    let def_uuid = Uuid::parse_str(&defendant_id)
        .map_err(|_| ServerFnError::new("Invalid defendant_id UUID"))?;
    let rows = charge::list_by_defendant(pool, &court_id, def_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn get_charge(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::charge;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = charge::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn create_charge(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::charge;

    let pool = get_db().await;
    let req: shared_types::CreateChargeRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = charge::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn update_charge(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::charge;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: shared_types::UpdateChargeRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = charge::update(pool, &court_id, uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_charge(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::charge;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    charge::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Motion Server Functions ────────────────────────────

#[server]
pub async fn list_motions_by_case(
    court_id: String,
    case_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = motion::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn get_motion(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = motion::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn create_motion(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::motion;

    let pool = get_db().await;
    let req: shared_types::CreateMotionRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = motion::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn update_motion(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: shared_types::UpdateMotionRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = motion::update(pool, &court_id, uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_motion(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    motion::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Case Note Server Functions ─────────────────────────

#[server]
pub async fn list_case_notes(
    court_id: String,
    case_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_note;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = case_note::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn get_case_note(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_note;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = case_note::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn create_case_note(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_note;

    let pool = get_db().await;
    let req: shared_types::CreateCaseNoteRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = case_note::create(pool, &court_id, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn update_case_note(
    court_id: String,
    id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_note;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let req: shared_types::UpdateCaseNoteRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = case_note::update(pool, &court_id, uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_case_note(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_note;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    case_note::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Rule Server Functions ──────────────────────────────

#[server]
pub async fn list_rules(court_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::rule;

    let pool = get_db().await;
    let rows = rule::list_all(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn get_rule(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::rule;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = rule::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn create_rule(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::rule;

    let pool = get_db().await;
    let v: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = rule::create(
        pool,
        &court_id,
        v["name"].as_str().unwrap_or_default(),
        v["description"].as_str().unwrap_or_default(),
        v["source"].as_str().unwrap_or_default(),
        v["category"].as_str().unwrap_or_default(),
        v["priority"].as_i64().unwrap_or(0) as i32,
        v["status"].as_str().unwrap_or("active"),
        v["jurisdiction"].as_str(),
        v["citation"].as_str(),
        None,
        v.get("conditions").unwrap_or(&serde_json::json!({})),
        v.get("actions").unwrap_or(&serde_json::json!({})),
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_rule(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::rule;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    rule::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Conflict Check Server Functions ────────────────────

#[server]
pub async fn run_conflict_check(
    court_id: String,
    attorney_id: String,
    party_names: Vec<String>,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::conflict_check;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = conflict_check::run_check(pool, &court_id, att_uuid, &party_names)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn list_conflicts_by_attorney(
    court_id: String,
    attorney_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::conflict_check;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = conflict_check::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

// ── Bar Admissions ──────────────────────────────────────────

#[server]
pub async fn list_bar_admissions(
    court_id: String,
    attorney_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::bar_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = bar_admission::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_bar_admission(
    court_id: String,
    attorney_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::bar_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let req: shared_types::CreateBarAdmissionRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = bar_admission::create(pool, &court_id, att_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_bar_admission(
    court_id: String,
    attorney_id: String,
    state: String,
) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::bar_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    bar_admission::delete_by_state(pool, &court_id, att_uuid, &state)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Federal Admissions ──────────────────────────────────────

#[server]
pub async fn list_federal_admissions(
    court_id: String,
    attorney_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::federal_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = federal_admission::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_federal_admission(
    court_id: String,
    attorney_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::federal_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let req: shared_types::CreateFederalAdmissionRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = federal_admission::create(pool, &court_id, att_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn delete_federal_admission(
    court_id: String,
    attorney_id: String,
    court_name: String,
) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::federal_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    federal_admission::delete_by_court_name(pool, &court_id, att_uuid, &court_name)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── CJA Appointments ────────────────────────────────────────

#[server]
pub async fn list_cja_appointments(
    court_id: String,
    attorney_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::cja_appointment;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = cja_appointment::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_cja_appointment(
    court_id: String,
    attorney_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::cja_appointment;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let req: shared_types::CreateCjaAppointmentRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = cja_appointment::create(pool, &court_id, att_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn list_pending_cja_vouchers(court_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::cja_appointment;

    let pool = get_db().await;
    let rows = cja_appointment::list_pending_vouchers(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

// ── Pro Hac Vice ────────────────────────────────────────────

#[server]
pub async fn list_pro_hac_vice(
    court_id: String,
    attorney_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::pro_hac_vice;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = pro_hac_vice::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_pro_hac_vice(
    court_id: String,
    attorney_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::pro_hac_vice;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let req: shared_types::CreateProHacViceRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = pro_hac_vice::create(pool, &court_id, att_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn update_pro_hac_vice_status(
    court_id: String,
    attorney_id: String,
    case_id: String,
    new_status: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::pro_hac_vice;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let row = pro_hac_vice::update_status(pool, &court_id, att_uuid, case_uuid, &new_status)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("PHV record not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

// ── Discipline Records ──────────────────────────────────────

#[server]
pub async fn list_discipline_records(
    court_id: String,
    attorney_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::discipline;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = discipline::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn create_discipline_record(
    court_id: String,
    attorney_id: String,
    body: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::discipline;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let req: shared_types::CreateDisciplineRecordRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = discipline::create(pool, &court_id, att_uuid, req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

// ── Practice Areas ──────────────────────────────────────────

#[server]
pub async fn list_practice_areas(
    court_id: String,
    attorney_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::practice_area;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = practice_area::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn add_practice_area(
    court_id: String,
    attorney_id: String,
    area: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::practice_area;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = practice_area::add(pool, &court_id, att_uuid, &area)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn remove_practice_area(
    court_id: String,
    attorney_id: String,
    area: String,
) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::practice_area;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    practice_area::remove(pool, &court_id, att_uuid, &area)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── ECF Registration ────────────────────────────────────────

#[server]
pub async fn get_ecf_registration(
    court_id: String,
    attorney_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::ecf_registration;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = ecf_registration::find_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn upsert_ecf_registration(
    court_id: String,
    attorney_id: String,
    status: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::ecf_registration;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = ecf_registration::upsert(pool, &court_id, att_uuid, &status)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

#[server]
pub async fn revoke_ecf_registration(
    court_id: String,
    attorney_id: String,
) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::ecf_registration;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    ecf_registration::revoke(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Judge Sub-Domain ────────────────────────────────────────

#[server]
pub async fn list_assignments_by_judge(
    court_id: String,
    judge_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_assignment;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let rows = case_assignment::list_by_judge(pool, &court_id, j_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

#[server]
pub async fn list_recusals_by_judge(
    court_id: String,
    judge_id: String,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::recusal_motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let rows = recusal_motion::list_by_judge(pool, &court_id, j_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

// ── Court Server Functions ────────────────────────────────

#[server]
pub async fn list_courts() -> Result<String, ServerFnError> {
    use crate::db::get_db;

    let pool = get_db().await;
    let rows = sqlx::query_as!(
        shared_types::Court,
        r#"SELECT id, name, court_type, tier, created_at FROM courts ORDER BY name"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

/// Persist the user's preferred court selection to the database.
#[server]
pub async fn set_preferred_court(court_id: String) -> Result<(), ServerFnError> {
    use crate::auth::{cookies, jwt};
    use crate::db::get_db;

    let ctx = dioxus::fullstack::FullstackContext::current()
        .ok_or_else(|| ServerFnError::new("No fullstack context"))?;
    let headers = ctx.parts_mut().headers.clone();
    let token = cookies::extract_access_token(&headers)
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;
    let claims = jwt::validate_access_token(&token)
        .map_err(|_| ServerFnError::new("Invalid token"))?;

    let db = get_db().await;
    sqlx::query!(
        "UPDATE users SET preferred_court_id = $1 WHERE id = $2",
        court_id,
        claims.sub
    )
    .execute(db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}
