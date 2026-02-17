use axum::{
    body::Bytes,
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post, put},
    Json, Router,
};
use shared_types::{
    AppError, ApproveDeviceRequest, AuthResponse, AuthUser, CheckoutRequest, CheckoutResponse,
    CreateProductRequest, CreateUserRequest, DashboardStats, DeviceFlowInitResponse,
    DeviceFlowPollResponse, ForgotPasswordRequest, InitiateDeviceRequest, LoginRequest,
    MessageResponse, PollDeviceRequest, Product, RegisterRequest, ResetPasswordRequest,
    SendPhoneVerificationRequest, SubscriptionStatus, UpdateProductRequest, UpdateTierRequest,
    UpdateUserRequest, User, UserTier, VerifyPhoneRequest,
};
use sqlx::{Pool, Postgres};

use crate::auth::{extractors::AuthRequired, jwt, password as pw};
use crate::db::AppState;
use crate::error_convert::{SqlxErrorExt, ValidateRequest};

// ── Users ──────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/users",
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "users"
)]
#[tracing::instrument(skip(pool))]
pub async fn list_users(State(pool): State<Pool<Postgres>>) -> Result<Json<Vec<User>>, AppError> {
    let users = sqlx::query_as!(
        User,
        "SELECT id, username, display_name, role, tier FROM users"
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;
    Ok(Json(users))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}",
    params(("user_id" = i64, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "users"
)]
#[tracing::instrument(skip(pool))]
pub async fn get_user(
    State(pool): State<Pool<Postgres>>,
    Path(user_id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, username, display_name, role, tier FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found(format!("User with id {} not found", user_id)))?;
    Ok(Json(user))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = User),
        (status = 422, description = "Validation error", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "users"
)]
#[tracing::instrument(skip(pool))]
pub async fn create_user(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    payload.validate_request()?;

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username, display_name) VALUES ($1, $2) RETURNING id, username, display_name, role, tier",
        payload.username,
        payload.display_name
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;
    Ok((StatusCode::CREATED, Json(user)))
}

#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}",
    params(("user_id" = i64, Path, description = "User ID")),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = User),
        (status = 404, description = "User not found", body = AppError),
        (status = 422, description = "Validation error", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "users"
)]
#[tracing::instrument(skip(pool))]
pub async fn update_user(
    State(pool): State<Pool<Postgres>>,
    Path(user_id): Path<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<User>, AppError> {
    payload.validate_request()?;

    let user = sqlx::query_as!(
        User,
        "UPDATE users SET username = $2, display_name = $3 WHERE id = $1 RETURNING id, username, display_name, role, tier",
        user_id,
        payload.username,
        payload.display_name
    )
    .fetch_optional(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found(format!("User with id {} not found", user_id)))?;
    Ok(Json(user))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}",
    params(("user_id" = i64, Path, description = "User ID")),
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "users"
)]
#[tracing::instrument(skip(pool))]
pub async fn delete_user(
    State(pool): State<Pool<Postgres>>,
    Path(user_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?;
    if result.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "User with id {} not found",
            user_id
        )))
    }
}

// ── Products ───────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/products",
    responses(
        (status = 200, description = "List of products", body = Vec<Product>),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "products"
)]
#[tracing::instrument(skip(pool))]
pub async fn list_products(
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<Product>>, AppError> {
    let rows = sqlx::query!(
        "SELECT id, name, description, price, category, status, created_at FROM products ORDER BY id DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let products: Vec<Product> = rows
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
    Ok(Json(products))
}

#[utoipa::path(
    post,
    path = "/api/v1/products",
    request_body = CreateProductRequest,
    responses(
        (status = 201, description = "Product created", body = Product),
        (status = 422, description = "Validation error", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "products"
)]
#[tracing::instrument(skip(pool))]
pub async fn create_product(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<Product>), AppError> {
    payload.validate_request()?;

    let row = sqlx::query!(
        "INSERT INTO products (name, description, price, category, status) VALUES ($1, $2, $3, $4, $5) RETURNING id, name, description, price, category, status, created_at",
        payload.name,
        payload.description,
        payload.price,
        payload.category,
        payload.status
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let product = Product {
        id: row.id,
        name: row.name,
        description: row.description,
        price: row.price,
        category: row.category,
        status: row.status,
        created_at: row.created_at.to_string(),
    };
    Ok((StatusCode::CREATED, Json(product)))
}

#[utoipa::path(
    put,
    path = "/api/v1/products/{product_id}",
    params(("product_id" = i64, Path, description = "Product ID")),
    request_body = UpdateProductRequest,
    responses(
        (status = 200, description = "Product updated", body = Product),
        (status = 404, description = "Product not found", body = AppError),
        (status = 422, description = "Validation error", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "products"
)]
#[tracing::instrument(skip(pool))]
pub async fn update_product(
    State(pool): State<Pool<Postgres>>,
    Path(product_id): Path<i64>,
    Json(payload): Json<UpdateProductRequest>,
) -> Result<Json<Product>, AppError> {
    payload.validate_request()?;

    let row = sqlx::query!(
        "UPDATE products SET name = $2, description = $3, price = $4, category = $5, status = $6 WHERE id = $1 RETURNING id, name, description, price, category, status, created_at",
        product_id,
        payload.name,
        payload.description,
        payload.price,
        payload.category,
        payload.status
    )
    .fetch_optional(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?
    .ok_or_else(|| {
        AppError::not_found(format!("Product with id {} not found", product_id))
    })?;

    let product = Product {
        id: row.id,
        name: row.name,
        description: row.description,
        price: row.price,
        category: row.category,
        status: row.status,
        created_at: row.created_at.to_string(),
    };
    Ok(Json(product))
}

#[utoipa::path(
    delete,
    path = "/api/v1/products/{product_id}",
    params(("product_id" = i64, Path, description = "Product ID")),
    responses(
        (status = 204, description = "Product deleted"),
        (status = 404, description = "Product not found", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "products"
)]
#[tracing::instrument(skip(pool))]
pub async fn delete_product(
    State(pool): State<Pool<Postgres>>,
    Path(product_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query!("DELETE FROM products WHERE id = $1", product_id)
        .execute(&pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?;
    if result.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "Product with id {} not found",
            product_id
        )))
    }
}

// ── Dashboard ──────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/stats",
    responses(
        (status = 200, description = "Dashboard statistics", body = DashboardStats),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "dashboard"
)]
#[tracing::instrument(skip(pool))]
pub async fn get_dashboard_stats(
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<DashboardStats>, AppError> {
    let total_users = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?
        .unwrap_or(0);

    let total_products = sqlx::query_scalar!("SELECT COUNT(*) FROM products")
        .fetch_one(&pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?
        .unwrap_or(0);

    let active_products =
        sqlx::query_scalar!("SELECT COUNT(*) FROM products WHERE status = 'active'")
            .fetch_one(&pool)
            .await
            .map_err(SqlxErrorExt::into_app_error)?
            .unwrap_or(0);

    let recent_users = sqlx::query_as!(
        User,
        "SELECT id, username, display_name, role, tier FROM users ORDER BY id DESC LIMIT 5"
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(Json(DashboardStats {
        total_users,
        total_products,
        active_products,
        recent_users,
    }))
}

// ── Auth ───────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered", body = AuthResponse),
        (status = 422, description = "Validation error (e.g. duplicate email)", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "auth"
)]
#[tracing::instrument(skip(pool, payload))]
pub async fn register(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    let password_hash =
        pw::hash_password(&payload.password).map_err(|e| AppError::internal(e.to_string()))?;

    let user = sqlx::query!(
        r#"INSERT INTO users (username, email, password_hash, display_name)
           VALUES ($1, $2, $3, $4)
           RETURNING id, username, display_name, email, role, tier, avatar_url,
                     email_verified, phone_number, phone_verified,
                     email_notifications_enabled, push_notifications_enabled, weekly_digest_enabled"#,
        payload.username,
        payload.email,
        password_hash,
        payload.display_name
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let user_email = user.email.unwrap_or_default();
    let user_role = crate::auth::maybe_promote_admin(&pool, user.id, &user_email, user.role).await;
    let user_tier = UserTier::from_str_or_default(&user.tier);

    // New users have no court roles yet
    let court_roles = std::collections::HashMap::<String, String>::new();

    let access_token =
        jwt::create_access_token(user.id, &user_email, &user_role, user_tier.as_str(), &court_roles)
            .map_err(|e| AppError::internal(e.to_string()))?;

    let (refresh_token, expires_at) =
        jwt::create_refresh_token(user.id, &user_email, &user_role, user_tier.as_str(), &court_roles)
            .map_err(|e| AppError::internal(e.to_string()))?;

    // Store the hash of the refresh token — never persist raw JWTs
    let refresh_hash = jwt::hash_token(&refresh_token);
    sqlx::query!(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        user.id,
        refresh_hash,
        expires_at
    )
    .execute(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    // Fire-and-forget: send welcome + verification emails
    let pool_clone = pool.clone();
    let email_clone = user_email.clone();
    let name_clone = user.display_name.clone();
    let uid = user.id;
    tokio::spawn(async move {
        crate::mailgun::send_welcome_email(&email_clone, &name_clone).await;
        if let Ok(token) = crate::mailgun::create_verification_token(&pool_clone, uid).await {
            crate::mailgun::send_verification_email(&email_clone, &token).await;
        }
    });

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
        has_password: true,
        court_roles,
        court_tiers: Default::default(),
        preferred_court_id: None,
    };

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            user: auth_user,
            access_token,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "auth"
)]
#[tracing::instrument(skip(pool, payload))]
pub async fn login(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = sqlx::query!(
        r#"SELECT id, username, display_name, email, password_hash, role, tier, avatar_url,
                  email_verified, phone_number, phone_verified, oauth_provider,
                  email_notifications_enabled, push_notifications_enabled, weekly_digest_enabled
           FROM users WHERE email = $1"#,
        payload.email
    )
    .fetch_optional(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::unauthorized("Invalid email or password"))?;

    let password_hash = match user.password_hash {
        Some(ref hash) => hash.clone(),
        None => {
            let provider = user
                .oauth_provider
                .as_deref()
                .unwrap_or("a social provider");
            let msg = format!(
                "This account uses {} sign-in. Please use that to log in.",
                provider.to_uppercase()
            );
            return Err(AppError::unauthorized(msg));
        }
    };

    let valid = pw::verify_password(&payload.password, &password_hash)
        .map_err(|e| AppError::internal(e.to_string()))?;

    if !valid {
        return Err(AppError::unauthorized("Invalid email or password"));
    }

    let user_email = user.email.unwrap_or_default();
    let user_role = crate::auth::maybe_promote_admin(&pool, user.id, &user_email, user.role).await;
    let user_tier = UserTier::from_str_or_default(&user.tier);

    let court_roles: std::collections::HashMap<String, String> = sqlx::query_scalar!(
        "SELECT court_roles FROM users WHERE id = $1",
        user.id
    )
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten()
    .and_then(|v| serde_json::from_value(v).ok())
    .unwrap_or_default();

    let access_token =
        jwt::create_access_token(user.id, &user_email, &user_role, user_tier.as_str(), &court_roles)
            .map_err(|e| AppError::internal(e.to_string()))?;

    let (refresh_token, expires_at) =
        jwt::create_refresh_token(user.id, &user_email, &user_role, user_tier.as_str(), &court_roles)
            .map_err(|e| AppError::internal(e.to_string()))?;

    // Store the hash of the refresh token — never persist raw JWTs
    let refresh_hash = jwt::hash_token(&refresh_token);
    sqlx::query!(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        user.id,
        refresh_hash,
        expires_at
    )
    .execute(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    // Fire-and-forget: security alert SMS on login if phone is verified
    if user.phone_verified {
        let pool_clone = pool.clone();
        let uid = user.id;
        tokio::spawn(async move {
            crate::twilio::send_security_alert(
                &pool_clone,
                uid,
                "New login detected on your account.",
            )
            .await;
        });
    }

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
        has_password: true,
        court_roles,
        court_tiers: Default::default(),
        preferred_court_id: None,
    };

    Ok(Json(AuthResponse {
        user: auth_user,
        access_token,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 204, description = "Logged out"),
        (status = 401, description = "Not authenticated", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "auth",
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip(pool, auth))]
pub async fn logout(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
) -> Result<StatusCode, AppError> {
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked = TRUE WHERE user_id = $1 AND revoked = FALSE",
        auth.0.sub
    )
    .execute(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}/tier",
    params(("user_id" = i64, Path, description = "User ID")),
    request_body = UpdateTierRequest,
    responses(
        (status = 200, description = "Tier updated", body = User),
        (status = 401, description = "Not authenticated", body = AppError),
        (status = 403, description = "Forbidden — admin role required", body = AppError),
        (status = 404, description = "User not found", body = AppError),
        (status = 422, description = "Invalid tier value", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "users",
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip(pool, auth))]
pub async fn update_user_tier(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
    Path(user_id): Path<i64>,
    Json(payload): Json<UpdateTierRequest>,
) -> Result<Json<User>, AppError> {
    if auth.0.role != "admin" {
        return Err(AppError::forbidden(
            "Admin role required to change user tiers",
        ));
    }

    let valid_tiers = ["free", "pro", "enterprise"];
    let tier_lower = payload.tier.to_lowercase();
    if !valid_tiers.contains(&tier_lower.as_str()) {
        return Err(AppError::validation(
            "Invalid tier value",
            Default::default(),
        ));
    }

    let user = sqlx::query_as!(
        User,
        "UPDATE users SET tier = $2 WHERE id = $1 RETURNING id, username, display_name, role, tier",
        user_id,
        tier_lower
    )
    .fetch_optional(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found(format!("User with id {} not found", user_id)))?;

    Ok(Json(user))
}

// ── Avatar Upload ───────────────────────────────────────

const MAX_AVATAR_SIZE: usize = 2 * 1024 * 1024; // 2 MB

#[utoipa::path(
    post,
    path = "/api/v1/users/me/avatar",
    responses(
        (status = 200, description = "Avatar uploaded", body = AuthUser),
        (status = 401, description = "Not authenticated", body = AppError),
        (status = 422, description = "Validation error", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "users",
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip(pool, auth, multipart))]
pub async fn upload_avatar(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
    mut multipart: Multipart,
) -> Result<Json<AuthUser>, AppError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::validation(e.to_string(), Default::default()))?
    {
        let ct = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let allowed = ["image/jpeg", "image/png", "image/webp"];
        if !allowed.contains(&ct.as_str()) {
            return Err(AppError::validation(
                "Only JPEG, PNG, and WebP images are allowed",
                Default::default(),
            ));
        }

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::internal(e.to_string()))?;

        if data.len() > MAX_AVATAR_SIZE {
            return Err(AppError::validation(
                "Avatar must be under 2 MB",
                Default::default(),
            ));
        }

        content_type = Some(ct);
        file_bytes = Some(data.to_vec());
        break;
    }

    let bytes =
        file_bytes.ok_or_else(|| AppError::validation("No file provided", Default::default()))?;
    let ct = content_type.unwrap_or_default();

    let avatar_url = crate::s3::upload_avatar(auth.0.sub, &ct, &bytes)
        .await
        .map_err(|e| AppError::internal(e))?;

    let user = sqlx::query!(
        r#"UPDATE users SET avatar_url = $2 WHERE id = $1
           RETURNING id, username, display_name, email, password_hash, role, tier, avatar_url,
                     email_verified, phone_number, phone_verified,
                     email_notifications_enabled, push_notifications_enabled, weekly_digest_enabled"#,
        auth.0.sub,
        avatar_url
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        use crate::error_convert::SqlxErrorExt;
        e.into_app_error()
    })?;

    Ok(Json(AuthUser {
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
        court_roles: std::collections::HashMap::new(),
        court_tiers: Default::default(),
        preferred_court_id: None,
    }))
}

// ── Billing ──────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/billing/checkout",
    request_body = CheckoutRequest,
    responses(
        (status = 200, description = "Checkout session created", body = CheckoutResponse),
        (status = 401, description = "Not authenticated", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "billing",
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip(pool, auth))]
pub async fn create_checkout(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
    Json(payload): Json<CheckoutRequest>,
) -> Result<Json<CheckoutResponse>, AppError> {
    let user_email = auth.0.email.clone();

    let url = match payload.checkout_type.as_str() {
        "subscription" => {
            let tier = payload.tier.as_deref().ok_or_else(|| {
                AppError::validation("Tier is required for subscriptions", Default::default())
            })?;
            let cid = payload.court_id.as_deref().unwrap_or("");
            crate::stripe::checkout::create_subscription_checkout(
                &pool,
                auth.0.sub,
                &user_email,
                tier,
                cid,
            )
            .await
            .map_err(|e| AppError::internal(e))?
        }
        "onetime" => {
            let price_cents = payload.price_cents.ok_or_else(|| {
                AppError::validation(
                    "price_cents is required for one-time payments",
                    Default::default(),
                )
            })?;
            let name = payload
                .product_name
                .as_deref()
                .unwrap_or("One-time purchase");
            let desc = payload.product_description.as_deref().unwrap_or("");
            crate::stripe::checkout::create_onetime_checkout(
                &pool,
                auth.0.sub,
                &user_email,
                price_cents,
                name,
                desc,
            )
            .await
            .map_err(|e| AppError::internal(e))?
        }
        _ => {
            return Err(AppError::validation(
                "checkout_type must be 'subscription' or 'onetime'",
                Default::default(),
            ));
        }
    };

    Ok(Json(CheckoutResponse { url }))
}

#[utoipa::path(
    post,
    path = "/api/v1/billing/portal",
    responses(
        (status = 200, description = "Portal session created", body = CheckoutResponse),
        (status = 401, description = "Not authenticated", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "billing",
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip(pool, auth))]
pub async fn create_portal(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
) -> Result<Json<CheckoutResponse>, AppError> {
    let url = crate::stripe::portal::create_portal_session(&pool, auth.0.sub)
        .await
        .map_err(|e| AppError::internal(e))?;

    Ok(Json(CheckoutResponse { url }))
}

#[utoipa::path(
    get,
    path = "/api/v1/billing/subscription",
    responses(
        (status = 200, description = "Current subscription status", body = SubscriptionStatus),
        (status = 401, description = "Not authenticated", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "billing",
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip(pool, auth))]
pub async fn get_subscription(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
) -> Result<Json<SubscriptionStatus>, AppError> {
    let sub = sqlx::query!(
        r#"SELECT status, stripe_price_id, current_period_end, cancel_at_period_end
           FROM subscriptions WHERE user_id = $1
           ORDER BY created_at DESC LIMIT 1"#,
        auth.0.sub
    )
    .fetch_optional(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let status = match sub {
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
    };

    Ok(Json(status))
}

#[utoipa::path(
    post,
    path = "/api/v1/billing/cancel",
    responses(
        (status = 200, description = "Subscription canceled", body = MessageResponse),
        (status = 401, description = "Not authenticated", body = AppError),
        (status = 404, description = "No active subscription", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "billing",
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip(pool, auth))]
pub async fn cancel_subscription(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
) -> Result<Json<MessageResponse>, AppError> {
    crate::stripe::checkout::cancel_subscription(&pool, auth.0.sub)
        .await
        .map_err(|e| AppError::internal(e))?;

    Ok(Json(MessageResponse {
        message: "Subscription canceled successfully".to_string(),
    }))
}

// ── Webhooks ─────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/webhooks/stripe",
    request_body(content = String, description = "Raw webhook payload"),
    responses(
        (status = 200, description = "Webhook processed")
    ),
    tag = "webhooks"
)]
pub async fn stripe_webhook(
    State(pool): State<Pool<Postgres>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    crate::stripe::webhooks::handle_stripe_webhook(axum::extract::State(pool), headers, body).await
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct MailgunWebhookPayload {
    pub timestamp: String,
    pub token: String,
    pub signature: String,
    pub event: Option<String>,
    pub recipient: Option<String>,
}

#[utoipa::path(
    post,
    path = "/webhooks/mailgun",
    responses(
        (status = 200, description = "Webhook processed")
    ),
    tag = "webhooks"
)]
#[tracing::instrument(skip(pool, payload))]
pub async fn mailgun_webhook(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<MailgunWebhookPayload>,
) -> StatusCode {
    let signing_key = std::env::var("MAILGUN_WEBHOOK_SIGNING_KEY").unwrap_or_default();

    if !crate::mailgun::verify_webhook_signature(
        &signing_key,
        &payload.timestamp,
        &payload.token,
        &payload.signature,
    ) {
        tracing::warn!("Invalid Mailgun webhook signature");
        return StatusCode::OK;
    }

    if let Some(recipient) = &payload.recipient {
        if let Some(event) = &payload.event {
            if event == "bounced" || event == "complained" {
                crate::mailgun::handle_bounce_event(&pool, recipient).await;
            }
        }
    }

    StatusCode::OK
}

// ── Email Verification & Password Reset ──────────────

#[derive(Debug, serde::Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/verify-email",
    params(("token" = String, Query, description = "Verification token")),
    responses(
        (status = 302, description = "Redirect to settings on success"),
        (status = 302, description = "Redirect to settings on failure")
    ),
    tag = "auth"
)]
#[tracing::instrument(skip(pool))]
pub async fn verify_email(
    State(pool): State<Pool<Postgres>>,
    Query(query): Query<VerifyEmailQuery>,
) -> axum::response::Redirect {
    let base = crate::stripe::client::app_base_url();
    match crate::mailgun::verify_email_token(&pool, &query.token).await {
        Ok(_) => axum::response::Redirect::to(&format!("{}/settings/?verified=success", base)),
        Err(e) => {
            tracing::warn!(error = %e, "Email verification failed");
            axum::response::Redirect::to(&format!("{}/settings/?verified=failed", base))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Reset email sent (always returns success)", body = MessageResponse)
    ),
    tag = "auth"
)]
#[tracing::instrument(skip(pool, payload))]
pub async fn forgot_password(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Json<MessageResponse> {
    // Always return success to prevent email enumeration
    let pool_clone = pool.clone();
    let email = payload.email.clone();
    tokio::spawn(async move {
        // Check if user exists
        let user_exists = sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE email = $1", email)
            .fetch_one(&pool_clone)
            .await
            .unwrap_or(Some(0))
            .unwrap_or(0);

        if user_exists > 0 {
            if let Ok(token) =
                crate::mailgun::create_password_reset_token(&pool_clone, &email).await
            {
                crate::mailgun::send_password_reset_email(&email, &token).await;
            }
        }
    });

    Json(MessageResponse {
        message: "If an account with that email exists, a password reset link has been sent."
            .to_string(),
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful", body = MessageResponse),
        (status = 400, description = "Invalid or expired token", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "auth"
)]
#[tracing::instrument(skip(pool, payload))]
pub async fn reset_password(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    payload.validate_request()?;

    let email = crate::mailgun::validate_password_reset_token(&pool, &payload.token)
        .await
        .map_err(|e| AppError::validation(e, Default::default()))?;

    let password_hash =
        pw::hash_password(&payload.new_password).map_err(|e| AppError::internal(e.to_string()))?;

    sqlx::query!(
        "UPDATE users SET password_hash = $2 WHERE email = $1",
        email,
        password_hash
    )
    .execute(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    // Revoke all refresh tokens for security
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked = TRUE WHERE user_id = (SELECT id FROM users WHERE email = $1) AND revoked = FALSE",
        email
    )
    .execute(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    // Fire-and-forget: send security alert if phone verified
    let pool_clone = pool.clone();
    let email_clone = email.clone();
    tokio::spawn(async move {
        let user_id = sqlx::query_scalar!("SELECT id FROM users WHERE email = $1", email_clone)
            .fetch_optional(&pool_clone)
            .await
            .ok()
            .flatten();

        if let Some(uid) = user_id {
            crate::twilio::send_security_alert(
                &pool_clone,
                uid,
                "Your password was recently changed.",
            )
            .await;
        }
    });

    Ok(Json(MessageResponse {
        message: "Password reset successfully. Please log in with your new password.".to_string(),
    }))
}

// ── Phone Verification ───────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/account/send-verification",
    request_body = SendPhoneVerificationRequest,
    responses(
        (status = 200, description = "Verification code sent", body = MessageResponse),
        (status = 401, description = "Not authenticated", body = AppError),
        (status = 422, description = "Validation error", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "account",
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip(pool, auth))]
pub async fn send_phone_verification(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
    Json(payload): Json<SendPhoneVerificationRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    payload.validate_request()?;

    crate::twilio::send_verification_code(&pool, auth.0.sub, &payload.phone_number).await?;

    Ok(Json(MessageResponse {
        message: "Verification code sent.".to_string(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/account/verify-phone",
    request_body = VerifyPhoneRequest,
    responses(
        (status = 200, description = "Phone verified", body = AuthUser),
        (status = 401, description = "Not authenticated", body = AppError),
        (status = 422, description = "Invalid code", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "account",
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip(pool, auth, payload))]
pub async fn verify_phone(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
    Json(payload): Json<VerifyPhoneRequest>,
) -> Result<Json<AuthUser>, AppError> {
    payload.validate_request()?;

    crate::twilio::verify_code(&pool, auth.0.sub, &payload.phone_number, &payload.code).await?;

    let user = sqlx::query!(
        r#"SELECT id, username, display_name, email, password_hash, role, tier, avatar_url,
                  email_verified, phone_number, phone_verified,
                  email_notifications_enabled, push_notifications_enabled, weekly_digest_enabled
           FROM users WHERE id = $1"#,
        auth.0.sub
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(Json(AuthUser {
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
        court_roles: std::collections::HashMap::new(),
        court_tiers: Default::default(),
        preferred_court_id: None,
    }))
}

// ── Device Authorization Flow (RFC 8628) ─────────────

#[utoipa::path(
    post,
    path = "/api/v1/auth/device/initiate",
    request_body = InitiateDeviceRequest,
    responses(
        (status = 200, description = "Device authorization initiated", body = DeviceFlowInitResponse),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "auth"
)]
#[tracing::instrument(skip(pool))]
pub async fn initiate_device(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<InitiateDeviceRequest>,
) -> Result<Json<DeviceFlowInitResponse>, AppError> {
    use crate::auth::device_flow;

    let base_url = crate::stripe::client::app_base_url();

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
            payload.client_platform,
            expires_at
        )
        .execute(&pool)
        .await;

        match result {
            Ok(_) => {
                let verification_uri = format!("{}/activate", base_url);
                let verification_uri_complete =
                    Some(format!("{}/activate?code={}", base_url, user_code));

                return Ok(Json(DeviceFlowInitResponse {
                    device_code: raw_device_code,
                    user_code,
                    verification_uri,
                    verification_uri_complete,
                    expires_in: device_flow::DEVICE_CODE_EXPIRY_SECONDS,
                    interval: device_flow::DEVICE_POLL_INTERVAL_SECONDS,
                }));
            }
            Err(e) => {
                if e.as_database_error()
                    .is_some_and(|db_err| db_err.is_unique_violation())
                {
                    attempts += 1;
                    if attempts >= 5 {
                        return Err(AppError::internal(
                            "Failed to generate unique device code after retries",
                        ));
                    }
                    continue;
                }
                return Err(e.into_app_error());
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/device/poll",
    request_body = PollDeviceRequest,
    responses(
        (status = 200, description = "Device authorization status", body = DeviceFlowPollResponse),
        (status = 404, description = "Invalid device code", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "auth"
)]
#[tracing::instrument(skip(pool, payload))]
pub async fn poll_device(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<PollDeviceRequest>,
) -> Result<Json<DeviceFlowPollResponse>, AppError> {
    use crate::auth::{device_flow, jwt};
    use shared_types::DeviceAuthStatus;
    use std::time::Duration;

    let code_hash = device_flow::hash_device_code(&payload.device_code);

    let row = sqlx::query!(
        r#"SELECT id, user_id, status, expires_at
           FROM device_authorizations
           WHERE device_code = $1"#,
        code_hash
    )
    .fetch_optional(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found("Invalid device code"))?;

    // Check if expired
    if row.expires_at < chrono::Utc::now() {
        if row.status == "pending" {
            let _ = sqlx::query!(
                "UPDATE device_authorizations SET status = 'expired' WHERE id = $1",
                row.id
            )
            .execute(&pool)
            .await;
        }
        return Ok(Json(DeviceFlowPollResponse {
            status: DeviceAuthStatus::Expired,
            user: None,
        }));
    }

    match row.status.as_str() {
        "approved" => {
            let user_id = row
                .user_id
                .ok_or_else(|| AppError::internal("Approved device auth missing user_id"))?;

            let user = sqlx::query!(
                r#"SELECT id, username, display_name, email, password_hash, role, tier, avatar_url,
                          email_verified, phone_number, phone_verified,
                          email_notifications_enabled, push_notifications_enabled, weekly_digest_enabled
                   FROM users WHERE id = $1"#,
                user_id
            )
            .fetch_optional(&pool)
            .await
            .map_err(SqlxErrorExt::into_app_error)?
            .ok_or_else(|| AppError::not_found("User not found"))?;

            let user_email = user.email.unwrap_or_default();
            let user_tier = UserTier::from_str_or_default(&user.tier);

            let court_roles: std::collections::HashMap<String, String> = sqlx::query_scalar!(
                "SELECT court_roles FROM users WHERE id = $1",
                user.id
            )
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

            let _access_token =
                jwt::create_access_token(user.id, &user_email, &user.role, user_tier.as_str(), &court_roles)
                    .map_err(|e| AppError::internal(e.to_string()))?;

            let (_refresh_token, refresh_expires_at) =
                jwt::create_refresh_token(user.id, &user_email, &user.role, user_tier.as_str(), &court_roles)
                    .map_err(|e| AppError::internal(e.to_string()))?;

            let refresh_hash = jwt::hash_token(&_refresh_token);
            sqlx::query!(
                "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
                user.id,
                refresh_hash,
                refresh_expires_at
            )
            .execute(&pool)
            .await
            .map_err(SqlxErrorExt::into_app_error)?;

            // Delete the device auth row (one-time use)
            let _ = sqlx::query!("DELETE FROM device_authorizations WHERE id = $1", row.id)
                .execute(&pool)
                .await;

            let auth_user = AuthUser {
                id: user.id,
                username: user.username,
                display_name: user.display_name,
                email: user_email,
                role: user.role,
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

            Ok(Json(DeviceFlowPollResponse {
                status: DeviceAuthStatus::Approved,
                user: Some(auth_user),
            }))
        }
        "pending" => {
            tokio::time::sleep(Duration::from_secs(
                device_flow::DEVICE_POLL_INTERVAL_SECONDS as u64,
            ))
            .await;

            Ok(Json(DeviceFlowPollResponse {
                status: DeviceAuthStatus::Pending,
                user: None,
            }))
        }
        _ => Ok(Json(DeviceFlowPollResponse {
            status: DeviceAuthStatus::Expired,
            user: None,
        })),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/device/approve",
    request_body = ApproveDeviceRequest,
    responses(
        (status = 200, description = "Device authorized", body = MessageResponse),
        (status = 401, description = "Not authenticated", body = AppError),
        (status = 404, description = "Code not found", body = AppError),
        (status = 422, description = "Invalid or expired code", body = AppError),
        (status = 500, description = "Internal server error", body = AppError)
    ),
    tag = "auth",
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip(pool, auth))]
pub async fn approve_device(
    State(pool): State<Pool<Postgres>>,
    auth: AuthRequired,
    Json(payload): Json<ApproveDeviceRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    use crate::auth::device_flow;

    let normalized = device_flow::normalize_user_code(&payload.user_code).ok_or_else(|| {
        AppError::validation(
            "Invalid code format. Expected 8 characters like ABCD-EFGH.",
            Default::default(),
        )
    })?;

    let row = sqlx::query!(
        r#"SELECT id, status, expires_at
           FROM device_authorizations
           WHERE user_code = $1"#,
        normalized
    )
    .fetch_optional(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?
    .ok_or_else(|| AppError::not_found("Code not found. It may have expired."))?;

    if row.expires_at < chrono::Utc::now() {
        let _ = sqlx::query!(
            "UPDATE device_authorizations SET status = 'expired' WHERE id = $1",
            row.id
        )
        .execute(&pool)
        .await;
        return Err(AppError::validation(
            "This code has expired. Please try again on your device.",
            Default::default(),
        ));
    }

    if row.status != "pending" {
        return Err(AppError::validation(
            "This code has already been used or expired.",
            Default::default(),
        ));
    }

    sqlx::query!(
        r#"UPDATE device_authorizations
           SET status = 'approved', user_id = $2, approved_at = NOW()
           WHERE id = $1"#,
        row.id,
        auth.0.sub
    )
    .execute(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(Json(MessageResponse {
        message: "Device authorized successfully. You can close this page.".to_string(),
    }))
}

/// All v1 API routes (paths are relative — no /api prefix).
fn api_v1_routes() -> Router<AppState> {
    Router::new()
        // Users
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/{user_id}",
            get(get_user).put(update_user).delete(delete_user),
        )
        .route("/users/{user_id}/tier", put(update_user_tier))
        .route("/users/me/avatar", post(upload_avatar))
        // Products
        .route("/products", get(list_products).post(create_product))
        .route(
            "/products/{product_id}",
            put(update_product).delete(delete_product),
        )
        // Dashboard
        .route("/dashboard/stats", get(get_dashboard_stats))
        // Auth
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/verify-email", get(verify_email))
        .route("/auth/forgot-password", post(forgot_password))
        .route("/auth/reset-password", post(reset_password))
        // Device Authorization Flow
        .route("/auth/device/initiate", post(initiate_device))
        .route("/auth/device/poll", post(poll_device))
        .route("/auth/device/approve", post(approve_device))
        // Billing
        .route("/billing/checkout", post(create_checkout))
        .route("/billing/portal", post(create_portal))
        .route("/billing/subscription", get(get_subscription))
        .route("/billing/cancel", post(cancel_subscription))
        // Phone verification
        .route("/account/send-verification", post(send_phone_verification))
        .route("/account/verify-phone", post(verify_phone))
}

/// Build the REST API router with all resource routes.
pub fn rest_router() -> Router<AppState> {
    let mut router = Router::new().nest("/api/v1", api_v1_routes());

    // Backward-compat: unversioned /api/* alias (controlled by env var)
    if std::env::var("API_ENABLE_UNVERSIONED")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true)
    {
        router = router.nest("/api", api_v1_routes());
    }

    // Webhooks stay unversioned — external services (Stripe, Mailgun) are configured once
    router
        .route("/webhooks/stripe", post(stripe_webhook))
        .route("/webhooks/mailgun", post(mailgun_webhook))
}
