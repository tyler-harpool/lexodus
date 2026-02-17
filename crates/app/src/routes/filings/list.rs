use dioxus::prelude::*;
use shared_types::{
    CaseSearchResponse, FilingListItem, PaginatedResponse, PaginationMeta, VALID_DOCUMENT_TYPES,
};
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
pub fn FilingListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);

    // Sheet state for creating a filing
    let mut show_sheet = use_signal(|| false);
    let mut form_case_id = use_signal(String::new);
    let mut form_document_type = use_signal(|| "Motion".to_string());
    let mut form_title = use_signal(String::new);
    let mut form_filed_by = use_signal(String::new);
    let mut form_is_sealed = use_signal(|| false);

    // Validation state
    let mut validation_result = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    // Load available cases for the case selector
    let cases_for_select = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            match server::api::search_cases(court, None, None, None, None, None, Some(100)).await {
                Ok(json) => serde_json::from_str::<CaseSearchResponse>(&json)
                    .ok()
                    .map(|r| r.cases),
                Err(_) => None,
            }
        }
    });

    // Load filings data
    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let p = *page.read();
        async move {
            let search = if q.is_empty() { None } else { Some(q) };
            match server::api::list_all_filings(court, search, Some(p), Some(20)).await {
                Ok(json) => {
                    serde_json::from_str::<PaginatedResponse<FilingListItem>>(&json).ok()
                }
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_case_id.set(String::new());
        form_document_type.set("Motion".to_string());
        form_title.set(String::new());
        form_filed_by.set(String::new());
        form_is_sealed.set(false);
        validation_result.set(None);
        submitting.set(false);
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

    let handle_validate = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let case_id = form_case_id.read().clone();
        let document_type = form_document_type.read().clone();
        let title = form_title.read().clone();
        let filed_by = form_filed_by.read().clone();
        let is_sealed = *form_is_sealed.read();

        if case_id.is_empty() || title.trim().is_empty() || filed_by.trim().is_empty() {
            toast.error(
                "Case, title, and filed by are required.".to_string(),
                ToastOptions::new(),
            );
            return;
        }

        spawn(async move {
            let body = serde_json::json!({
                "case_id": case_id,
                "document_type": document_type,
                "title": title,
                "filed_by": filed_by,
                "is_sealed": is_sealed,
            });

            match server::api::validate_filing_request(court, body.to_string()).await {
                Ok(json) => {
                    validation_result.set(Some(json.clone()));
                    // Check if valid
                    if let Ok(resp) =
                        serde_json::from_str::<shared_types::ValidateFilingResponse>(&json)
                    {
                        if resp.valid {
                            toast.success("Validation passed.".to_string(), ToastOptions::new());
                        } else {
                            let msgs: Vec<String> =
                                resp.errors.iter().map(|e| e.message.clone()).collect();
                            toast.error(
                                format!("Validation failed: {}", msgs.join(", ")),
                                ToastOptions::new(),
                            );
                        }
                    }
                }
                Err(e) => {
                    toast.error(format!("Validation error: {}", e), ToastOptions::new());
                }
            }
        });
    };

    let handle_submit = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let case_id = form_case_id.read().clone();
        let document_type = form_document_type.read().clone();
        let title = form_title.read().clone();
        let filed_by = form_filed_by.read().clone();
        let is_sealed = *form_is_sealed.read();

        if case_id.is_empty() || title.trim().is_empty() || filed_by.trim().is_empty() {
            toast.error(
                "Case, title, and filed by are required.".to_string(),
                ToastOptions::new(),
            );
            return;
        }

        submitting.set(true);

        spawn(async move {
            let body = serde_json::json!({
                "case_id": case_id,
                "document_type": document_type,
                "title": title,
                "filed_by": filed_by,
                "is_sealed": is_sealed,
            });

            match server::api::submit_filing(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    submitting.set(false);
                    toast.success(
                        "Filing submitted successfully.".to_string(),
                        ToastOptions::new(),
                    );
                }
                Err(e) => {
                    submitting.set(false);
                    toast.error(format!("Submit failed: {}", e), ToastOptions::new());
                }
            }
        });
    };

    rsx! {
        div { class: "container",
            PageHeader {
                PageTitle { "Filings" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "New Filing"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by filing type or filed by...",
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
                    FilingTable { filings: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No filings found for this court district." }
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

            // Create filing Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Filing" }
                        SheetDescription {
                            "Submit an electronic filing to the court."
                        }
                        SheetClose { on_close: move |_| show_sheet.set(false) }
                    }

                    Form {
                        onsubmit: handle_submit,

                        div {
                            class: "sheet-form",

                            // Case selector
                            label { class: "input-label", "Case *" }
                            select {
                                class: "input",
                                value: form_case_id.read().clone(),
                                onchange: move |e: FormEvent| form_case_id.set(e.value().to_string()),
                                option { value: "", "-- Select a case --" }
                                {match &*cases_for_select.read() {
                                    Some(Some(cases)) => rsx! {
                                        for c in cases.iter() {
                                            option {
                                                value: "{c.id}",
                                                "{c.case_number} — {c.title}"
                                            }
                                        }
                                    },
                                    _ => rsx! {
                                        option { value: "", disabled: true, "Loading cases..." }
                                    },
                                }}
                            }

                            // Document type selector
                            label { class: "input-label", "Document Type *" }
                            select {
                                class: "input",
                                value: form_document_type.read().clone(),
                                onchange: move |e: FormEvent| form_document_type.set(e.value().to_string()),
                                for dt in VALID_DOCUMENT_TYPES.iter() {
                                    option { value: *dt, "{dt}" }
                                }
                            }

                            Input {
                                label: "Title *",
                                value: form_title.read().clone(),
                                on_input: move |e: FormEvent| form_title.set(e.value().to_string()),
                                placeholder: "e.g., Motion to Dismiss",
                            }

                            Input {
                                label: "Filed By *",
                                value: form_filed_by.read().clone(),
                                on_input: move |e: FormEvent| form_filed_by.set(e.value().to_string()),
                                placeholder: "e.g., Jane Smith, Esq.",
                            }

                            label { class: "input-label", "Sealed" }
                            select {
                                class: "input",
                                value: if *form_is_sealed.read() { "true" } else { "false" },
                                onchange: move |e: FormEvent| {
                                    form_is_sealed.set(e.value() == "true");
                                },
                                option { value: "false", "No" }
                                option { value: "true", "Yes" }
                            }

                            // Validation result display
                            if let Some(ref json) = *validation_result.read() {
                                if let Ok(resp) = serde_json::from_str::<shared_types::ValidateFilingResponse>(json) {
                                    div { class: "validation-result",
                                        if resp.valid {
                                            Badge { variant: BadgeVariant::Primary, "Valid" }
                                        } else {
                                            Badge { variant: BadgeVariant::Destructive, "Invalid" }
                                            for err in resp.errors.iter() {
                                                p { class: "text-error", "{err.field}: {err.message}" }
                                            }
                                        }
                                        for warn in resp.warnings.iter() {
                                            p { class: "text-warning", "{warn.field}: {warn.message}" }
                                        }
                                    }
                                }
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: handle_validate,
                                    "Validate"
                                }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    if *submitting.read() { "Submitting..." } else { "Submit Filing" }
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
fn FilingTable(filings: Vec<FilingListItem>) -> Element {
    if filings.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No filings found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Filing Type" }
                DataTableColumn { "Filed By" }
                DataTableColumn { "Date" }
                DataTableColumn { "Status" }
                DataTableColumn { "Case" }
            }
            DataTableBody {
                for filing in filings {
                    FilingRow { filing: filing }
                }
            }
        }
    }
}

#[component]
fn FilingRow(filing: FilingListItem) -> Element {
    let id = filing.id.clone();
    let status_variant = filing_status_badge_variant(&filing.status);
    let date_display = filing.filed_date.chars().take(10).collect::<String>();
    let case_id_short = if filing.case_id.len() > 8 {
        format!("{}...", &filing.case_id[..8])
    } else {
        filing.case_id.clone()
    };

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::FilingDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{filing.filing_type}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{filing.filing_type}" }
                                span { class: "hover-card-id", "Filed by: {filing.filed_by}" }
                                span { class: "hover-card-id", "Date: {date_display}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: status_variant,
                                        "{filing.status}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{filing.filed_by}" }
            DataTableCell { "{date_display}" }
            DataTableCell {
                Badge { variant: status_variant, "{filing.status}" }
            }
            DataTableCell { "{case_id_short}" }
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

/// Map filing status to an appropriate badge variant.
fn filing_status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Filed" | "Accepted" => BadgeVariant::Primary,
        "Pending" | "Under Review" => BadgeVariant::Secondary,
        "Rejected" => BadgeVariant::Destructive,
        "Returned" => BadgeVariant::Outline,
        _ => BadgeVariant::Outline,
    }
}
