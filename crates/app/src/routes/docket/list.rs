use dioxus::prelude::*;
use shared_types::{
    CaseSearchResponse, DocketEntryResponse, DocketSearchResponse, DOCKET_ENTRY_TYPES,
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

/// Per-page limit for docket entry search results.
const PER_PAGE: i64 = 20;

#[component]
pub fn DocketListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut page = use_signal(|| 0i64); // offset-based: 0, PER_PAGE, 2*PER_PAGE ...
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);

    // Sheet state for creating a docket entry
    let mut show_sheet = use_signal(|| false);
    let mut form_case_id = use_signal(String::new);
    let mut form_entry_type = use_signal(|| "motion".to_string());
    let mut form_description = use_signal(String::new);
    let mut form_filed_by = use_signal(String::new);

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

    // Load docket entries
    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let offset = *page.read();
        async move {
            let search = if q.is_empty() { None } else { Some(q) };
            match server::api::search_docket_entries(
                court,
                None,
                None,
                search,
                Some(offset),
                Some(PER_PAGE),
            )
            .await
            {
                Ok(json) => serde_json::from_str::<DocketSearchResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_case_id.set(String::new());
        form_entry_type.set("motion".to_string());
        form_description.set(String::new());
        form_filed_by.set(String::new());
    };

    let open_create = move |_| {
        reset_form();
        show_sheet.set(true);
    };

    let handle_search = move |_| {
        search_query.set(search_input.read().clone());
        page.set(0);
    };

    let handle_clear = move |_| {
        search_input.set(String::new());
        search_query.set(String::new());
        page.set(0);
    };

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let case_id_str = form_case_id.read().clone();

        if case_id_str.is_empty() {
            toast.error("Case is required.".to_string(), ToastOptions::new());
            return;
        }
        if form_description.read().trim().is_empty() {
            toast.error("Description is required.".to_string(), ToastOptions::new());
            return;
        }

        let body = serde_json::json!({
            "case_id": case_id_str,
            "entry_type": form_entry_type.read().clone(),
            "description": form_description.read().clone(),
            "filed_by": opt_str(&form_filed_by.read()),
        });

        spawn(async move {
            match server::api::create_docket_entry(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Docket entry created successfully".to_string(),
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
                PageTitle { "Docket" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "New Entry"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search docket entries...",
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
                    DocketTable { entries: resp.entries.clone() }
                    OffsetPagination {
                        total: resp.total,
                        offset: page,
                        per_page: PER_PAGE,
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No docket entries found for this court district." }
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

            // Create docket entry Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Docket Entry" }
                        SheetDescription {
                            "File a new entry on the case docket."
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

                            // Entry type selector
                            label { class: "input-label", "Entry Type *" }
                            select {
                                class: "input",
                                value: form_entry_type.read().clone(),
                                onchange: move |e: FormEvent| form_entry_type.set(e.value().to_string()),
                                for et in DOCKET_ENTRY_TYPES.iter() {
                                    option { value: *et, "{format_entry_type(et)}" }
                                }
                            }

                            Input {
                                label: "Description *",
                                value: form_description.read().clone(),
                                on_input: move |e: FormEvent| form_description.set(e.value().to_string()),
                                placeholder: "e.g., Motion to suppress evidence",
                            }

                            Input {
                                label: "Filed By",
                                value: form_filed_by.read().clone(),
                                on_input: move |e: FormEvent| form_filed_by.set(e.value().to_string()),
                                placeholder: "e.g., Attorney name or party",
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Create Entry"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Table component displaying docket entries.
#[component]
fn DocketTable(entries: Vec<DocketEntryResponse>) -> Element {
    if entries.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No docket entries found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Entry #" }
                DataTableColumn { "Type" }
                DataTableColumn { "Description" }
                DataTableColumn { "Case" }
                DataTableColumn { "Date Filed" }
            }
            DataTableBody {
                for entry in entries {
                    DocketRow { entry: entry }
                }
            }
        }
    }
}

/// Individual row in the docket table with HoverCard preview.
#[component]
fn DocketRow(entry: DocketEntryResponse) -> Element {
    let id = entry.id.clone();
    let type_variant = entry_type_badge_variant(&entry.entry_type);
    let case_id_short = if entry.case_id.len() > 8 {
        format!("{}...", &entry.case_id[..8])
    } else {
        entry.case_id.clone()
    };
    let date_display = entry.date_filed.chars().take(10).collect::<String>();
    let filed_by_display = entry
        .filed_by
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let sealed_label = if entry.is_sealed { "Sealed" } else { "Public" };

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::DocketDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "#{entry.entry_number}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name",
                                    "Entry #{entry.entry_number}"
                                }
                                span { class: "hover-card-id",
                                    "Type: {format_entry_type(&entry.entry_type)}"
                                }
                                span { class: "hover-card-id",
                                    "Case: {entry.case_id}"
                                }
                                span { class: "hover-card-id",
                                    "Filed by: {filed_by_display}"
                                }
                                div { class: "hover-card-meta",
                                    Badge { variant: type_variant,
                                        "{format_entry_type(&entry.entry_type)}"
                                    }
                                    Badge {
                                        variant: if entry.is_sealed { BadgeVariant::Destructive } else { BadgeVariant::Secondary },
                                        "{sealed_label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell {
                Badge { variant: type_variant, "{format_entry_type(&entry.entry_type)}" }
            }
            DataTableCell {
                span { class: "truncate-text", "{entry.description}" }
            }
            DataTableCell { "{case_id_short}" }
            DataTableCell { "{date_display}" }
        }
    }
}

/// Offset-based pagination control for docket search results.
#[component]
fn OffsetPagination(total: i64, offset: Signal<i64>, per_page: i64) -> Element {
    let current_page = (*offset.read() / per_page) + 1;
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i64;
    let has_prev = *offset.read() > 0;
    let has_next = *offset.read() + per_page < total;

    rsx! {
        div { class: "pagination",
            if has_prev {
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let current = *offset.read();
                        offset.set((current - per_page).max(0));
                    },
                    "Previous"
                }
            }
            span { class: "pagination-info",
                "Page {current_page} of {total_pages} ({total} total)"
            }
            if has_next {
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let current = *offset.read();
                        offset.set(current + per_page);
                    },
                    "Next"
                }
            }
        }
    }
}

/// Map entry type to an appropriate badge variant.
fn entry_type_badge_variant(entry_type: &str) -> BadgeVariant {
    match entry_type {
        "motion" | "response" | "reply" => BadgeVariant::Primary,
        "order" | "minute_order" | "scheduling_order" => BadgeVariant::Secondary,
        "protective_order" | "sealing_order" => BadgeVariant::Destructive,
        "complaint" | "indictment" | "information" | "criminal_complaint" => BadgeVariant::Outline,
        "judgment" | "verdict" | "sentence" => BadgeVariant::Destructive,
        "notice" | "hearing_notice" | "notice_of_appeal" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// Format an entry type slug into a human-readable label.
fn format_entry_type(entry_type: &str) -> String {
    entry_type
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    format!("{}{}", upper, chars.collect::<String>())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Return `serde_json::Value::Null` for empty strings, or the string value.
fn opt_str(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(s.to_string())
    }
}
