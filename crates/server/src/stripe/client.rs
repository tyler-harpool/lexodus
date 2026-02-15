use std::sync::OnceLock;
use stripe::Client;
use tokio::sync::broadcast;

use shared_types::BillingEvent;

static BILLING_TX: OnceLock<broadcast::Sender<(i64, BillingEvent)>> = OnceLock::new();

/// Create a Stripe API client from the secret key environment variable.
pub fn stripe_client() -> Result<Client, String> {
    Ok(Client::new(stripe_secret_key()?))
}

pub fn stripe_secret_key() -> Result<String, String> {
    std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| "STRIPE_SECRET_KEY is not configured".to_string())
}

pub fn stripe_webhook_secret() -> Result<String, String> {
    std::env::var("STRIPE_WEBHOOK_SECRET")
        .map_err(|_| "STRIPE_WEBHOOK_SECRET is not configured".to_string())
}

/// Map a subscription tier name to the corresponding Stripe Price ID.
pub fn stripe_price_for_tier(tier: &str) -> Result<String, String> {
    match tier.to_lowercase().as_str() {
        "pro" => std::env::var("STRIPE_PRICE_PRO")
            .map_err(|_| "STRIPE_PRICE_PRO not configured".to_string()),
        "enterprise" => std::env::var("STRIPE_PRICE_ENTERPRISE")
            .map_err(|_| "STRIPE_PRICE_ENTERPRISE not configured".to_string()),
        _ => Err(format!("No Stripe price configured for tier: {}", tier)),
    }
}

pub fn app_base_url() -> String {
    std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

/// Get the global billing event broadcast sender.
pub fn billing_broadcast() -> &'static broadcast::Sender<(i64, BillingEvent)> {
    BILLING_TX.get_or_init(|| {
        let (tx, _) = broadcast::channel(64);
        tx
    })
}

/// Subscribe to billing events.
pub fn billing_subscribe() -> broadcast::Receiver<(i64, BillingEvent)> {
    billing_broadcast().subscribe()
}
