use axum::{extract::FromRequestParts, http::request::Parts};
use shared_types::{AppError, UserRole, UserTier};

use super::jwt::Claims;

/// Extractor that requires authentication. Returns 401 if no valid token.
pub struct AuthRequired(pub Claims);

impl<S: Send + Sync> FromRequestParts<S> for AuthRequired {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(AuthRequired)
            .ok_or_else(|| AppError::unauthorized("Authentication required"))
    }
}

/// Extractor that optionally extracts auth claims. Never fails.
pub struct MaybeAuth(pub Option<Claims>);

impl<S: Send + Sync> FromRequestParts<S> for MaybeAuth {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(MaybeAuth(parts.extensions.get::<Claims>().cloned()))
    }
}

/// Extractor that requires authentication AND a specific court role.
/// Returns 401 if unauthenticated, 403 if the user's role does not satisfy the required role.
///
/// Role constants (match `UserRole` variants):
/// - 0 = Public    (any authenticated user — rarely used with this extractor)
/// - 1 = Attorney  (any authenticated user with an assigned role)
/// - 2 = Clerk
/// - 3 = Judge
/// - 4 = Admin     (satisfies all roles)
pub struct RoleRequired<const ROLE: u8>(pub Claims);

impl<const ROLE: u8, S: Send + Sync> FromRequestParts<S> for RoleRequired<ROLE> {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or_else(|| AppError::unauthorized("Authentication required"))?;

        let user_role = UserRole::from_str_or_default(&claims.role);
        let required_role = match ROLE {
            1 => UserRole::Attorney,
            2 => UserRole::Clerk,
            3 => UserRole::Judge,
            4 => UserRole::Admin,
            _ => UserRole::Public,
        };

        if !user_role.satisfies(&required_role) {
            return Err(AppError::forbidden(format!(
                "{} role or higher required",
                required_role.as_str()
            )));
        }

        Ok(RoleRequired(claims))
    }
}

/// Extractor that requires authentication AND a minimum user tier.
/// Returns 401 if unauthenticated, 403 if insufficient tier.
pub struct TierRequired<const TIER: u8>(pub Claims);

impl<const TIER: u8, S: Send + Sync> FromRequestParts<S> for TierRequired<TIER> {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or_else(|| AppError::unauthorized("Authentication required"))?;

        let user_tier = UserTier::from_str_or_default(&claims.tier);
        let required_tier = match TIER {
            0 => UserTier::Free,
            1 => UserTier::Pro,
            2 => UserTier::Enterprise,
            _ => UserTier::Enterprise,
        };

        if !user_tier.has_access(&required_tier) {
            return Err(AppError::forbidden(format!(
                "{} tier or higher required",
                required_tier.as_str()
            )));
        }

        Ok(TierRequired(claims))
    }
}
