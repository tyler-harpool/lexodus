use crate::auth::use_auth;
use crate::routes::Route;
use dioxus::prelude::*;
use shared_types::FeatureFlags;
use shared_ui::{
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Input, Label, Separator,
};
use std::collections::HashMap;

/// Register page with email/password and OAuth options.
#[component]
pub fn Register() -> Element {
    let mut auth = use_auth();
    let flags: FeatureFlags = use_context();
    let mut username = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut display_name = use_signal(String::new);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut field_errors = use_signal(HashMap::<String, String>::new);
    let mut loading = use_signal(|| false);

    // Pre-fetch OAuth URLs (hook must always run — short-circuit inside async)
    let oauth_enabled = flags.oauth;
    let oauth_urls = use_server_future(move || async move {
        if !oauth_enabled {
            return (None, None);
        }
        let google = server::api::oauth_authorize_url("google".to_string(), None)
            .await
            .ok();
        let github = server::api::oauth_authorize_url("github".to_string(), None)
            .await
            .ok();
        (google, github)
    })?;

    let (google_url, github_url) = oauth_urls.read().as_ref().cloned().unwrap_or((None, None));

    // Redirect to dashboard if already authenticated
    if auth.is_authenticated() {
        navigator().push(Route::Dashboard {});
    }

    let handle_register = move |evt: FormEvent| async move {
        evt.prevent_default();
        loading.set(true);
        error_msg.set(None);
        field_errors.set(HashMap::new());

        match server::api::register(username(), email(), password(), display_name()).await {
            Ok(user) => {
                auth.set_user(user);
                navigator().push(Route::Dashboard {});
            }
            Err(e) => {
                let err_str = e.to_string();
                let fe = shared_types::AppError::parse_field_errors(&err_str);
                if fe.is_empty() {
                    error_msg.set(Some(shared_types::AppError::friendly_message(&err_str)));
                } else {
                    field_errors.set(fe);
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
                    CardTitle { "Create Account" }
                    CardDescription { "Create an account to get started" }
                }

                CardContent {
                    if let Some(err) = error_msg() {
                        div { class: "auth-error", "{err}" }
                    }

                    if flags.oauth {
                        // Always render <a> tags so the DOM tree matches between
                        // SSR and web client hydration. On desktop/mobile there is
                        // no SSR, so we intercept clicks via the Dioxus router to
                        // navigate to the device-code flow instead.
                        div { class: "auth-oauth-buttons",
                            a {
                                class: "auth-oauth-btn",
                                href: google_url.as_deref().unwrap_or("/device-auth"),
                                onclick: move |evt: MouseEvent| {
                                    if cfg!(feature = "desktop") || cfg!(feature = "mobile") {
                                        evt.prevent_default();
                                        navigator().push(Route::DeviceAuth {});
                                    }
                                },
                                "Continue with Google"
                            }
                            a {
                                class: "auth-oauth-btn",
                                href: github_url.as_deref().unwrap_or("/device-auth"),
                                onclick: move |evt: MouseEvent| {
                                    if cfg!(feature = "desktop") || cfg!(feature = "mobile") {
                                        evt.prevent_default();
                                        navigator().push(Route::DeviceAuth {});
                                    }
                                },
                                "Continue with GitHub"
                            }
                        }

                        div { class: "auth-divider",
                            Separator {}
                            span { class: "auth-divider-text", "or" }
                            Separator {}
                        }
                    }

                    // Registration form
                    form { onsubmit: handle_register,
                        div { class: "auth-field",
                            Label { html_for: "display_name", "Display Name" }
                            Input {
                                id: "display_name",
                                placeholder: "Your display name",
                                value: display_name(),
                                on_input: move |e: FormEvent| display_name.set(e.value()),
                            }
                            if let Some(err) = field_errors().get("display_name") {
                                div { class: "auth-field-error", "{err}" }
                            }
                        }
                        div { class: "auth-field",
                            Label { html_for: "username", "Username" }
                            Input {
                                id: "username",
                                placeholder: "Choose a username",
                                value: username(),
                                on_input: move |e: FormEvent| username.set(e.value()),
                            }
                            if let Some(err) = field_errors().get("username") {
                                div { class: "auth-field-error", "{err}" }
                            }
                        }
                        div { class: "auth-field",
                            Label { html_for: "email", "Email" }
                            Input {
                                input_type: "email",
                                id: "email",
                                placeholder: "you@example.com",
                                value: email(),
                                on_input: move |e: FormEvent| email.set(e.value()),
                            }
                            if let Some(err) = field_errors().get("email") {
                                div { class: "auth-field-error", "{err}" }
                            }
                        }
                        div { class: "auth-field",
                            Label { html_for: "password", "Password" }
                            Input {
                                input_type: "password",
                                id: "password",
                                placeholder: "Create a password",
                                value: password(),
                                on_input: move |e: FormEvent| password.set(e.value()),
                            }
                            if let Some(err) = field_errors().get("password") {
                                div { class: "auth-field-error", "{err}" }
                            }
                        }
                        button {
                            r#type: "submit",
                            class: "auth-submit button",
                            disabled: loading(),
                            if loading() { "Creating account..." } else { "Create Account" }
                        }
                    }
                }

                CardFooter {
                    p { class: "auth-link",
                        "Already have an account? "
                        Link { to: Route::Login { redirect: None }, "Sign in" }
                    }
                }
            }
        }
    }
}
