use dioxus::prelude::*;
use shared_types::{
    JudicialOrderResponse, PaginatedResponse, PaginationMeta, ORDER_TYPES, ORDER_STATUSES,
    CaseSearchResponse,
};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Form, FormSelect, Input,
    PageActions, PageHeader, PageTitle, SearchBar, Separator, Sheet, SheetClose, SheetContent,
    SheetDescription, SheetFooter, SheetHeader, SheetSide, SheetTitle, Skeleton,
};
use shared_ui::{use_toast, HoverCard, HoverCardContent, HoverCardTrigger, ToastOptions};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn OrderListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);

    // Sheet state for creating a new order
    let mut show_sheet = use_signal(|| false);
    let mut form_title = use_signal(String::new);
    let mut form_order_type = use_signal(|| ORDER_TYPES[0].to_string());
    let mut form_case_id = use_signal(String::new);
    let mut form_judge_id = use_signal(String::new);
    let mut form_content = use_signal(String::new);
    let mut form_status = use_signal(|| "Draft".to_string());

    // Load cases for the case selector in the create form
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

    // Load judges for the judge selector
    let judges_for_select = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            match server::api::list_judges(court).await {
                Ok(json) => serde_json::from_str::<Vec<serde_json::Value>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    // Load orders data
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

    let mut reset_form = move || {
        form_title.set(String::new());
        form_order_type.set(ORDER_TYPES[0].to_string());
        form_case_id.set(String::new());
        form_judge_id.set(String::new());
        form_content.set(String::new());
        form_status.set("Draft".to_string());
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

        if form_title.read().trim().is_empty() {
            toast.error("Title is required.".to_string(), ToastOptions::new());
            return;
        }
        if form_case_id.read().is_empty() {
            toast.error("Case is required.".to_string(), ToastOptions::new());
            return;
        }
        if form_judge_id.read().is_empty() {
            toast.error("Judge is required.".to_string(), ToastOptions::new());
            return;
        }

        let body = serde_json::json!({
            "case_id": form_case_id.read().clone(),
            "judge_id": form_judge_id.read().clone(),
            "order_type": form_order_type.read().clone(),
            "title": form_title.read().trim().to_string(),
            "content": form_content.read().clone(),
            "status": form_status.read().clone(),
        });

        spawn(async move {
            match server::api::create_order(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Order created successfully".to_string(),
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
                PageTitle { "Orders" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
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

            // Create order Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Order" }
                        SheetDescription {
                            "Create a new judicial order."
                        }
                        SheetClose { on_close: move |_| show_sheet.set(false) }
                    }

                    Form {
                        onsubmit: handle_save,

                        div {
                            class: "sheet-form",

                            Input {
                                label: "Title *",
                                value: form_title.read().clone(),
                                on_input: move |e: FormEvent| form_title.set(e.value().to_string()),
                                placeholder: "e.g., Scheduling Order",
                            }

                            FormSelect {
                                label: "Order Type *",
                                value: "{form_order_type}",
                                onchange: move |e: Event<FormData>| form_order_type.set(e.value()),
                                for ot in ORDER_TYPES.iter() {
                                    option { value: *ot, "{ot}" }
                                }
                            }

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

                            // Judge selector
                            label { class: "input-label", "Judge *" }
                            select {
                                class: "input",
                                value: form_judge_id.read().clone(),
                                onchange: move |e: FormEvent| form_judge_id.set(e.value().to_string()),
                                option { value: "", "-- Select a judge --" }
                                {match &*judges_for_select.read() {
                                    Some(Some(judges)) => rsx! {
                                        for j in judges.iter() {
                                            option {
                                                value: j["id"].as_str().unwrap_or(""),
                                                {j["name"].as_str().unwrap_or("Unknown")}
                                            }
                                        }
                                    },
                                    _ => rsx! {
                                        option { value: "", disabled: true, "Loading judges..." }
                                    },
                                }}
                            }

                            FormSelect {
                                label: "Status",
                                value: "{form_status}",
                                onchange: move |e: Event<FormData>| form_status.set(e.value()),
                                for s in ORDER_STATUSES.iter() {
                                    option { value: *s, "{s}" }
                                }
                            }

                            label { class: "input-label", "Content" }
                            textarea {
                                class: "input",
                                rows: 4,
                                value: form_content.read().clone(),
                                oninput: move |e: FormEvent| form_content.set(e.value().to_string()),
                                placeholder: "Order content...",
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Create Order"
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
