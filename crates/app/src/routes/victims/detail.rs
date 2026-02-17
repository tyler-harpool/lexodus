use dioxus::prelude::*;
use shared_types::{VictimNotificationResponse, VictimResponse};
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, Card, CardContent, CardHeader, CardTitle, DetailGrid, DetailItem, DetailList,
    PageActions, PageHeader, PageTitle, Skeleton, TabContent, TabList, TabTrigger, Tabs,
};
use shared_ui::{use_toast, ToastOptions};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn VictimDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let victim_id = id.clone();
    let toast = use_toast();

    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let data = use_resource(move || {
        let court = court_id.clone();
        let vid = victim_id.clone();
        async move {
            match server::api::get_victim(court, vid).await {
                Ok(json) => serde_json::from_str::<VictimResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let vid = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_victim(court, vid).await {
                Ok(()) => {
                    toast.success(
                        "Victim deleted successfully".to_string(),
                        ToastOptions::new(),
                    );
                    let nav = navigator();
                    nav.push(Route::VictimList {});
                }
                Err(e) => {
                    toast.error(format!("{}", e), ToastOptions::new());
                    deleting.set(false);
                    show_delete_confirm.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(victim)) => rsx! {
                    PageHeader {
                        PageTitle { "{victim.name}" }
                        PageActions {
                            Link { to: Route::VictimList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                            Button {
                                variant: ButtonVariant::Destructive,
                                onclick: move |_| show_delete_confirm.set(true),
                                "Delete"
                            }
                        }
                    }

                    AlertDialogRoot {
                        open: show_delete_confirm(),
                        on_open_change: move |v| show_delete_confirm.set(v),
                        AlertDialogContent {
                            AlertDialogTitle { "Delete Victim" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this victim record? This action cannot be undone."
                            }
                            AlertDialogActions {
                                AlertDialogCancel { "Cancel" }
                                AlertDialogAction {
                                    on_click: handle_delete,
                                    if *deleting.read() { "Deleting..." } else { "Delete" }
                                }
                            }
                        }
                    }

                    Tabs { default_value: "profile", horizontal: true,
                        TabList {
                            TabTrigger { value: "profile", index: 0usize, "Profile" }
                            TabTrigger { value: "notifications", index: 1usize, "Notifications" }
                        }
                        TabContent { value: "profile", index: 0usize,
                            ProfileTab { victim: victim.clone() }
                        }
                        TabContent { value: "notifications", index: 1usize,
                            NotificationsTab { victim_id: id.clone() }
                        }
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Victim Not Found" }
                                p { "The victim record you're looking for doesn't exist in this court district." }
                                Link { to: Route::VictimList {},
                                    Button { "Back to List" }
                                }
                            }
                        }
                    }
                },
                None => rsx! {
                    div { class: "loading",
                        Skeleton {}
                        Skeleton {}
                        Skeleton {}
                    }
                },
            }
        }
    }
}

/// Profile tab showing the victim's personal and notification information.
#[component]
fn ProfileTab(victim: VictimResponse) -> Element {
    let v = &victim;
    let email_display = v
        .notification_email
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let phone_display = v
        .notification_phone
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let mail_display = if v.notification_mail {
        "Yes".to_string()
    } else {
        "No".to_string()
    };
    let type_variant = victim_type_badge_variant(&v.victim_type);

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Personal Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Name", value: v.name.clone() }
                        DetailItem { label: "Type",
                            Badge { variant: type_variant, "{v.victim_type}" }
                        }
                        DetailItem { label: "Case ID", value: v.case_id.clone() }
                        DetailItem {
                            label: "Created",
                            value: v.created_at.chars().take(10).collect::<String>()
                        }
                        DetailItem {
                            label: "Updated",
                            value: v.updated_at.chars().take(10).collect::<String>()
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Notification Preferences" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Email", value: email_display }
                        DetailItem { label: "Phone", value: phone_display }
                        DetailItem { label: "Postal Mail", value: mail_display }
                    }
                }
            }
        }
    }
}

/// Notifications tab listing all notifications sent to this victim.
#[component]
fn NotificationsTab(victim_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let notifications = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let vid = victim_id.clone();
        async move {
            match server::api::list_victim_notifications(court, vid).await {
                Ok(json) => serde_json::from_str::<Vec<VictimNotificationResponse>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        match &*notifications.read() {
            Some(Some(list)) if !list.is_empty() => rsx! {
                div { class: "charges-list",
                    for notif in list.iter() {
                        NotificationCard { notification: notif.clone() }
                    }
                }
            },
            Some(_) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No notifications have been sent to this victim." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// Individual notification card display.
#[component]
fn NotificationCard(notification: VictimNotificationResponse) -> Element {
    let method_variant = method_badge_variant(&notification.method);
    let ack_variant = if notification.acknowledged {
        BadgeVariant::Primary
    } else {
        BadgeVariant::Outline
    };
    let ack_label = if notification.acknowledged {
        "Acknowledged"
    } else {
        "Pending"
    };

    let sent_date = notification
        .sent_at
        .chars()
        .take(10)
        .collect::<String>();

    let ack_date = notification
        .acknowledged_at
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());

    let summary_display = notification
        .content_summary
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        Card {
            CardHeader {
                CardTitle { "{notification.notification_type}" }
            }
            CardContent {
                DetailList {
                    DetailItem { label: "Method",
                        Badge { variant: method_variant, "{notification.method}" }
                    }
                    DetailItem { label: "Sent", value: sent_date }
                    DetailItem { label: "Summary", value: summary_display }
                    DetailItem { label: "Status",
                        Badge { variant: ack_variant, "{ack_label}" }
                    }
                    DetailItem { label: "Acknowledged At", value: ack_date }
                }
            }
        }
    }
}

/// Map victim type to a badge variant.
fn victim_type_badge_variant(victim_type: &str) -> BadgeVariant {
    match victim_type {
        "Individual" => BadgeVariant::Primary,
        "Organization" => BadgeVariant::Secondary,
        "Government" => BadgeVariant::Outline,
        "Minor" => BadgeVariant::Destructive,
        "Deceased" => BadgeVariant::Destructive,
        "Anonymous" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}

/// Map notification method to a badge variant.
fn method_badge_variant(method: &str) -> BadgeVariant {
    match method {
        "Email" => BadgeVariant::Primary,
        "Phone" => BadgeVariant::Secondary,
        "Mail" => BadgeVariant::Outline,
        "In-App" => BadgeVariant::Primary,
        "Fax" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}
