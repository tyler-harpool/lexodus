use dioxus::prelude::*;
use shared_types::{EvidenceResponse, PaginatedResponse, PaginationMeta};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Input, PageActions, PageHeader,
    PageTitle, SearchBar, Skeleton,
};
use shared_ui::{HoverCard, HoverCardContent, HoverCardTrigger};

use super::form_sheet::{EvidenceFormSheet, FormMode};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn EvidenceListPage() -> Element {
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
            match server::api::list_all_evidence(court, search, Some(p), Some(20)).await {
                Ok(json) => {
                    serde_json::from_str::<PaginatedResponse<EvidenceResponse>>(&json).ok()
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
                PageTitle { "Evidence" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_create.set(true),
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

            EvidenceFormSheet {
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

