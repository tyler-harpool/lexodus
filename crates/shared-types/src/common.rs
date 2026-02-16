use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Physical/mailing address (nested in API responses).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Address {
    pub street1: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street2: Option<String>,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country: String,
}

/// Paginated response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

/// Pagination metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PaginationMeta {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, page: i64, limit: i64, total: i64) -> Self {
        let total_pages = if limit > 0 {
            (total + limit - 1) / limit
        } else {
            1
        };
        let has_next = page < total_pages;
        let has_prev = page > 1;

        Self {
            data: items,
            meta: PaginationMeta {
                page,
                limit,
                total,
                total_pages,
                has_next,
                has_prev,
            },
        }
    }
}

/// Court/tenant record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Court {
    pub id: String,
    pub name: String,
    pub court_type: String,
    /// Subscription tier for this court (free, pro, enterprise).
    #[serde(default = "default_tier")]
    pub tier: String,
    pub created_at: DateTime<Utc>,
}

fn default_tier() -> String {
    "free".to_string()
}

/// Tenant statistics response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TenantStats {
    pub court_id: String,
    pub attorney_count: i64,
}

/// Request to initialize a tenant/court.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct InitTenantRequest {
    pub id: String,
    pub name: String,
    #[serde(default = "default_court_type")]
    pub court_type: String,
}

fn default_court_type() -> String {
    "district".to_string()
}

/// Helper to normalize pagination params with safe defaults.
pub fn normalize_pagination(page: Option<i64>, limit: Option<i64>) -> (i64, i64) {
    let page = page.unwrap_or(1).max(1);
    let limit = limit.unwrap_or(20).clamp(1, 100);
    (page, limit)
}

// ── Court Role Admission Workflow ──────────────────────

/// A pending or resolved court role request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CourtRoleRequestResponse {
    pub id: String,
    pub user_id: i64,
    pub court_id: String,
    pub requested_role: String,
    pub status: String,
    pub requested_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Request body for submitting a court role request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SubmitCourtRoleRequest {
    pub court_id: String,
    pub requested_role: String,
}

/// Request body for reviewing (approve/deny) a court role request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReviewCourtRoleRequest {
    pub approved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Request body for directly setting a court role (admin or clerk).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetCourtRoleRequest {
    pub user_id: i64,
    pub court_id: String,
    pub role: String,
}

/// A single court membership entry for a user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CourtMembership {
    pub court_id: String,
    pub role: String,
}
