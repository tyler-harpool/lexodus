use dioxus::prelude::*;
use shared_types::{DefendantResponse, PaginatedResponse, PaginationMeta};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Input, PageActions, PageHeader,
    PageTitle, SearchBar, Skeleton,
};
use shared_ui::{HoverCard, HoverCardContent, HoverCardTrigger};

use super::form_sheet::{DefendantFormSheet, FormMode};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn DefendantListPage() -> Element {
    let ctx = use_context::<CourtContext>();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);
    let mut show_create = use_signal(|| false);

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

    let handle_search = move |_| {
        search_query.set(search_input.read().clone());
        page.set(1);
    };

    let handle_clear = move |_| {
        search_input.set(String::new());
        search_query.set(String::new());
        page.set(1);
    };

    rsx! {
        div { class: "container",
            PageHeader {
                PageTitle { "Defendants" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_create.set(true),
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

            DefendantFormSheet {
                mode: FormMode::Create,
                initial: None,
                open: show_create(),
                on_close: move |_| show_create.set(false),
                on_saved: move |_| data.restart(),
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

