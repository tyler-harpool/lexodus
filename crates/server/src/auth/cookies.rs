use axum::http::{header, HeaderMap, HeaderValue};
use cookie::Cookie;
use std::sync::{Arc, Mutex};

use super::jwt;

pub const CYBER_ACCESS: &str = "cyber_access";
pub const CYBER_REFRESH: &str = "cyber_refresh";

fn cookie_secure() -> bool {
    std::env::var("COOKIE_SECURE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(false)
}

fn cookie_domain() -> Option<String> {
    std::env::var("COOKIE_DOMAIN")
        .ok()
        .filter(|d| !d.is_empty())
}

/// Build a Set-Cookie header value for the access token.
pub fn build_access_cookie(token: &str, max_age_minutes: i64) -> HeaderValue {
    let mut cookie = Cookie::build((CYBER_ACCESS, token))
        .http_only(true)
        .same_site(cookie::SameSite::Lax)
        .path("/")
        .max_age(cookie::time::Duration::seconds(max_age_minutes * 60))
        .secure(cookie_secure());

    if let Some(domain) = cookie_domain() {
        cookie = cookie.domain(domain);
    }

    HeaderValue::from_str(&cookie.build().to_string()).expect("cookie header value should be valid")
}

/// Build a Set-Cookie header value for the refresh token.
pub fn build_refresh_cookie(token: &str, max_age_days: i64) -> HeaderValue {
    let mut cookie = Cookie::build((CYBER_REFRESH, token))
        .http_only(true)
        .same_site(cookie::SameSite::Lax)
        .path("/")
        .max_age(cookie::time::Duration::seconds(max_age_days * 86400))
        .secure(cookie_secure());

    if let Some(domain) = cookie_domain() {
        cookie = cookie.domain(domain);
    }

    HeaderValue::from_str(&cookie.build().to_string()).expect("cookie header value should be valid")
}

/// Build Set-Cookie headers that clear both auth cookies.
pub fn build_clear_cookies() -> (HeaderValue, HeaderValue) {
    let access = Cookie::build((CYBER_ACCESS, ""))
        .http_only(true)
        .same_site(cookie::SameSite::Lax)
        .path("/")
        .max_age(cookie::time::Duration::ZERO)
        .build();

    let refresh = Cookie::build((CYBER_REFRESH, ""))
        .http_only(true)
        .same_site(cookie::SameSite::Lax)
        .path("/")
        .max_age(cookie::time::Duration::ZERO)
        .build();

    (
        HeaderValue::from_str(&access.to_string()).expect("clear cookie should be valid"),
        HeaderValue::from_str(&refresh.to_string()).expect("clear cookie should be valid"),
    )
}

/// Extract the access token from cookies (preferred) or Bearer header (fallback).
pub fn extract_access_token(headers: &HeaderMap) -> Option<String> {
    // Try cookie first
    if let Some(token) = extract_cookie(headers, CYBER_ACCESS) {
        return Some(token);
    }

    // Fallback to Bearer header for REST API clients
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }

    None
}

/// Extract the refresh token from cookies.
pub fn extract_refresh_token(headers: &HeaderMap) -> Option<String> {
    extract_cookie(headers, CYBER_REFRESH)
}

/// Parse a specific cookie value from the Cookie header.
fn extract_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    for header_value in headers.get_all(header::COOKIE) {
        if let Ok(cookie_str) = header_value.to_str() {
            for piece in cookie_str.split(';') {
                if let Ok(c) = Cookie::parse(piece.trim().to_string()) {
                    if c.name() == name {
                        return Some(c.value().to_string());
                    }
                }
            }
        }
    }
    None
}

/// Set both access and refresh cookies on the response using current JWT expiry config.
pub fn set_auth_cookies(headers: &mut HeaderMap, access_token: &str, refresh_token: &str) {
    let access_minutes = jwt::access_token_expiry_minutes();
    let refresh_days = jwt::refresh_token_expiry_days();

    headers.append(
        header::SET_COOKIE,
        build_access_cookie(access_token, access_minutes),
    );
    headers.append(
        header::SET_COOKIE,
        build_refresh_cookie(refresh_token, refresh_days),
    );
}

/// Clear both auth cookies on the response.
pub fn clear_auth_cookies(headers: &mut HeaderMap) {
    let (access, refresh) = build_clear_cookies();
    headers.append(header::SET_COOKIE, access);
    headers.append(header::SET_COOKIE, refresh);
}

/// Pending cookie action to be picked up by the auth middleware.
/// Stored in request extensions as `Arc<Mutex<>>` so server functions can populate it.
#[derive(Clone, Debug)]
pub enum PendingCookieAction {
    Set {
        access_token: String,
        refresh_token: String,
    },
    Clear,
}

/// Shared slot for server functions to communicate cookie actions to the middleware.
#[derive(Clone, Debug, Default)]
pub struct CookieSlot(pub Arc<Mutex<Option<PendingCookieAction>>>);

/// Schedule auth cookies to be set by the middleware.
/// Called from server functions — reads the CookieSlot from FullstackContext extensions.
pub fn schedule_auth_cookies(access_token: &str, refresh_token: &str) {
    if let Some(ctx) = dioxus::fullstack::FullstackContext::current() {
        let parts = ctx.parts_mut();
        if let Some(slot) = parts.extensions.get::<CookieSlot>() {
            *slot.0.lock().unwrap() = Some(PendingCookieAction::Set {
                access_token: access_token.to_string(),
                refresh_token: refresh_token.to_string(),
            });
        }
    }
}

/// Schedule auth cookies to be cleared by the middleware.
/// Called from server functions — reads the CookieSlot from FullstackContext extensions.
pub fn schedule_clear_cookies() {
    if let Some(ctx) = dioxus::fullstack::FullstackContext::current() {
        let parts = ctx.parts_mut();
        if let Some(slot) = parts.extensions.get::<CookieSlot>() {
            *slot.0.lock().unwrap() = Some(PendingCookieAction::Clear);
        }
    }
}

// ── Post-OAuth redirect cookie (BFF pattern) ────────────

/// Cookie name for the post-OAuth redirect path.
const POST_OAUTH_REDIRECT: &str = "post_oauth_redirect";

/// Shared slot for server functions to schedule a post-OAuth redirect cookie.
#[derive(Clone, Debug, Default)]
pub struct RedirectSlot(pub Arc<Mutex<Option<String>>>);

/// Schedule a post-OAuth redirect cookie to be set by the middleware.
/// Called from the `oauth_authorize_url` server function when a redirect_after is provided.
pub fn schedule_redirect_cookie(path: &str) {
    if let Some(ctx) = dioxus::fullstack::FullstackContext::current() {
        let parts = ctx.parts_mut();
        if let Some(slot) = parts.extensions.get::<RedirectSlot>() {
            *slot.0.lock().unwrap() = Some(path.to_string());
        }
    }
}

/// Extract the post-OAuth redirect path from request cookies.
pub fn extract_redirect_cookie(headers: &HeaderMap) -> Option<String> {
    extract_cookie(headers, POST_OAUTH_REDIRECT)
}

/// Build a Set-Cookie header for the post-OAuth redirect path (HTTP-only, 10 min TTL).
pub fn build_redirect_cookie(path: &str) -> HeaderValue {
    let cookie = Cookie::build((POST_OAUTH_REDIRECT, path))
        .http_only(true)
        .same_site(cookie::SameSite::Lax)
        .path("/")
        .max_age(cookie::time::Duration::seconds(600))
        .build();
    HeaderValue::from_str(&cookie.to_string()).expect("redirect cookie should be valid")
}

/// Build a Set-Cookie header that clears the post-OAuth redirect cookie.
pub fn build_clear_redirect_cookie() -> HeaderValue {
    let cookie = Cookie::build((POST_OAUTH_REDIRECT, ""))
        .http_only(true)
        .same_site(cookie::SameSite::Lax)
        .path("/")
        .max_age(cookie::time::Duration::ZERO)
        .build();
    HeaderValue::from_str(&cookie.to_string()).expect("clear redirect cookie should be valid")
}
