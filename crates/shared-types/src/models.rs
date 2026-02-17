use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User subscription tier controlling feature access.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum UserTier {
    #[default]
    Free,
    Pro,
    Enterprise,
}

impl UserTier {
    /// Numeric rank for tier comparison.
    fn rank(&self) -> u8 {
        match self {
            UserTier::Free => 0,
            UserTier::Pro => 1,
            UserTier::Enterprise => 2,
        }
    }

    /// Check if this tier grants access to a feature requiring `required` tier.
    pub fn has_access(&self, required: &UserTier) -> bool {
        self.rank() >= required.rank()
    }

    /// Parse a tier string, defaulting to the base tier for unknown values.
    pub fn from_str_or_default(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pro" => UserTier::Pro,
            "enterprise" => UserTier::Enterprise,
            _ => UserTier::Free,
        }
    }

    /// Serialize to lowercase string for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            UserTier::Free => "free",
            UserTier::Pro => "pro",
            UserTier::Enterprise => "enterprise",
        }
    }
}

/// Court user role controlling access to filing operations.
///
/// - `Public` — unauthenticated or unknown role. View-only access to public records.
/// - `Attorney` — can file documents, view public + SealedAttorneysOnly.
/// - `Clerk` — can add docket entries, seal/unseal, strike/replace, promote.
/// - `Judge` — can issue orders, seal/unseal.
/// - `Admin` — full access (superset of all roles).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum UserRole {
    #[default]
    Public,
    Attorney,
    Clerk,
    Judge,
    Admin,
}

impl UserRole {
    /// Parse from JWT `role` claim. Unknown values default to Public.
    pub fn from_str_or_default(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "attorney" => UserRole::Attorney,
            "clerk" => UserRole::Clerk,
            "judge" => UserRole::Judge,
            "admin" => UserRole::Admin,
            _ => UserRole::Public,
        }
    }

    /// Lowercase string for database / JWT storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Public => "public",
            UserRole::Attorney => "attorney",
            UserRole::Clerk => "clerk",
            UserRole::Judge => "judge",
            UserRole::Admin => "admin",
        }
    }

    /// Returns true if this role satisfies the `required` role.
    /// Admin satisfies all roles. Clerk and Judge satisfy themselves + Attorney + Public.
    pub fn satisfies(&self, required: &UserRole) -> bool {
        match self {
            UserRole::Admin => true,
            UserRole::Clerk => matches!(required, UserRole::Clerk | UserRole::Attorney | UserRole::Public),
            UserRole::Judge => matches!(required, UserRole::Judge | UserRole::Attorney | UserRole::Public),
            UserRole::Attorney => matches!(required, UserRole::Attorney | UserRole::Public),
            UserRole::Public => matches!(required, UserRole::Public),
        }
    }
}

/// Supported OAuth identity providers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum OAuthProvider {
    Google,
    GitHub,
}

impl OAuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::GitHub => "github",
        }
    }

    pub fn parse_provider(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "google" => Some(OAuthProvider::Google),
            "github" => Some(OAuthProvider::GitHub),
            _ => None,
        }
    }
}

/// Parameters received from an OAuth callback redirect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OAuthCallbackParams {
    pub code: String,
    pub state: String,
}

/// A user in the system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct User {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub tier: String,
}

/// A product available in the catalog.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub category: String,
    pub status: String,
    pub created_at: String,
}

/// User row with their role in a specific court (flat DTO for the membership UI).
/// Server resolves `court_roles[court_id]` into a plain string so the UI
/// never has to deserialize a `HashMap` across the WASM boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UserWithMembership {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub tier: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone_number: Option<String>,
    /// Role in the requested court ("" if none).
    #[serde(default)]
    pub court_role: String,
    /// All court memberships: court_id -> role.
    #[serde(default)]
    pub all_court_roles: std::collections::HashMap<String, String>,
}

/// Aggregated dashboard statistics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DashboardStats {
    pub total_users: i64,
    pub total_products: i64,
    pub active_products: i64,
    pub recent_users: Vec<User>,
}

/// Login request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
pub struct LoginRequest {
    #[cfg_attr(
        feature = "validation",
        validate(email(message = "Valid email is required"))
    )]
    pub email: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 8, message = "Password must be at least 8 characters"))
    )]
    pub password: String,
}

/// Register request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
pub struct RegisterRequest {
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 3, message = "Username must be at least 3 characters"))
    )]
    pub username: String,
    #[cfg_attr(
        feature = "validation",
        validate(email(message = "Valid email is required"))
    )]
    pub email: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 8, message = "Password must be at least 8 characters"))
    )]
    pub password: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Display name is required"))
    )]
    pub display_name: String,
}

/// Authenticated user info (safe to send to client).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub role: String,
    #[serde(default)]
    pub tier: UserTier,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub email_verified: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(default)]
    pub phone_verified: bool,
    #[serde(default = "default_true")]
    pub email_notifications_enabled: bool,
    #[serde(default)]
    pub push_notifications_enabled: bool,
    #[serde(default = "default_true")]
    pub weekly_digest_enabled: bool,
    #[serde(default)]
    pub has_password: bool,
    /// Per-court role memberships: maps court_id -> role string.
    #[serde(default)]
    pub court_roles: HashMap<String, String>,
    /// Per-court subscription tiers: maps court_id -> tier string.
    #[serde(default)]
    pub court_tiers: HashMap<String, String>,
    /// Last-selected court district (cross-platform persistence).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_court_id: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Premium analytics data returned by the tier-gated endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PremiumAnalytics {
    pub total_revenue: f64,
    pub avg_product_price: f64,
    pub products_by_category: Vec<CategoryCount>,
    pub users_last_30_days: i64,
}

/// Category name with a count of products in that category.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CategoryCount {
    pub category: String,
    pub count: i64,
}

/// Refresh token request (used by REST/OpenAPI).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Billing webhook event variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum BillingEvent {
    SubscriptionUpdated {
        tier: String,
        status: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        court_id: Option<String>,
    },
    PaymentSucceeded { amount_cents: i64 },
    PaymentFailed { message: String },
}

// ── Device Authorization Flow (RFC 8628) ─────────────

/// Status of a device authorization request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum DeviceAuthStatus {
    #[default]
    Pending,
    Approved,
    Expired,
}

impl DeviceAuthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceAuthStatus::Pending => "pending",
            DeviceAuthStatus::Approved => "approved",
            DeviceAuthStatus::Expired => "expired",
        }
    }

    pub fn from_str_or_default(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "approved" => DeviceAuthStatus::Approved,
            "expired" => DeviceAuthStatus::Expired,
            _ => DeviceAuthStatus::Pending,
        }
    }
}

/// Response from initiating a device authorization flow.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeviceFlowInitResponse {
    #[serde(default)]
    pub device_code: String,
    #[serde(default)]
    pub user_code: String,
    #[serde(default)]
    pub verification_uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_uri_complete: Option<String>,
    #[serde(default)]
    pub expires_in: i64,
    #[serde(default)]
    pub interval: i64,
}

/// Response from polling a device authorization request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeviceFlowPollResponse {
    #[serde(default)]
    pub status: DeviceAuthStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<AuthUser>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_serialization_roundtrip() {
        let user = User {
            id: 1,
            username: "tharpool".into(),
            display_name: "Tyler".into(),
            role: "user".into(),
            tier: "free".into(),
        };

        let json = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&json).unwrap();

        assert_eq!(user, deserialized);
    }

    #[test]
    fn user_deserializes_from_api_json() {
        let json = r#"{"id": 42, "username": "demo", "display_name": "Demo User", "role": "admin", "tier": "pro"}"#;
        let user: User = serde_json::from_str(json).unwrap();

        assert_eq!(user.id, 42);
        assert_eq!(user.username, "demo");
        assert_eq!(user.role, "admin");
        assert_eq!(user.tier, "pro");
    }

    #[test]
    fn product_serialization_roundtrip() {
        let product = Product {
            id: 1,
            name: "Widget".into(),
            description: "A test widget".into(),
            price: 29.99,
            category: "Hardware".into(),
            status: "active".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
        };

        let json = serde_json::to_string(&product).unwrap();
        let deserialized: Product = serde_json::from_str(&json).unwrap();

        assert_eq!(product, deserialized);
    }

    #[test]
    fn user_tier_has_access_same_tier() {
        assert!(UserTier::Free.has_access(&UserTier::Free));
        assert!(UserTier::Pro.has_access(&UserTier::Pro));
        assert!(UserTier::Enterprise.has_access(&UserTier::Enterprise));
    }

    #[test]
    fn user_tier_has_access_higher_tier() {
        assert!(UserTier::Pro.has_access(&UserTier::Free));
        assert!(UserTier::Enterprise.has_access(&UserTier::Free));
        assert!(UserTier::Enterprise.has_access(&UserTier::Pro));
    }

    #[test]
    fn user_tier_denies_lower_tier() {
        assert!(!UserTier::Free.has_access(&UserTier::Pro));
        assert!(!UserTier::Free.has_access(&UserTier::Enterprise));
        assert!(!UserTier::Pro.has_access(&UserTier::Enterprise));
    }

    #[test]
    fn user_tier_from_str_or_default_known_values() {
        assert_eq!(UserTier::from_str_or_default("pro"), UserTier::Pro);
        assert_eq!(UserTier::from_str_or_default("Pro"), UserTier::Pro);
        assert_eq!(UserTier::from_str_or_default("PRO"), UserTier::Pro);
        assert_eq!(
            UserTier::from_str_or_default("enterprise"),
            UserTier::Enterprise
        );
        assert_eq!(
            UserTier::from_str_or_default("Enterprise"),
            UserTier::Enterprise
        );
        assert_eq!(UserTier::from_str_or_default("free"), UserTier::Free);
    }

    #[test]
    fn user_tier_from_str_or_default_unknown_falls_to_base() {
        assert_eq!(UserTier::from_str_or_default(""), UserTier::Free);
        assert_eq!(UserTier::from_str_or_default("gold"), UserTier::Free);
        assert_eq!(UserTier::from_str_or_default("invalid"), UserTier::Free);
    }

    #[test]
    fn user_tier_as_str_roundtrip() {
        for tier in [UserTier::Free, UserTier::Pro, UserTier::Enterprise] {
            let s = tier.as_str();
            let parsed = UserTier::from_str_or_default(s);
            assert_eq!(tier, parsed);
        }
    }

    #[test]
    fn oauth_provider_parse_valid() {
        assert_eq!(
            OAuthProvider::parse_provider("google"),
            Some(OAuthProvider::Google)
        );
        assert_eq!(
            OAuthProvider::parse_provider("Google"),
            Some(OAuthProvider::Google)
        );
        assert_eq!(
            OAuthProvider::parse_provider("github"),
            Some(OAuthProvider::GitHub)
        );
        assert_eq!(
            OAuthProvider::parse_provider("GitHub"),
            Some(OAuthProvider::GitHub)
        );
    }

    #[test]
    fn oauth_provider_parse_invalid_returns_none() {
        assert_eq!(OAuthProvider::parse_provider("facebook"), None);
        assert_eq!(OAuthProvider::parse_provider(""), None);
        assert_eq!(OAuthProvider::parse_provider("twitter"), None);
    }
}
