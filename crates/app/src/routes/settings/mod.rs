mod advanced_section;
mod appearance_section;
mod billing_section;
mod notifications_section;
mod phone_section;
mod profile_section;
mod security_section;

use crate::auth::use_auth;
use dioxus::prelude::*;
use shared_types::FeatureFlags;
use shared_ui::{
    use_toast, Accordion, MenubarContent, MenubarItem, MenubarMenu, MenubarRoot, MenubarSeparator,
    MenubarTrigger, Separator, ToastOptions,
};

use advanced_section::AdvancedSection;
use appearance_section::AppearanceSection;
use billing_section::BillingSection;
use notifications_section::NotificationsSection;
use phone_section::PhoneSection;
use profile_section::ProfileSection;
use security_section::SecuritySection;

/// Settings page with menubar navigation, accordion sections, and advanced collapsible.
///
/// Decomposed into sub-components to keep each function's stack frame small
/// enough for WASM's limited stack (avoids allocator panic during hydration).
#[component]
pub fn Settings(billing: Option<String>, verified: Option<String>) -> Element {
    let mut auth = use_auth();
    let flags: FeatureFlags = use_context();
    let toast = use_toast();
    let mut resending_verification = use_signal(|| false);

    let email_verified = {
        let guard = auth.current_user.read();
        guard.as_ref().map(|u| u.email_verified).unwrap_or(false)
    };

    // Handle query params on mount (from Stripe redirect or email verification).
    // Runs once — not re-triggered by auth state changes.
    let billing_param = billing.clone();
    let verified_param = verified.clone();
    let mut handled = use_signal(|| false);
    use_effect(move || {
        if handled() {
            return;
        }
        handled.set(true);

        if let Some(ref status) = billing_param {
            match status.as_str() {
                "success" => {
                    spawn(async move {
                        if let Ok(Some(user)) = server::api::get_current_user().await {
                            auth.set_user(user);
                        }
                    });
                    toast.success(
                        "Subscription activated successfully!".to_string(),
                        ToastOptions::new(),
                    );
                }
                "cancelled" => {
                    toast.info("Checkout was cancelled.".to_string(), ToastOptions::new());
                }
                _ => {}
            }
        }
        if let Some(ref status) = verified_param {
            match status.as_str() {
                "success" => {
                    spawn(async move {
                        if let Ok(Some(user)) = server::api::get_current_user().await {
                            auth.set_user(user);
                        }
                    });
                    toast.success(
                        "Email verified successfully!".to_string(),
                        ToastOptions::new(),
                    );
                }
                "failed" => {
                    toast.error(
                        "Email verification failed. The link may be expired.".to_string(),
                        ToastOptions::new(),
                    );
                }
                _ => {}
            }
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./settings.css") }

        div {
            class: "settings-page",

            h1 {
                class: "settings-title",
                "Settings"
            }

            // Email verification banner (only when mailgun is enabled)
            if flags.mailgun && !email_verified {
                div {
                    class: "settings-banner settings-banner-warning",
                    div {
                        class: "settings-banner-content",
                        span { "Your email address has not been verified." }
                        shared_ui::Button {
                            variant: shared_ui::ButtonVariant::Primary,
                            disabled: resending_verification(),
                            onclick: move |_| async move {
                                resending_verification.set(true);
                                match server::api::resend_verification_email().await {
                                    Ok(resp) => {
                                        toast.success(resp.message, ToastOptions::new());
                                    }
                                    Err(e) => {
                                        toast.error(
                                            shared_types::AppError::friendly_message(&e.to_string()),
                                            ToastOptions::new(),
                                        );
                                    }
                                }
                                resending_verification.set(false);
                            },
                            if resending_verification() { "Sending..." } else { "Resend Verification Email" }
                        }
                    }
                }
            }

            // Menubar
            MenubarRoot {
                MenubarMenu {
                    index: 0usize,
                    MenubarTrigger { "General" }
                    MenubarContent {
                        MenubarItem { index: 0usize, value: "profile",
                            on_select: move |_: String| { toast.info("Profile selected".to_string(), ToastOptions::new()); },
                            "Profile"
                        }
                        MenubarItem { index: 1usize, value: "account",
                            on_select: move |_: String| { toast.info("Account selected".to_string(), ToastOptions::new()); },
                            "Account"
                        }
                        MenubarItem { index: 2usize, value: "security",
                            on_select: move |_: String| { toast.info("Security selected".to_string(), ToastOptions::new()); },
                            "Security"
                        }
                    }
                }

                MenubarSeparator {}

                MenubarMenu {
                    index: 1usize,
                    MenubarTrigger { "Appearance" }
                    MenubarContent {
                        MenubarItem { index: 0usize, value: "theme",
                            on_select: move |_: String| { toast.info("Theme selected".to_string(), ToastOptions::new()); },
                            "Theme"
                        }
                        MenubarItem { index: 1usize, value: "layout",
                            on_select: move |_: String| { toast.info("Layout selected".to_string(), ToastOptions::new()); },
                            "Layout"
                        }
                        MenubarItem { index: 2usize, value: "fonts",
                            on_select: move |_: String| { toast.info("Fonts selected".to_string(), ToastOptions::new()); },
                            "Fonts"
                        }
                    }
                }

                MenubarSeparator {}

                MenubarMenu {
                    index: 2usize,
                    MenubarTrigger { "Notifications" }
                    MenubarContent {
                        MenubarItem { index: 0usize, value: "email-notifs",
                            on_select: move |_: String| { toast.info("Email notifications selected".to_string(), ToastOptions::new()); },
                            "Email"
                        }
                        MenubarItem { index: 1usize, value: "push-notifs",
                            on_select: move |_: String| { toast.info("Push notifications selected".to_string(), ToastOptions::new()); },
                            "Push"
                        }
                        MenubarItem { index: 2usize, value: "digest",
                            on_select: move |_: String| { toast.info("Digest selected".to_string(), ToastOptions::new()); },
                            "Digest"
                        }
                    }
                }
            }

            Separator {}

            // Accordion sections — each is its own component with its own stack frame.
            // Indices must be sequential 0..N-1 for the Accordion primitive's
            // keyboard navigation to work. Compute offsets for conditional sections.
            {
                let stripe_offset = if flags.stripe { 1usize } else { 0 };
                let twilio_offset = if flags.twilio { 1usize } else { 0 };

                rsx! {
                    Accordion {
                        ProfileSection { index: 0 }
                        SecuritySection { index: 1 }
                        if flags.stripe {
                            BillingSection { index: 2 }
                        }
                        if flags.twilio {
                            PhoneSection { index: 2 + stripe_offset }
                        }
                        AppearanceSection { index: 2 + stripe_offset + twilio_offset }
                        NotificationsSection { index: 3 + stripe_offset + twilio_offset }
                    }
                }
            }

            Separator {}

            // Advanced settings (collapsible)
            AdvancedSection {}
        }
    }
}
