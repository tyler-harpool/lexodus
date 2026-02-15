use sqlx::{Pool, Postgres};
use stripe::{BillingPortalSession, CreateBillingPortalSession, CustomerId};

use super::client::{app_base_url, stripe_client};

/// Create a Stripe Customer Portal session for managing subscriptions.
#[tracing::instrument(skip(pool))]
pub async fn create_portal_session(pool: &Pool<Postgres>, user_id: i64) -> Result<String, String> {
    let customer_id = sqlx::query_scalar!(
        "SELECT stripe_customer_id FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?
    .ok_or_else(|| "No Stripe customer found. Please subscribe first.".to_string())?;

    let customer_id: CustomerId = customer_id
        .parse()
        .map_err(|e| format!("Invalid customer ID: {}", e))?;

    let client = stripe_client()?;
    let base_url = app_base_url();
    let return_url = format!("{}/settings", base_url);

    let mut params = CreateBillingPortalSession::new(customer_id);
    params.return_url = Some(&return_url);

    let session = BillingPortalSession::create(&client, params)
        .await
        .map_err(|e| format!("Failed to create portal session: {}", e))?;

    Ok(session.url)
}
