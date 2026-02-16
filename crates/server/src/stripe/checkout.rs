use shared_types::BillingEvent;
use sqlx::{Pool, Postgres};
use stripe::{
    CheckoutSession, CheckoutSessionMode, CreateCheckoutSession, CreateCheckoutSessionLineItems,
    CreateCustomer, Customer, CustomerId, Subscription, SubscriptionId,
};

use super::client::{app_base_url, billing_broadcast, stripe_client, stripe_price_for_tier};

/// Ensure the user has a Stripe customer ID, creating one if needed.
#[tracing::instrument(skip(pool))]
pub async fn ensure_stripe_customer(
    pool: &Pool<Postgres>,
    user_id: i64,
    email: &str,
) -> Result<CustomerId, String> {
    let existing = sqlx::query_scalar!(
        "SELECT stripe_customer_id FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    if let Some(customer_id) = existing {
        return customer_id
            .parse::<CustomerId>()
            .map_err(|e| format!("Invalid customer ID: {}", e));
    }

    let client = stripe_client()?;
    let mut params = CreateCustomer::new();
    params.email = Some(email);
    params.metadata = Some(
        [("user_id".to_string(), user_id.to_string())]
            .into_iter()
            .collect(),
    );

    let customer = Customer::create(&client, params)
        .await
        .map_err(|e| format!("Stripe customer creation failed: {}", e))?;

    sqlx::query!(
        "UPDATE users SET stripe_customer_id = $2 WHERE id = $1",
        user_id,
        customer.id.as_str()
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to store customer ID: {}", e))?;

    Ok(customer.id)
}

/// Create a Stripe Checkout session for a subscription.
/// `court_id` identifies the court whose tier will be updated on completion.
#[tracing::instrument(skip(pool))]
pub async fn create_subscription_checkout(
    pool: &Pool<Postgres>,
    user_id: i64,
    email: &str,
    tier: &str,
    court_id: &str,
) -> Result<String, String> {
    let customer_id = ensure_stripe_customer(pool, user_id, email).await?;
    let price_id = stripe_price_for_tier(tier)?;
    let base_url = app_base_url();
    let client = stripe_client()?;

    let success_url = format!("{}/settings?billing=success", base_url);
    let cancel_url = format!("{}/settings?billing=cancelled", base_url);

    let mut metadata: std::collections::HashMap<String, String> = [
        ("user_id".to_string(), user_id.to_string()),
        ("tier".to_string(), tier.to_string()),
    ]
    .into_iter()
    .collect();

    if !court_id.is_empty() {
        metadata.insert("court_id".to_string(), court_id.to_string());
    }

    let mut params = CreateCheckoutSession::new();
    params.customer = Some(customer_id);
    params.mode = Some(CheckoutSessionMode::Subscription);
    params.success_url = Some(&success_url);
    params.cancel_url = Some(&cancel_url);
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        price: Some(price_id),
        quantity: Some(1),
        ..Default::default()
    }]);
    params.metadata = Some(metadata);

    let session = CheckoutSession::create(&client, params)
        .await
        .map_err(|e| format!("Failed to create checkout session: {}", e))?;

    session
        .url
        .ok_or_else(|| "No URL returned from Stripe".to_string())
}

/// Create a Stripe Checkout session for a one-time payment.
#[tracing::instrument(skip(pool))]
pub async fn create_onetime_checkout(
    pool: &Pool<Postgres>,
    user_id: i64,
    email: &str,
    price_cents: i64,
    product_name: &str,
    product_description: &str,
) -> Result<String, String> {
    let customer_id = ensure_stripe_customer(pool, user_id, email).await?;
    let base_url = app_base_url();
    let client = stripe_client()?;

    let success_url = format!("{}/settings?billing=success", base_url);
    let cancel_url = format!("{}/settings?billing=cancelled", base_url);

    let mut params = CreateCheckoutSession::new();
    params.customer = Some(customer_id);
    params.mode = Some(CheckoutSessionMode::Payment);
    params.success_url = Some(&success_url);
    params.cancel_url = Some(&cancel_url);
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        price_data: Some(stripe::CreateCheckoutSessionLineItemsPriceData {
            currency: stripe::Currency::USD,
            unit_amount: Some(price_cents),
            product_data: Some(stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
                name: product_name.to_string(),
                description: Some(product_description.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        quantity: Some(1),
        ..Default::default()
    }]);
    params.metadata = Some(
        [("user_id".to_string(), user_id.to_string())]
            .into_iter()
            .collect(),
    );

    let session = CheckoutSession::create(&client, params)
        .await
        .map_err(|e| format!("Failed to create checkout session: {}", e))?;

    session
        .url
        .ok_or_else(|| "No URL returned from Stripe".to_string())
}

/// Cancel ALL active subscriptions for a user immediately via Stripe.
#[tracing::instrument(skip(pool))]
pub async fn cancel_subscription(pool: &Pool<Postgres>, user_id: i64) -> Result<(), String> {
    let active_subs: Vec<String> = sqlx::query_scalar!(
        r#"SELECT stripe_subscription_id FROM subscriptions
           WHERE user_id = $1 AND status = 'active'"#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    if active_subs.is_empty() {
        return Err("No active subscription found".to_string());
    }

    let client = stripe_client()?;
    for sub_id in &active_subs {
        let sid: SubscriptionId = sub_id
            .parse()
            .map_err(|e| format!("Invalid subscription ID {}: {}", sub_id, e))?;

        if let Err(e) =
            Subscription::cancel(&client, &sid, stripe::CancelSubscription::default()).await
        {
            tracing::error!(sub_id, %e, "Failed to cancel subscription in Stripe");
        }
    }

    // Mark all active subscriptions as canceled in DB
    sqlx::query!(
        "UPDATE subscriptions SET status = 'canceled', updated_at = NOW() WHERE user_id = $1 AND status = 'active'",
        user_id
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update subscription status: {}", e))?;

    sqlx::query!("UPDATE users SET tier = 'free' WHERE id = $1", user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to downgrade user tier: {}", e))?;

    // Also downgrade any courts linked via subscriptions
    sqlx::query!(
        r#"UPDATE courts SET tier = 'free'
           WHERE id IN (SELECT DISTINCT court_id FROM subscriptions WHERE user_id = $1 AND court_id IS NOT NULL)"#,
        user_id
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to downgrade court tiers: {}", e))?;

    // Revoke tokens so next request picks up the new tier
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked = TRUE WHERE user_id = $1 AND revoked = FALSE",
        user_id
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to revoke tokens: {}", e))?;

    let _ = billing_broadcast().send((
        user_id,
        BillingEvent::SubscriptionUpdated {
            tier: "free".to_string(),
            status: "canceled".to_string(),
            court_id: None,
        },
    ));

    tracing::info!(
        user_id,
        canceled_count = active_subs.len(),
        "All subscriptions canceled from app"
    );
    Ok(())
}
