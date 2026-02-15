use crate::auth::use_auth;
use dioxus::prelude::*;
use shared_ui::{
    use_toast, AccordionContent, AccordionItem, AccordionTrigger, Button, ButtonVariant, Form,
    Input, Label, ToastOptions,
};

const MIN_PASSWORD_LENGTH: usize = 8;

/// Security accordion section: change password or set initial password.
///
/// If the user has a password (`has_password`), shows current + new + confirm fields.
/// If the user signed up via OAuth and has no password, shows only new + confirm fields
/// and calls `set_oauth_account_password` so they can also use email/password login
/// (critical for mobile/desktop where OAuth isn't available).
#[component]
pub fn SecuritySection(index: usize) -> Element {
    let auth = use_auth();
    let toast = use_toast();

    let has_password = use_memo(move || {
        auth.current_user
            .read()
            .as_ref()
            .map(|u| u.has_password)
            .unwrap_or(false)
    });

    let user_email = use_memo(move || {
        auth.current_user
            .read()
            .as_ref()
            .map(|u| u.email.clone())
            .unwrap_or_default()
    });

    let mut current_password = use_signal(String::new);
    let mut new_password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let mut error_msg = use_signal(|| Option::<String>::None);

    rsx! {
        AccordionItem {
            index: index,

            AccordionTrigger { "Security" }
            AccordionContent {
                div {
                    class: "settings-section",

                    p {
                        class: "settings-section-desc",
                        if has_password() {
                            "Change your account password. You'll need your current password to set a new one."
                        } else {
                            "Set a password so you can also sign in with your email and password. This is required for mobile and desktop apps where OAuth isn't available."
                        }
                    }

                    Form {
                        onsubmit: move |_evt| async move {
                            saving.set(true);
                            error_msg.set(None);

                            let pw = new_password();
                            let confirm = confirm_password();

                            if pw.len() < MIN_PASSWORD_LENGTH {
                                error_msg.set(Some(format!(
                                    "Password must be at least {} characters",
                                    MIN_PASSWORD_LENGTH
                                )));
                                saving.set(false);
                                return;
                            }

                            if pw != confirm {
                                error_msg.set(Some("Passwords do not match".to_string()));
                                saving.set(false);
                                return;
                            }

                            let result = if has_password() {
                                server::api::change_password(current_password(), pw).await
                            } else {
                                server::api::set_oauth_account_password(user_email(), pw).await
                            };

                            match result {
                                Ok(resp) => {
                                    toast.success(resp.message, ToastOptions::new());
                                    current_password.set(String::new());
                                    new_password.set(String::new());
                                    confirm_password.set(String::new());
                                }
                                Err(e) => {
                                    let msg = shared_types::AppError::friendly_message(
                                        &e.to_string(),
                                    );
                                    error_msg.set(Some(msg.clone()));
                                    toast.error(msg, ToastOptions::new());
                                }
                            }

                            saving.set(false);
                        },

                        div {
                            class: "settings-form",

                            if let Some(err) = error_msg() {
                                div { class: "auth-error", "{err}" }
                            }

                            if has_password() {
                                div {
                                    class: "settings-field",
                                    Label { html_for: "current-password", "Current Password" }
                                    Input {
                                        input_type: "password",
                                        value: current_password(),
                                        placeholder: "Enter your current password",
                                        label: "",
                                        on_input: move |evt: FormEvent| {
                                            current_password.set(evt.value());
                                        },
                                    }
                                }
                            }

                            div {
                                class: "settings-field",
                                Label { html_for: "new-password", "New Password" }
                                Input {
                                    input_type: "password",
                                    value: new_password(),
                                    placeholder: "At least 8 characters",
                                    label: "",
                                    on_input: move |evt: FormEvent| {
                                        new_password.set(evt.value());
                                    },
                                }
                            }

                            div {
                                class: "settings-field",
                                Label { html_for: "confirm-password", "Confirm New Password" }
                                Input {
                                    input_type: "password",
                                    value: confirm_password(),
                                    placeholder: "Re-enter your new password",
                                    label: "",
                                    on_input: move |evt: FormEvent| {
                                        confirm_password.set(evt.value());
                                    },
                                }
                            }

                            Button {
                                variant: ButtonVariant::Primary,
                                disabled: saving(),
                                if saving() {
                                    if has_password() { "Changing Password..." } else { "Setting Password..." }
                                } else {
                                    if has_password() { "Change Password" } else { "Set Password" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
