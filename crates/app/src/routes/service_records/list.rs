use dioxus::prelude::*;
use shared_types::{PaginatedResponse, PaginationMeta, ServiceRecordResponse};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Form, Input, PageActions,
    PageHeader, PageTitle, SearchBar, Separator, Sheet, SheetClose, SheetContent, SheetDescription,
    SheetFooter, SheetHeader, SheetSide, SheetTitle, Skeleton,
};
use shared_ui::{use_toast, HoverCard, HoverCardContent, HoverCardTrigger, ToastOptions};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn ServiceRecordListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);

    // Sheet state for creating a service record
    let mut show_sheet = use_signal(|| false);
    let mut form_document_id = use_signal(String::new);
    let mut form_party_id = use_signal(String::new);
    let mut form_service_method = use_signal(|| "ECF".to_string());
    let mut form_served_by = use_signal(String::new);
    let mut form_service_date = use_signal(String::new);
    let mut form_notes = use_signal(String::new);

    // Load service records data
    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let p = *page.read();
        async move {
            let search = if q.is_empty() { None } else { Some(q) };
            match server::api::list_all_service_records(court, search, Some(p), Some(20)).await {
                Ok(json) => {
                    serde_json::from_str::<PaginatedResponse<ServiceRecordResponse>>(&json).ok()
                }
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_document_id.set(String::new());
        form_party_id.set(String::new());
        form_service_method.set("ECF".to_string());
        form_served_by.set(String::new());
        form_service_date.set(String::new());
        form_notes.set(String::new());
    };

    let open_create = move |_| {
        reset_form();
        show_sheet.set(true);
    };

    let handle_search = move |_| {
        search_query.set(search_input.read().clone());
        page.set(1);
    };

    let handle_clear = move |_| {
        search_input.set(String::new());
        search_query.set(String::new());
        page.set(1);
    };

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();

        if form_document_id.read().trim().is_empty() {
            toast.error("Document ID is required.".to_string(), ToastOptions::new());
            return;
        }
        if form_party_id.read().trim().is_empty() {
            toast.error("Party ID is required.".to_string(), ToastOptions::new());
            return;
        }
        if form_served_by.read().trim().is_empty() {
            toast.error("Served By is required.".to_string(), ToastOptions::new());
            return;
        }

        let body = serde_json::json!({
            "document_id": form_document_id.read().clone(),
            "party_id": form_party_id.read().clone(),
            "service_method": form_service_method.read().clone(),
            "served_by": form_served_by.read().clone(),
            "service_date": opt_str(&form_service_date.read()),
            "notes": opt_str(&form_notes.read()),
        });

        spawn(async move {
            match server::api::create_service_record(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Service record created successfully".to_string(),
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
                PageTitle { "Service Records" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "New Service Record"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by party name, served by, or method...",
                    label: "",
                    on_input: move |evt: FormEvent| search_input.set(evt.value().to_string()),
                }
                Button { onclick: handle_search, "Search" }
                if !search_query.read().is_empty() {
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: handle_clear,
                        "Clear"
                    }
                }
            }

            match &*data.read() {
                Some(Some(resp)) => rsx! {
                    ServiceRecordTable { records: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No service records found for this court district." }
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

            // Create service record Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Service Record" }
                        SheetDescription {
                            "Record service of a document on a party."
                        }
                        SheetClose { on_close: move |_| show_sheet.set(false) }
                    }

                    Form {
                        onsubmit: handle_save,

                        div {
                            class: "sheet-form",

                            Input {
                                label: "Document ID *",
                                value: form_document_id.read().clone(),
                                on_input: move |e: FormEvent| form_document_id.set(e.value().to_string()),
                                placeholder: "UUID of the document",
                            }

                            Input {
                                label: "Party ID *",
                                value: form_party_id.read().clone(),
                                on_input: move |e: FormEvent| form_party_id.set(e.value().to_string()),
                                placeholder: "UUID of the party being served",
                            }

                            label { class: "input-label", "Service Method *" }
                            select {
                                class: "input",
                                value: form_service_method.read().clone(),
                                onchange: move |e: FormEvent| form_service_method.set(e.value().to_string()),
                                for method in SERVICE_METHODS.iter() {
                                    option { value: *method, "{method}" }
                                }
                            }

                            Input {
                                label: "Served By *",
                                value: form_served_by.read().clone(),
                                on_input: move |e: FormEvent| form_served_by.set(e.value().to_string()),
                                placeholder: "Name of the person who served",
                            }

                            Input {
                                label: "Service Date",
                                input_type: "datetime-local",
                                value: form_service_date.read().clone(),
                                on_input: move |e: FormEvent| form_service_date.set(e.value().to_string()),
                            }

                            Input {
                                label: "Notes",
                                value: form_notes.read().clone(),
                                on_input: move |e: FormEvent| form_notes.set(e.value().to_string()),
                                placeholder: "Optional notes",
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Create Record"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Valid service methods for the form dropdown.
const SERVICE_METHODS: &[&str] = &[
    "Electronic",
    "Mail",
    "Personal Service",
    "Waiver",
    "Publication",
    "Certified Mail",
    "Express Mail",
    "ECF",
    "Other",
];

#[component]
fn ServiceRecordTable(records: Vec<ServiceRecordResponse>) -> Element {
    if records.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No service records found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Document" }
                DataTableColumn { "Party" }
                DataTableColumn { "Method" }
                DataTableColumn { "Date" }
                DataTableColumn { "Successful" }
            }
            DataTableBody {
                for record in records {
                    ServiceRecordRow { record: record }
                }
            }
        }
    }
}

#[component]
fn ServiceRecordRow(record: ServiceRecordResponse) -> Element {
    let id = record.id.clone();
    let doc_id_short = truncate_id(&record.document_id);
    let date_display = record
        .service_date
        .chars()
        .take(10)
        .collect::<String>();
    let successful_variant = if record.successful {
        BadgeVariant::Primary
    } else {
        BadgeVariant::Destructive
    };
    let successful_label = if record.successful { "Yes" } else { "No" };

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::ServiceRecordDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{doc_id_short}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "Method: {record.service_method}" }
                                span { class: "hover-card-id", "Party: {record.party_name}" }
                                span { class: "hover-card-id", "Date: {date_display}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: successful_variant, "{successful_label}" }
                                    if record.proof_of_service_filed {
                                        Badge { variant: BadgeVariant::Secondary, "Proof Filed" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{record.party_name}" }
            DataTableCell {
                Badge { variant: method_badge_variant(&record.service_method), "{record.service_method}" }
            }
            DataTableCell { "{date_display}" }
            DataTableCell {
                Badge { variant: successful_variant, "{successful_label}" }
            }
        }
    }
}

#[component]
fn PaginationControls(meta: PaginationMeta, page: Signal<i64>) -> Element {
    rsx! {
        div { class: "pagination",
            if meta.has_prev {
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let current = *page.read();
                        page.set(current - 1);
                    },
                    "Previous"
                }
            }
            span { class: "pagination-info",
                "Page {meta.page} of {meta.total_pages} ({meta.total} total)"
            }
            if meta.has_next {
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let current = *page.read();
                        page.set(current + 1);
                    },
                    "Next"
                }
            }
        }
    }
}

/// Map service method to a badge variant.
fn method_badge_variant(method: &str) -> BadgeVariant {
    match method {
        "ECF" | "Electronic" => BadgeVariant::Primary,
        "Personal Service" => BadgeVariant::Secondary,
        "Mail" | "Certified Mail" | "Express Mail" => BadgeVariant::Outline,
        "Publication" => BadgeVariant::Outline,
        _ => BadgeVariant::Outline,
    }
}

/// Truncate a UUID string for display purposes.
fn truncate_id(id: &str) -> String {
    if id.len() > 8 {
        format!("{}...", &id[..8])
    } else {
        id.to_string()
    }
}

/// Return `serde_json::Value::Null` for empty strings, or the string value.
fn opt_str(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(s.to_string())
    }
}
