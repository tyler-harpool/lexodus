use crate::auth::use_auth;
use crate::routes::Route;
use crate::CourtContext;
use dioxus::prelude::*;
use shared_ui::{
    use_toast, AccordionContent, AccordionItem, AccordionTrigger, AlertDialogAction,
    AlertDialogActions, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
    AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button, ButtonVariant, Separator,
    ToastOptions,
};

/// Billing & Subscription accordion section with cancel dialog.
/// Tier is now per-court — reads from CourtContext.
#[component]
pub fn BillingSection(index: usize) -> Element {
    let mut auth = use_auth();
    let toast = use_toast();
    let ctx = use_context::<CourtContext>();

    let current_tier = ctx.court_tier.read().as_str().to_string();
    let court_id = ctx.court_id.read().clone();

    let mut billing_loading = use_signal(|| false);
    let mut cancel_dialog_open = use_signal(|| false);
    let mut canceling_sub = use_signal(|| false);

    rsx! {
        AccordionItem {
            index: index,

            AccordionTrigger { "Billing & Subscription" }
            AccordionContent {
                div {
                    class: "settings-section",

                    // Current plan display
                    div {
                        class: "billing-plan-row",
                        span {
                            class: "settings-toggle-label",
                            "Current Plan"
                        }
                        Badge {
                            variant: match current_tier.as_str() {
                                "enterprise" => BadgeVariant::Primary,
                                "pro" => BadgeVariant::Secondary,
                                _ => BadgeVariant::Outline,
                            },
                            {current_tier.to_uppercase()}
                        }
                    }

                    // Show which court this applies to
                    div {
                        class: "billing-plan-row",
                        span {
                            class: "settings-toggle-label settings-toggle-muted",
                            "Managing court: {court_id}"
                        }
                    }

                    Separator {}

                    // Upgrade / Manage actions
                    div {
                        class: "billing-actions",
                        p {
                            class: "billing-desc",
                            match current_tier.as_str() {
                                "enterprise" => "This court is on the top-tier plan. Manage the subscription below.",
                                "pro" => "Upgrade to Enterprise for more features, or manage the current subscription.",
                                _ => "Upgrade this court's plan to unlock analytics, admin controls, and more.",
                            }
                        }
                        div {
                            class: "billing-buttons",

                            // Show Pro upgrade for free tier courts only
                            if current_tier == "free" {
                                Button {
                                    variant: ButtonVariant::Primary,
                                    disabled: billing_loading(),
                                    onclick: move |_| async move {
                                        billing_loading.set(true);
                                        let cid = ctx.court_id.read().clone();
                                        match server::api::create_billing_checkout(
                                            "subscription".to_string(),
                                            Some("pro".to_string()),
                                            None,
                                            None,
                                            None,
                                            Some(cid),
                                        ).await {
                                            Ok(resp) => {
                                                navigator().push(
                                                    NavigationTarget::<Route>::External(resp.url),
                                                );
                                            }
                                            Err(e) => {
                                                toast.error(
                                                    shared_types::AppError::friendly_message(&e.to_string()),
                                                    ToastOptions::new(),
                                                );
                                            }
                                        }
                                        billing_loading.set(false);
                                    },
                                    if billing_loading() { "Loading..." } else { "Upgrade to Pro" }
                                }
                            }

                            // Show Enterprise upgrade for free and pro courts
                            if current_tier != "enterprise" {
                                Button {
                                    variant: if current_tier == "free" { ButtonVariant::Outline } else { ButtonVariant::Primary },
                                    disabled: billing_loading(),
                                    onclick: move |_| async move {
                                        billing_loading.set(true);
                                        let cid = ctx.court_id.read().clone();
                                        match server::api::create_billing_checkout(
                                            "subscription".to_string(),
                                            Some("enterprise".to_string()),
                                            None,
                                            None,
                                            None,
                                            Some(cid),
                                        ).await {
                                            Ok(resp) => {
                                                navigator().push(
                                                    NavigationTarget::<Route>::External(resp.url),
                                                );
                                            }
                                            Err(e) => {
                                                toast.error(
                                                    shared_types::AppError::friendly_message(&e.to_string()),
                                                    ToastOptions::new(),
                                                );
                                            }
                                        }
                                        billing_loading.set(false);
                                    },
                                    if billing_loading() { "Loading..." } else { "Upgrade to Enterprise" }
                                }
                            }

                            // Show Cancel + Manage for any paid tier
                            if current_tier != "free" {
                                Button {
                                    variant: ButtonVariant::Destructive,
                                    disabled: billing_loading() || canceling_sub(),
                                    onclick: move |_| {
                                        cancel_dialog_open.set(true);
                                    },
                                    "Cancel Subscription"
                                }
                                Button {
                                    variant: ButtonVariant::Outline,
                                    disabled: billing_loading(),
                                    onclick: move |_| async move {
                                        billing_loading.set(true);
                                        match server::api::create_billing_portal().await {
                                            Ok(resp) => {
                                                navigator().push(
                                                    NavigationTarget::<Route>::External(resp.url),
                                                );
                                            }
                                            Err(e) => {
                                                toast.error(
                                                    shared_types::AppError::friendly_message(&e.to_string()),
                                                    ToastOptions::new(),
                                                );
                                            }
                                        }
                                        billing_loading.set(false);
                                    },
                                    if billing_loading() { "Loading..." } else { "Manage Subscription" }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Cancel Subscription confirmation dialog
        AlertDialogRoot {
            open: cancel_dialog_open(),
            on_open_change: move |val: bool| cancel_dialog_open.set(val),

            AlertDialogContent {
                AlertDialogTitle { "Cancel Subscription" }
                AlertDialogDescription {
                    "This court's subscription will be canceled immediately and downgraded to the free tier. This action cannot be undone."
                }
                AlertDialogActions {
                    AlertDialogCancel { "Keep Subscription" }
                    AlertDialogAction {
                        on_click: move |_| async move {
                            canceling_sub.set(true);
                            cancel_dialog_open.set(false);
                            match server::api::cancel_subscription().await {
                                Ok(_) => {
                                    if let Ok(Some(user)) = server::api::get_current_user().await {
                                        auth.set_user(user);
                                    }
                                    crate::notify::send_if_enabled(
                                        &auth,
                                        "Subscription Cancelled",
                                        "Your court's subscription has been cancelled.",
                                    );
                                    toast.success(
                                        "Subscription canceled successfully.".to_string(),
                                        ToastOptions::new(),
                                    );
                                }
                                Err(e) => {
                                    toast.error(
                                        shared_types::AppError::friendly_message(&e.to_string()),
                                        ToastOptions::new(),
                                    );
                                }
                            }
                            canceling_sub.set(false);
                        },
                        "Yes, Cancel"
                    }
                }
            }
        }
    }
}
