use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect, Response},
};
use oauth2::{AuthorizationCode, TokenResponse};
use shared_types::{OAuthProvider, UserTier};
use sqlx::{Pool, Postgres};
use tracing::{debug, error, info};

use super::{cookies, jwt, oauth, oauth_state};

/// Query parameters received from the OAuth provider callback.
#[derive(Debug, serde::Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

/// Axum handler for `/auth/callback/{provider}`.
/// Exchanges the authorization code for tokens, fetches user info,
/// upserts the user, creates JWTs, sets HTTP-only cookies, and redirects to `/`.
pub async fn oauth_callback(
    State(pool): State<Pool<Postgres>>,
    Path(provider_str): Path<String>,
    headers: axum::http::HeaderMap,
    Query(params): Query<CallbackQuery>,
) -> Result<Response, Response> {
    info!(provider = %provider_str, "OAuth callback received");

    let error_redirect = |msg: &str| {
        error!(error = %msg, "OAuth callback failed");
        Redirect::to(&format!("/login?error={}", urlencoding::encode(msg))).into_response()
    };

    let provider = OAuthProvider::parse_provider(&provider_str)
        .ok_or_else(|| error_redirect("Unknown OAuth provider"))?;

    debug!("Verifying CSRF state");

    // Parse the state parameter: "{csrf}|{redirect}" or just "{csrf}"
    let (csrf_key, redirect_from_state) = oauth::parse_oauth_state(&params.state);

    // Verify CSRF state and retrieve PKCE verifier + redirect path from store
    let (verifier, redirect_in_store) = oauth_state::take_state(csrf_key)
        .await
        .ok_or_else(|| error_redirect("Invalid or expired OAuth state"))?;

    // Redirect priority: state store > state parameter > cookie > "/"
    let redirect_after = redirect_in_store.or_else(|| redirect_from_state.map(String::from));

    debug!("CSRF state verified, exchanging code for token");

    // Exchange code for access token
    let client = oauth::build_oauth_client(&provider)
        .map_err(|e| error_redirect(&format!("OAuth config error: {}", e)))?;

    let http_client = reqwest::Client::new();
    let token_response = client
        .exchange_code(AuthorizationCode::new(params.code))
        .set_pkce_verifier(verifier)
        .request_async(&http_client)
        .await
        .map_err(|e| error_redirect(&format!("Token exchange failed: {}", e)))?;

    let access_token_str = token_response.access_token().secret();
    debug!("Token exchange succeeded, fetching user info");

    // Fetch user info from the provider
    let user_info = match &provider {
        OAuthProvider::Google => {
            let info = oauth::fetch_google_user_info(access_token_str)
                .await
                .map_err(|e| error_redirect(&e))?;

            oauth::OAuthUserInfo {
                provider: OAuthProvider::Google,
                provider_id: info.sub,
                email: info.email.unwrap_or_default(),
                display_name: info.name.unwrap_or_else(|| "Google User".to_string()),
                avatar_url: info.picture,
            }
        }
        OAuthProvider::GitHub => {
            let info = oauth::fetch_github_user_info(access_token_str)
                .await
                .map_err(|e| error_redirect(&e))?;

            oauth::OAuthUserInfo {
                provider: OAuthProvider::GitHub,
                provider_id: info.id.to_string(),
                email: info.email.unwrap_or_default(),
                display_name: info.name.unwrap_or_else(|| info.login.clone()),
                avatar_url: info.avatar_url,
            }
        }
    };

    debug!(email = %user_info.email, display_name = %user_info.display_name, "User info fetched");

    if user_info.email.is_empty() {
        return Err(error_redirect(
            "Could not retrieve email from OAuth provider",
        ));
    }

    // Upsert user in the database
    let (user_id, role, tier_str) = oauth::upsert_oauth_user(&pool, &user_info)
        .await
        .map_err(|e| error_redirect(&e))?;

    let tier = UserTier::from_str_or_default(&tier_str);
    debug!(user_id, role = %role, tier = %tier_str, "User upserted");

    // Load court_roles from the user record
    let court_roles: std::collections::HashMap<String, String> = sqlx::query_scalar!(
        "SELECT court_roles FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten()
    .and_then(|v| serde_json::from_value(v).ok())
    .unwrap_or_default();

    // Create JWTs
    let jwt_access = jwt::create_access_token(user_id, &user_info.email, &role, tier.as_str(), &court_roles)
        .map_err(|e| error_redirect(&format!("JWT error: {}", e)))?;

    let (jwt_refresh, expires_at) =
        jwt::create_refresh_token(user_id, &user_info.email, &role, tier.as_str(), &court_roles)
            .map_err(|e| error_redirect(&format!("JWT error: {}", e)))?;

    // Store the hash of the refresh token — never persist raw JWTs
    let refresh_hash = jwt::hash_token(&jwt_refresh);
    sqlx::query!(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        user_id,
        refresh_hash,
        expires_at
    )
    .execute(&pool)
    .await
    .map_err(|e| error_redirect(&format!("DB error: {}", e)))?;

    // Build redirect response with auth cookies.
    // Redirect priority: state param/store (already combined) > cookie > "/"
    let cookie_redirect = cookies::extract_redirect_cookie(&headers);
    let destination = redirect_after
        .as_deref()
        .or(cookie_redirect.as_deref())
        .unwrap_or("/");

    let mut response = Redirect::to(destination).into_response();
    cookies::set_auth_cookies(response.headers_mut(), &jwt_access, &jwt_refresh);

    // Clear the redirect cookie after use
    if cookie_redirect.is_some() {
        response.headers_mut().append(
            axum::http::header::SET_COOKIE,
            cookies::build_clear_redirect_cookie(),
        );
    }

    info!(user_id, redirect = %destination, "OAuth login successful");
    Ok(response)
}
