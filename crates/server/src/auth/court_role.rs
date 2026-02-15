use shared_types::UserRole;

/// Resolve effective role for a user in a specific court from JWT claims.
/// No DB query — reads from claims.court_roles (populated at login).
///
/// - Global admin bypasses court membership.
/// - Court-specific role is looked up from JWT claims.
/// - Falls back to Public if no court membership exists.
pub fn resolve_court_role(claims: &super::jwt::Claims, court_id: &str) -> UserRole {
    if claims.role == "admin" {
        return UserRole::Admin;
    }
    claims
        .court_roles
        .get(court_id)
        .map(|r| UserRole::from_str_or_default(r))
        .unwrap_or(UserRole::Public)
}
