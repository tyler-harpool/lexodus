use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    PkceCodeChallenge, RedirectUrl, Scope, TokenUrl,
};
use shared_types::OAuthProvider;

use super::oauth_state;

/// Concrete OAuth client type with auth URL, token URL, and redirect URL set.
type ConfiguredClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

/// Environment variable names for OAuth configuration.
struct OAuthEnvConfig {
    client_id_var: &'static str,
    client_secret_var: &'static str,
    redirect_url_var: &'static str,
    auth_url: &'static str,
    token_url: &'static str,
}

const GOOGLE_CONFIG: OAuthEnvConfig = OAuthEnvConfig {
    client_id_var: "OAUTH_GOOGLE_CLIENT_ID",
    client_secret_var: "OAUTH_GOOGLE_CLIENT_SECRET",
    redirect_url_var: "OAUTH_GOOGLE_REDIRECT_URL",
    auth_url: "https://accounts.google.com/o/oauth2/v2/auth",
    token_url: "https://oauth2.googleapis.com/token",
};

const GITHUB_CONFIG: OAuthEnvConfig = OAuthEnvConfig {
    client_id_var: "OAUTH_GITHUB_CLIENT_ID",
    client_secret_var: "OAUTH_GITHUB_CLIENT_SECRET",
    redirect_url_var: "OAUTH_GITHUB_REDIRECT_URL",
    auth_url: "https://github.com/login/oauth/authorize",
    token_url: "https://github.com/login/oauth/access_token",
};

fn env_config(provider: &OAuthProvider) -> &'static OAuthEnvConfig {
    match provider {
        OAuthProvider::Google => &GOOGLE_CONFIG,
        OAuthProvider::GitHub => &GITHUB_CONFIG,
    }
}

/// Build an OAuth2 client for the given provider.
pub fn build_oauth_client(provider: &OAuthProvider) -> Result<ConfiguredClient, String> {
    let config = env_config(provider);

    let client_id = std::env::var(config.client_id_var)
        .map_err(|_| format!("{} not set", config.client_id_var))?;
    let client_secret = std::env::var(config.client_secret_var)
        .map_err(|_| format!("{} not set", config.client_secret_var))?;
    let redirect_url = std::env::var(config.redirect_url_var)
        .map_err(|_| format!("{} not set", config.redirect_url_var))?;

    let client = BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_auth_uri(AuthUrl::new(config.auth_url.to_string()).map_err(|e| e.to_string())?)
        .set_token_uri(TokenUrl::new(config.token_url.to_string()).map_err(|e| e.to_string())?)
        .set_redirect_uri(RedirectUrl::new(redirect_url).map_err(|e| e.to_string())?);

    Ok(client)
}

/// Scopes for each provider.
fn scopes(provider: &OAuthProvider) -> Vec<Scope> {
    match provider {
        OAuthProvider::Google => vec![
            Scope::new("openid".to_string()),
            Scope::new("email".to_string()),
            Scope::new("profile".to_string()),
        ],
        OAuthProvider::GitHub => vec![
            Scope::new("read:user".to_string()),
            Scope::new("user:email".to_string()),
        ],
    }
}

/// Generate an OAuth authorization URL and store the CSRF state.
/// `redirect_after` is an optional path to redirect to after login (e.g. "/activate").
///
/// The redirect path is encoded directly in the OAuth `state` parameter as `{csrf}|{path}`.
/// This round-trips through the OAuth provider, so the callback can always recover it —
/// no cookies or in-memory lookups needed for the redirect.
pub async fn get_authorize_url(
    provider: &OAuthProvider,
    redirect_after: Option<String>,
) -> Result<String, String> {
    let client = build_oauth_client(provider)?;
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate a random CSRF token, then optionally append the redirect path.
    let csrf_token = CsrfToken::new_random();
    let csrf_secret = csrf_token.secret().clone();

    let state_param = match &redirect_after {
        Some(path) => format!("{}|{}", csrf_secret, path),
        None => csrf_secret.clone(),
    };

    let mut auth_request = client
        .authorize_url(|| CsrfToken::new(state_param))
        .set_pkce_challenge(pkce_challenge);

    for scope in scopes(provider) {
        auth_request = auth_request.add_scope(scope);
    }

    let (url, _) = auth_request.url();

    // Store using just the CSRF secret as key (without the redirect suffix)
    oauth_state::store_state(csrf_secret, pkce_verifier, redirect_after).await;

    Ok(url.to_string())
}

/// Parse the OAuth state parameter: `{csrf_token}` or `{csrf_token}|{redirect_path}`.
/// Returns the CSRF key (for state store lookup) and optional redirect path.
pub fn parse_oauth_state(state: &str) -> (&str, Option<&str>) {
    match state.split_once('|') {
        Some((csrf, redirect)) => (csrf, Some(redirect)),
        None => (state, None),
    }
}

/// Google user info from the userinfo endpoint.
#[derive(Debug, serde::Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

/// GitHub user info from the API.
#[derive(Debug, serde::Deserialize)]
pub struct GitHubUserInfo {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// GitHub email from the API (for private emails).
#[derive(Debug, serde::Deserialize)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
}

/// Fetch user info from Google using an access token.
pub async fn fetch_google_user_info(access_token: &str) -> Result<GoogleUserInfo, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Google user info: {}", e))?;

    resp.json::<GoogleUserInfo>()
        .await
        .map_err(|e| format!("Failed to parse Google user info: {}", e))
}

/// Fetch user info from GitHub using an access token.
pub async fn fetch_github_user_info(access_token: &str) -> Result<GitHubUserInfo, String> {
    let client = reqwest::Client::new();
    let mut user_info: GitHubUserInfo = client
        .get("https://api.github.com/user")
        .bearer_auth(access_token)
        .header("User-Agent", "lexodus")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch GitHub user info: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub user info: {}", e))?;

    // If email is not public, fetch from the emails endpoint
    if user_info.email.is_none() {
        let emails: Vec<GitHubEmail> = client
            .get("https://api.github.com/user/emails")
            .bearer_auth(access_token)
            .header("User-Agent", "lexodus")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch GitHub emails: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse GitHub emails: {}", e))?;

        user_info.email = emails
            .into_iter()
            .find(|e| e.primary && e.verified)
            .map(|e| e.email);
    }

    Ok(user_info)
}

/// User info unified from any OAuth provider.
pub struct OAuthUserInfo {
    pub provider: OAuthProvider,
    pub provider_id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

/// Insert or update a user from an OAuth login.
/// Returns the user's database ID, role, and tier.
pub async fn upsert_oauth_user(
    db: &sqlx::PgPool,
    info: &OAuthUserInfo,
) -> Result<(i64, String, String), String> {
    let provider_str = info.provider.as_str();

    // Try to find existing user by OAuth provider + ID
    let existing = sqlx::query!(
        "SELECT id, role, tier FROM users WHERE oauth_provider = $1 AND oauth_provider_id = $2",
        provider_str,
        info.provider_id
    )
    .fetch_optional(db)
    .await
    .map_err(|e| format!("DB lookup failed: {}", e))?;

    if let Some(row) = existing {
        // Update display name and avatar on each login
        sqlx::query!(
            "UPDATE users SET display_name = $2, avatar_url = $3, updated_at = NOW() WHERE id = $1",
            row.id,
            info.display_name,
            info.avatar_url.as_deref(),
        )
        .execute(db)
        .await
        .map_err(|e| format!("DB update failed: {}", e))?;

        let role = super::maybe_promote_admin(db, row.id, &info.email, row.role).await;
        return Ok((row.id, role, row.tier));
    }

    // Check if a user with this email already exists (link OAuth to existing account)
    let by_email = sqlx::query!(
        "SELECT id, role, tier FROM users WHERE email = $1",
        info.email
    )
    .fetch_optional(db)
    .await
    .map_err(|e| format!("DB email lookup failed: {}", e))?;

    if let Some(row) = by_email {
        // Link OAuth provider to existing account
        sqlx::query!(
            "UPDATE users SET oauth_provider = $2, oauth_provider_id = $3, avatar_url = $4, updated_at = NOW() WHERE id = $1",
            row.id,
            provider_str,
            info.provider_id,
            info.avatar_url.as_deref(),
        )
        .execute(db)
        .await
        .map_err(|e| format!("DB link failed: {}", e))?;

        let role = super::maybe_promote_admin(db, row.id, &info.email, row.role).await;
        return Ok((row.id, role, row.tier));
    }

    // Create new user
    let username = info.email.split('@').next().unwrap_or("user").to_string();

    let row = sqlx::query!(
        r#"INSERT INTO users (username, email, display_name, oauth_provider, oauth_provider_id, avatar_url)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, role, tier"#,
        username,
        info.email,
        info.display_name,
        provider_str,
        info.provider_id,
        info.avatar_url.as_deref(),
    )
    .fetch_one(db)
    .await
    .map_err(|e| format!("DB insert failed: {}", e))?;

    let role = super::maybe_promote_admin(db, row.id, &info.email, row.role).await;
    Ok((row.id, role, row.tier))
}
