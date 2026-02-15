use sha2::{Digest, Sha256};
use sqlx::{Pool, Postgres};
use tracing;

// --- Environment helpers ---

fn mailgun_api_key() -> Result<String, String> {
    std::env::var("MAILGUN_API_KEY").map_err(|_| "MAILGUN_API_KEY is not configured".to_string())
}

fn mailgun_domain() -> Result<String, String> {
    std::env::var("MAILGUN_DOMAIN").map_err(|_| "MAILGUN_DOMAIN is not configured".to_string())
}

fn mailgun_from() -> Result<String, String> {
    match std::env::var("MAILGUN_FROM") {
        Ok(v) => Ok(v),
        Err(_) => Ok(format!("{} <noreply@{}>", app_name(), mailgun_domain()?)),
    }
}

fn app_base_url() -> String {
    std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

fn app_name() -> String {
    std::env::var("APP_NAME").unwrap_or_else(|_| "Lexodus".to_string())
}

// --- Core email sending ---

#[tracing::instrument(skip(html_body))]
pub async fn send_email(to: &str, subject: &str, html_body: &str) -> Result<(), String> {
    let domain = mailgun_domain()?;
    let url = format!("https://api.mailgun.net/v3/{}/messages", domain);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .basic_auth("api", Some(mailgun_api_key()?))
        .form(&[
            ("from", mailgun_from()?),
            ("to", to.to_string()),
            ("subject", subject.to_string()),
            ("html", html_body.to_string()),
        ])
        .send()
        .await
        .map_err(|e| format!("Mailgun request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Mailgun API error ({}): {}", status, body));
    }

    tracing::info!(to = to, subject = subject, "Email sent successfully");
    Ok(())
}

// --- Higher-level helpers ---

pub async fn send_welcome_email(to: &str, display_name: &str) {
    let html = templates::welcome_html(display_name, &app_name());
    if let Err(e) = send_email(to, &format!("Welcome to {}", app_name()), &html).await {
        tracing::error!(error = %e, to = to, "Failed to send welcome email");
    }
}

pub async fn send_verification_email(to: &str, token: &str) {
    let html = templates::verification_html(token, &app_base_url());
    if let Err(e) = send_email(to, "Verify your email", &html).await {
        tracing::error!(error = %e, to = to, "Failed to send verification email");
    }
}

pub async fn send_password_reset_email(to: &str, token: &str) {
    let html = templates::password_reset_html(token, &app_base_url());
    if let Err(e) = send_email(to, "Reset your password", &html).await {
        tracing::error!(error = %e, to = to, "Failed to send password reset email");
    }
}

pub async fn send_receipt_email(to: &str, tier: &str, amount_cents: i64) {
    let html = templates::receipt_html(tier, amount_cents, &app_name());
    if let Err(e) = send_email(to, "Payment receipt", &html).await {
        tracing::error!(error = %e, to = to, "Failed to send receipt email");
    }
}

// --- Email verification token ---

pub async fn create_verification_token(
    pool: &Pool<Postgres>,
    user_id: i64,
) -> Result<String, String> {
    let token = uuid::Uuid::new_v4().to_string();
    let token_hash = hash_token(&token);
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

    sqlx::query!(
        "INSERT INTO email_verifications (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        user_id,
        token_hash,
        expires_at
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create verification token: {}", e))?;

    Ok(token)
}

pub async fn verify_email_token(pool: &Pool<Postgres>, token: &str) -> Result<i64, String> {
    let token_hash = hash_token(token);

    let record = sqlx::query!(
        r#"UPDATE email_verifications
           SET verified_at = NOW()
           WHERE token_hash = $1 AND expires_at > NOW() AND verified_at IS NULL
           RETURNING user_id"#,
        token_hash
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?
    .ok_or_else(|| "Invalid or expired verification token".to_string())?;

    sqlx::query!(
        "UPDATE users SET email_verified = true WHERE id = $1",
        record.user_id
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update user: {}", e))?;

    Ok(record.user_id)
}

// --- Password reset token ---

pub async fn create_password_reset_token(
    pool: &Pool<Postgres>,
    email: &str,
) -> Result<String, String> {
    let token = uuid::Uuid::new_v4().to_string();
    let token_hash = hash_token(&token);
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

    sqlx::query!(
        "INSERT INTO password_resets (email, token_hash, expires_at) VALUES ($1, $2, $3)",
        email,
        token_hash,
        expires_at
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create reset token: {}", e))?;

    Ok(token)
}

pub async fn validate_password_reset_token(
    pool: &Pool<Postgres>,
    token: &str,
) -> Result<String, String> {
    let token_hash = hash_token(token);

    let record = sqlx::query!(
        r#"UPDATE password_resets
           SET used = true
           WHERE token_hash = $1 AND expires_at > NOW() AND used = false
           RETURNING email"#,
        token_hash
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?
    .ok_or_else(|| "Invalid or expired reset token".to_string())?;

    Ok(record.email)
}

// --- Webhook verification ---

pub fn verify_webhook_signature(
    signing_key: &str,
    timestamp: &str,
    token: &str,
    signature: &str,
) -> bool {
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<sha2::Sha256>;

    let Ok(mut mac) = HmacSha256::new_from_slice(signing_key.as_bytes()) else {
        return false;
    };
    mac.update(timestamp.as_bytes());
    mac.update(token.as_bytes());

    let expected = hex::encode(mac.finalize().into_bytes());
    expected == signature
}

pub async fn handle_bounce_event(pool: &Pool<Postgres>, recipient: &str) {
    if let Err(e) = sqlx::query!(
        "UPDATE users SET email_bounced = true WHERE email = $1",
        recipient
    )
    .execute(pool)
    .await
    {
        tracing::error!(error = %e, email = recipient, "Failed to mark email as bounced");
    }
}

// --- Token hashing (reuse SHA-256 pattern from jwt) ---

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_token_consistent_sha256() {
        let hash = hash_token("test-token-123");
        // SHA-256 always produces 64 hex chars
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        // Same input always produces same output
        assert_eq!(hash, hash_token("test-token-123"));
    }

    #[test]
    fn hash_token_different_inputs_differ() {
        assert_ne!(hash_token("token-a"), hash_token("token-b"));
    }

    #[test]
    fn verify_webhook_signature_valid() {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<sha2::Sha256>;

        let key = "test-signing-key";
        let timestamp = "1234567890";
        let token = "abc123";

        // Compute expected signature
        let mut mac = HmacSha256::new_from_slice(key.as_bytes()).unwrap();
        mac.update(timestamp.as_bytes());
        mac.update(token.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        assert!(verify_webhook_signature(key, timestamp, token, &signature));
    }

    #[test]
    fn verify_webhook_signature_invalid() {
        assert!(!verify_webhook_signature(
            "key",
            "timestamp",
            "token",
            "badsignature"
        ));
    }

    #[test]
    fn verify_webhook_signature_wrong_key() {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<sha2::Sha256>;

        let timestamp = "1234567890";
        let token = "abc123";

        let mut mac = HmacSha256::new_from_slice(b"correct-key").unwrap();
        mac.update(timestamp.as_bytes());
        mac.update(token.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        assert!(!verify_webhook_signature(
            "wrong-key",
            timestamp,
            token,
            &signature
        ));
    }

    #[test]
    fn welcome_template_contains_name_and_app() {
        let html = templates::welcome_html("Alice", "TestApp");
        assert!(html.contains("Alice"));
        assert!(html.contains("TestApp"));
    }

    #[test]
    fn verification_template_contains_link() {
        let html = templates::verification_html("tok-123", "https://example.com");
        assert!(html.contains("https://example.com/api/v1/auth/verify-email?token=tok-123"));
    }

    #[test]
    fn password_reset_template_contains_link() {
        let html = templates::password_reset_html("reset-tok", "https://example.com");
        assert!(html.contains("https://example.com/reset-password?token=reset-tok"));
    }

    #[test]
    fn receipt_template_formats_amount() {
        let html = templates::receipt_html("Pro", 1999, "TestApp");
        assert!(html.contains("$19.99"));
        assert!(html.contains("Pro"));
        assert!(html.contains("TestApp"));
    }
}

// --- Email templates ---

mod templates {
    pub fn welcome_html(display_name: &str, app_name: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: 'Courier New', monospace; background: #0a0a0f; color: #e0e0e0; padding: 20px;">
  <div style="max-width: 600px; margin: 0 auto; border: 1px solid #00f0ff; padding: 30px;">
    <h1 style="color: #00f0ff; text-align: center;">Welcome to {app_name}</h1>
    <p>Hey {display_name},</p>
    <p>Your account has been created. You're now part of the grid.</p>
    <p style="color: #888;">— The {app_name} Team</p>
  </div>
</body>
</html>"#,
            app_name = app_name,
            display_name = display_name
        )
    }

    pub fn verification_html(token: &str, base_url: &str) -> String {
        let link = format!("{}/api/v1/auth/verify-email?token={}", base_url, token);
        format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: 'Courier New', monospace; background: #0a0a0f; color: #e0e0e0; padding: 20px;">
  <div style="max-width: 600px; margin: 0 auto; border: 1px solid #00f0ff; padding: 30px;">
    <h1 style="color: #00f0ff; text-align: center;">Verify Your Email</h1>
    <p>Click the link below to verify your email address:</p>
    <p style="text-align: center;">
      <a href="{link}" style="display: inline-block; background: #00f0ff; color: #0a0a0f; padding: 12px 24px; text-decoration: none; font-weight: bold;">Verify Email</a>
    </p>
    <p style="color: #888; font-size: 12px;">This link expires in 24 hours.</p>
  </div>
</body>
</html>"#,
            link = link
        )
    }

    pub fn password_reset_html(token: &str, base_url: &str) -> String {
        let link = format!("{}/reset-password?token={}", base_url, token);
        format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: 'Courier New', monospace; background: #0a0a0f; color: #e0e0e0; padding: 20px;">
  <div style="max-width: 600px; margin: 0 auto; border: 1px solid #00f0ff; padding: 30px;">
    <h1 style="color: #00f0ff; text-align: center;">Reset Your Password</h1>
    <p>Click the link below to reset your password:</p>
    <p style="text-align: center;">
      <a href="{link}" style="display: inline-block; background: #ff6b00; color: #0a0a0f; padding: 12px 24px; text-decoration: none; font-weight: bold;">Reset Password</a>
    </p>
    <p style="color: #888; font-size: 12px;">This link expires in 1 hour. If you didn't request this, ignore this email.</p>
  </div>
</body>
</html>"#,
            link = link
        )
    }

    pub fn receipt_html(tier: &str, amount_cents: i64, app_name: &str) -> String {
        let amount = format!("${:.2}", amount_cents as f64 / 100.0);
        format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: 'Courier New', monospace; background: #0a0a0f; color: #e0e0e0; padding: 20px;">
  <div style="max-width: 600px; margin: 0 auto; border: 1px solid #00f0ff; padding: 30px;">
    <h1 style="color: #00f0ff; text-align: center;">Payment Receipt</h1>
    <p>Your payment has been processed:</p>
    <table style="width: 100%; border-collapse: collapse; margin: 20px 0;">
      <tr><td style="padding: 8px; border-bottom: 1px solid #333;">Plan</td><td style="padding: 8px; border-bottom: 1px solid #333; text-align: right; color: #00f0ff;">{tier}</td></tr>
      <tr><td style="padding: 8px; border-bottom: 1px solid #333;">Amount</td><td style="padding: 8px; border-bottom: 1px solid #333; text-align: right; color: #00ff41;">{amount}</td></tr>
    </table>
    <p style="color: #888;">— The {app_name} Team</p>
  </div>
</body>
</html>"#,
            tier = tier,
            amount = amount,
            app_name = app_name
        )
    }
}
