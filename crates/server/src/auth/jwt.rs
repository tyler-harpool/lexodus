use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Token type discriminator — prevents using a refresh token as an access token.
const TOKEN_TYPE_ACCESS: &str = "access";
const TOKEN_TYPE_REFRESH: &str = "refresh";

/// JWT claims stored in access and refresh tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub email: String,
    pub role: String,
    pub tier: String,
    pub exp: i64,
    pub iat: i64,
    /// Unique token identifier — prevents hash collisions when multiple
    /// tokens are issued for the same user within the same second.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    /// Token type: "access" or "refresh". Prevents token confusion attacks
    /// where a refresh token is used as an access token or vice versa.
    #[serde(default)]
    pub typ: String,
    /// Per-court role memberships carried in the token.
    #[serde(default)]
    pub court_roles: HashMap<String, String>,
}

/// Compute the SHA-256 hash of a raw JWT string, returned as a hex-encoded string.
/// Used to store refresh tokens safely — the raw token goes to the client cookie
/// while only the hash is persisted in the database.
pub fn hash_token(raw_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

pub fn access_token_expiry_minutes() -> i64 {
    std::env::var("JWT_ACCESS_TOKEN_EXPIRY_MINUTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15)
}

pub fn refresh_token_expiry_days() -> i64 {
    std::env::var("JWT_REFRESH_TOKEN_EXPIRY_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7)
}

pub fn create_access_token(
    user_id: i64,
    email: &str,
    role: &str,
    tier: &str,
    court_roles: &HashMap<String, String>,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        tier: tier.to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::minutes(access_token_expiry_minutes())).timestamp(),
        jti: Some(uuid::Uuid::new_v4().to_string()),
        typ: TOKEN_TYPE_ACCESS.to_string(),
        court_roles: court_roles.clone(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
}

pub fn create_refresh_token(
    user_id: i64,
    email: &str,
    role: &str,
    tier: &str,
    court_roles: &HashMap<String, String>,
) -> Result<(String, chrono::DateTime<Utc>), jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expires_at = now + Duration::days(refresh_token_expiry_days());
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        tier: tier.to_string(),
        iat: now.timestamp(),
        exp: expires_at.timestamp(),
        jti: Some(uuid::Uuid::new_v4().to_string()),
        typ: TOKEN_TYPE_REFRESH.to_string(),
        court_roles: court_roles.clone(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )?;
    Ok((token, expires_at))
}

/// Validate an access token. Rejects tokens with `typ: "refresh"` to prevent
/// token confusion attacks where a refresh token is presented as an access token.
/// Allows empty `typ` for backward compatibility with pre-migration tokens.
pub fn validate_access_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )?;
    if token_data.claims.typ == TOKEN_TYPE_REFRESH {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
    }
    Ok(token_data.claims)
}

/// Validate a refresh token. Requires `typ: "refresh"` — rejects access tokens
/// and legacy tokens without a type claim.
pub fn validate_refresh_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )?;
    if token_data.claims.typ != TOKEN_TYPE_REFRESH {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
    }
    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_secret() {
        std::env::set_var("JWT_SECRET", "test-secret-key-for-jwt-unit-tests");
    }

    #[test]
    fn create_and_validate_access_token() {
        setup_test_secret();
        let token = create_access_token(42, "test@example.com", "user", "free", &HashMap::new()).unwrap();
        let claims = validate_access_token(&token).unwrap();
        assert_eq!(claims.sub, 42);
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, "user");
        assert_eq!(claims.tier, "free");
        assert_eq!(claims.typ, TOKEN_TYPE_ACCESS);
    }

    #[test]
    fn expired_token_rejected() {
        setup_test_secret();
        let now = Utc::now();
        let claims = Claims {
            sub: 1,
            email: "expired@test.com".to_string(),
            role: "user".to_string(),
            tier: "free".to_string(),
            iat: (now - Duration::hours(2)).timestamp(),
            exp: (now - Duration::hours(1)).timestamp(),
            jti: None,
            typ: TOKEN_TYPE_ACCESS.to_string(),
            court_roles: HashMap::new(),
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret().as_bytes()),
        )
        .unwrap();

        assert!(validate_access_token(&token).is_err());
    }

    #[test]
    fn invalid_token_rejected() {
        setup_test_secret();
        assert!(validate_access_token("not.a.valid.jwt").is_err());
        assert!(validate_access_token("").is_err());
    }

    #[test]
    fn claims_contain_correct_fields() {
        setup_test_secret();
        let token = create_access_token(99, "admin@co.com", "admin", "enterprise", &HashMap::new()).unwrap();
        let claims = validate_access_token(&token).unwrap();
        assert_eq!(claims.sub, 99);
        assert_eq!(claims.email, "admin@co.com");
        assert_eq!(claims.role, "admin");
        assert_eq!(claims.tier, "enterprise");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn refresh_token_has_later_expiry() {
        setup_test_secret();
        let access = create_access_token(1, "a@b.com", "user", "free", &HashMap::new()).unwrap();
        let (refresh, _) = create_refresh_token(1, "a@b.com", "user", "free", &HashMap::new()).unwrap();

        let access_claims = validate_access_token(&access).unwrap();
        let refresh_claims = validate_refresh_token(&refresh).unwrap();

        assert!(refresh_claims.exp > access_claims.exp);
    }

    #[test]
    fn refresh_token_rejected_by_access_validator() {
        setup_test_secret();
        let (refresh, _) = create_refresh_token(1, "a@b.com", "user", "free", &HashMap::new()).unwrap();
        // A refresh token must NOT pass access token validation
        assert!(validate_access_token(&refresh).is_err());
    }

    #[test]
    fn access_token_rejected_by_refresh_validator() {
        setup_test_secret();
        let access = create_access_token(1, "a@b.com", "user", "free", &HashMap::new()).unwrap();
        // An access token must NOT pass refresh token validation
        assert!(validate_refresh_token(&access).is_err());
    }

    #[test]
    fn hash_token_produces_consistent_hex() {
        let token = "eyJhbGciOiJIUzI1NiJ9.test-payload.signature";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);
        assert_eq!(hash1, hash2);
        // SHA-256 produces 64 hex characters
        assert_eq!(hash1.len(), 64);
        assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn different_tokens_produce_different_hashes() {
        let hash1 = hash_token("token-aaa");
        let hash2 = hash_token("token-bbb");
        assert_ne!(hash1, hash2);
    }
}
