# CM/ECF Modernization Gaps — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add MFA (TOTP), filing fee payments (Stripe), and a free public access portal (CourtListener-compatible API) to close the 3 critical gaps for full CM/ECF replacement.

**Architecture:** 3 phases sharing the existing Dioxus + Axum + Postgres + Stripe stack. Feature-flagged for incremental rollout. No new services or crates beyond `totp-rs`, `qrcode`, and `aes-gcm`.

**Tech Stack:** Rust, Dioxus 0.7, Axum, PostgreSQL, sqlx, Stripe (async-stripe), totp-rs, aes-gcm, qrcode

**Design Doc:** `docs/plans/2026-02-19-cmecf-gaps-design.md`

---

## Phase 1: MFA (TOTP)

### Task 1: Add MFA Dependencies and Feature Flag

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Modify: `crates/server/Cargo.toml`
- Modify: `crates/shared-types/src/feature_flags.rs`

**Step 1: Add workspace dependencies**

In root `Cargo.toml` under `[workspace.dependencies]`, add:

```toml
totp-rs = { version = "5", features = ["gen_secret", "qr"] }
qrcode = { version = "0.14", default-features = false, features = ["svg"] }
aes-gcm = "0.10"
```

**Step 2: Add server crate dependencies**

In `crates/server/Cargo.toml` under `[dependencies]`, add:

```toml
totp-rs = { workspace = true, optional = true }
qrcode = { workspace = true, optional = true }
aes-gcm = { workspace = true, optional = true }
```

Add `"totp-rs"`, `"qrcode"`, `"aes-gcm"` to the `server` feature list in `[features]`.

**Step 3: Add feature flags**

In `crates/shared-types/src/feature_flags.rs`, add to the `FeatureFlags` struct:

```rust
#[serde(default)]
pub mfa: bool,
#[serde(default)]
pub mfa_required: bool,
#[serde(default)]
pub public_portal: bool,
```

**Step 4: Verify compilation**

Run: `cargo check -p server -p shared-types`
Expected: Compiles with warnings only (no errors)

**Step 5: Commit**

```bash
git add Cargo.toml crates/server/Cargo.toml crates/shared-types/src/feature_flags.rs
git commit -m "chore: add MFA dependencies and feature flags (totp-rs, aes-gcm, qrcode)"
```

---

### Task 2: MFA Database Migrations

**Files:**
- Create: `migrations/20260301000092_add_mfa_to_users.sql`
- Create: `migrations/20260301000092_add_mfa_to_users.down.sql`
- Create: `migrations/20260301000093_create_mfa_recovery_codes.sql`
- Create: `migrations/20260301000093_create_mfa_recovery_codes.down.sql`

**Step 1: Create users MFA columns migration**

`migrations/20260301000092_add_mfa_to_users.sql`:

```sql
ALTER TABLE users
    ADD COLUMN totp_secret_enc BYTEA,
    ADD COLUMN mfa_enabled BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN mfa_enforced_at TIMESTAMPTZ;
```

Down migration:

```sql
ALTER TABLE users
    DROP COLUMN IF EXISTS totp_secret_enc,
    DROP COLUMN IF EXISTS mfa_enabled,
    DROP COLUMN IF EXISTS mfa_enforced_at;
```

**Step 2: Create recovery codes table migration**

`migrations/20260301000093_create_mfa_recovery_codes.sql`:

```sql
CREATE TABLE IF NOT EXISTS mfa_recovery_codes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_hash   TEXT NOT NULL,
    used        BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_mfa_recovery_codes_user ON mfa_recovery_codes(user_id);
```

Down migration:

```sql
DROP TABLE IF EXISTS mfa_recovery_codes;
```

**Step 3: Run migrations on dev DB**

Run: `sqlx migrate run`
Expected: Both migrations applied

**Step 4: Verify compilation (sqlx macro re-check)**

Run: `cargo check -p server`
Expected: Compiles (sqlx macros see new columns)

**Step 5: Commit**

```bash
git add migrations/20260301000092* migrations/20260301000093*
git commit -m "feat(mfa): add users MFA columns and recovery codes table"
```

---

### Task 3: MFA Encryption Module

**Files:**
- Create: `crates/server/src/auth/mfa_crypto.rs`
- Modify: `crates/server/src/auth/mod.rs`

**Step 1: Write tests for encryption round-trip**

Create `crates/tests/src/mfa_crypto_tests.rs` and add `mod mfa_crypto_tests;` to `crates/tests/src/lib.rs`:

```rust
use server::auth::mfa_crypto;

#[test]
fn encrypt_decrypt_round_trip() {
    std::env::set_var("MFA_ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    let plaintext = b"JBSWY3DPEHPK3PXP";
    let encrypted = mfa_crypto::encrypt_totp_secret(plaintext).expect("encrypt failed");
    let decrypted = mfa_crypto::decrypt_totp_secret(&encrypted).expect("decrypt failed");
    assert_eq!(plaintext.as_slice(), decrypted.as_slice());
}

#[test]
fn different_encryptions_produce_different_ciphertext() {
    std::env::set_var("MFA_ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    let plaintext = b"JBSWY3DPEHPK3PXP";
    let a = mfa_crypto::encrypt_totp_secret(plaintext).unwrap();
    let b = mfa_crypto::encrypt_totp_secret(plaintext).unwrap();
    assert_ne!(a, b, "nonces should differ");
}

#[test]
fn decrypt_with_wrong_key_fails() {
    std::env::set_var("MFA_ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    let encrypted = mfa_crypto::encrypt_totp_secret(b"secret").unwrap();
    std::env::set_var("MFA_ENCRYPTION_KEY", "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789");
    assert!(mfa_crypto::decrypt_totp_secret(&encrypted).is_err());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p tests mfa_crypto -- --test-threads=1`
Expected: FAIL — module doesn't exist

**Step 3: Implement encryption module**

Create `crates/server/src/auth/mfa_crypto.rs`:

```rust
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, AeadCore, Key, Nonce,
};

fn get_key() -> Key<Aes256Gcm> {
    let hex_key = std::env::var("MFA_ENCRYPTION_KEY")
        .expect("MFA_ENCRYPTION_KEY must be set (64 hex chars = 32 bytes)");
    let key_bytes = hex::decode(&hex_key)
        .expect("MFA_ENCRYPTION_KEY must be valid hex");
    assert_eq!(key_bytes.len(), 32, "MFA_ENCRYPTION_KEY must be 32 bytes (64 hex chars)");
    *Key::<Aes256Gcm>::from_slice(&key_bytes)
}

/// Encrypt a TOTP secret. Returns nonce (12 bytes) || ciphertext.
pub fn encrypt_totp_secret(plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(&get_key());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {e}"))?;
    let mut result = nonce.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

/// Decrypt a TOTP secret. Input is nonce (12 bytes) || ciphertext.
pub fn decrypt_totp_secret(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 12 {
        return Err("Ciphertext too short".to_string());
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(&get_key());
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {e}"))
}
```

Add `pub mod mfa_crypto;` to `crates/server/src/auth/mod.rs`.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p tests mfa_crypto -- --test-threads=1`
Expected: 3 passed

**Step 5: Commit**

```bash
git add crates/server/src/auth/mfa_crypto.rs crates/server/src/auth/mod.rs crates/tests/src/mfa_crypto_tests.rs crates/tests/src/lib.rs
git commit -m "feat(mfa): AES-256-GCM encryption for TOTP secrets"
```

---

### Task 4: MFA Setup and Verify Server Functions

**Files:**
- Create: `crates/server/src/auth/mfa.rs`
- Modify: `crates/server/src/auth/mod.rs`
- Create: `crates/tests/src/mfa_setup_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Write tests for MFA setup flow**

Create `crates/tests/src/mfa_setup_tests.rs`:

```rust
use crate::common::*;

#[tokio::test]
async fn mfa_setup_returns_qr_and_secret() {
    let (app, _pool, _guard) = test_app().await;
    let token = create_test_token("admin");
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/mfa/setup")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = parse_body(resp).await;
    assert!(body["qr_svg"].is_string());
    assert!(body["secret"].is_string());
    assert!(!body["secret"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn mfa_verify_with_wrong_code_returns_400() {
    let (app, _pool, _guard) = test_app().await;
    let token = create_test_token("admin");
    // Setup first
    let _ = app.clone().oneshot(
        axum::http::Request::builder()
            .method("POST")
            .uri("/api/mfa/setup")
            .header("authorization", format!("Bearer {token}"))
            .body(axum::body::Body::empty())
            .unwrap(),
    ).await.unwrap();
    // Verify with wrong code
    let (status, _) = post_json_authed(&app, "/api/mfa/verify", serde_json::json!({"code": "000000"}), &token).await;
    assert_eq!(status, 400);
}

#[tokio::test]
async fn mfa_disable_requires_valid_code() {
    let (app, _pool, _guard) = test_app().await;
    let token = create_test_token("admin");
    let (status, _) = post_json_authed(&app, "/api/mfa/disable", serde_json::json!({"code": "000000"}), &token).await;
    // Should fail since MFA not enabled or code wrong
    assert!(status == 400 || status == 404);
}
```

Note: You may need to add a `post_json_authed` helper to `common.rs` if one doesn't exist, or adapt the existing `post_json` to accept a bearer token. Check the pattern used for authenticated requests in existing tests.

**Step 2: Implement MFA server functions**

Create `crates/server/src/auth/mfa.rs`:

```rust
use crate::auth::{mfa_crypto, password};
use crate::db::get_db;
use crate::error_convert::SqlxErrorExt;
use shared_types::AppError;
use totp_rs::{Algorithm, Secret, TOTP};

const ISSUER: &str = "Lexodus";

/// Generate a new TOTP secret and QR code for the authenticated user.
/// Stores the encrypted secret in users.totp_secret_enc but does NOT enable MFA yet.
pub async fn setup_mfa(user_id: i64) -> Result<(String, String), AppError> {
    let pool = get_db().await;
    let secret = Secret::generate_secret();
    let secret_base32 = secret.to_encoded().to_string();

    // Fetch user email for TOTP account name
    let user = sqlx::query!("SELECT email FROM users WHERE id = $1", user_id)
        .fetch_one(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes().map_err(|e| AppError::BadRequest(e.to_string()))?,
    )
    .map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Generate QR code SVG
    let qr_url = totp.get_url(&user.email, ISSUER);
    let qr_svg = qrcode::QrCode::new(qr_url.as_bytes())
        .map_err(|e| AppError::Internal(format!("QR generation failed: {e}")))?
        .render::<qrcode::render::svg::Color>()
        .build();

    // Encrypt and store secret (MFA not yet enabled — needs verify step)
    let encrypted = mfa_crypto::encrypt_totp_secret(secret_base32.as_bytes())
        .map_err(|e| AppError::Internal(e))?;

    sqlx::query!(
        "UPDATE users SET totp_secret_enc = $1 WHERE id = $2",
        &encrypted,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((qr_svg, secret_base32))
}

/// Verify a TOTP code and enable MFA for the user. Also generates recovery codes.
pub async fn verify_and_enable(user_id: i64, code: &str) -> Result<Vec<String>, AppError> {
    let pool = get_db().await;

    let user = sqlx::query!(
        "SELECT totp_secret_enc, mfa_enabled FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let enc = user.totp_secret_enc
        .ok_or(AppError::BadRequest("MFA not set up — call /api/mfa/setup first".into()))?;

    let secret_bytes = mfa_crypto::decrypt_totp_secret(&enc)
        .map_err(|e| AppError::Internal(e))?;
    let secret_str = String::from_utf8(secret_bytes)
        .map_err(|_| AppError::Internal("Invalid TOTP secret encoding".into()))?;

    let secret = Secret::Encoded(secret_str);
    let totp = TOTP::new(
        Algorithm::SHA1, 6, 1, 30,
        secret.to_bytes().map_err(|e| AppError::BadRequest(e.to_string()))?,
    )
    .map_err(|e| AppError::BadRequest(e.to_string()))?;

    if !totp.check_current(code).map_err(|e| AppError::Internal(e.to_string()))? {
        return Err(AppError::BadRequest("Invalid TOTP code".into()));
    }

    // Enable MFA
    sqlx::query!(
        "UPDATE users SET mfa_enabled = true, mfa_enforced_at = NOW() WHERE id = $1",
        user_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    // Generate 8 recovery codes
    let mut codes = Vec::with_capacity(8);
    for _ in 0..8 {
        let code = format!(
            "{:04x}-{:04x}",
            rand::random::<u16>(),
            rand::random::<u16>(),
        );
        let hash = password::hash_password(&code)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        sqlx::query!(
            "INSERT INTO mfa_recovery_codes (user_id, code_hash) VALUES ($1, $2)",
            user_id,
            hash,
        )
        .execute(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?;
        codes.push(code);
    }

    Ok(codes)
}

/// Validate a TOTP code for login challenge. Returns true if valid.
pub async fn validate_code(user_id: i64, code: &str) -> Result<bool, AppError> {
    let pool = get_db().await;

    let user = sqlx::query!(
        "SELECT totp_secret_enc FROM users WHERE id = $1 AND mfa_enabled = true",
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?
    .ok_or(AppError::NotFound)?;

    let enc = user.totp_secret_enc
        .ok_or(AppError::Internal("MFA enabled but no secret stored".into()))?;

    let secret_bytes = mfa_crypto::decrypt_totp_secret(&enc)
        .map_err(|e| AppError::Internal(e))?;
    let secret_str = String::from_utf8(secret_bytes)
        .map_err(|_| AppError::Internal("Invalid TOTP secret encoding".into()))?;

    let secret = Secret::Encoded(secret_str);
    let totp = TOTP::new(
        Algorithm::SHA1, 6, 1, 30,
        secret.to_bytes().map_err(|e| AppError::Internal(e.to_string()))?,
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(totp.check_current(code).map_err(|e| AppError::Internal(e.to_string()))?)
}

/// Check a recovery code. If valid, marks it as used and returns true.
pub async fn use_recovery_code(user_id: i64, code: &str) -> Result<bool, AppError> {
    let pool = get_db().await;

    let rows = sqlx::query!(
        "SELECT id, code_hash FROM mfa_recovery_codes WHERE user_id = $1 AND used = false",
        user_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    for row in rows {
        if password::verify_password(code, &row.code_hash) {
            sqlx::query!(
                "UPDATE mfa_recovery_codes SET used = true WHERE id = $1",
                row.id,
            )
            .execute(pool)
            .await
            .map_err(SqlxErrorExt::into_app_error)?;
            return Ok(true);
        }
    }

    Ok(false)
}

/// Disable MFA for a user. Requires valid TOTP or recovery code.
pub async fn disable_mfa(user_id: i64, code: &str) -> Result<(), AppError> {
    let valid_totp = validate_code(user_id, code).await.unwrap_or(false);
    let valid_recovery = if !valid_totp {
        use_recovery_code(user_id, code).await.unwrap_or(false)
    } else {
        false
    };

    if !valid_totp && !valid_recovery {
        return Err(AppError::BadRequest("Invalid code".into()));
    }

    let pool = get_db().await;
    sqlx::query!(
        "UPDATE users SET totp_secret_enc = NULL, mfa_enabled = false, mfa_enforced_at = NULL WHERE id = $1",
        user_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    sqlx::query!("DELETE FROM mfa_recovery_codes WHERE user_id = $1", user_id)
        .execute(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?;

    Ok(())
}
```

Add `pub mod mfa;` to `crates/server/src/auth/mod.rs`.

**Step 3: Wire up REST endpoints**

Create `crates/server/src/rest/mfa.rs` with routes:
- `POST /api/mfa/setup` — requires auth, calls `mfa::setup_mfa`, returns `{ qr_svg, secret }`
- `POST /api/mfa/verify` — requires auth, body `{ code }`, calls `mfa::verify_and_enable`, returns `{ recovery_codes: [...] }`
- `POST /api/mfa/disable` — requires auth, body `{ code }`, calls `mfa::disable_mfa`
- `POST /api/mfa/challenge` — body `{ challenge_token, code }`, validates MFA and issues JWT

Add `mod mfa;` to `crates/server/src/rest/mod.rs` and merge the router.

**Step 4: Run tests**

Run: `cargo test -p tests mfa -- --test-threads=1`
Expected: All MFA tests pass

**Step 5: Commit**

```bash
git add crates/server/src/auth/mfa.rs crates/server/src/auth/mod.rs crates/server/src/rest/mfa.rs crates/server/src/rest/mod.rs crates/tests/src/mfa_setup_tests.rs crates/tests/src/lib.rs
git commit -m "feat(mfa): TOTP setup, verify, disable, and login challenge endpoints"
```

---

### Task 5: Modify Login Flow for MFA Challenge

**Files:**
- Modify: `crates/server/src/api.rs` (login function, ~line 625-743)
- Modify: `crates/server/src/auth/jwt.rs` (add challenge token type)
- Create: `crates/tests/src/mfa_login_tests.rs`

**Step 1: Write tests**

```rust
#[tokio::test]
async fn login_with_mfa_enabled_returns_challenge() {
    // Setup: create user, enable MFA, then attempt login
    // Expected: 200 with { mfa_required: true, challenge_token: "..." } instead of JWT cookies
}

#[tokio::test]
async fn mfa_challenge_with_valid_code_returns_jwt() {
    // Use challenge_token + valid TOTP code
    // Expected: 200 with JWT cookies set
}

#[tokio::test]
async fn mfa_challenge_with_recovery_code_works() {
    // Use challenge_token + recovery code
    // Expected: 200 with JWT cookies, recovery code marked used
}

#[tokio::test]
async fn mfa_challenge_expired_token_returns_401() {
    // Use expired challenge token
    // Expected: 401
}
```

**Step 2: Add challenge token to JWT module**

In `crates/server/src/auth/jwt.rs`, add:

```rust
pub fn create_mfa_challenge_token(user_id: i64) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = (chrono::Utc::now() + chrono::Duration::minutes(5)).timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        email: String::new(),
        role: String::new(),
        tier: String::new(),
        exp,
        iat: chrono::Utc::now().timestamp() as usize,
        jti: uuid::Uuid::new_v4().to_string(),
        typ: "mfa_challenge".to_string(),
        court_roles: std::collections::HashMap::new(),
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret().as_bytes()))
}

pub fn validate_mfa_challenge_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.set_required_spec_claims(&["exp", "sub", "typ"]);
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret().as_bytes()), &validation)?;
    if data.claims.typ != "mfa_challenge" {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
    }
    Ok(data.claims)
}
```

**Step 3: Modify login function**

In `crates/server/src/api.rs`, after password verification (around line 673), add MFA check:

```rust
// After password is verified as valid...
let mfa_enabled: bool = sqlx::query_scalar!(
    "SELECT mfa_enabled FROM users WHERE id = $1", user_id
)
.fetch_one(pool)
.await
.unwrap_or(false);

if mfa_enabled {
    let challenge_token = jwt::create_mfa_challenge_token(user_id)
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    return Ok(serde_json::json!({
        "mfa_required": true,
        "challenge_token": challenge_token,
    }).to_string());
}

// Existing JWT creation continues here for non-MFA users...
```

**Step 4: Run all tests**

Run: `cargo test -p tests -- --test-threads=1`
Expected: All tests pass (existing login tests still work for non-MFA users, new MFA tests pass)

**Step 5: Commit**

```bash
git commit -m "feat(mfa): login challenge flow — MFA users get challenge token instead of JWT"
```

---

### Task 6: MFA UI Pages

**Files:**
- Create: `crates/app/src/routes/settings/mfa.rs`
- Modify: `crates/app/src/routes/settings/mod.rs`
- Create: `crates/app/src/routes/mfa_challenge.rs`
- Modify: `crates/app/src/routes/mod.rs`

**Step 1: MFA Settings component**

Create `crates/app/src/routes/settings/mfa.rs` with a component that:
- Shows MFA status (enabled/disabled)
- "Enable MFA" button → calls `server::api::mfa_setup()` → shows QR code + manual key
- 6-digit input for verification → calls `server::api::mfa_verify(code)` → shows recovery codes once
- "Disable MFA" button → requires code input → calls `server::api::mfa_disable(code)`

Use existing shared-ui components: `Card`, `CardContent`, `Button`, `Input`, `Badge`.

**Step 2: MFA Challenge page**

Create `crates/app/src/routes/mfa_challenge.rs`:
- Shown after login when `mfa_required: true` returned
- 6-digit TOTP input
- "Use recovery code" link toggles to text input
- Submits to `server::api::mfa_challenge(challenge_token, code)`
- On success: stores JWT, navigates to dashboard

**Step 3: Add route**

In `crates/app/src/routes/mod.rs`, add OUTSIDE AuthGuard layout:

```rust
#[route("/mfa-challenge")]
MfaChallenge {},
```

**Step 4: Verify compilation**

Run: `cargo check -p app`
Expected: Compiles

**Step 5: Commit**

```bash
git commit -m "feat(mfa): settings UI for MFA setup and login challenge page"
```

---

## Phase 2: Filing Fees (Stripe)

### Task 7: Filing Fee Database Schema

**Files:**
- Create: `migrations/20260301000094_create_filing_fee_schedule.sql`
- Create: `migrations/20260301000094_create_filing_fee_schedule.down.sql`
- Create: `migrations/20260301000095_create_filing_payments.sql`
- Create: `migrations/20260301000095_create_filing_payments.down.sql`
- Create: `migrations/20260301000096_add_fee_status_to_cases.sql`
- Create: `migrations/20260301000096_add_fee_status_to_cases.down.sql`

**Step 1: Create filing_fee_schedule table**

```sql
CREATE TABLE IF NOT EXISTS filing_fee_schedule (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL REFERENCES courts(id),
    entry_type      TEXT NOT NULL,
    fee_cents       INTEGER NOT NULL CHECK (fee_cents >= 0),
    description     TEXT NOT NULL DEFAULT '',
    effective_date  DATE NOT NULL DEFAULT CURRENT_DATE,
    active          BOOLEAN NOT NULL DEFAULT true,
    UNIQUE(court_id, entry_type, effective_date)
);

CREATE INDEX idx_filing_fee_schedule_court ON filing_fee_schedule(court_id);
CREATE INDEX idx_filing_fee_schedule_lookup ON filing_fee_schedule(court_id, entry_type, active);
```

**Step 2: Create filing_payments table**

```sql
CREATE TABLE IF NOT EXISTS filing_payments (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id                    TEXT NOT NULL REFERENCES courts(id),
    filing_id                   UUID REFERENCES filings(id),
    user_id                     BIGINT NOT NULL REFERENCES users(id),
    stripe_checkout_session_id  TEXT,
    stripe_payment_intent_id    TEXT,
    amount_cents                INTEGER NOT NULL,
    status                      TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'paid', 'refunded', 'waived', 'failed')),
    fee_waiver_type             TEXT CHECK (fee_waiver_type IN ('ifp', 'government', 'cja')),
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    paid_at                     TIMESTAMPTZ
);

CREATE INDEX idx_filing_payments_court ON filing_payments(court_id);
CREATE INDEX idx_filing_payments_filing ON filing_payments(filing_id);
CREATE INDEX idx_filing_payments_user ON filing_payments(user_id);
CREATE INDEX idx_filing_payments_status ON filing_payments(court_id, status);
```

**Step 3: Add fee_status to cases**

```sql
ALTER TABLE civil_cases ADD COLUMN fee_status TEXT NOT NULL DEFAULT 'pending'
    CHECK (fee_status IN ('pending', 'paid', 'ifp', 'government', 'waived'));
ALTER TABLE criminal_cases ADD COLUMN fee_status TEXT NOT NULL DEFAULT 'pending'
    CHECK (fee_status IN ('pending', 'paid', 'ifp', 'government', 'waived'));
```

**Step 4: Run migrations, verify compilation**

Run: `sqlx migrate run && cargo check -p server`

**Step 5: Commit**

```bash
git add migrations/20260301000094* migrations/20260301000095* migrations/20260301000096*
git commit -m "feat(fees): filing fee schedule, payments, and fee_status schema"
```

---

### Task 8: Seed Filing Fee Schedule

**Files:**
- Create: `migrations/20260301000097_seed_filing_fee_schedule.sql`
- Create: `migrations/20260301000097_seed_filing_fee_schedule.down.sql`

**Step 1: Seed Judicial Conference fee schedule for both districts**

```sql
-- Federal filing fee schedule (Judicial Conference, effective 2026)
-- Applied to both test districts

INSERT INTO filing_fee_schedule (court_id, entry_type, fee_cents, description) VALUES
-- district9
('district9', 'complaint',        40200, 'Civil case filing fee'),
('district9', 'notice_of_appeal', 60500, 'Appeal filing fee'),
('district9', 'motion',            0,    'Motion (no fee)'),
('district9', 'habeas_corpus',      500, 'Habeas corpus petition'),
('district9', 'miscellaneous',    49000, 'Miscellaneous filing'),
-- district12
('district12', 'complaint',        40200, 'Civil case filing fee'),
('district12', 'notice_of_appeal', 60500, 'Appeal filing fee'),
('district12', 'motion',            0,    'Motion (no fee)'),
('district12', 'habeas_corpus',      500, 'Habeas corpus petition'),
('district12', 'miscellaneous',    49000, 'Miscellaneous filing')
ON CONFLICT DO NOTHING;
```

Down: `DELETE FROM filing_fee_schedule WHERE court_id IN ('district9', 'district12');`

**Step 2: Run migration, commit**

```bash
sqlx migrate run
git add migrations/20260301000097*
git commit -m "feat(fees): seed Judicial Conference fee schedule for test districts"
```

---

### Task 9: Filing Fee Repo and REST Endpoints

**Files:**
- Create: `crates/server/src/repo/filing_payment.rs`
- Modify: `crates/server/src/repo/mod.rs`
- Create: `crates/server/src/rest/filing_fee.rs`
- Modify: `crates/server/src/rest/mod.rs`
- Create: `crates/tests/src/filing_fee_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Write tests**

```rust
#[tokio::test]
async fn lookup_filing_fee_returns_amount() {
    let (app, _pool, _guard) = test_app().await;
    let (status, body) = get_with_court(&app, "/api/filing-fees/lookup?entry_type=complaint", "district9").await;
    assert_eq!(status, 200);
    assert_eq!(body["fee_cents"].as_i64(), Some(40200));
}

#[tokio::test]
async fn lookup_filing_fee_zero_for_motions() {
    let (app, _pool, _guard) = test_app().await;
    let (status, body) = get_with_court(&app, "/api/filing-fees/lookup?entry_type=motion", "district9").await;
    assert_eq!(status, 200);
    assert_eq!(body["fee_cents"].as_i64(), Some(0));
}

#[tokio::test]
async fn lookup_filing_fee_unknown_type_returns_404() {
    let (app, _pool, _guard) = test_app().await;
    let (status, _) = get_with_court(&app, "/api/filing-fees/lookup?entry_type=nonexistent", "district9").await;
    assert_eq!(status, 404);
}
```

**Step 2: Implement repo module**

`crates/server/src/repo/filing_payment.rs` with functions:
- `lookup_fee(pool, court_id, entry_type) -> Result<Option<i32>, AppError>` — query `filing_fee_schedule`
- `create_payment(pool, court_id, filing_id, user_id, amount_cents, waiver_type) -> Result<FilingPayment, AppError>`
- `update_payment_status(pool, payment_id, status, stripe_payment_intent_id) -> Result<(), AppError>`
- `get_payment_by_filing(pool, filing_id) -> Result<Option<FilingPayment>, AppError>`

**Step 3: Implement REST endpoints**

`crates/server/src/rest/filing_fee.rs`:
- `GET /api/filing-fees/lookup?entry_type=X` — lookup fee amount
- `GET /api/filing-fees/schedule` — list all fees for court
- `POST /api/filing-fees/:filing_id/checkout` — create Stripe Checkout for a pending filing
- `POST /api/filing-fees/:payment_id/refund` — admin-only refund

**Step 4: Run tests, commit**

Run: `cargo test -p tests filing_fee -- --test-threads=1`

```bash
git commit -m "feat(fees): filing fee lookup, payments repo, and REST endpoints"
```

---

### Task 10: Wire Filing Fees into Filing Submission

**Files:**
- Modify: `crates/server/src/rest/filing.rs` (~line 69-112)
- Modify: `crates/server/src/stripe/sync.rs`
- Create: `crates/tests/src/filing_payment_flow_tests.rs`

**Step 1: Modify filing submission**

In `submit_filing()`, after creating the filing but before returning:

1. Look up fee: `filing_payment::lookup_fee(pool, court_id, entry_type)`
2. If fee > 0 and no waiver:
   - Create `filing_payments` row with `status = 'pending'`
   - Create Stripe Checkout Session using existing `create_onetime_checkout()` with metadata `{ filing_id, court_id }`
   - Return `{ status: "pending_payment", checkout_url: "..." }` instead of the normal response
3. If fee == 0 or waiver applies:
   - Create `filing_payments` row with `status = 'waived'`
   - Proceed normally

**Step 2: Extend webhook handler**

In `crates/server/src/stripe/sync.rs`, in `handle_checkout_completed()`:
- Check metadata for `filing_id` key
- If present: update `filing_payments.status = 'paid'`, advance filing status, trigger NEF
- If absent: existing subscription logic continues

**Step 3: Write integration tests**

Test the full flow: submit filing → get checkout URL → simulate webhook → verify filing advances.

**Step 4: Run all tests, commit**

Run: `cargo test -p tests -- --test-threads=1`

```bash
git commit -m "feat(fees): wire filing fees into submission flow with Stripe checkout"
```

---

## Phase 3: Public Access Portal

### Task 11: Public API Endpoints (CourtListener-Compatible)

**Files:**
- Create: `crates/server/src/rest/public_api.rs`
- Modify: `crates/server/src/rest/mod.rs`
- Create: `crates/tests/src/public_api_tests.rs`

**Step 1: Write tests**

```rust
#[tokio::test]
async fn public_search_returns_cases() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "9:26-cr-99901").await;
    // No auth header — public endpoint
    let resp = app.clone().oneshot(
        axum::http::Request::builder()
            .uri("/api/v1/public/search/?q=United+States&court=district9")
            .body(axum::body::Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = parse_body(resp).await;
    assert!(body["results"].is_array());
}

#[tokio::test]
async fn public_search_excludes_sealed_cases() {
    // Create sealed case, verify it doesn't appear in public search
}

#[tokio::test]
async fn public_docket_returns_entries() {
    // Create case + docket entries, fetch via public API
}

#[tokio::test]
async fn public_docket_excludes_sealed_entries() {
    // Sealed entries filtered out
}

#[tokio::test]
async fn public_opinions_returns_published_only() {
    // Only published opinions, not drafts
}
```

**Step 2: Implement public API routes**

`crates/server/src/rest/public_api.rs`:

```rust
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/public/search/", get(public_search))
        .route("/api/v1/public/dockets/:id/", get(public_docket))
        .route("/api/v1/public/docket-entries/", get(public_docket_entries))
        .route("/api/v1/public/opinions/", get(public_opinions))
        .route("/api/v1/public/people/", get(public_people))
        .route("/api/v1/public/courts/", get(public_courts))
}
```

Each handler:
- No auth required (no `AuthRequired` extractor)
- Adds `WHERE is_sealed = false` to all queries
- Maps internal field names to CourtListener names in response:
  - `case_number` → `docket_number`
  - `court_id` → `court`
  - `title` → `case_name`
  - Adds `absolute_url` pointing to `/public/case/:id`
- Returns paginated results with `count`, `next`, `previous`, `results` (CourtListener pattern)

**Step 3: Feature-flag the router**

In `crates/server/src/rest/mod.rs`, conditionally merge:

```rust
if flags.public_portal {
    router = router.merge(public_api::public_router());
}
```

**Step 4: Run tests, commit**

```bash
git commit -m "feat(public): CourtListener-compatible public API endpoints"
```

---

### Task 12: Public Rate Limiting (Per-IP)

**Files:**
- Modify: `crates/server/src/rate_limit.rs`
- Modify: `crates/server/src/rest/public_api.rs`

**Step 1: Add per-IP rate limit middleware**

Create a second `RateLimitState` instance configured for public endpoints:
- 60 req/min per IP (anonymous)
- Key extraction: `X-Forwarded-For` header or `ConnectInfo<SocketAddr>`

**Step 2: Apply to public router**

```rust
pub fn public_router() -> Router<AppState> {
    let public_rate_limit = RateLimitState::new(60, Duration::from_secs(60));
    Router::new()
        .route(...)
        .layer(axum::middleware::from_fn_with_state(
            public_rate_limit,
            rate_limit_by_ip,
        ))
}
```

**Step 3: Test rate limiting**

```rust
#[tokio::test]
async fn public_api_rate_limited_after_60_requests() {
    // Send 61 requests, verify 429 on the 61st
}
```

**Step 4: Commit**

```bash
git commit -m "feat(public): per-IP rate limiting for public API (60 req/min)"
```

---

### Task 13: Bulk Data Export Endpoint

**Files:**
- Create: `crates/server/src/rest/public_bulk.rs`
- Modify: `crates/server/src/rest/public_api.rs`

**Step 1: Implement bulk export**

`GET /api/v1/public/bulk/` returns JSON listing available bulk files:
```json
{
  "cases": "/api/v1/public/bulk/cases.jsonl",
  "docket_entries": "/api/v1/public/bulk/docket-entries.jsonl",
  "opinions": "/api/v1/public/bulk/opinions.jsonl",
  "generated_at": "2026-02-19T00:00:00Z"
}
```

Each bulk endpoint streams JSONL (one JSON object per line) from a database query with `WHERE is_sealed = false`. Rate limited to 1 req/hour per IP.

**Step 2: Test, commit**

```bash
git commit -m "feat(public): bulk JSONL export endpoints for Free Law Project integration"
```

---

### Task 14: Public UI Routes

**Files:**
- Create: `crates/app/src/routes/public/mod.rs`
- Create: `crates/app/src/routes/public/search.rs`
- Create: `crates/app/src/routes/public/case_view.rs`
- Create: `crates/app/src/routes/public/docket_view.rs`
- Create: `crates/app/src/routes/public/opinion_list.rs`
- Modify: `crates/app/src/routes/mod.rs`

**Step 1: Add public routes to Route enum**

In `crates/app/src/routes/mod.rs`, add OUTSIDE the `#[layout(AuthGuard)]` block:

```rust
#[route("/public/search")]
PublicSearch {},

#[route("/public/case/:id")]
PublicCaseView { id: String },

#[route("/public/case/:case_id/docket")]
PublicDocketView { case_id: String },

#[route("/public/opinions")]
PublicOpinionList {},
```

**Step 2: Implement components**

Each public page:
- Minimal layout (no sidebar, no auth-required features)
- Uses shared-ui components (Card, DataTable, Badge, Pagination, SearchBar)
- Calls `/api/v1/public/*` endpoints
- Links to document downloads
- Shows "Login to File" call-to-action for attorneys

**Step 3: Verify compilation, commit**

Run: `cargo check -p app`

```bash
git commit -m "feat(public): public UI pages for case search, docket view, and opinions"
```

---

### Task 15: Final Integration Test and Cleanup

**Files:**
- Modify: `crates/tests/src/common.rs` (add `mfa_recovery_codes`, `filing_fee_schedule`, `filing_payments` to TRUNCATE)
- Run full test suite

**Step 1: Update test TRUNCATE**

Add to the TRUNCATE list in `common.rs`:
```sql
mfa_recovery_codes, filing_fee_schedule, filing_payments
```

**Step 2: Run full test suite**

Run: `cargo test -p tests -- --test-threads=1`
Expected: All tests pass (existing 388 + new MFA + filing fee + public API tests)

**Step 3: Verify compilation of all crates**

Run: `cargo check --workspace`

**Step 4: Final commit**

```bash
git commit -m "chore: update test truncation and verify full test suite"
```

---

## Summary

| Phase | Tasks | New Tables | New Endpoints | New Crates |
|---|---|---|---|---|
| 1. MFA | 1-6 | `mfa_recovery_codes` + 3 columns on `users` | 4 (`/api/mfa/*`) | `totp-rs`, `qrcode`, `aes-gcm` |
| 2. Filing Fees | 7-10 | `filing_fee_schedule`, `filing_payments` + `fee_status` column | 4 (`/api/filing-fees/*`) | None |
| 3. Public Portal | 11-14 | None | 8 (`/api/v1/public/*`) | None |
| Cleanup | 15 | None | None | None |

**Total: 15 tasks, 3 new tables, 3 new columns, 16 new endpoints, 3 new crates**
