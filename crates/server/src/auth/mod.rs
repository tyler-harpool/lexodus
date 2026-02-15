pub mod cookies;
pub mod court_role;
pub mod device_flow;
pub mod extractors;
pub mod jwt;
pub mod middleware;
pub mod oauth;
pub mod oauth_callback;
pub mod oauth_state;
pub mod password;

/// Check if the given email matches the `ADMIN_EMAIL` env var (case-insensitive).
/// Returns `false` if the env var is empty or unset.
pub fn is_admin_email(email: &str) -> bool {
    match std::env::var("ADMIN_EMAIL") {
        Ok(admin) if !admin.is_empty() => admin.eq_ignore_ascii_case(email),
        _ => false,
    }
}

/// If the email matches `ADMIN_EMAIL`, promote the user to admin in the database.
/// Returns the (possibly updated) role string. DB errors are non-fatal — the
/// current role is returned unchanged on failure.
pub async fn maybe_promote_admin(
    db: &sqlx::PgPool,
    user_id: i64,
    email: &str,
    current_role: String,
) -> String {
    if !is_admin_email(email) || current_role == "admin" {
        return current_role;
    }

    match sqlx::query!("UPDATE users SET role = 'admin' WHERE id = $1", user_id)
        .execute(db)
        .await
    {
        Ok(_) => {
            tracing::info!(
                user_id,
                email,
                "Auto-promoted user to admin via ADMIN_EMAIL"
            );
            "admin".to_string()
        }
        Err(e) => {
            tracing::error!(user_id, email, %e, "Failed to auto-promote admin");
            current_role
        }
    }
}
