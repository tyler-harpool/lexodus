use dioxus::prelude::*;

// ── Court Server Functions ────────────────────────────────

#[server]
pub async fn list_courts() -> Result<Vec<shared_types::Court>, ServerFnError> {
    use crate::db::get_db;

    let pool = get_db().await;
    let rows = sqlx::query_as!(
        shared_types::Court,
        r#"SELECT id, name, court_type, tier, created_at FROM courts ORDER BY name"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows)
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
