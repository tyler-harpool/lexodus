use dioxus::prelude::*;
use shared_types::{DeadlineResponse, DeadlineSearchResponse};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Form, FormSelect, Input,
    PageActions, PageHeader, PageTitle, Pagination, SearchBar, Separator, Sheet, SheetClose,
    SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide, SheetTitle, Skeleton,
    Textarea,
};
use shared_ui::{use_toast, HoverCard, HoverCardContent, HoverCardTrigger, ToastOptions};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn DeadlineListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut offset = use_signal(|| 0i64);
    let mut search_status = use_signal(String::new);
    let limit: i64 = 20;

    // Sheet state for creating deadlines
    let mut show_sheet = use_signal(|| false);
    let mut form_title = use_signal(String::new);
    let mut form_due_at = use_signal(String::new);
    let mut form_rule_code = use_signal(String::new);
    let mut form_notes = use_signal(String::new);

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let st = search_status.read().clone();
        let off = *offset.read();
        async move {
            let result = server::api::search_deadlines(
                court,
                if st.is_empty() { None } else { Some(st) },
                None, // case_id
                None, // date_from
                None, // date_to
                Some(off),
                Some(limit),
            )
            .await;

            match result {
                Ok(json) => serde_json::from_str::<DeadlineSearchResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_title.set(String::new());
        form_due_at.set(String::new());
        form_rule_code.set(String::new());
        form_notes.set(String::new());
    };

    let open_create = move |_| {
        reset_form();
        show_sheet.set(true);
    };

    let handle_clear = move |_| {
        search_status.set(String::new());
        offset.set(0);
    };

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let t = form_title.read().clone();
        let d = form_due_at.read().clone();
        let r = form_rule_code.read().clone();
        let n = form_notes.read().clone();

        spawn(async move {
            if t.trim().is_empty() || d.trim().is_empty() {
                toast.error(
                    "Title and due date are required.".to_string(),
                    ToastOptions::new(),
                );
                return;
            }

            // Convert HTML datetime-local to RFC3339
            let due_rfc3339 = format!("{}:00Z", d);

            let body = serde_json::json!({
                "title": t.trim(),
                "due_at": due_rfc3339,
                "rule_code": if r.is_empty() { None::<String> } else { Some(r) },
                "notes": if n.is_empty() { None::<String> } else { Some(n) },
            });

            match server::api::create_deadline(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Deadline created successfully".to_string(),
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
                PageTitle { "Deadlines" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "New Deadline"
                    }
                }
            }

            SearchBar {
                FormSelect {
                    value: "{search_status}",
                    onchange: move |evt: Event<FormData>| {
                        search_status.set(evt.value().to_string());
                        offset.set(0);
                    },
                    option { value: "", "All Statuses" }
                    option { value: "open", "Open" }
                    option { value: "met", "Met" }
                    option { value: "extended", "Extended" }
                    option { value: "cancelled", "Cancelled" }
                    option { value: "expired", "Expired" }
                }
                if !search_status.read().is_empty() {
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: handle_clear,
                        "Clear Filters"
                    }
                }
            }

            match &*data.read() {
                Some(Some(resp)) => rsx! {
                    DeadlineTable { deadlines: resp.deadlines.clone() }
                    Pagination {
                        total: resp.total,
                        offset: offset,
                        limit: limit,
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No deadlines found for this court district." }
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

            // Create deadline Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Deadline" }
                        SheetDescription {
                            "Set a deadline with an associated rule and due date."
                        }
                        SheetClose { on_close: move |_| show_sheet.set(false) }
                    }

                    Form {
                        onsubmit: handle_save,

                        div {
                            class: "sheet-form",

                            Input {
                                label: "Title *",
                                value: form_title.read().clone(),
                                on_input: move |e: FormEvent| form_title.set(e.value().to_string()),
                                placeholder: "e.g., File Motion Response",
                            }

                            Input {
                                label: "Due Date *",
                                input_type: "datetime-local",
                                value: form_due_at.read().clone(),
                                on_input: move |e: FormEvent| form_due_at.set(e.value().to_string()),
                            }

                            Input {
                                label: "Rule Code",
                                value: form_rule_code.read().clone(),
                                on_input: move |e: FormEvent| form_rule_code.set(e.value().to_string()),
                                placeholder: "e.g., FRCP 12(b)",
                            }

                            Textarea {
                                label: "Notes",
                                value: form_notes.read().clone(),
                                on_input: move |e: FormEvent| form_notes.set(e.value().to_string()),
                                placeholder: "Optional notes...",
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Create Deadline"
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
fn DeadlineTable(deadlines: Vec<DeadlineResponse>) -> Element {
    if deadlines.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No deadlines found." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Title" }
                DataTableColumn { "Due Date" }
                DataTableColumn { "Rule" }
                DataTableColumn { "Status" }
            }
            DataTableBody {
                for dl in deadlines {
                    DeadlineRow { deadline: dl }
                }
            }
        }
    }
}

#[component]
fn DeadlineRow(deadline: DeadlineResponse) -> Element {
    let id = deadline.id.clone();
    let badge_variant = status_badge_variant(&deadline.status);
    let display_date = format_due_date(&deadline.due_at);
    let rule_display = deadline.rule_code.clone().unwrap_or_default();
    let notes_preview = deadline
        .notes
        .as_deref()
        .unwrap_or("No notes");
    let notes_display = if notes_preview.len() > 100 {
        format!("{}...", &notes_preview[..100])
    } else {
        notes_preview.to_string()
    };

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::DeadlineDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { "{deadline.title}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{deadline.title}" }
                                span { class: "hover-card-username", "Due: {display_date}" }
                                if !rule_display.is_empty() {
                                    span { class: "hover-card-id", "Rule: {rule_display}" }
                                }
                                span { class: "hover-card-id", "{notes_display}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: badge_variant, "{deadline.status}" }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{display_date}" }
            DataTableCell { "{rule_display}" }
            DataTableCell {
                Badge { variant: badge_variant, "{deadline.status}" }
            }
        }
    }
}

fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "open" => BadgeVariant::Primary,
        "met" => BadgeVariant::Secondary,
        "extended" => BadgeVariant::Outline,
        "cancelled" | "expired" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}

fn format_due_date(date_str: &str) -> String {
    if date_str.len() >= 16 {
        date_str[..16].replace('T', " ")
    } else {
        date_str.to_string()
    }
}
