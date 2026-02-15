use crate::routes::Route;
use dioxus::prelude::*;
use shared_ui::{
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Input, Label,
};

/// Forgot password page — lets a user request a password reset email.
/// Always shows a success message regardless of whether the email exists
/// (email enumeration protection).
#[component]
pub fn ForgotPassword() -> Element {
    let mut email = use_signal(String::new);
    let mut submitted = use_signal(|| false);
    let mut loading = use_signal(|| false);

    let handle_submit = move |evt: FormEvent| async move {
        evt.prevent_default();
        loading.set(true);

        let _ = server::api::forgot_password(email()).await;

        submitted.set(true);
        loading.set(false);
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("../routes/login.css") }

        div { class: "auth-page",
            Card {
                class: "auth-card",

                CardHeader {
                    CardTitle { "Reset Password" }
                    CardDescription { "Enter your email to receive a password reset link" }
                }

                CardContent {
                    if submitted() {
                        div { class: "auth-success",
                            "If an account with that email exists, a password reset link has been sent. Check your inbox."
                        }
                    } else {
                        form { onsubmit: handle_submit,
                            div { class: "auth-field",
                                Label { html_for: "email", "Email" }
                                Input {
                                    input_type: "email",
                                    id: "email",
                                    placeholder: "user@example.com",
                                    value: email(),
                                    on_input: move |e: FormEvent| email.set(e.value()),
                                }
                            }
                            button {
                                r#type: "submit",
                                class: "auth-submit button",
                                disabled: loading(),
                                if loading() { "Sending..." } else { "Send Reset Link" }
                            }
                        }
                    }
                }

                CardFooter {
                    p { class: "auth-link",
                        "Remember your password? "
                        Link { to: Route::Login { redirect: None }, "Sign in" }
                    }
                }
            }
        }
    }
}
