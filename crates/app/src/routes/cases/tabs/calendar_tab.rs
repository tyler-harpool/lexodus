use dioxus::prelude::*;
use shared_types::CalendarEntryResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant,
    DataTable, DataTableBody, DataTableCell, DataTableColumn, DataTableHeader, DataTableRow,
    Form, FormSelect, Input, Separator,
    Sheet, SheetClose, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle,
    Skeleton,
};
use shared_ui::{use_toast, ToastOptions};

use crate::CourtContext;

/// Map event status to badge variant.
fn event_status_variant(status: &str) -> BadgeVariant {
    match status {
        "completed" => BadgeVariant::Secondary,
        "cancelled" => BadgeVariant::Destructive,
        "in_progress" => BadgeVariant::Primary,
        _ => BadgeVariant::Primary, // scheduled
    }
}

#[component]
pub fn CalendarTab(case_id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut show_sheet = use_signal(|| false);
    let mut form_event_type = use_signal(|| "hearing".to_string());
    let mut form_courtroom = use_signal(String::new);
    let mut form_scheduled_date = use_signal(String::new);
    let mut form_duration = use_signal(|| "60".to_string());
    let mut form_description = use_signal(String::new);

    let case_id_save = case_id.clone();

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move {
            server::api::list_calendar_by_case(court, cid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<CalendarEntryResponse>>(&json).ok())
        }
    });

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let cid = case_id_save.clone();
        let etype = form_event_type.read().clone();
        let courtroom = form_courtroom.read().clone();
        let date = form_scheduled_date.read().clone();
        let dur = form_duration.read().clone();
        let desc = form_description.read().clone();

        spawn(async move {
            if date.trim().is_empty() {
                toast.error("Date is required.".to_string(), ToastOptions::new());
                return;
            }
            let duration_min: i32 = dur.parse().unwrap_or(60);
            let body = serde_json::json!({
                "case_id": cid,
                "event_type": etype,
                "scheduled_date": format!("{}:00Z", date),
                "duration_minutes": duration_min,
                "courtroom": courtroom.trim(),
                "description": desc.trim(),
            });
            match server::api::schedule_calendar_event(court, body.to_string()).await {
                Ok(_) => {
                    toast.success("Event scheduled.".to_string(), ToastOptions::new());
                    show_sheet.set(false);
                    form_description.set(String::new());
                    form_courtroom.set(String::new());
                    form_scheduled_date.set(String::new());
                    data.restart();
                }
                Err(e) => toast.error(format!("Error: {e}"), ToastOptions::new()),
            }
        });
    };

    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-md);",
            h3 { "Case Calendar" }
            Button {
                variant: ButtonVariant::Primary,
                onclick: move |_| show_sheet.set(true),
                "Schedule Event"
            }
        }

        match &*data.read() {
            Some(Some(events)) if !events.is_empty() => rsx! {
                DataTable {
                    DataTableHeader {
                        DataTableColumn { "Date" }
                        DataTableColumn { "Event Type" }
                        DataTableColumn { "Courtroom" }
                        DataTableColumn { "Duration" }
                        DataTableColumn { "Status" }
                    }
                    DataTableBody {
                        for evt in events.iter() {
                            DataTableRow {
                                DataTableCell {
                                    {crate::format_helpers::format_datetime_human(&evt.scheduled_date)}
                                }
                                DataTableCell {
                                    Badge { variant: BadgeVariant::Secondary,
                                        {crate::format_helpers::format_snake_case_title(&evt.event_type)}
                                    }
                                }
                                DataTableCell { {evt.courtroom.clone()} }
                                DataTableCell {
                                    {format!("{} min", evt.duration_minutes)}
                                }
                                DataTableCell {
                                    Badge {
                                        variant: event_status_variant(&evt.status),
                                        {crate::format_helpers::format_snake_case_title(&evt.status)}
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Some(Some(_)) => rsx! {
                p { class: "empty-state", "No events scheduled for this case." }
            },
            Some(None) => rsx! {
                p { class: "error-state", "Failed to load calendar events." }
            },
            None => rsx! {
                Skeleton { style: "width: 100%; height: 200px" }
            },
        }

        Sheet {
            open: show_sheet(),
            on_close: move |_| show_sheet.set(false),
            side: SheetSide::Right,
            SheetContent {
                SheetHeader {
                    SheetTitle { "Schedule Event" }
                    SheetClose { on_close: move |_| show_sheet.set(false) }
                }
                Form {
                    onsubmit: handle_save,
                    div { class: "sheet-form",
                        FormSelect {
                            label: "Event Type",
                            value: "{form_event_type}",
                            onchange: move |e: Event<FormData>| form_event_type.set(e.value()),
                            option { value: "hearing", "Hearing" }
                            option { value: "trial", "Trial" }
                            option { value: "sentencing", "Sentencing" }
                            option { value: "arraignment", "Arraignment" }
                            option { value: "conference", "Conference" }
                            option { value: "motion_hearing", "Motion Hearing" }
                        }
                        Input {
                            label: "Date & Time",
                            input_type: "datetime-local",
                            value: form_scheduled_date(),
                            on_input: move |e: FormEvent| form_scheduled_date.set(e.value()),
                        }
                        Input {
                            label: "Courtroom",
                            value: form_courtroom(),
                            on_input: move |e: FormEvent| form_courtroom.set(e.value()),
                            placeholder: "e.g., Courtroom 4B",
                        }
                        Input {
                            label: "Duration (minutes)",
                            input_type: "number",
                            value: form_duration(),
                            on_input: move |e: FormEvent| form_duration.set(e.value()),
                        }
                        Input {
                            label: "Description",
                            value: form_description(),
                            on_input: move |e: FormEvent| form_description.set(e.value()),
                            placeholder: "Event description",
                        }
                    }
                    Separator {}
                    SheetFooter {
                        div { class: "sheet-footer-actions",
                            SheetClose { on_close: move |_| show_sheet.set(false) }
                            Button { variant: ButtonVariant::Primary, "Schedule" }
                        }
                    }
                }
            }
        }
    }
}
