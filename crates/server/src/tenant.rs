use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use shared_types::AppError;

/// Extractor that resolves the court/tenant ID from the request.
///
/// Priority:
/// 1. `X-Tenant-ID` header
/// 2. `X-Court-District` header
/// 3. Host subdomain (e.g., `sdny.lexodus.gov` -> `sdny`)
/// 4. `?tenant=xxx` query param
#[derive(Debug, Clone)]
pub struct CourtId(pub String);

impl CourtId {
    /// Sanitize a tenant ID to lowercase alphanumeric + hyphens.
    fn sanitize(raw: &str) -> String {
        raw.trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect()
    }
}

impl<S> FromRequestParts<S> for CourtId
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. X-Tenant-ID header
        if let Some(val) = parts.headers.get("x-tenant-id") {
            if let Ok(s) = val.to_str() {
                let sanitized = Self::sanitize(s);
                if !sanitized.is_empty() {
                    return Ok(CourtId(sanitized));
                }
            }
        }

        // 2. X-Court-District header
        if let Some(val) = parts.headers.get("x-court-district") {
            if let Ok(s) = val.to_str() {
                let sanitized = Self::sanitize(s);
                if !sanitized.is_empty() {
                    return Ok(CourtId(sanitized));
                }
            }
        }

        // 3. Host subdomain
        if let Some(host) = parts.headers.get("host") {
            if let Ok(h) = host.to_str() {
                let parts_host: Vec<&str> = h.split('.').collect();
                if parts_host.len() >= 3 {
                    let sanitized = Self::sanitize(parts_host[0]);
                    if !sanitized.is_empty() {
                        return Ok(CourtId(sanitized));
                    }
                }
            }
        }

        // 4. Query parameter ?tenant=xxx
        if let Some(query) = &parts.uri.query() {
            for pair in query.split('&') {
                if let Some(val) = pair.strip_prefix("tenant=") {
                    let sanitized = Self::sanitize(val);
                    if !sanitized.is_empty() {
                        return Ok(CourtId(sanitized));
                    }
                }
            }
        }

        Err(AppError::bad_request(
            "Missing required header: X-Court-District",
        ))
    }
}
