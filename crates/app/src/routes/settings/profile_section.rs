use crate::auth::use_auth;
use crate::ProfileState;
use dioxus::prelude::*;
use shared_ui::{
    use_toast, AccordionContent, AccordionItem, AccordionTrigger, Avatar, AvatarFallback,
    AvatarImage, Button, ButtonVariant, Form, Input, Label, ToastOptions,
};

/// Profile accordion section: avatar upload and profile form.
#[component]
pub fn ProfileSection(index: usize) -> Element {
    let mut auth = use_auth();
    let profile: ProfileState = use_context();
    let toast = use_toast();

    let mut profile_name = use_signal(move || (profile.display_name)());
    let mut profile_email = use_signal(move || (profile.email)());

    let mut saving = use_signal(|| false);
    let mut profile_error = use_signal(|| Option::<String>::None);
    let mut profile_field_errors = use_signal(std::collections::HashMap::<String, String>::new);
    let mut uploading_avatar = use_signal(|| false);
    let mut avatar_popup_open = use_signal(|| false);

    rsx! {
        AccordionItem {
            index: index,
            default_open: true,

            AccordionTrigger { "Profile" }
            AccordionContent {
                div {
                    class: "settings-section",

                    // Avatar preview and upload
                    div {
                        class: "settings-avatar-section",
                        div {
                            class: "settings-avatar-preview",
                            onclick: move |_| {
                                if profile.avatar_url.read().is_some() {
                                    avatar_popup_open.set(true);
                                }
                            },
                            Avatar {
                                if let Some(url) = profile.avatar_url.read().as_ref() {
                                    AvatarImage { src: url.clone() }
                                }
                                AvatarFallback {
                                    {profile_name().split_whitespace().filter_map(|w| w.chars().next()).take(2).collect::<String>().to_uppercase()}
                                }
                            }
                        }
                        label {
                            class: if uploading_avatar() { "button avatar-upload-label disabled" } else { "button avatar-upload-label" },
                            "data-style": "outline",
                            input {
                                r#type: "file",
                                accept: "image/jpeg,image/png,image/webp",
                                class: "avatar-upload-input",
                                onchange: move |evt: FormEvent| async move {
                                    uploading_avatar.set(true);
                                    let files = evt.files();
                                    if let Some(file) = files.first() {
                                        if file.size() > 2 * 1024 * 1024 {
                                            toast.error("Avatar must be under 2 MB".to_string(), ToastOptions::new());
                                        } else {
                                            let content_type = file.content_type()
                                                .unwrap_or_else(|| "image/jpeg".to_string());
                                            match file.read_bytes().await {
                                                Ok(bytes) => {
                                                    use base64::Engine as _;
                                                    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                                                    match server::api::upload_user_avatar(encoded, content_type).await {
                                                        Ok(user) => {
                                                            auth.set_user(user);
                                                            toast.success("Avatar uploaded".to_string(), ToastOptions::new());
                                                        }
                                                        Err(e) => {
                                                            toast.error(
                                                                shared_types::AppError::friendly_message(&e.to_string()),
                                                                ToastOptions::new(),
                                                            );
                                                        }
                                                    }
                                                }
                                                Err(_) => {
                                                    toast.error("Failed to read file".to_string(), ToastOptions::new());
                                                }
                                            }
                                        }
                                    }
                                    uploading_avatar.set(false);
                                },
                            }
                            if uploading_avatar() { "Uploading..." } else { "Upload Avatar" }
                        }
                    }

                    Form {
                        onsubmit: move |_evt| async move {
                            saving.set(true);
                            profile_error.set(None);
                            profile_field_errors.set(std::collections::HashMap::new());

                            match server::api::update_profile(
                                profile_name(),
                                profile_email(),
                            )
                            .await
                            {
                                Ok(user) => {
                                    auth.set_user(user);
                                    toast.success(
                                        "Profile updated successfully".to_string(),
                                        ToastOptions::new(),
                                    );
                                }
                                Err(e) => {
                                    let err_str = e.to_string();
                                    let field_errs =
                                        shared_types::AppError::parse_field_errors(
                                            &err_str,
                                        );
                                    if field_errs.is_empty() {
                                        profile_error.set(Some(
                                            shared_types::AppError::friendly_message(
                                                &err_str,
                                            ),
                                        ));
                                    } else {
                                        profile_field_errors.set(field_errs);
                                    }
                                    toast.error(
                                        "Failed to update profile".to_string(),
                                        ToastOptions::new(),
                                    );
                                }
                            }
                            saving.set(false);
                        },

                        div {
                            class: "settings-form",

                            if let Some(err) = profile_error() {
                                div { class: "auth-error", "{err}" }
                            }

                            div {
                                class: "settings-field",
                                Label { html_for: "profile-name", "Display Name" }
                                Input {
                                    value: profile_name(),
                                    placeholder: "Enter your name",
                                    label: "",
                                    on_input: move |evt: FormEvent| {
                                        profile_name.set(evt.value());
                                    },
                                }
                                if let Some(err) = profile_field_errors().get("display_name") {
                                    div { class: "settings-field-error", "{err}" }
                                }
                            }

                            div {
                                class: "settings-field",
                                Label { html_for: "profile-email", "Email Address" }
                                Input {
                                    value: profile_email(),
                                    placeholder: "Enter your email",
                                    label: "",
                                    on_input: move |evt: FormEvent| {
                                        profile_email.set(evt.value());
                                    },
                                }
                                if let Some(err) = profile_field_errors().get("email") {
                                    div { class: "settings-field-error", "{err}" }
                                }
                            }

                            Button {
                                variant: ButtonVariant::Primary,
                                disabled: saving(),
                                if saving() { "Saving..." } else { "Save Profile" }
                            }
                        }
                    }
                }
            }
        }

        // Avatar popup overlay
        if avatar_popup_open() {
            div {
                class: "avatar-popup-overlay",
                onclick: move |_| avatar_popup_open.set(false),

                div {
                    class: "avatar-popup-frame",
                    onclick: move |evt: MouseEvent| evt.stop_propagation(),

                    if let Some(url) = profile.avatar_url.read().as_ref() {
                        img {
                            class: "avatar-popup-image",
                            src: url.clone(),
                            alt: "Avatar",
                        }
                    }
                }
            }
        }
    }
}
