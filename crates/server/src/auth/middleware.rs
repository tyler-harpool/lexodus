use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::Response;
use sqlx::{Pool, Postgres};

use super::cookies::{self, CookieSlot, PendingCookieAction, RedirectSlot};
use super::jwt::{self, hash_token, validate_access_token, validate_refresh_token};

/// Permissive auth middleware that handles authentication and cookie management.
///
/// On each request:
/// 1. Validates the access token from cookies (or Bearer header fallback)
/// 2. If expired, attempts transparent refresh using the refresh cookie
/// 3. Inserts a `CookieSlot` so server functions can schedule cookie changes
/// 4. After the handler runs, applies any pending cookie actions to the response
///
/// Does NOT reject unauthenticated requests — downstream handlers decide authorization.
pub async fn auth_middleware(
    State(pool): State<Pool<Postgres>>,
    mut req: Request,
    next: Next,
) -> Response {
    let headers = req.headers().clone();
    let mut refresh_cookies: Option<(String, String)> = None;

    // Validate access token and insert Claims into extensions
    let access_token = cookies::extract_access_token(&headers);
    let mut needs_refresh = access_token.is_none();

    if let Some(token) = access_token {
        match validate_access_token(&token) {
            Ok(claims) => {
                req.extensions_mut().insert(claims);
            }
            Err(_) => {
                needs_refresh = true;
            }
        }
    }

    // Transparent refresh: access token missing (cookie expired) or invalid
    if needs_refresh {
        if let Some(refresh_token) = cookies::extract_refresh_token(&headers) {
            if let Some((new_access, new_refresh)) =
                try_transparent_refresh(&pool, &refresh_token, &mut req).await
            {
                refresh_cookies = Some((new_access, new_refresh));
            }
        }
    }

    // Insert slots so server functions can schedule cookie changes
    let cookie_slot = CookieSlot::default();
    let redirect_slot = RedirectSlot::default();
    req.extensions_mut().insert(cookie_slot.clone());
    req.extensions_mut().insert(redirect_slot.clone());

    let mut response = next.run(req).await;

    // Apply cookies from transparent refresh
    if let Some((access, refresh)) = refresh_cookies {
        cookies::set_auth_cookies(response.headers_mut(), &access, &refresh);
    }

    // Apply any cookie action scheduled by server functions
    if let Some(action) = cookie_slot.0.lock().unwrap().take() {
        match action {
            PendingCookieAction::Set {
                access_token,
                refresh_token,
            } => {
                cookies::set_auth_cookies(response.headers_mut(), &access_token, &refresh_token);
            }
            PendingCookieAction::Clear => {
                cookies::clear_auth_cookies(response.headers_mut());
            }
        }
    }

    // Apply post-OAuth redirect cookie if scheduled by server functions
    if let Some(path) = redirect_slot.0.lock().unwrap().take() {
        response
            .headers_mut()
            .append(header::SET_COOKIE, cookies::build_redirect_cookie(&path));
    }

    response
}

/// Attempt to transparently refresh the session using the refresh token.
/// On success: inserts new Claims into request extensions and returns
/// the new token pair for the middleware to set as cookies.
async fn try_transparent_refresh(
    pool: &Pool<Postgres>,
    refresh_token: &str,
    req: &mut Request,
) -> Option<(String, String)> {
    // Use validate_refresh_token — only accepts tokens with typ: "refresh"
    let claims = validate_refresh_token(refresh_token).ok()?;

    // Look up by hash, not raw token — the DB stores SHA-256 hashes
    let token_hash = hash_token(refresh_token);
    let stored = sqlx::query!(
        "SELECT id, revoked FROM refresh_tokens WHERE token_hash = $1 AND user_id = $2",
        token_hash,
        claims.sub
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    if stored.revoked {
        return None;
    }

    // Revoke old refresh token
    let _ = sqlx::query!(
        "UPDATE refresh_tokens SET revoked = TRUE WHERE id = $1",
        stored.id
    )
    .execute(pool)
    .await;

    // Issue new tokens — reuse court_roles from the old token (no extra DB hit;
    // court role changes take effect on next full login, 15-min window max)
    let new_access =
        jwt::create_access_token(claims.sub, &claims.email, &claims.role, &claims.tier, &claims.court_roles).ok()?;
    let (new_refresh, expires_at) =
        jwt::create_refresh_token(claims.sub, &claims.email, &claims.role, &claims.tier, &claims.court_roles).ok()?;

    // Store the hash of the new refresh token
    let new_refresh_hash = hash_token(&new_refresh);
    let _ = sqlx::query!(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        claims.sub,
        new_refresh_hash,
        expires_at
    )
    .execute(pool)
    .await;

    // Validate the new access token to get fresh claims
    let new_claims = validate_access_token(&new_access).ok()?;
    req.extensions_mut().insert(new_claims);

    Some((new_access, new_refresh))
}
