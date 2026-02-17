use dioxus::prelude::*;
use shared_types::{
    CaseSearchResponse, EvidenceResponse, PaginatedResponse, PaginationMeta, EVIDENCE_TYPES,
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
pub fn EvidenceListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);

    // Sheet state for creating evidence
    let mut show_sheet = use_signal(|| false);
    let mut form_description = use_signal(String::new);
    let mut form_case_id = use_signal(String::new);
    let mut form_evidence_type = use_signal(|| "Physical".to_string());
    let mut form_seized_date = use_signal(String::new);
    let mut form_seized_by = use_signal(String::new);
    let mut form_location = use_signal(String::new);

    // Load available cases for the case selector in the create form
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

    // Load evidence data
    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let p = *page.read();
        async move {
            let search = if q.is_empty() { None } else { Some(q) };
            match server::api::list_all_evidence(court, search, Some(p), Some(20)).await {
                Ok(json) => {
                    serde_json::from_str::<PaginatedResponse<EvidenceResponse>>(&json).ok()
                }
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_description.set(String::new());
        form_case_id.set(String::new());
        form_evidence_type.set("Physical".to_string());
        form_seized_date.set(String::new());
        form_seized_by.set(String::new());
        form_location.set(String::new());
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
        let case_id_str = form_case_id.read().clone();

        if form_description.read().trim().is_empty() {
            toast.error("Description is required.".to_string(), ToastOptions::new());
            return;
        }
        if case_id_str.is_empty() {
            toast.error("Case is required.".to_string(), ToastOptions::new());
            return;
        }

        let body = serde_json::json!({
            "case_id": case_id_str,
            "description": form_description.read().clone(),
            "evidence_type": form_evidence_type.read().clone(),
            "seized_date": opt_date(&form_seized_date.read()),
            "seized_by": opt_str(&form_seized_by.read()),
            "location": opt_str(&form_location.read()),
        });

        spawn(async move {
            match server::api::create_evidence(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Evidence created successfully".to_string(),
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
                PageTitle { "Evidence" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "New Evidence"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by description...",
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
                    EvidenceTable { evidence_items: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No evidence found for this court district." }
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

            // Create evidence Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Evidence" }
                        SheetDescription {
                            "Add an evidence item to an existing case."
                        }
                        SheetClose { on_close: move |_| show_sheet.set(false) }
                    }

                    Form {
                        onsubmit: handle_save,

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

                            Input {
                                label: "Description *",
                                value: form_description.read().clone(),
                                on_input: move |e: FormEvent| form_description.set(e.value().to_string()),
                                placeholder: "e.g., Recovered laptop from suspect's residence",
                            }

                            label { class: "input-label", "Evidence Type" }
                            select {
                                class: "input",
                                value: form_evidence_type.read().clone(),
                                onchange: move |e: FormEvent| form_evidence_type.set(e.value().to_string()),
                                for et in EVIDENCE_TYPES.iter() {
                                    option { value: *et, "{et}" }
                                }
                            }

                            Input {
                                label: "Seized Date",
                                input_type: "date",
                                value: form_seized_date.read().clone(),
                                on_input: move |e: FormEvent| form_seized_date.set(e.value().to_string()),
                            }

                            Input {
                                label: "Seized By",
                                value: form_seized_by.read().clone(),
                                on_input: move |e: FormEvent| form_seized_by.set(e.value().to_string()),
                                placeholder: "e.g., Special Agent Smith",
                            }

                            Input {
                                label: "Storage Location",
                                value: form_location.read().clone(),
                                on_input: move |e: FormEvent| form_location.set(e.value().to_string()),
                                placeholder: "e.g., Evidence Locker B-12",
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Create Evidence"
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
fn EvidenceTable(evidence_items: Vec<EvidenceResponse>) -> Element {
    if evidence_items.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No evidence found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Description" }
                DataTableColumn { "Type" }
                DataTableColumn { "Location" }
                DataTableColumn { "Sealed" }
            }
            DataTableBody {
                for item in evidence_items {
                    EvidenceRow { evidence: item }
                }
            }
        }
    }
}

#[component]
fn EvidenceRow(evidence: EvidenceResponse) -> Element {
    let id = evidence.id.clone();
    let type_variant = evidence_type_badge_variant(&evidence.evidence_type);
    let seized_date_display = evidence
        .seized_date
        .clone()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());
    let seized_by_display = evidence
        .seized_by
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let case_id_short = if evidence.case_id.len() > 8 {
        format!("{}...", &evidence.case_id[..8])
    } else {
        evidence.case_id.clone()
    };
    let location_display = if evidence.location.is_empty() {
        "--".to_string()
    } else {
        evidence.location.clone()
    };

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::EvidenceDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{evidence.description}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{evidence.description}" }
                                span { class: "hover-card-id", "Case: {case_id_short}" }
                                span { class: "hover-card-id", "Seized: {seized_date_display}" }
                                span { class: "hover-card-id", "By: {seized_by_display}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: type_variant,
                                        "{evidence.evidence_type}"
                                    }
                                    if evidence.is_sealed {
                                        Badge { variant: BadgeVariant::Destructive,
                                            "Sealed"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell {
                Badge { variant: type_variant, "{evidence.evidence_type}" }
            }
            DataTableCell { "{location_display}" }
            DataTableCell {
                if evidence.is_sealed {
                    Badge { variant: BadgeVariant::Destructive, "Sealed" }
                } else {
                    Badge { variant: BadgeVariant::Secondary, "Open" }
                }
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

/// Map evidence type to an appropriate badge variant.
fn evidence_type_badge_variant(evidence_type: &str) -> BadgeVariant {
    match evidence_type {
        "Physical" => BadgeVariant::Primary,
        "Documentary" => BadgeVariant::Secondary,
        "Digital" => BadgeVariant::Outline,
        "Testimonial" => BadgeVariant::Primary,
        "Demonstrative" => BadgeVariant::Secondary,
        "Forensic" => BadgeVariant::Destructive,
        _ => BadgeVariant::Outline,
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

/// Return `serde_json::Value::Null` for empty date strings, or the date string value.
fn opt_date(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(s.to_string())
    }
}
