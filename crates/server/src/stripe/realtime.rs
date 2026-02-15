use shared_types::BillingEvent;

use super::client::billing_subscribe;

/// Type alias for the billing WebSocket broadcast receiver.
pub type BillingReceiver = tokio::sync::broadcast::Receiver<(i64, BillingEvent)>;

/// Create a new subscription to billing events.
pub fn subscribe_for_user() -> BillingReceiver {
    billing_subscribe()
}
