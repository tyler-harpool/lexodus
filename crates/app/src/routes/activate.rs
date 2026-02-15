use crate::routes::Route;
use dioxus::prelude::*;
use shared_ui::{Card, CardContent, CardDescription, CardHeader, CardTitle, Input, Label};

/// Activate page — user enters a device code from their mobile/desktop app.
/// Public route (outside AuthGuard). Auth is validated server-side when the
/// user submits the code, avoiding SSR hydration issues with embedded futures.
#[component]
pub fn Activate(code: Option<String>) -> Element {
    let mut user_code = use_signal(move || code.unwrap_or_default());
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut success_msg = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);
    let mut needs_login = use_signal(|| false);

    let handle_approve = move |evt: FormEvent| async move {
        evt.prevent_default();
        let code_val = user_code();
        if code_val.is_empty() {
            error_msg.set(Some("Please enter a device code.".to_string()));
            return;
        }
        loading.set(true);
        error_msg.set(None);
        success_msg.set(None);
        needs_login.set(false);

        match server::api::approve_device(code_val).await {
            Ok(resp) => {
                success_msg.set(Some(resp.message));
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("Authentication required") || err_str.contains("UNAUTHORIZED") {
                    needs_login.set(true);
                } else {
                    error_msg.set(Some(shared_types::AppError::friendly_message(&err_str)));
                }
            }
        }
        loading.set(false);
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./login.css") }

        div { class: "auth-page",
            Card {
                class: "auth-card",

                CardHeader {
                    CardTitle { "Authorize Device" }
                    CardDescription { "Enter the code shown on your device" }
                }

                CardContent {
                    if let Some(msg) = success_msg() {
                        div { class: "auth-success",
                            "{msg}"
                        }
                        p { class: "device-auth-hint",
                            "You can close this page now."
                        }
                    } else {
                        if needs_login() {
                            div { class: "auth-error",
                                "You need to sign in first."
                            }
                            div { class: "device-auth-instructions",
                                Link {
                                    to: Route::Login { redirect: Some("/activate".to_string()) },
                                    class: "auth-submit button",
                                    "Sign In"
                                }
                            }
                        }

                        if let Some(err) = error_msg() {
                            div { class: "auth-error", "{err}" }
                        }

                        form { onsubmit: handle_approve,
                            div { class: "auth-field",
                                Label { html_for: "user_code", "Device Code" }
                                Input {
                                    input_type: "text",
                                    id: "user_code",
                                    placeholder: "ABCD-EFGH",
                                    value: user_code(),
                                    on_input: move |e: FormEvent| user_code.set(e.value()),
                                }
                            }
                            button {
                                r#type: "submit",
                                class: "auth-submit button",
                                if loading() { "Authorizing..." } else { "Authorize Device" }
                            }
                        }
                    }
                }
            }
        }
    }
}
