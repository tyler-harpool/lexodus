use dioxus::prelude::*;
use shared_types::{PaginatedResponse, PaginationMeta, PartyResponse};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Input, PageActions, PageHeader,
    PageTitle, SearchBar, Skeleton,
};
use shared_ui::{HoverCard, HoverCardContent, HoverCardTrigger};

use super::form_sheet::{FormMode, PartyFormSheet};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn PartyListPage() -> Element {
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
            match server::api::list_all_parties(court, search, Some(p), Some(20)).await {
                Ok(json) => {
                    serde_json::from_str::<PaginatedResponse<PartyResponse>>(&json).ok()
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
                PageTitle { "Parties" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_create.set(true),
                        "New Party"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by party name...",
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
                    PartyTable { parties: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No parties found for this court district." }
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

            PartyFormSheet {
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
fn PartyTable(parties: Vec<PartyResponse>) -> Element {
    if parties.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No parties found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Name" }
                DataTableColumn { "Type" }
                DataTableColumn { "Entity Type" }
                DataTableColumn { "Case" }
                DataTableColumn { "Status" }
            }
            DataTableBody {
                for party in parties {
                    PartyRow { party: party }
                }
            }
        }
    }
}

#[component]
fn PartyRow(party: PartyResponse) -> Element {
    let id = party.id.clone();
    let status_variant = party_status_badge_variant(&party.status);
    let case_id_short = if party.case_id.len() > 8 {
        format!("{}...", &party.case_id[..8])
    } else {
        party.case_id.clone()
    };
    let email_display = party
        .email
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let phone_display = party
        .phone
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::PartyDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{party.name}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{party.name}" }
                                span { class: "hover-card-id", "Type: {party.party_type}" }
                                span { class: "hover-card-id", "Entity: {party.entity_type}" }
                                span { class: "hover-card-id", "Email: {email_display}" }
                                span { class: "hover-card-id", "Phone: {phone_display}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: status_variant,
                                        "{party.status}"
                                    }
                                    if party.pro_se {
                                        Badge { variant: BadgeVariant::Outline,
                                            "Pro Se"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{party.party_type}" }
            DataTableCell { "{party.entity_type}" }
            DataTableCell { "{case_id_short}" }
            DataTableCell {
                Badge { variant: status_variant, "{party.status}" }
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

/// Map party status to an appropriate badge variant.
fn party_status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Terminated" | "Dismissed" | "Deceased" => BadgeVariant::Destructive,
        "Defaulted" | "In Contempt" => BadgeVariant::Destructive,
        "Settled" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

