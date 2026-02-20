use dioxus::prelude::*;
use shared_types::{AttorneyResponse, PaginationMeta};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Input, PageActions, PageHeader,
    PageTitle, SearchBar, Skeleton,
};
use shared_ui::{HoverCard, HoverCardContent, HoverCardTrigger};

use super::form_sheet::{AttorneyFormSheet, FormMode};
use crate::auth::{can, use_user_role, Action};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn AttorneyListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let role = use_user_role();
    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);
    let mut show_sheet = use_signal(|| false);

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let p = *page.read();
        async move {
            let result = if q.is_empty() {
                server::api::list_attorneys(court, Some(p), Some(20)).await
            } else {
                server::api::search_attorneys(court, q, Some(p), Some(20)).await
            };

            result.ok()
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
                PageTitle { "Attorneys" }
                PageActions {
                    if can(&role, Action::ManageAttorneys) {
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| show_sheet.set(true),
                            "New Attorney"
                        }
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by name, bar number, email, or firm...",
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
                    AttorneyTable { attorneys: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No attorneys found for this court district." }
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

            AttorneyFormSheet {
                mode: FormMode::Create,
                initial: None,
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                on_saved: move |_| data.restart(),
            }
        }
    }
}

#[component]
fn AttorneyTable(attorneys: Vec<AttorneyResponse>) -> Element {
    if attorneys.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No attorneys found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Name" }
                DataTableColumn { "Bar Number" }
                DataTableColumn { "Email" }
                DataTableColumn { "Firm" }
                DataTableColumn { "Status" }
            }
            DataTableBody {
                for attorney in attorneys {
                    AttorneyRow { attorney: attorney }
                }
            }
        }
    }
}

#[component]
fn AttorneyRow(attorney: AttorneyResponse) -> Element {
    let id = attorney.id.clone();
    let badge_variant = status_badge_variant(&attorney.status);
    let full_name = format!("{}, {}", attorney.last_name, attorney.first_name);
    let firm_display = attorney.firm_name.clone().unwrap_or_else(|| "--".to_string());
    let city_state = format!("{}, {}", attorney.address.city, attorney.address.state);

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::AttorneyDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{full_name}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{full_name}" }
                                span { class: "hover-card-username", "Bar: {attorney.bar_number}" }
                                span { class: "hover-card-id", "{attorney.email}" }
                                span { class: "hover-card-id", "{attorney.phone}" }
                                if !firm_display.is_empty() && firm_display != "--" {
                                    span { class: "hover-card-id", "Firm: {firm_display}" }
                                }
                                span { class: "hover-card-id", "{city_state}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: badge_variant, "{attorney.status}" }
                                    if attorney.cja_panel_member {
                                        Badge { variant: BadgeVariant::Outline, "CJA Panel" }
                                    }
                                    if attorney.cases_handled > 0 {
                                        Badge { variant: BadgeVariant::Secondary, "{attorney.cases_handled} cases" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{attorney.bar_number}" }
            DataTableCell { "{attorney.email}" }
            DataTableCell { "{firm_display}" }
            DataTableCell {
                Badge { variant: badge_variant, "{attorney.status}" }
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

fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Inactive" => BadgeVariant::Secondary,
        "Suspended" => BadgeVariant::Destructive,
        "Retired" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}
