use serde::{Deserialize, Serialize};

#[cfg(feature = "validation")]
use validator::Validate;

/// Request DTO for creating a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validation", derive(Validate))]
pub struct CreateUserRequest {
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 3, message = "Username must be at least 3 characters"))
    )]
    pub username: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Display name is required"))
    )]
    pub display_name: String,
}

/// Request DTO for updating a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validation", derive(Validate))]
pub struct UpdateUserRequest {
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 3, message = "Username must be at least 3 characters"))
    )]
    pub username: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Display name is required"))
    )]
    pub display_name: String,
}

/// Request DTO for creating a product.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validation", derive(Validate))]
pub struct CreateProductRequest {
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Product name is required"))
    )]
    pub name: String,
    pub description: String,
    #[cfg_attr(
        feature = "validation",
        validate(range(min = 0.0, message = "Price must be non-negative"))
    )]
    pub price: f64,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Category is required"))
    )]
    pub category: String,
    pub status: String,
}

/// Request DTO for updating a product.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validation", derive(Validate))]
pub struct UpdateProductRequest {
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Product name is required"))
    )]
    pub name: String,
    pub description: String,
    #[cfg_attr(
        feature = "validation",
        validate(range(min = 0.0, message = "Price must be non-negative"))
    )]
    pub price: f64,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Category is required"))
    )]
    pub category: String,
    pub status: String,
}

/// Request DTO for updating the current user's profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validation", derive(Validate))]
pub struct UpdateProfileRequest {
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Display name is required"))
    )]
    pub display_name: String,
    #[cfg_attr(
        feature = "validation",
        validate(email(message = "Valid email is required"))
    )]
    pub email: String,
}

/// Response returned after successful authentication (login or register).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AuthResponse {
    pub user: crate::AuthUser,
    pub access_token: String,
}

/// Request DTO for updating a user's subscription tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateTierRequest {
    pub tier: String,
}

// --- Billing Types ---

/// Request DTO for creating a Stripe checkout session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CheckoutRequest {
    pub checkout_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_cents: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_description: Option<String>,
    /// Court ID for subscription checkout (tier applies to the court, not the user).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub court_id: Option<String>,
}

/// Response containing the Stripe checkout session URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CheckoutResponse {
    pub url: String,
}

/// Current subscription status for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SubscriptionStatus {
    pub active: bool,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_period_end: Option<String>,
    pub cancel_at_period_end: bool,
}

// --- Email/Password Reset Types ---

/// Request DTO for initiating a password reset email.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validation", derive(Validate))]
pub struct ForgotPasswordRequest {
    #[cfg_attr(
        feature = "validation",
        validate(email(message = "Valid email is required"))
    )]
    pub email: String,
}

/// Request DTO for resetting a password with a token.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validation", derive(Validate))]
pub struct ResetPasswordRequest {
    pub token: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 8, message = "Password must be at least 8 characters"))
    )]
    pub new_password: String,
}

/// Generic message response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MessageResponse {
    pub message: String,
}

// --- Phone Verification Types ---

/// Request DTO for sending a phone verification code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validation", derive(Validate))]
pub struct SendPhoneVerificationRequest {
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 10, max = 15, message = "Phone number must be 10-15 characters"))
    )]
    pub phone_number: String,
}

/// Request DTO for verifying a phone number with a code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validation", derive(Validate))]
pub struct VerifyPhoneRequest {
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 10, max = 15, message = "Phone number must be 10-15 characters"))
    )]
    pub phone_number: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(equal = 6, message = "Code must be 6 digits"))
    )]
    pub code: String,
}

// ── Device Authorization Flow (RFC 8628) ─────────────

/// Request DTO for initiating a device authorization flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct InitiateDeviceRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_platform: Option<String>,
}

/// Request DTO for polling a device authorization status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PollDeviceRequest {
    pub device_code: String,
}

/// Request DTO for approving a device authorization (entered by user in browser).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ApproveDeviceRequest {
    pub user_code: String,
}
