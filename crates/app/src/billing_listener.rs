use crate::auth::use_auth;
use dioxus::prelude::*;
use shared_types::BillingEvent;
#[allow(unused_imports)]
use shared_ui::{use_toast, ToastOptions};

/// Maximum consecutive errors before the polling loop stops.
/// The user can refresh the page to restart the listener.
#[allow(dead_code)]
const MAX_CONSECUTIVE_ERRORS: u32 = 10;

/// Headless component that long-polls for billing events and fires
/// desktop notifications + in-app toasts when they arrive.
///
/// Mount once inside `AppLayout` so it runs on every authenticated page.
#[component]
#[allow(unused_variables, unused_mut)]
pub fn BillingListener() -> Element {
    let mut auth = use_auth();
    let toast = use_toast();

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        // During SSR this coroutine would block the render thread because
        // server functions execute as direct calls, not HTTP requests.
        // Only run the long-poll loop on the hydrated client.
        #[cfg(feature = "server")]
        return;

        #[cfg(not(feature = "server"))]
        {
            let mut consecutive_errors: u32 = 0;

            loop {
                match server::api::poll_billing_event().await {
                    Ok(Some(event)) => {
                        consecutive_errors = 0;
                        let (title, body) = notification_content(&event);

                        // Desktop notification (respects push_notifications_enabled)
                        crate::notify::send_if_enabled(&auth, &title, &body);

                        // In-app toast so web users see it too
                        match &event {
                            BillingEvent::PaymentFailed { .. } => {
                                toast.error(body.clone(), ToastOptions::new());
                            }
                            _ => {
                                toast.success(body.clone(), ToastOptions::new());
                            }
                        }

                        // Refresh auth state so the tier badge updates
                        if let Ok(Some(user)) = server::api::get_current_user().await {
                            auth.set_user(user);
                        }
                    }
                    Ok(None) => {
                        // Timeout — immediately retry (long-poll loop)
                        consecutive_errors = 0;
                    }
                    Err(_) => {
                        consecutive_errors += 1;
                        if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                            break;
                        }
                        // The next server function call provides natural rate-limiting
                        // via network round-trip time, so no explicit sleep is needed.
                    }
                }
            }
        }
    });

    // Headless — renders nothing
    rsx! {}
}

/// Map a `BillingEvent` to a (title, body) pair for notifications.
#[allow(dead_code)]
fn notification_content(event: &BillingEvent) -> (String, String) {
    match event {
        BillingEvent::SubscriptionUpdated { status, tier } => match status.as_str() {
            "active" => (
                "Subscription Activated".to_string(),
                format!("Welcome to {}!", capitalize(tier)),
            ),
            "canceled" => (
                "Subscription Ended".to_string(),
                "You're now on the free plan.".to_string(),
            ),
            other => (
                "Subscription Updated".to_string(),
                format!("Status: {}", capitalize(other)),
            ),
        },
        BillingEvent::PaymentSucceeded { amount_cents } => {
            let dollars = *amount_cents as f64 / 100.0;
            (
                "Payment Successful".to_string(),
                format!("${:.2} processed.", dollars),
            )
        }
        BillingEvent::PaymentFailed { message } => ("Payment Failed".to_string(), message.clone()),
    }
}

/// Capitalize the first letter of a string.
#[allow(dead_code)]
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
