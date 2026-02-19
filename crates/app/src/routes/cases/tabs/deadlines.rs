use dioxus::prelude::*;
use shared_types::DeadlineSearchResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant,
    DataTable, DataTableBody, DataTableCell, DataTableColumn, DataTableHeader, DataTableRow,
    Form, Input, Separator, Textarea,
    Sheet, SheetClose, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle,
    Skeleton,
};
use shared_ui::{use_toast, ToastOptions};

use crate::CourtContext;

/// Map deadline status to a badge variant for visual urgency.
fn status_variant(status: &str) -> BadgeVariant {
    match status {
        "missed" => BadgeVariant::Destructive,
        "open" => BadgeVariant::Primary,
        "met" => BadgeVariant::Secondary,
        _ => BadgeVariant::Secondary,
    }
}

#[component]
pub fn DeadlinesTab(case_id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // Create deadline form state
    let mut show_create_sheet = use_signal(|| false);
    let mut create_title = use_signal(String::new);
    let mut create_due_date = use_signal(String::new);
    let mut create_rule_code = use_signal(String::new);
    let mut create_notes = use_signal(String::new);

    // Extension request form state
    let mut show_extension_sheet = use_signal(|| false);
    let mut selected_deadline_id = use_signal(String::new);
    let mut ext_reason = use_signal(String::new);
    let mut ext_new_date = use_signal(String::new);

    let case_id_create = case_id.clone();

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move {
            let result = server::api::search_deadlines(
                court,
                None,
                Some(cid),
                None,
                None,
                None,
                Some(50),
            )
            .await;
            match result {
                Ok(json) => serde_json::from_str::<DeadlineSearchResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let mut open_extension = move |deadline_id: String| {
        selected_deadline_id.set(deadline_id);
        ext_reason.set(String::new());
        ext_new_date.set(String::new());
        show_extension_sheet.set(true);
    };

    let handle_create_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let cid = case_id_create.clone();
        let title = create_title.read().clone();
        let due_date = create_due_date.read().clone();
        let rule_code = create_rule_code.read().clone();
        let notes = create_notes.read().clone();

        spawn(async move {
            if title.trim().is_empty() {
                toast.error("Title is required.".to_string(), ToastOptions::new());
                return;
            }
            if due_date.is_empty() {
                toast.error("Due date is required.".to_string(), ToastOptions::new());
                return;
            }
            let mut body = serde_json::json!({
                "title": title.trim(),
                "case_id": cid,
                "due_at": format!("{due_date}T00:00:00Z"),
            });
            if !rule_code.trim().is_empty() {
                body["rule_code"] = serde_json::Value::String(rule_code.trim().to_string());
            }
            if !notes.trim().is_empty() {
                body["notes"] = serde_json::Value::String(notes.trim().to_string());
            }
            match server::api::create_deadline(court, body.to_string()).await {
                Ok(_) => {
                    toast.success("Deadline created.".to_string(), ToastOptions::new());
                    show_create_sheet.set(false);
                    create_title.set(String::new());
                    create_due_date.set(String::new());
                    create_rule_code.set(String::new());
                    create_notes.set(String::new());
                    data.restart();
                }
                Err(e) => toast.error(format!("Error: {e}"), ToastOptions::new()),
            }
        });
    };

    let handle_extension_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let did = selected_deadline_id.read().clone();
        let reason = ext_reason.read().clone();
        let new_date = ext_new_date.read().clone();

        spawn(async move {
            if reason.trim().is_empty() {
                toast.error("Reason is required.".to_string(), ToastOptions::new());
                return;
            }
            let body = serde_json::json!({
                "requested_by": "current_user",
                "reason": reason.trim(),
                "requested_new_date": if new_date.is_empty() { None } else { Some(format!("{new_date}T00:00:00Z")) },
            });
            match server::api::create_extension_request_fn(court, did, body.to_string()).await {
                Ok(_) => {
                    toast.success("Extension requested.".to_string(), ToastOptions::new());
                    show_extension_sheet.set(false);
                    data.restart();
                }
                Err(e) => toast.error(format!("Error: {e}"), ToastOptions::new()),
            }
        });
    };

    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-md);",
            h3 { "Case Deadlines" }
            Button {
                variant: ButtonVariant::Primary,
                onclick: move |_| show_create_sheet.set(true),
                "Add Deadline"
            }
        }

        match &*data.read() {
            Some(Some(resp)) => {
                let deadlines = &resp.deadlines;
                if deadlines.is_empty() {
                    rsx! { p { class: "empty-state", "No deadlines for this case." } }
                } else {
                    rsx! {
                        DataTable {
                            DataTableHeader {
                                DataTableColumn { "Title" }
                                DataTableColumn { "Due Date" }
                                DataTableColumn { "Status" }
                                DataTableColumn { "Rule" }
                                DataTableColumn { "Actions" }
                            }
                            DataTableBody {
                                for dl in deadlines.iter() {
                                    {
                                        let dl_id = dl.id.clone();
                                        let display_status = dl.status.replace('_', " ");
                                        let due = if dl.due_at.len() >= 10 { &dl.due_at[..10] } else { &dl.due_at };
                                        rsx! {
                                            DataTableRow {
                                                DataTableCell { {dl.title.clone()} }
                                                DataTableCell { {due.to_string()} }
                                                DataTableCell {
                                                    Badge { variant: status_variant(&dl.status),
                                                        {display_status}
                                                    }
                                                }
                                                DataTableCell {
                                                    {dl.rule_code.clone().unwrap_or_else(|| "—".to_string())}
                                                }
                                                DataTableCell {
                                                    Button {
                                                        variant: ButtonVariant::Outline,
                                                        onclick: move |_| open_extension(dl_id.clone()),
                                                        "Request Extension"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Some(None) => rsx! {
                p { class: "error-state", "Failed to load deadlines." }
            },
            None => rsx! {
                Skeleton { style: "width: 100%; height: 200px" }
            },
        }

        // Create Deadline Sheet
        Sheet {
            open: show_create_sheet(),
            on_close: move |_| show_create_sheet.set(false),
            side: SheetSide::Right,
            SheetContent {
                SheetHeader {
                    SheetTitle { "Add Deadline" }
                    SheetClose { on_close: move |_| show_create_sheet.set(false) }
                }
                Form {
                    onsubmit: handle_create_save,
                    div { class: "sheet-form",
                        Input {
                            label: "Title",
                            value: create_title(),
                            on_input: move |e: FormEvent| create_title.set(e.value()),
                            placeholder: "e.g. Discovery cutoff",
                        }
                        Input {
                            label: "Due Date",
                            input_type: "date",
                            value: create_due_date(),
                            on_input: move |e: FormEvent| create_due_date.set(e.value()),
                        }
                        Input {
                            label: "Rule Code (optional)",
                            value: create_rule_code(),
                            on_input: move |e: FormEvent| create_rule_code.set(e.value()),
                            placeholder: "e.g. FRCP 26(a)",
                        }
                        Textarea {
                            label: "Notes (optional)",
                            value: create_notes(),
                            on_input: move |e: FormEvent| create_notes.set(e.value()),
                            placeholder: "Additional notes about this deadline",
                        }
                    }
                    Separator {}
                    SheetFooter {
                        div { class: "sheet-footer-actions",
                            SheetClose { on_close: move |_| show_create_sheet.set(false) }
                            Button { variant: ButtonVariant::Primary, "Create Deadline" }
                        }
                    }
                }
            }
        }

        // Extension Request Sheet
        Sheet {
            open: show_extension_sheet(),
            on_close: move |_| show_extension_sheet.set(false),
            side: SheetSide::Right,
            SheetContent {
                SheetHeader {
                    SheetTitle { "Request Extension" }
                    SheetClose { on_close: move |_| show_extension_sheet.set(false) }
                }
                Form {
                    onsubmit: handle_extension_save,
                    div { class: "sheet-form",
                        Input {
                            label: "Reason",
                            value: ext_reason(),
                            on_input: move |e: FormEvent| ext_reason.set(e.value()),
                            placeholder: "Reason for extension request",
                        }
                        Input {
                            label: "Requested New Date",
                            input_type: "date",
                            value: ext_new_date(),
                            on_input: move |e: FormEvent| ext_new_date.set(e.value()),
                        }
                    }
                    Separator {}
                    SheetFooter {
                        div { class: "sheet-footer-actions",
                            SheetClose { on_close: move |_| show_extension_sheet.set(false) }
                            Button { variant: ButtonVariant::Primary, "Submit Request" }
                        }
                    }
                }
            }
        }
    }
}
