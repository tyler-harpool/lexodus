use crate::auth::use_auth;
use dioxus::prelude::*;
use shared_ui::{
    use_toast, AccordionContent, AccordionItem, AccordionTrigger, Badge, BadgeVariant, Button,
    ButtonVariant, Input, Label, Separator, ToastOptions,
};

/// Phone Verification accordion section.
#[component]
pub fn PhoneSection(index: usize) -> Element {
    let mut auth = use_auth();
    let toast = use_toast();

    let phone_verified = {
        let guard = auth.current_user.read();
        guard.as_ref().map(|u| u.phone_verified).unwrap_or(false)
    };
    let current_phone = {
        let guard = auth.current_user.read();
        guard
            .as_ref()
            .and_then(|u| u.phone_number.clone())
            .unwrap_or_default()
    };

    let mut phone_input = use_signal(String::new);
    let mut phone_code_input = use_signal(String::new);
    let mut phone_sending = use_signal(|| false);
    let mut phone_verifying = use_signal(|| false);
    let mut phone_code_sent = use_signal(|| false);

    rsx! {
        AccordionItem {
            index: index,

            AccordionTrigger { "Phone Verification" }
            AccordionContent {
                div {
                    class: "settings-section",

                    if phone_verified {
                        div {
                            class: "billing-plan-row",
                            span {
                                class: "settings-toggle-label",
                                "Phone Number"
                            }
                            div {
                                class: "phone-verified-row",
                                span { "{current_phone}" }
                                Badge {
                                    variant: BadgeVariant::Primary,
                                    "Verified"
                                }
                            }
                        }

                        Separator {}

                        p {
                            class: "billing-desc",
                            "Your phone number is verified. You will receive security and billing alerts via SMS."
                        }
                    } else if phone_code_sent() {
                        div {
                            class: "settings-form",

                            p {
                                class: "billing-desc",
                                "A 6-digit code has been sent to your phone. Enter it below to verify."
                            }

                            div {
                                class: "settings-field",
                                Label { html_for: "phone-code", "Verification Code" }
                                Input {
                                    value: phone_code_input(),
                                    placeholder: "123456",
                                    label: "",
                                    on_input: move |evt: FormEvent| {
                                        phone_code_input.set(evt.value());
                                    },
                                }
                            }

                            div {
                                class: "billing-buttons",
                                Button {
                                    variant: ButtonVariant::Primary,
                                    disabled: phone_verifying() || phone_code_input().len() != 6,
                                    onclick: move |_| async move {
                                        phone_verifying.set(true);
                                        match server::api::verify_phone(
                                            phone_input(),
                                            phone_code_input(),
                                        ).await {
                                            Ok(user) => {
                                                auth.set_user(user);
                                                phone_code_sent.set(false);
                                                toast.success(
                                                    "Phone number verified!".to_string(),
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
                                        phone_verifying.set(false);
                                    },
                                    if phone_verifying() { "Verifying..." } else { "Verify Code" }
                                }
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    onclick: move |_| {
                                        phone_code_sent.set(false);
                                        phone_code_input.set(String::new());
                                    },
                                    "Back"
                                }
                            }
                        }
                    } else {
                        div {
                            class: "settings-form",

                            p {
                                class: "billing-desc",
                                "Add a phone number to receive security alerts and billing notifications via SMS."
                            }

                            div {
                                class: "settings-field",
                                Label { html_for: "phone-number", "Phone Number" }
                                Input {
                                    value: phone_input(),
                                    placeholder: "+1 (555) 123-4567",
                                    label: "",
                                    on_input: move |evt: FormEvent| {
                                        phone_input.set(evt.value());
                                    },
                                }
                            }

                            Button {
                                variant: ButtonVariant::Primary,
                                disabled: phone_sending() || phone_input().len() < 10,
                                onclick: move |_| async move {
                                    phone_sending.set(true);
                                    match server::api::send_phone_verification(phone_input()).await {
                                        Ok(_) => {
                                            phone_code_sent.set(true);
                                            toast.success(
                                                "Verification code sent!".to_string(),
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
                                    phone_sending.set(false);
                                },
                                if phone_sending() { "Sending..." } else { "Send Verification Code" }
                            }
                        }
                    }
                }
            }
        }
    }
}
