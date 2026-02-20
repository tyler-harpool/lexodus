use dioxus::prelude::*;
use shared_types::CalendarEntryResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, FormSelect,
    PageActions, PageHeader, PageTitle, Pagination, SearchBar, Skeleton,
};
use shared_ui::{
    Calendar, CalendarGrid, CalendarHeader, CalendarMonthTitle, CalendarNavigation,
    CalendarNextMonthButton, CalendarPreviousMonthButton, Date, UtcDateTime,
};

use super::form_sheet::{CalendarFormSheet, FormMode};
use crate::auth::{can, use_user_role, Action};
use crate::auth::use_auth;
use crate::components::scope_toggle::{resolve_scope_filter, ListScope, ScopeToggle};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn CalendarListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let role = use_user_role();

    let mut offset = use_signal(|| 0i64);
    let mut search_event_type = use_signal(String::new);
    let mut search_status = use_signal(String::new);
    let limit: i64 = 20;

    let mut selected_date = use_signal(|| None::<Date>);
    let mut view_date = use_signal(|| UtcDateTime::now().date());

    let mut show_sheet = use_signal(|| false);
    let mut prefill_date = use_signal(|| None::<String>);

    // Scope toggle: My Items vs All Court (only rendered for Judge/Attorney)
    let scope = use_signal(|| ListScope::MyItems);
    let auth = use_auth();

    let resource_role = role.clone();
    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let et = search_event_type.read().clone();
        let st = search_status.read().clone();
        let off = *offset.read();
        let user = auth.current_user.read();
        let linked_attorney = user.as_ref().and_then(|u| u.linked_attorney_id.clone());
        let linked_judge = user.as_ref().and_then(|u| u.linked_judge_id.clone());
        let role_clone = resource_role.clone();
        let scope_filter = resolve_scope_filter(*scope.read(), &role_clone, &linked_attorney, &linked_judge);
        async move {
            // If in "My Items" mode with an attorney_id, use the dedicated endpoint
            if let Some(attorney_id) = scope_filter.attorney_id {
                let events = server::api::list_calendar_events_for_attorney(
                    court,
                    attorney_id,
                    None, // date_from (show all)
                )
                .await
                .unwrap_or_default();

                return Some(shared_types::CalendarSearchResponse {
                    total: events.len() as i64,
                    events,
                });
            }

            // For judge scope, pass judge_id to the search endpoint
            let judge_filter = scope_filter.judge_id;

            let result = server::api::search_calendar_events(
                court,
                judge_filter, // judge_id filter
                None, // courtroom
                if et.is_empty() { None } else { Some(et) },
                if st.is_empty() { None } else { Some(st) },
                None, // date_from
                None, // date_to
                Some(off),
                Some(limit),
            )
            .await;

            result.ok()
        }
    });

    let handle_clear = move |_| {
        search_event_type.set(String::new());
        search_status.set(String::new());
        offset.set(0);
    };

    rsx! {
        div { class: "container",
            PageHeader {
                PageTitle { "Calendar" }
                PageActions {
                    if can(&role, Action::CreateCase) {
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                prefill_date.set(None);
                                show_sheet.set(true);
                            },
                            "Schedule Event"
                        }
                    }
                }
            }

            // My Items / All Court toggle (only for Judge and Attorney roles)
            ScopeToggle { scope: scope }

            div { class: "calendar-layout",
                div { class: "calendar-widget",
                    Calendar {
                        selected_date: selected_date,
                        on_date_change: move |date: Option<Date>| {
                            selected_date.set(date);
                            if let Some(d) = date {
                                let iso = format!(
                                    "{}-{:02}-{:02}T09:00:00Z",
                                    d.year(),
                                    d.month() as u8,
                                    d.day()
                                );
                                prefill_date.set(Some(iso));
                                show_sheet.set(true);
                            }
                        },
                        view_date: view_date,
                        on_view_change: move |new_view: Date| {
                            view_date.set(new_view);
                        },
                        CalendarHeader {
                            CalendarNavigation {
                                CalendarPreviousMonthButton { "\u{2039}" }
                                CalendarMonthTitle {}
                                CalendarNextMonthButton { "\u{203a}" }
                            }
                        }
                        CalendarGrid {}
                    }

                    if let Some(date) = selected_date() {
                        div { class: "selected-date-info",
                            Badge {
                                variant: BadgeVariant::Primary,
                                "{date.year()}-{date.month() as u8:02}-{date.day():02}"
                            }
                        }
                    }
                }

                div { class: "calendar-events-list",
                    h2 { class: "calendar-events-title", "Upcoming Events" }

                    SearchBar {
                        FormSelect {
                            value: "{search_event_type}",
                            onchange: move |evt: Event<FormData>| {
                                search_event_type.set(evt.value().to_string());
                                offset.set(0);
                            },
                            option { value: "", "All Event Types" }
                            option { value: "initial_appearance", "Initial Appearance" }
                            option { value: "arraignment", "Arraignment" }
                            option { value: "bail_hearing", "Bail Hearing" }
                            option { value: "plea_hearing", "Plea Hearing" }
                            option { value: "trial_date", "Trial Date" }
                            option { value: "sentencing", "Sentencing" }
                            option { value: "motion_hearing", "Motion Hearing" }
                            option { value: "pretrial_conference", "Pretrial Conference" }
                            option { value: "status_conference", "Status Conference" }
                            option { value: "jury_trial", "Jury Trial" }
                            option { value: "bench_trial", "Bench Trial" }
                            option { value: "emergency_hearing", "Emergency Hearing" }
                        }
                        FormSelect {
                            value: "{search_status}",
                            onchange: move |evt: Event<FormData>| {
                                search_status.set(evt.value().to_string());
                                offset.set(0);
                            },
                            option { value: "", "All Statuses" }
                            option { value: "scheduled", "Scheduled" }
                            option { value: "confirmed", "Confirmed" }
                            option { value: "in_progress", "In Progress" }
                            option { value: "completed", "Completed" }
                            option { value: "cancelled", "Cancelled" }
                            option { value: "postponed", "Postponed" }
                            option { value: "recessed", "Recessed" }
                            option { value: "continued", "Continued" }
                        }
                        if !search_event_type.read().is_empty() || !search_status.read().is_empty() {
                            Button {
                                variant: ButtonVariant::Secondary,
                                onclick: handle_clear,
                                "Clear"
                            }
                        }
                    }

                    match &*data.read() {
                        Some(Some(resp)) => rsx! {
                            CalendarTable { events: resp.events.clone() }
                            Pagination {
                                total: resp.total,
                                offset: offset,
                                limit: limit,
                            }
                        },
                        Some(None) => rsx! {
                            Card {
                                CardContent {
                                    p { "No calendar events found." }
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

            CalendarFormSheet {
                mode: FormMode::Create,
                initial: None,
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                on_saved: move |_| data.restart(),
                prefill_date: prefill_date.read().clone(),
            }
        }
    }
}

#[component]
fn CalendarTable(events: Vec<CalendarEntryResponse>) -> Element {
    if events.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No calendar events found." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Event Type" }
                DataTableColumn { "Scheduled Date" }
                DataTableColumn { "Courtroom" }
                DataTableColumn { "Duration" }
                DataTableColumn { "Status" }
            }
            DataTableBody {
                for event in events {
                    CalendarRow { event: event }
                }
            }
        }
    }
}

#[component]
fn CalendarRow(event: CalendarEntryResponse) -> Element {
    let id = event.id.clone();
    let badge_variant = status_badge_variant(&event.status);
    let display_type = format_event_type(&event.event_type);
    let display_date = format_scheduled_date(&event.scheduled_date);

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::CalendarDetail { id: id.clone() });
            },
            DataTableCell { "{display_type}" }
            DataTableCell { "{display_date}" }
            DataTableCell { "{event.courtroom}" }
            DataTableCell { "{event.duration_minutes} min" }
            DataTableCell {
                Badge { variant: badge_variant, "{event.status}" }
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
    crate::format_helpers::format_snake_case_title(et)
}

fn format_scheduled_date(date_str: &str) -> String {
    crate::format_helpers::format_datetime_human(date_str)
}
