use dioxus::prelude::*;
use shared_types::{JudicialOrderResponse, PaginatedResponse, PaginationMeta};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Input,
    PageActions, PageHeader, PageTitle, SearchBar, Skeleton,
};
use shared_ui::{HoverCard, HoverCardContent, HoverCardTrigger};

use super::form_sheet::{OrderFormSheet, FormMode};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn OrderListPage() -> Element {
    let ctx = use_context::<CourtContext>();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);
    let mut show_sheet = use_signal(|| false);

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let p = *page.read();
        async move {
            let search = if q.is_empty() { None } else { Some(q) };
            match server::api::list_all_orders(court, search, Some(p), Some(20)).await {
                Ok(json) => {
                    serde_json::from_str::<PaginatedResponse<JudicialOrderResponse>>(&json).ok()
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
                PageTitle { "Orders" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_sheet.set(true),
                        "New Order"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by order title...",
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
                    OrderTable { orders: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No orders found for this court district." }
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

            OrderFormSheet {
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
fn OrderTable(orders: Vec<JudicialOrderResponse>) -> Element {
    if orders.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No orders found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Title" }
                DataTableColumn { "Type" }
                DataTableColumn { "Status" }
                DataTableColumn { "Sealed" }
                DataTableColumn { "Created" }
            }
            DataTableBody {
                for order in orders {
                    OrderRow { order: order }
                }
            }
        }
    }
}

#[component]
fn OrderRow(order: JudicialOrderResponse) -> Element {
    let id = order.id.clone();
    let status_variant = status_badge_variant(&order.status);
    let type_variant = type_badge_variant(&order.order_type);
    let sealed_label = if order.is_sealed { "Yes" } else { "No" };
    let sealed_variant = if order.is_sealed {
        BadgeVariant::Destructive
    } else {
        BadgeVariant::Secondary
    };
    let created_display = order.created_at.chars().take(10).collect::<String>();
    let case_id_short = if order.case_id.len() > 8 {
        format!("{}...", &order.case_id[..8])
    } else {
        order.case_id.clone()
    };
    let judge_id_short = if order.judge_id.len() > 8 {
        format!("{}...", &order.judge_id[..8])
    } else {
        order.judge_id.clone()
    };

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::OrderDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{order.title}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{order.title}" }
                                span { class: "hover-card-id", "Case: {case_id_short}" }
                                span { class: "hover-card-id", "Judge: {judge_id_short}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: type_variant,
                                        "{order.order_type}"
                                    }
                                    Badge { variant: status_variant,
                                        "{order.status}"
                                    }
                                    if order.is_sealed {
                                        Badge { variant: BadgeVariant::Destructive, "Sealed" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell {
                Badge { variant: type_variant, "{order.order_type}" }
            }
            DataTableCell {
                Badge { variant: status_variant, "{order.status}" }
            }
            DataTableCell {
                Badge { variant: sealed_variant, "{sealed_label}" }
            }
            DataTableCell { "{created_display}" }
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

/// Map order status to an appropriate badge variant.
fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Draft" => BadgeVariant::Outline,
        "Pending Signature" => BadgeVariant::Secondary,
        "Signed" => BadgeVariant::Primary,
        "Filed" => BadgeVariant::Primary,
        "Vacated" => BadgeVariant::Destructive,
        "Amended" | "Superseded" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// Map order type to an appropriate badge variant.
fn type_badge_variant(order_type: &str) -> BadgeVariant {
    match order_type {
        "Scheduling" | "Procedural" => BadgeVariant::Secondary,
        "Protective" | "Restraining" => BadgeVariant::Primary,
        "Dismissal" | "Sentencing" => BadgeVariant::Destructive,
        "Detention" => BadgeVariant::Destructive,
        "Release" => BadgeVariant::Primary,
        "Sealing" => BadgeVariant::Secondary,
        "Standing" => BadgeVariant::Outline,
        _ => BadgeVariant::Outline,
    }
}
