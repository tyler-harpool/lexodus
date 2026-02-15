use crate::auth::use_auth;
use dioxus::prelude::*;
use shared_ui::{
    use_toast, AccordionContent, AccordionItem, AccordionTrigger, Separator, Switch, SwitchThumb,
    ToastOptions,
};

/// Notifications accordion section: email, push, and digest toggles.
/// Values are loaded from the authenticated user and persisted to the database.
#[component]
pub fn NotificationsSection(index: usize) -> Element {
    let mut auth = use_auth();
    let toast = use_toast();

    // Initialize signals from current user (falls back to defaults if not logged in)
    let (init_email, init_push, init_digest) = {
        let guard = auth.current_user.read();
        match guard.as_ref() {
            Some(user) => (
                user.email_notifications_enabled,
                user.push_notifications_enabled,
                user.weekly_digest_enabled,
            ),
            None => (true, false, true),
        }
    };

    let mut email_notifs = use_signal(move || init_email);
    let mut push_notifs = use_signal(move || init_push);
    let mut weekly_digest = use_signal(move || init_digest);
    let mut saving = use_signal(|| false);

    // Persist all three preferences to the database
    let save_preferences = move |_: ()| async move {
        saving.set(true);
        match server::api::update_notification_preferences(
            email_notifs(),
            push_notifs(),
            weekly_digest(),
        )
        .await
        {
            Ok(user) => {
                auth.set_user(user);
            }
            Err(e) => {
                toast.error(
                    shared_types::AppError::friendly_message(&e.to_string()),
                    ToastOptions::new(),
                );
            }
        }
        saving.set(false);
    };

    rsx! {
        AccordionItem {
            index: index,

            AccordionTrigger { "Notifications" }
            AccordionContent {
                div {
                    class: "settings-section",

                    div {
                        class: "settings-toggle-row",
                        span {
                            class: "settings-toggle-label",
                            "Email notifications"
                        }
                        Switch {
                            checked: Some(email_notifs()),
                            on_checked_change: move |val: bool| {
                                email_notifs.set(val);
                                spawn(save_preferences(()));
                            },
                            SwitchThumb {}
                        }
                    }

                    Separator {}

                    div {
                        class: "settings-toggle-row",
                        span {
                            class: "settings-toggle-label",
                            "Push notifications"
                        }
                        Switch {
                            checked: Some(push_notifs()),
                            on_checked_change: move |val: bool| {
                                push_notifs.set(val);
                                spawn(save_preferences(()));
                                if val {
                                    crate::notify::send(
                                        "Push notifications enabled",
                                        "You will now receive desktop notifications.",
                                    );
                                }
                            },
                            SwitchThumb {}
                        }
                    }

                    Separator {}

                    div {
                        class: "settings-toggle-row",
                        span {
                            class: "settings-toggle-label",
                            "Weekly digest"
                        }
                        Switch {
                            checked: Some(weekly_digest()),
                            on_checked_change: move |val: bool| {
                                weekly_digest.set(val);
                                spawn(save_preferences(()));
                            },
                            SwitchThumb {}
                        }
                    }

                    if saving() {
                        div {
                            class: "settings-saving-indicator",
                            "Saving..."
                        }
                    }
                }
            }
        }
    }
}
