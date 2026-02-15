use rand::Rng;
use sha2::{Digest, Sha256};
use shared_types::AppError;
use sqlx::{Pool, Postgres};

// --- Environment helpers ---

fn twilio_account_sid() -> Result<String, String> {
    std::env::var("TWILIO_ACCOUNT_SID")
        .map_err(|_| "TWILIO_ACCOUNT_SID is not configured".to_string())
}

fn twilio_auth_token() -> Result<String, String> {
    std::env::var("TWILIO_AUTH_TOKEN")
        .map_err(|_| "TWILIO_AUTH_TOKEN is not configured".to_string())
}

fn twilio_from_number() -> Result<String, String> {
    std::env::var("TWILIO_FROM_NUMBER")
        .map_err(|_| "TWILIO_FROM_NUMBER is not configured".to_string())
}

fn app_name() -> String {
    std::env::var("APP_NAME").unwrap_or_else(|_| "Lexodus".to_string())
}

// --- Core SMS sending ---

/// Send an SMS message via Twilio REST API.
#[tracing::instrument(skip(message))]
pub async fn send_sms(to: &str, message: &str) -> Result<(), String> {
    let sid = twilio_account_sid()?;
    let auth_token = twilio_auth_token()?;
    let from = twilio_from_number()?;

    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        sid
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .basic_auth(&sid, Some(&auth_token))
        .form(&[
            ("From", from),
            ("To", to.to_string()),
            ("Body", message.to_string()),
        ])
        .send()
        .await
        .map_err(|e| format!("Twilio request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Twilio API error ({}): {}", status, body));
    }

    tracing::info!(to = to, "SMS sent successfully");
    Ok(())
}

// --- Verification ---

const MAX_CODES_PER_HOUR: i64 = 3;
const MAX_VERIFY_ATTEMPTS: i32 = 5;
const CODE_EXPIRY_MINUTES: i64 = 10;

/// Send a verification code to a phone number.
/// Rate-limited to MAX_CODES_PER_HOUR per phone number.
#[tracing::instrument(skip(pool))]
pub async fn send_verification_code(
    pool: &Pool<Postgres>,
    user_id: i64,
    phone: &str,
) -> Result<(), AppError> {
    // Rate limit check
    let recent_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM sms_verifications
           WHERE phone_number = $1 AND created_at > NOW() - INTERVAL '1 hour'"#,
        phone
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?
    .unwrap_or(0);

    if recent_count >= MAX_CODES_PER_HOUR {
        return Err(AppError::validation(
            "Too many verification codes sent. Please try again later.",
            Default::default(),
        ));
    }

    // Generate 6-digit code
    let code: u32 = rand::thread_rng().gen_range(100_000..1_000_000);
    let code_str = code.to_string();
    let code_hash = hash_code(&code_str);

    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(CODE_EXPIRY_MINUTES);

    // Store hashed code
    sqlx::query!(
        "INSERT INTO sms_verifications (phone_number, code_hash, expires_at) VALUES ($1, $2, $3)",
        phone,
        code_hash,
        expires_at
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    // Send SMS
    let message = format!("Your {} verification code is: {}", app_name(), code_str);
    send_sms(phone, &message)
        .await
        .map_err(|e| AppError::internal(e))?;

    Ok(())
}

/// Verify a phone number with a code.
/// Updates user's phone_number and phone_verified on success.
#[tracing::instrument(skip(pool, code))]
pub async fn verify_code(
    pool: &Pool<Postgres>,
    user_id: i64,
    phone: &str,
    code: &str,
) -> Result<(), AppError> {
    // Find the latest unexpired, unverified code for this phone
    let record = sqlx::query!(
        r#"SELECT id, code_hash, attempts FROM sms_verifications
           WHERE phone_number = $1 AND expires_at > NOW() AND verified = false
           ORDER BY created_at DESC LIMIT 1"#,
        phone
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?
    .ok_or_else(|| {
        AppError::validation(
            "No pending verification code found. Please request a new one.",
            Default::default(),
        )
    })?;

    // Check attempt limit
    if record.attempts >= MAX_VERIFY_ATTEMPTS {
        return Err(AppError::validation(
            "Too many failed attempts. Please request a new code.",
            Default::default(),
        ));
    }

    // Increment attempts
    sqlx::query!(
        "UPDATE sms_verifications SET attempts = attempts + 1 WHERE id = $1",
        record.id
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    // Verify code hash
    let code_hash = hash_code(code);
    if code_hash != record.code_hash {
        return Err(AppError::validation(
            "Invalid verification code.",
            Default::default(),
        ));
    }

    // Mark as verified
    sqlx::query!(
        "UPDATE sms_verifications SET verified = true WHERE id = $1",
        record.id
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    // Update user's phone info
    sqlx::query!(
        "UPDATE users SET phone_number = $2, phone_verified = true WHERE id = $1",
        user_id,
        phone
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_code_consistent_sha256() {
        let hash = hash_code("123456");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(hash, hash_code("123456"));
    }

    #[test]
    fn hash_code_different_inputs_differ() {
        assert_ne!(hash_code("123456"), hash_code("654321"));
    }

    #[test]
    fn rate_limit_constants_are_reasonable() {
        assert!(MAX_CODES_PER_HOUR > 0 && MAX_CODES_PER_HOUR <= 10);
        assert!(MAX_VERIFY_ATTEMPTS > 0 && MAX_VERIFY_ATTEMPTS <= 10);
        assert!(CODE_EXPIRY_MINUTES > 0 && CODE_EXPIRY_MINUTES <= 30);
    }
}

// --- Notification helpers ---

/// Send a billing alert SMS to a user's verified phone number.
pub async fn send_billing_alert(pool: &Pool<Postgres>, user_id: i64, message: &str) {
    match get_verified_phone(pool, user_id).await {
        Some(phone) => {
            let msg = format!("[{} Billing] {}", app_name(), message);
            if let Err(e) = send_sms(&phone, &msg).await {
                tracing::error!(error = %e, user_id, "Failed to send billing alert SMS");
            }
        }
        None => {
            tracing::debug!(user_id, "No verified phone for billing alert");
        }
    }
}

/// Send a security alert SMS to a user's verified phone number.
pub async fn send_security_alert(pool: &Pool<Postgres>, user_id: i64, message: &str) {
    match get_verified_phone(pool, user_id).await {
        Some(phone) => {
            let msg = format!("[{} Security] {}", app_name(), message);
            if let Err(e) = send_sms(&phone, &msg).await {
                tracing::error!(error = %e, user_id, "Failed to send security alert SMS");
            }
        }
        None => {
            tracing::debug!(user_id, "No verified phone for security alert");
        }
    }
}

// --- Internal helpers ---

async fn get_verified_phone(pool: &Pool<Postgres>, user_id: i64) -> Option<String> {
    sqlx::query_scalar!(
        "SELECT phone_number FROM users WHERE id = $1 AND phone_verified = true",
        user_id
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
}

fn hash_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    format!("{:x}", hasher.finalize())
}
