use crate::auth::use_auth;
use crate::routes::Route;
use dioxus::prelude::*;
use shared_types::FeatureFlags;
use shared_ui::{
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Input, Label, Separator,
};
use std::collections::HashMap;

/// Login page with email/password and OAuth options.
/// Accepts an optional `redirect` query param — after login, navigates there
/// instead of Dashboard (used by `/activate` for device-auth flow).
/// On mobile/desktop, OAuth buttons route to DeviceAuth (device code flow).
#[component]
pub fn Login(redirect: Option<String>) -> Element {
    let mut auth = use_auth();
    let flags: FeatureFlags = use_context();
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut field_errors = use_signal(HashMap::<String, String>::new);
    let mut loading = use_signal(|| false);

    // When an OAuth account has no password, show the set-password form
    let mut show_set_password = use_signal(|| false);
    let mut new_password = use_signal(String::new);
    let mut set_pw_loading = use_signal(|| false);
    let mut set_pw_msg = use_signal(|| Option::<(bool, String)>::None);

    // Pre-fetch OAuth authorize URLs so the buttons can link directly.
    // On mobile/desktop (or when OAuth is disabled) the URLs will be None
    // and the buttons fall back to the device-auth flow.
    let oauth_enabled = flags.oauth;
    let redirect_for_oauth = redirect.clone();
    let oauth_urls = use_server_future(move || {
        let redir = redirect_for_oauth.clone();
        async move {
            if !oauth_enabled {
                return (None, None);
            }
            let google = server::api::oauth_authorize_url("google".to_string(), redir.clone())
                .await
                .ok();
            let github = server::api::oauth_authorize_url("github".to_string(), redir)
                .await
                .ok();
            (google, github)
        }
    })?;

    let (google_url, github_url) = oauth_urls.read().as_ref().cloned().unwrap_or((None, None));

    // Store redirect in a signal so closures can read it without moving ownership
    let redirect_target = use_signal(move || redirect);

    // Navigate to the redirect target or Dashboard
    let go_to_destination = move || {
        if let Some(ref path) = *redirect_target.read() {
            navigator().push(NavigationTarget::<Route>::External(path.clone()));
        } else {
            navigator().push(Route::Dashboard {});
        }
    };

    // Redirect to dashboard if already authenticated
    if auth.is_authenticated() {
        go_to_destination();
    }

    let handle_login = move |evt: FormEvent| async move {
        evt.prevent_default();
        loading.set(true);
        error_msg.set(None);
        field_errors.set(HashMap::new());

        match server::api::login(email(), password()).await {
            Ok(user) => {
                auth.set_user(user);
                go_to_destination();
            }
            Err(e) => {
                let err_str = e.to_string();
                // Server returns "NO_PASSWORD" for OAuth accounts without a password
                if err_str.contains("NO_PASSWORD") {
                    show_set_password.set(true);
                } else {
                    let fe = shared_types::AppError::parse_field_errors(&err_str);
                    if fe.is_empty() {
                        error_msg.set(Some(shared_types::AppError::friendly_message(&err_str)));
                    } else {
                        field_errors.set(fe);
                    }
                }
            }
        }
        loading.set(false);
    };

    let handle_set_password = move |evt: FormEvent| async move {
        evt.prevent_default();
        set_pw_loading.set(true);
        set_pw_msg.set(None);

        match server::api::set_oauth_account_password(email(), new_password()).await {
            Ok(resp) => {
                set_pw_msg.set(Some((true, resp.message)));
                // Reset so they can log in with the new password
                show_set_password.set(false);
                error_msg.set(None);
            }
            Err(e) => {
                set_pw_msg.set(Some((
                    false,
                    shared_types::AppError::friendly_message(&e.to_string()),
                )));
            }
        }
        set_pw_loading.set(false);
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./login.css") }

        div { class: "auth-page",
            Card {
                class: "auth-card",

                CardHeader {
                    CardTitle { "Sign In" }
                    CardDescription { "Enter your credentials to access your account" }
                }

                CardContent {
                    if let Some(err) = error_msg() {
                        div { class: "auth-error", "{err}" }
                    }

                    // Success/error feedback from set-password
                    if let Some((success, msg)) = set_pw_msg() {
                        div {
                            class: if success { "auth-success" } else { "auth-error" },
                            "{msg}"
                        }
                    }

                    if show_set_password() {
                        // Set password form for OAuth accounts
                        div { class: "auth-set-password",
                            p { class: "auth-set-password-info",
                                "This account was created with OAuth and has no password. Set one below to sign in."
                            }
                            form { onsubmit: handle_set_password,
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
                                button {
                                    r#type: "submit",
                                    class: "auth-submit button",
                                    disabled: set_pw_loading(),
                                    if set_pw_loading() { "Setting password..." } else { "Set Password" }
                                }
                            }

                            div { class: "auth-divider",
                                Separator {}
                                span { class: "auth-divider-text", "or" }
                                Separator {}
                            }
                        }
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

                    // Email/Password form
                    form { onsubmit: handle_login,
                        div { class: "auth-field",
                            Label { html_for: "email", "Email" }
                            Input {
                                input_type: "email",
                                id: "email",
                                placeholder: "user@example.com",
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
                                placeholder: "Enter your password",
                                value: password(),
                                on_input: move |e: FormEvent| password.set(e.value()),
                            }
                            if let Some(err) = field_errors().get("password") {
                                div { class: "auth-field-error", "{err}" }
                            }
                        }
                        div { class: "auth-forgot-password",
                            Link { to: Route::ForgotPassword {}, "Forgot password?" }
                        }
                        button {
                            r#type: "submit",
                            class: "auth-submit button",
                            disabled: loading(),
                            if loading() { "Signing in..." } else { "Sign In" }
                        }
                    }
                }

                CardFooter {
                    p { class: "auth-link",
                        "Don't have an account? "
                        Link { to: Route::Register {}, "Create one" }
                    }
                }
            }
        }
    }
}
