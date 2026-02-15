use dioxus::prelude::*;
use shared_types::{CalendarEntryResponse, CalendarSearchResponse};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Form, FormSelect, Input,
    PageActions, PageHeader, PageTitle, Pagination, SearchBar, Separator, Sheet, SheetClose,
    SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide, SheetTitle, Skeleton,
    Textarea,
};
use shared_ui::{
    use_toast, Calendar, CalendarGrid, CalendarHeader, CalendarMonthTitle, CalendarNavigation,
    CalendarNextMonthButton, CalendarPreviousMonthButton, Date, ToastOptions, UtcDateTime,
};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn CalendarListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut offset = use_signal(|| 0i64);
    let mut search_event_type = use_signal(String::new);
    let mut search_status = use_signal(String::new);
    let limit: i64 = 20;

    let mut selected_date = use_signal(|| None::<Date>);
    let mut view_date = use_signal(|| UtcDateTime::now().date());

    // Sheet state for scheduling events
    let mut show_sheet = use_signal(|| false);
    let mut form_case_id = use_signal(String::new);
    let mut form_judge_id = use_signal(String::new);
    let mut form_event_type = use_signal(|| "motion_hearing".to_string());
    let mut form_scheduled_date = use_signal(String::new);
    let mut form_duration = use_signal(|| "60".to_string());
    let mut form_courtroom = use_signal(String::new);
    let mut form_description = use_signal(String::new);
    let mut form_participants = use_signal(String::new);

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let et = search_event_type.read().clone();
        let st = search_status.read().clone();
        let off = *offset.read();
        async move {
            let result = server::api::search_calendar_events(
                court,
                None, // judge_id
                None, // courtroom
                if et.is_empty() { None } else { Some(et) },
                if st.is_empty() { None } else { Some(st) },
                None, // date_from
                None, // date_to
                Some(off),
                Some(limit),
            )
            .await;

            match result {
                Ok(json) => serde_json::from_str::<CalendarSearchResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_case_id.set(String::new());
        form_judge_id.set(String::new());
        form_event_type.set("motion_hearing".to_string());
        form_scheduled_date.set(String::new());
        form_duration.set("60".to_string());
        form_courtroom.set(String::new());
        form_description.set(String::new());
        form_participants.set(String::new());
    };

    let open_create = move |_| {
        reset_form();
        show_sheet.set(true);
    };

    let handle_clear = move |_| {
        search_event_type.set(String::new());
        search_status.set(String::new());
        offset.set(0);
    };

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let participants_vec: Vec<String> = form_participants
            .read()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let body = serde_json::json!({
            "case_id": form_case_id.read().clone(),
            "judge_id": form_judge_id.read().clone(),
            "event_type": form_event_type.read().clone(),
            "scheduled_date": form_scheduled_date.read().clone(),
            "duration_minutes": form_duration.read().parse::<i32>().unwrap_or(60),
            "courtroom": form_courtroom.read().clone(),
            "description": form_description.read().clone(),
            "participants": participants_vec,
            "is_public": true,
        });

        spawn(async move {
            match server::api::schedule_calendar_event(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Event scheduled successfully".to_string(),
                        ToastOptions::new(),
                    );
                }
                Err(e) => {
                    toast.error(format!("{}", e), ToastOptions::new());
                }
            }
        });
    };

    rsx! {
        div { class: "container",
            PageHeader {
                PageTitle { "Calendar" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "Schedule Event"
                    }
                }
            }

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
                                reset_form();
                                form_scheduled_date.set(iso);
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

            // Schedule event Sheet (like Products "New Product")
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "Schedule Event" }
                        SheetDescription {
                            "Fill in the details to schedule a new calendar event."
                        }
                        SheetClose { on_close: move |_| show_sheet.set(false) }
                    }

                    Form {
                        onsubmit: handle_save,

                        div {
                            class: "sheet-form",

                            FormSelect {
                                label: "Event Type *",
                                value: "{form_event_type}",
                                onchange: move |evt: Event<FormData>| form_event_type.set(evt.value().to_string()),
                                option { value: "initial_appearance", "Initial Appearance" }
                                option { value: "arraignment", "Arraignment" }
                                option { value: "bail_hearing", "Bail Hearing" }
                                option { value: "plea_hearing", "Plea Hearing" }
                                option { value: "trial_date", "Trial Date" }
                                option { value: "sentencing", "Sentencing" }
                                option { value: "violation_hearing", "Violation Hearing" }
                                option { value: "status_conference", "Status Conference" }
                                option { value: "scheduling_conference", "Scheduling Conference" }
                                option { value: "settlement_conference", "Settlement Conference" }
                                option { value: "pretrial_conference", "Pretrial Conference" }
                                option { value: "motion_hearing", "Motion Hearing" }
                                option { value: "evidentiary_hearing", "Evidentiary Hearing" }
                                option { value: "jury_selection", "Jury Selection" }
                                option { value: "jury_trial", "Jury Trial" }
                                option { value: "bench_trial", "Bench Trial" }
                                option { value: "show_cause_hearing", "Show Cause Hearing" }
                                option { value: "contempt_hearing", "Contempt Hearing" }
                                option { value: "emergency_hearing", "Emergency Hearing" }
                                option { value: "telephonic", "Telephonic" }
                                option { value: "video_conference", "Video Conference" }
                            }

                            Input {
                                label: "Scheduled Date/Time *",
                                value: form_scheduled_date.read().clone(),
                                on_input: move |e: FormEvent| form_scheduled_date.set(e.value().to_string()),
                                placeholder: "2026-06-15T09:00:00Z",
                            }

                            Input {
                                label: "Case ID *",
                                value: form_case_id.read().clone(),
                                on_input: move |e: FormEvent| form_case_id.set(e.value().to_string()),
                                placeholder: "UUID of the case",
                            }

                            Input {
                                label: "Judge ID *",
                                value: form_judge_id.read().clone(),
                                on_input: move |e: FormEvent| form_judge_id.set(e.value().to_string()),
                                placeholder: "UUID of the judge",
                            }

                            Input {
                                label: "Duration (minutes) *",
                                input_type: "number",
                                value: form_duration.read().clone(),
                                on_input: move |e: FormEvent| form_duration.set(e.value().to_string()),
                            }

                            Input {
                                label: "Courtroom *",
                                value: form_courtroom.read().clone(),
                                on_input: move |e: FormEvent| form_courtroom.set(e.value().to_string()),
                                placeholder: "Courtroom 4A",
                            }

                            Textarea {
                                label: "Description *",
                                value: form_description.read().clone(),
                                on_input: move |e: FormEvent| form_description.set(e.value().to_string()),
                                placeholder: "Brief description of the event",
                            }

                            Input {
                                label: "Participants (comma-separated)",
                                value: form_participants.read().clone(),
                                on_input: move |e: FormEvent| form_participants.set(e.value().to_string()),
                                placeholder: "Prosecutor, Defense Counsel",
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Schedule"
                                }
                            }
                        }
                    }
                }
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

fn format_scheduled_date(date_str: &str) -> String {
    if date_str.len() >= 16 {
        date_str[..16].replace('T', " ")
    } else {
        date_str.to_string()
    }
}
