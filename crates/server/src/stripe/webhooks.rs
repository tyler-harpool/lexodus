use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode},
};
use sqlx::{Pool, Postgres};
use stripe::{Event, EventType, Webhook};

use super::sync;

/// Axum handler for Stripe webhook events.
/// Always returns 200 to prevent Stripe retry storms — errors are logged.
#[tracing::instrument(skip(pool, body, headers))]
pub async fn handle_stripe_webhook(
    axum::extract::State(pool): axum::extract::State<Pool<Postgres>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let signature = match headers.get("stripe-signature") {
        Some(sig) => match sig.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                tracing::warn!("Invalid stripe-signature header encoding");
                return StatusCode::OK;
            }
        },
        None => {
            tracing::warn!("Missing stripe-signature header");
            return StatusCode::OK;
        }
    };

    let payload = match std::str::from_utf8(&body) {
        Ok(p) => p,
        Err(_) => {
            tracing::warn!("Invalid UTF-8 in webhook body");
            return StatusCode::OK;
        }
    };

    let secret = match super::client::stripe_webhook_secret() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "Stripe webhook secret not configured");
            return StatusCode::OK;
        }
    };
    let event = match Webhook::construct_event(payload, &signature, &secret) {
        Ok(e) => e,
        Err(err) => {
            tracing::warn!(error = %err, "Stripe webhook signature verification failed");
            return StatusCode::OK;
        }
    };

    // Idempotency guard: skip already-processed events
    let event_id = event.id.as_str().to_string();
    let event_type_str = format!("{:?}", event.type_);

    let inserted = sqlx::query!(
        r#"INSERT INTO stripe_webhook_events (stripe_event_id, event_type)
           VALUES ($1, $2)
           ON CONFLICT (stripe_event_id) DO NOTHING"#,
        event_id,
        event_type_str
    )
    .execute(&pool)
    .await;

    match inserted {
        Ok(result) if result.rows_affected() == 0 => {
            tracing::info!(event_id = %event_id, "Duplicate webhook event, skipping");
            return StatusCode::OK;
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to record webhook event");
            return StatusCode::OK;
        }
        _ => {}
    }

    // Route to sync handlers
    if let Err(e) = route_event(&pool, &event).await {
        tracing::error!(
            error = %e,
            event_id = %event_id,
            event_type = %event_type_str,
            "Error processing Stripe webhook event"
        );
    }

    StatusCode::OK
}

async fn route_event(pool: &Pool<Postgres>, event: &Event) -> Result<(), String> {
    match event.type_ {
        EventType::CheckoutSessionCompleted => sync::handle_checkout_completed(pool, event).await,
        EventType::CustomerSubscriptionUpdated => {
            sync::handle_subscription_updated(pool, event).await
        }
        EventType::CustomerSubscriptionDeleted => {
            sync::handle_subscription_deleted(pool, event).await
        }
        EventType::InvoicePaymentSucceeded => {
            sync::handle_invoice_payment_succeeded(pool, event).await
        }
        EventType::InvoicePaymentFailed => sync::handle_invoice_payment_failed(pool, event).await,
        _ => {
            tracing::debug!(event_type = ?event.type_, "Unhandled Stripe event type");
            Ok(())
        }
    }
}
