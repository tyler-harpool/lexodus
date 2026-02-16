use shared_types::BillingEvent;
use sqlx::{Pool, Postgres};
use stripe::{Event, EventObject, Subscription, SubscriptionId, UpdateSubscription};

use super::client::{billing_broadcast, stripe_client};

/// Handle checkout.session.completed — create subscription or record payment.
#[tracing::instrument(skip(pool, event))]
pub async fn handle_checkout_completed(pool: &Pool<Postgres>, event: &Event) -> Result<(), String> {
    let session = match &event.data.object {
        EventObject::CheckoutSession(s) => s,
        _ => return Err("Expected CheckoutSession object".to_string()),
    };

    let metadata = &session.metadata;
    let user_id: i64 = metadata
        .as_ref()
        .and_then(|m| m.get("user_id"))
        .and_then(|v| v.parse().ok())
        .ok_or("Missing user_id in session metadata")?;

    match session.mode {
        stripe::CheckoutSessionMode::Subscription => {
            let subscription_id = session
                .subscription
                .as_ref()
                .map(|s| s.id().to_string())
                .ok_or("Missing subscription ID")?;

            let tier = metadata
                .as_ref()
                .and_then(|m| m.get("tier"))
                .map(|s| s.as_str())
                .unwrap_or("pro");

            let court_id = metadata
                .as_ref()
                .and_then(|m| m.get("court_id"))
                .cloned();

            sqlx::query!(
                r#"INSERT INTO subscriptions (user_id, stripe_subscription_id, stripe_price_id, status, court_id)
                   VALUES ($1, $2, $3, 'active', $4)
                   ON CONFLICT (stripe_subscription_id) DO UPDATE
                   SET status = 'active', court_id = COALESCE($4, subscriptions.court_id), updated_at = NOW()"#,
                user_id,
                subscription_id,
                tier,
                court_id.as_deref(),
            )
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to upsert subscription: {}", e))?;

            // Update users.tier (vestigial, kept for backward compat)
            sqlx::query!("UPDATE users SET tier = $2 WHERE id = $1", user_id, tier)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to update user tier: {}", e))?;

            // Update court tier (source of truth)
            if let Some(ref cid) = court_id {
                sqlx::query!("UPDATE courts SET tier = $2 WHERE id = $1", cid, tier)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("Failed to update court tier: {}", e))?;
            }

            // Revoke refresh tokens to force re-login with new tier claims
            sqlx::query!(
                "UPDATE refresh_tokens SET revoked = TRUE WHERE user_id = $1 AND revoked = FALSE",
                user_id
            )
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to revoke tokens: {}", e))?;

            // Propagate tier metadata to the Stripe subscription so that
            // subsequent `customer.subscription.updated` webhooks can read it.
            // Checkout session metadata is NOT automatically copied to the subscription.
            if let Ok(sid) = subscription_id.parse::<SubscriptionId>() {
                let client = stripe_client()?;
                let mut update_params = UpdateSubscription::new();
                let mut sub_metadata: std::collections::HashMap<String, String> = [
                    ("user_id".to_string(), user_id.to_string()),
                    ("tier".to_string(), tier.to_string()),
                ]
                .into_iter()
                .collect();
                if let Some(ref cid) = court_id {
                    sub_metadata.insert("court_id".to_string(), cid.clone());
                }
                update_params.metadata = Some(sub_metadata);
                if let Err(e) = Subscription::update(&client, &sid, update_params).await {
                    tracing::error!(%e, "Failed to set tier metadata on Stripe subscription");
                }
            }

            // Cancel any OTHER active subscriptions for this user (upgrade scenario)
            let old_subs: Vec<String> = sqlx::query_scalar!(
                r#"SELECT stripe_subscription_id FROM subscriptions
                   WHERE user_id = $1 AND status = 'active'
                     AND stripe_subscription_id != $2"#,
                user_id,
                subscription_id,
            )
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to query old subscriptions: {}", e))?;

            if !old_subs.is_empty() {
                let client = stripe_client()?;
                for old_sub_id in &old_subs {
                    match old_sub_id.parse::<SubscriptionId>() {
                        Ok(sid) => {
                            // Cancel immediately in Stripe
                            if let Err(e) = Subscription::cancel(
                                &client,
                                &sid,
                                stripe::CancelSubscription::default(),
                            )
                            .await
                            {
                                tracing::error!(sub_id = old_sub_id, %e, "Failed to cancel old subscription in Stripe");
                            } else {
                                tracing::info!(
                                    sub_id = old_sub_id,
                                    "Canceled old subscription during upgrade"
                                );
                            }
                        }
                        Err(e) => {
                            tracing::error!(sub_id = old_sub_id, %e, "Invalid subscription ID format");
                        }
                    }
                }

                // Mark old subscriptions as canceled in DB
                sqlx::query!(
                    r#"UPDATE subscriptions SET status = 'canceled', updated_at = NOW()
                       WHERE user_id = $1 AND status = 'active'
                         AND stripe_subscription_id != $2"#,
                    user_id,
                    subscription_id,
                )
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to cancel old subscriptions in DB: {}", e))?;

                tracing::info!(
                    user_id,
                    canceled_count = old_subs.len(),
                    "Canceled old subscriptions during upgrade to {}",
                    tier
                );
            }

            let _ = billing_broadcast().send((
                user_id,
                BillingEvent::SubscriptionUpdated {
                    tier: tier.to_string(),
                    status: "active".to_string(),
                    court_id: court_id.clone(),
                },
            ));

            tracing::info!(user_id, tier, ?court_id, "Subscription created via checkout");
        }
        stripe::CheckoutSessionMode::Payment => {
            let payment_intent_id = session
                .payment_intent
                .as_ref()
                .map(|pi| pi.id().to_string())
                .ok_or("Missing payment intent ID")?;

            let amount_total = session.amount_total.unwrap_or(0);

            sqlx::query!(
                r#"INSERT INTO payments (user_id, stripe_payment_intent_id, amount_cents, currency, status, description)
                   VALUES ($1, $2, $3, 'usd', 'succeeded', 'One-time payment')
                   ON CONFLICT (stripe_payment_intent_id) DO NOTHING"#,
                user_id,
                payment_intent_id,
                amount_total,
            )
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to record payment: {}", e))?;

            let _ = billing_broadcast().send((
                user_id,
                BillingEvent::PaymentSucceeded {
                    amount_cents: amount_total,
                },
            ));

            tracing::info!(user_id, amount_total, "One-time payment recorded");
        }
        _ => {
            tracing::debug!("Checkout session with unhandled mode");
        }
    }

    Ok(())
}

/// Handle customer.subscription.updated — sync subscription status and tier.
#[tracing::instrument(skip(pool, event))]
pub async fn handle_subscription_updated(
    pool: &Pool<Postgres>,
    event: &Event,
) -> Result<(), String> {
    let subscription = match &event.data.object {
        EventObject::Subscription(s) => s,
        _ => return Err("Expected Subscription object".to_string()),
    };

    let sub_id = subscription.id.as_str();
    // Normalize status to lowercase for consistent storage and comparison
    let status = format!("{:?}", subscription.status).to_lowercase();

    let db_sub = sqlx::query!(
        "SELECT user_id, stripe_price_id, court_id FROM subscriptions WHERE stripe_subscription_id = $1",
        sub_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    let db_sub = match db_sub {
        Some(row) => row,
        None => {
            tracing::warn!(sub_id, "Subscription not found in database");
            return Ok(());
        }
    };
    let user_id = db_sub.user_id;
    let court_id = db_sub.court_id;

    let cancel_at_period_end = subscription.cancel_at_period_end;
    let current_period_start =
        chrono::DateTime::from_timestamp(subscription.current_period_start, 0);
    let current_period_end = chrono::DateTime::from_timestamp(subscription.current_period_end, 0);

    sqlx::query!(
        r#"UPDATE subscriptions
           SET status = $2, cancel_at_period_end = $3,
               current_period_start = $4, current_period_end = $5,
               updated_at = NOW()
           WHERE stripe_subscription_id = $1"#,
        sub_id,
        status,
        cancel_at_period_end,
        current_period_start,
        current_period_end,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update subscription: {}", e))?;

    // Read tier from our DB first (set during checkout), then Stripe metadata as fallback.
    // Stripe does NOT auto-copy checkout session metadata to the subscription object,
    // so relying on subscription.metadata alone would default to "premium" on upgrades.
    let tier = db_sub.stripe_price_id.as_str();
    let tier = if tier.is_empty() {
        subscription
            .metadata
            .get("tier")
            .map(|s| s.as_str())
            .unwrap_or("pro")
    } else {
        tier
    };

    let effective_tier = if status == "active" { tier } else { "free" };

    // Update users.tier (vestigial, kept for backward compat)
    sqlx::query!(
        "UPDATE users SET tier = $2 WHERE id = $1",
        user_id,
        effective_tier
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update user tier: {}", e))?;

    // Update court tier (source of truth)
    if let Some(ref cid) = court_id {
        sqlx::query!("UPDATE courts SET tier = $2 WHERE id = $1", cid, effective_tier)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update court tier: {}", e))?;
    }

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
            tier: effective_tier.to_string(),
            status: status.clone(),
            court_id: court_id.clone(),
        },
    ));

    tracing::info!(user_id, %status, tier = effective_tier, ?court_id, "Subscription updated");
    Ok(())
}

/// Handle customer.subscription.deleted — downgrade user to free tier.
#[tracing::instrument(skip(pool, event))]
pub async fn handle_subscription_deleted(
    pool: &Pool<Postgres>,
    event: &Event,
) -> Result<(), String> {
    let subscription = match &event.data.object {
        EventObject::Subscription(s) => s,
        _ => return Err("Expected Subscription object".to_string()),
    };

    let sub_id = subscription.id.as_str();

    let sub_row = sqlx::query!(
        "SELECT user_id, court_id FROM subscriptions WHERE stripe_subscription_id = $1",
        sub_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    let sub_row = match sub_row {
        Some(row) => row,
        None => {
            tracing::warn!(sub_id, "Subscription not found for deletion");
            return Ok(());
        }
    };
    let user_id = sub_row.user_id;
    let court_id = sub_row.court_id;

    sqlx::query!(
        "UPDATE subscriptions SET status = 'canceled', updated_at = NOW() WHERE stripe_subscription_id = $1",
        sub_id
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update subscription: {}", e))?;

    // Check if user has any OTHER active subscription before downgrading
    let remaining_active = sqlx::query!(
        r#"SELECT stripe_price_id FROM subscriptions
           WHERE user_id = $1 AND status = 'active'
             AND stripe_subscription_id != $2
           ORDER BY created_at DESC LIMIT 1"#,
        user_id,
        sub_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to check remaining subscriptions: {}", e))?;

    let effective_tier = if let Some(remaining) = &remaining_active {
        remaining.stripe_price_id.as_str()
    } else {
        "free"
    };

    // Update users.tier (vestigial, kept for backward compat)
    sqlx::query!(
        "UPDATE users SET tier = $2 WHERE id = $1",
        user_id,
        effective_tier
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update user tier: {}", e))?;

    // Update court tier (source of truth)
    if let Some(ref cid) = court_id {
        sqlx::query!("UPDATE courts SET tier = $2 WHERE id = $1", cid, effective_tier)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update court tier: {}", e))?;
    }

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
            tier: effective_tier.to_string(),
            status: if remaining_active.is_some() {
                "active"
            } else {
                "canceled"
            }
            .to_string(),
            court_id: court_id.clone(),
        },
    ));

    tracing::info!(
        user_id,
        tier = effective_tier,
        ?court_id,
        "Subscription deleted, tier set to {}",
        effective_tier
    );
    Ok(())
}

/// Handle invoice.payment_succeeded — record the payment for audit trail.
#[tracing::instrument(skip(pool, event))]
pub async fn handle_invoice_payment_succeeded(
    pool: &Pool<Postgres>,
    event: &Event,
) -> Result<(), String> {
    let invoice = match &event.data.object {
        EventObject::Invoice(i) => i,
        _ => return Err("Expected Invoice object".to_string()),
    };

    let customer_id = invoice
        .customer
        .as_ref()
        .map(|c| c.id().to_string())
        .unwrap_or_default();

    let user_id = sqlx::query_scalar!(
        "SELECT id FROM users WHERE stripe_customer_id = $1",
        customer_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    let user_id = match user_id {
        Some(id) => id,
        None => {
            tracing::warn!(customer_id, "User not found for invoice payment");
            return Ok(());
        }
    };

    let payment_intent_id = invoice
        .payment_intent
        .as_ref()
        .map(|pi| pi.id().to_string())
        .unwrap_or_else(|| format!("inv_{}", invoice.id.as_str()));

    let amount = invoice.amount_paid.unwrap_or(0);
    let invoice_id = invoice.id.as_str().to_string();

    sqlx::query!(
        r#"INSERT INTO payments (user_id, stripe_payment_intent_id, stripe_invoice_id, amount_cents, currency, status)
           VALUES ($1, $2, $3, $4, 'usd', 'succeeded')
           ON CONFLICT (stripe_payment_intent_id) DO NOTHING"#,
        user_id,
        payment_intent_id,
        invoice_id,
        amount,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to record payment: {}", e))?;

    let _ = billing_broadcast().send((
        user_id,
        BillingEvent::PaymentSucceeded {
            amount_cents: amount,
        },
    ));

    tracing::info!(user_id, amount, "Invoice payment recorded");
    Ok(())
}

/// Handle invoice.payment_failed — log warning, Stripe dunning handles retries.
#[tracing::instrument(skip(pool, event))]
pub async fn handle_invoice_payment_failed(
    pool: &Pool<Postgres>,
    event: &Event,
) -> Result<(), String> {
    let invoice = match &event.data.object {
        EventObject::Invoice(i) => i,
        _ => return Err("Expected Invoice object".to_string()),
    };

    let customer_id = invoice
        .customer
        .as_ref()
        .map(|c| c.id().to_string())
        .unwrap_or_default();

    let user_id = sqlx::query_scalar!(
        "SELECT id FROM users WHERE stripe_customer_id = $1",
        customer_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    if let Some(user_id) = user_id {
        let _ = billing_broadcast().send((
            user_id,
            BillingEvent::PaymentFailed {
                message: "Payment failed. Please update your payment method.".to_string(),
            },
        ));

        // Fire-and-forget SMS alert if phone verified
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            crate::twilio::send_billing_alert(
                &pool_clone,
                user_id,
                "Your payment failed. Please update your payment method.",
            )
            .await;
        });
    }

    tracing::warn!(
        customer_id,
        "Invoice payment failed — Stripe dunning will handle retries"
    );
    Ok(())
}
