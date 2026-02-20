use dioxus::prelude::*;
use shared_types::{DashboardStats, FeatureFlags, MessageResponse, Product, User};

#[cfg(feature = "server")]
use crate::db::get_db;

#[cfg(feature = "server")]
use crate::error_convert::{AppErrorExt, SqlxErrorExt, ValidateRequest};

#[cfg(feature = "server")]
use shared_types::{CreateProductRequest, CreateUserRequest, UpdateProductRequest, UpdateUserRequest, UserTier};

#[cfg(feature = "server")]
use super::auth::*;

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
