use crate::auth::use_auth;
#[allow(unused_imports)]
use crate::routes::Route;
use dioxus::prelude::*;
use shared_types::{DeviceAuthStatus, DeviceFlowInitResponse};
use shared_ui::{Card, CardContent, CardDescription, CardHeader, CardTitle};

/// Maximum consecutive poll errors before giving up.
#[allow(dead_code)]
const MAX_POLL_ERRORS: u32 = 10;

/// Device authorization page — shown on mobile/desktop.
/// Initiates the flow, displays the code, and polls until approved.
#[component]
pub fn DeviceAuth() -> Element {
    #[allow(unused_mut, unused_variables)]
    let mut auth = use_auth();
    #[allow(unused_mut)]
    let mut status = use_signal(|| DeviceAuthStatus::Pending);
    #[allow(unused_mut)]
    let mut error_msg = use_signal(|| Option::<String>::None);
    #[allow(unused_mut)]
    let mut init_data = use_signal(|| Option::<DeviceFlowInitResponse>::None);

    // Single coroutine that initiates + polls (same pattern as BillingListener)
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        // Only run on hydrated client, not during SSR
        #[cfg(feature = "server")]
        return;

        #[cfg(not(feature = "server"))]
        {
            // Step 1: Initiate the device auth flow
            let data =
                match server::api::initiate_device_auth(Some(crate::client_platform().to_string()))
                    .await
                {
                    Ok(d) => d,
                    Err(e) => {
                        error_msg.set(Some(format!("Failed to start: {}", e)));
                        return;
                    }
                };

            let device_code = data.device_code.clone();
            init_data.set(Some(data));

            // Step 2: Poll until approved/expired
            let mut consecutive_errors: u32 = 0;

            loop {
                match server::api::poll_device_auth(device_code.clone()).await {
                    Ok(resp) => {
                        consecutive_errors = 0;
                        match resp.status {
                            DeviceAuthStatus::Approved => {
                                if let Some(user) = resp.user {
                                    auth.set_user(user);
                                    navigator().push(Route::Dashboard {});
                                }
                                status.set(DeviceAuthStatus::Approved);
                                break;
                            }
                            DeviceAuthStatus::Expired => {
                                status.set(DeviceAuthStatus::Expired);
                                break;
                            }
                            DeviceAuthStatus::Pending => {
                                // Server already slept ~5s, loop immediately
                            }
                        }
                    }
                    Err(e) => {
                        consecutive_errors += 1;
                        if consecutive_errors >= MAX_POLL_ERRORS {
                            error_msg.set(Some(format!("Connection lost: {}", e)));
                            break;
                        }
                        // The next server function call provides natural rate-limiting
                        // via network round-trip time, so no explicit sleep is needed.
                    }
                }
            }
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./login.css") }

        div { class: "auth-page",
            Card {
                class: "auth-card",

                CardHeader {
                    CardTitle { "Sign In" }
                    CardDescription { "Use another device to authorize this one" }
                }

                CardContent {
                    if let Some(err) = error_msg() {
                        div { class: "auth-error", "{err}" }
                    }

                    match init_data() {
                        None => rsx! {
                            p { class: "device-auth-hint", "Generating code..." }
                        },
                        Some(data) => {
                            match status() {
                                DeviceAuthStatus::Expired => rsx! {
                                    div { class: "auth-error", "Code expired." }
                                    p { class: "device-auth-hint",
                                        "Please refresh the page to try again."
                                    }
                                },
                                DeviceAuthStatus::Approved => rsx! {
                                    div { class: "auth-success", "Authorized! Redirecting..." }
                                },
                                DeviceAuthStatus::Pending => rsx! {
                                    div { class: "device-auth-instructions",
                                        p { "Open a browser and go to:" }
                                        div { class: "device-auth-url",
                                            "{data.verification_uri}"
                                        }
                                        p { "Then enter this code:" }
                                        div { class: "device-auth-code",
                                            "{data.user_code}"
                                        }
                                        p { class: "device-auth-hint",
                                            "Waiting for authorization..."
                                        }
                                    }
                                },
                            }
                        },
                    }
                }
            }
        }
    }
}
