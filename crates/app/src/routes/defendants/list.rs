use dioxus::prelude::*;
use shared_types::{
    CaseSearchResponse, DefendantResponse, PaginatedResponse, PaginationMeta, CUSTODY_STATUSES,
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
pub fn DefendantListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);

    // Sheet state for creating a defendant
    let mut show_sheet = use_signal(|| false);
    let mut form_name = use_signal(String::new);
    let mut form_case_id = use_signal(String::new);
    let mut form_usm_number = use_signal(String::new);
    let mut form_fbi_number = use_signal(String::new);
    let mut form_dob = use_signal(String::new);
    let mut form_citizenship = use_signal(|| "Unknown".to_string());
    let mut form_custody = use_signal(|| "Released".to_string());

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

    // Load defendants data
    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let p = *page.read();
        async move {
            let search = if q.is_empty() { None } else { Some(q) };
            match server::api::list_all_defendants(court, search, Some(p), Some(20)).await {
                Ok(json) => {
                    serde_json::from_str::<PaginatedResponse<DefendantResponse>>(&json).ok()
                }
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_name.set(String::new());
        form_case_id.set(String::new());
        form_usm_number.set(String::new());
        form_fbi_number.set(String::new());
        form_dob.set(String::new());
        form_citizenship.set("Unknown".to_string());
        form_custody.set("Released".to_string());
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

        if form_name.read().trim().is_empty() {
            toast.error("Name is required.".to_string(), ToastOptions::new());
            return;
        }
        if case_id_str.is_empty() {
            toast.error("Case is required.".to_string(), ToastOptions::new());
            return;
        }

        let body = serde_json::json!({
            "case_id": case_id_str,
            "name": form_name.read().clone(),
            "aliases": [],
            "usm_number": opt_str(&form_usm_number.read()),
            "fbi_number": opt_str(&form_fbi_number.read()),
            "date_of_birth": opt_date(&form_dob.read()),
            "citizenship_status": form_citizenship.read().clone(),
            "custody_status": form_custody.read().clone(),
        });

        spawn(async move {
            match server::api::create_defendant(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Defendant created successfully".to_string(),
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
                PageTitle { "Defendants" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "New Defendant"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by defendant name...",
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
                    DefendantTable { defendants: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No defendants found for this court district." }
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

            // Create defendant Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Defendant" }
                        SheetDescription {
                            "Add a defendant to an existing case."
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
                                label: "Full Name *",
                                value: form_name.read().clone(),
                                on_input: move |e: FormEvent| form_name.set(e.value().to_string()),
                                placeholder: "e.g., John Doe",
                            }

                            Input {
                                label: "USM Number",
                                value: form_usm_number.read().clone(),
                                on_input: move |e: FormEvent| form_usm_number.set(e.value().to_string()),
                                placeholder: "e.g., 12345-001",
                            }

                            Input {
                                label: "FBI Number",
                                value: form_fbi_number.read().clone(),
                                on_input: move |e: FormEvent| form_fbi_number.set(e.value().to_string()),
                            }

                            Input {
                                label: "Date of Birth",
                                input_type: "date",
                                value: form_dob.read().clone(),
                                on_input: move |e: FormEvent| form_dob.set(e.value().to_string()),
                            }

                            label { class: "input-label", "Custody Status" }
                            select {
                                class: "input",
                                value: form_custody.read().clone(),
                                onchange: move |e: FormEvent| form_custody.set(e.value().to_string()),
                                for status in CUSTODY_STATUSES.iter() {
                                    option { value: *status, "{status}" }
                                }
                            }

                            label { class: "input-label", "Citizenship Status" }
                            select {
                                class: "input",
                                value: form_citizenship.read().clone(),
                                onchange: move |e: FormEvent| form_citizenship.set(e.value().to_string()),
                                for status in shared_types::CITIZENSHIP_STATUSES.iter() {
                                    option { value: *status, "{status}" }
                                }
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Create Defendant"
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
fn DefendantTable(defendants: Vec<DefendantResponse>) -> Element {
    if defendants.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No defendants found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Name" }
                DataTableColumn { "Case" }
                DataTableColumn { "Custody Status" }
                DataTableColumn { "USM #" }
            }
            DataTableBody {
                for defendant in defendants {
                    DefendantRow { defendant: defendant }
                }
            }
        }
    }
}

#[component]
fn DefendantRow(defendant: DefendantResponse) -> Element {
    let id = defendant.id.clone();
    let custody_variant = custody_badge_variant(&defendant.custody_status);
    let usm_display = defendant
        .usm_number
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let dob_display = defendant
        .date_of_birth
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let case_id_short = if defendant.case_id.len() > 8 {
        format!("{}...", &defendant.case_id[..8])
    } else {
        defendant.case_id.clone()
    };

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::DefendantDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{defendant.name}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{defendant.name}" }
                                span { class: "hover-card-id", "USM: {usm_display}" }
                                span { class: "hover-card-id", "DOB: {dob_display}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: custody_variant,
                                        "{defendant.custody_status}"
                                    }
                                    Badge { variant: BadgeVariant::Secondary,
                                        "{defendant.citizenship_status}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{case_id_short}" }
            DataTableCell {
                Badge { variant: custody_variant, "{defendant.custody_status}" }
            }
            DataTableCell { "{usm_display}" }
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

/// Map custody status to an appropriate badge variant.
fn custody_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "In Custody" => BadgeVariant::Destructive,
        "Bail" | "Bond" => BadgeVariant::Primary,
        "Released" | "Supervised Release" => BadgeVariant::Secondary,
        "Fugitive" => BadgeVariant::Destructive,
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
