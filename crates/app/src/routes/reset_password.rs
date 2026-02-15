use crate::routes::Route;
use dioxus::prelude::*;
use shared_ui::{
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Input, Label,
};

const MIN_PASSWORD_LENGTH: usize = 8;

/// Reset password page — receives a token from the email link as a query param
/// and lets the user set a new password.
#[component]
pub fn ResetPassword(token: Option<String>) -> Element {
    let mut new_password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut success = use_signal(|| false);
    let mut loading = use_signal(|| false);
    let token_signal = use_signal(|| token.clone());

    let handle_submit = move |evt: FormEvent| async move {
        evt.prevent_default();
        error_msg.set(None);

        let pw = new_password();
        let confirm = confirm_password();

        if pw.len() < MIN_PASSWORD_LENGTH {
            error_msg.set(Some(format!(
                "Password must be at least {} characters",
                MIN_PASSWORD_LENGTH
            )));
            return;
        }

        if pw != confirm {
            error_msg.set(Some("Passwords do not match".to_string()));
            return;
        }

        let tk = match token_signal().as_deref() {
            Some(t) if !t.is_empty() => t.to_string(),
            _ => {
                error_msg.set(Some(
                    "Missing reset token. Please use the link from your email.".to_string(),
                ));
                return;
            }
        };

        loading.set(true);

        match server::api::reset_password(tk, pw).await {
            Ok(_) => {
                success.set(true);
            }
            Err(e) => {
                error_msg.set(Some(shared_types::AppError::friendly_message(
                    &e.to_string(),
                )));
            }
        }

        loading.set(false);
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("../routes/login.css") }

        div { class: "auth-page",
            Card {
                class: "auth-card",

                CardHeader {
                    CardTitle { "Set New Password" }
                    CardDescription { "Enter your new password below" }
                }

                CardContent {
                    if success() {
                        div { class: "auth-success",
                            "Password reset successfully. You can now sign in with your new password."
                        }
                    } else {
                        if let Some(err) = error_msg() {
                            div { class: "auth-error", "{err}" }
                        }

                        form { onsubmit: handle_submit,
                            div { class: "auth-field",
                                Label { html_for: "new_password", "New Password" }
                                Input {
                                    input_type: "password",
                                    id: "new_password",
                                    placeholder: "At least 8 characters",
                                    value: new_password(),
                                    on_input: move |e: FormEvent| new_password.set(e.value()),
                                }
                            }
                            div { class: "auth-field",
                                Label { html_for: "confirm_password", "Confirm Password" }
                                Input {
                                    input_type: "password",
                                    id: "confirm_password",
                                    placeholder: "Re-enter your password",
                                    value: confirm_password(),
                                    on_input: move |e: FormEvent| confirm_password.set(e.value()),
                                }
                            }
                            button {
                                r#type: "submit",
                                class: "auth-submit button",
                                disabled: loading(),
                                if loading() { "Resetting..." } else { "Reset Password" }
                            }
                        }
                    }
                }

                CardFooter {
                    p { class: "auth-link",
                        Link { to: Route::Login { redirect: None }, "Back to sign in" }
                    }
                }
            }
        }
    }
}
