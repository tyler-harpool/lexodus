use dioxus::prelude::*;
use shared_types::CalendarEntryResponse;
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, Card, CardContent, CardHeader, CardTitle, DetailFooter, DetailGrid, DetailItem,
    DetailList, PageActions, PageHeader, PageTitle, Skeleton,
};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn CalendarDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let event_id = id.clone();

    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let data = use_resource(move || {
        let court = court_id.clone();
        let eid = event_id.clone();
        async move {
            match server::api::get_calendar_event(court, eid).await {
                Ok(json) => serde_json::from_str::<CalendarEntryResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let eid = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_calendar_event(court, eid).await {
                Ok(()) => {
                    let nav = navigator();
                    nav.push(Route::CalendarList {});
                }
                Err(_) => {
                    deleting.set(false);
                    show_delete_confirm.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(event)) => rsx! {
                    PageHeader {
                        PageTitle { "{format_event_type(&event.event_type)}" }
                        PageActions {
                            Link { to: Route::CalendarList {},
                                Button { variant: ButtonVariant::Secondary, "Back to Calendar" }
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
                            AlertDialogTitle { "Delete Calendar Event" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this calendar event? This action cannot be undone."
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

                    DetailGrid {
                        Card {
                            CardHeader { CardTitle { "Event Information" } }
                            CardContent {
                                DetailList {
                                    DetailItem { label: "Event Type", value: format_event_type(&event.event_type) }
                                    DetailItem { label: "Scheduled Date", value: event.scheduled_date.clone() }
                                    DetailItem { label: "Duration", value: format!("{} minutes", event.duration_minutes) }
                                    DetailItem { label: "Courtroom", value: event.courtroom.clone() }
                                    DetailItem { label: "Description", value: event.description.clone() }
                                    DetailItem { label: "Status",
                                        Badge {
                                            variant: status_badge_variant(&event.status),
                                            "{event.status}"
                                        }
                                    }
                                    DetailItem { label: "Public",
                                        span {
                                            if event.is_public { "Yes" } else { "No" }
                                        }
                                    }
                                }
                            }
                        }

                        Card {
                            CardHeader { CardTitle { "Case & Judge" } }
                            CardContent {
                                DetailList {
                                    DetailItem { label: "Case ID", value: event.case_id.clone() }
                                    DetailItem { label: "Judge ID", value: event.judge_id.clone() }
                                    if let Some(reporter) = &event.court_reporter {
                                        DetailItem { label: "Court Reporter", value: reporter.clone() }
                                    }
                                }
                            }
                        }

                        Card {
                            CardHeader { CardTitle { "Participants" } }
                            CardContent {
                                if event.participants.is_empty() {
                                    p { "No participants listed." }
                                } else {
                                    DetailList {
                                        for participant in &event.participants {
                                            DetailItem {
                                                label: "Participant",
                                                value: participant.clone(),
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        Card {
                            CardHeader { CardTitle { "Timing" } }
                            CardContent {
                                if event.actual_start.is_some() || event.actual_end.is_some() || event.call_time.is_some() {
                                    DetailList {
                                        if let Some(start) = &event.actual_start {
                                            DetailItem { label: "Actual Start", value: start.clone() }
                                        }
                                        if let Some(end) = &event.actual_end {
                                            DetailItem { label: "Actual End", value: end.clone() }
                                        }
                                        if let Some(call) = &event.call_time {
                                            DetailItem { label: "Call Time", value: call.clone() }
                                        }
                                    }
                                } else {
                                    p { "No timing data recorded yet." }
                                }
                            }
                        }

                        if !event.notes.is_empty() {
                            Card {
                                CardHeader { CardTitle { "Notes" } }
                                CardContent {
                                    p { "{event.notes}" }
                                }
                            }
                        }
                    }

                    DetailFooter {
                        span { "ID: {event.id}" }
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Calendar Event Not Found" }
                                p { "The calendar event you're looking for doesn't exist in this court district." }
                                Link { to: Route::CalendarList {},
                                    Button { "Back to Calendar" }
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

fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "scheduled" | "confirmed" => BadgeVariant::Primary,
        "completed" => BadgeVariant::Secondary,
        "in_progress" => BadgeVariant::Outline,
        "cancelled" | "postponed" => BadgeVariant::Destructive,
        "recessed" | "continued" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}

fn format_event_type(et: &str) -> String {
    et.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
