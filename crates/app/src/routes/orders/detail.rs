use dioxus::prelude::*;
use shared_types::{JudicialOrderResponse, OrderTemplateResponse};
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, Card, CardContent, CardHeader, CardTitle, DetailGrid, DetailItem, DetailList,
    PageActions, PageHeader, PageTitle, Separator, Skeleton, TabContent, TabList, TabTrigger, Tabs,
};
use shared_ui::{use_toast, ToastOptions};

use super::form_sheet::{OrderFormSheet, FormMode};
use crate::auth::{can, use_user_role, Action};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn OrderDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let order_id = id.clone();
    let toast = use_toast();
    let role = use_user_role();
    let mut show_edit = use_signal(|| false);

    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);
    let mut generating_pdf = use_signal(|| false);
    let mut pdf_html = use_signal::<Option<String>>(|| None);

    let mut data = use_resource(move || {
        let court = court_id.clone();
        let oid = order_id.clone();
        async move {
            match server::api::get_order(court, oid).await {
                Ok(json) => serde_json::from_str::<JudicialOrderResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let oid = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_order(court, oid).await {
                Ok(()) => {
                    toast.success(
                        "Order deleted successfully".to_string(),
                        ToastOptions::new(),
                    );
                    let nav = navigator();
                    nav.push(Route::OrderList {});
                }
                Err(e) => {
                    toast.error(format!("{}", e), ToastOptions::new());
                    deleting.set(false);
                    show_delete_confirm.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(order)) => {
                    let pdf_order_id = id.clone();
                    let is_signed = order.status == "Signed" || order.status == "Filed";
                    let handle_generate_pdf = move |_: MouseEvent| {
                        let court = ctx.court_id.read().clone();
                        let oid = pdf_order_id.clone();
                        spawn(async move {
                            generating_pdf.set(true);
                            match server::api::generate_order_html(court, oid, is_signed).await {
                                Ok(html) => {
                                    pdf_html.set(Some(html));
                                }
                                Err(e) => {
                                    toast.error(format!("PDF generation failed: {}", e), ToastOptions::new());
                                }
                            }
                            generating_pdf.set(false);
                        });
                    };

                    rsx! {
                    PageHeader {
                        PageTitle { "{order.title}" }
                        PageActions {
                            Link { to: Route::OrderList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                            if can(&role, Action::GeneratePdf) {
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: handle_generate_pdf,
                                    if *generating_pdf.read() { "Generating..." } else { "Generate PDF" }
                                }
                            }
                            if can(&role, Action::Edit) {
                                Button {
                                    variant: ButtonVariant::Primary,
                                    onclick: move |_| show_edit.set(true),
                                    "Edit"
                                }
                            }
                            if can(&role, Action::Delete) {
                                Button {
                                    variant: ButtonVariant::Destructive,
                                    onclick: move |_| show_delete_confirm.set(true),
                                    "Delete"
                                }
                            }
                        }
                    }

                    AlertDialogRoot {
                        open: show_delete_confirm(),
                        on_open_change: move |v| show_delete_confirm.set(v),
                        AlertDialogContent {
                            AlertDialogTitle { "Delete Order" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this order? This action cannot be undone."
                            }
                            AlertDialogActions {
                                AlertDialogCancel { "Cancel" }
                                AlertDialogAction {
                                    on_click: handle_delete,
                                    if *deleting.read() { "Deleting..." } else { "Delete" }
                                }
                            }
                        }
                    }

                    Tabs { default_value: "content", horizontal: true,
                        TabList {
                            TabTrigger { value: "content", index: 0usize, "Content" }
                            TabTrigger { value: "workflow", index: 1usize, "Workflow" }
                            TabTrigger { value: "templates", index: 2usize, "Templates" }
                        }
                        TabContent { value: "content", index: 0usize,
                            ContentTab { order: order.clone() }
                        }
                        TabContent { value: "workflow", index: 1usize,
                            WorkflowTab { order_id: id.clone(), order: order.clone(), data: data }
                        }
                        TabContent { value: "templates", index: 2usize,
                            TemplatesTab {}
                        }
                    }

                    OrderFormSheet {
                        mode: FormMode::Edit,
                        initial: Some(order.clone()),
                        open: show_edit(),
                        on_close: move |_| show_edit.set(false),
                        on_saved: move |_| data.restart(),
                    }

                    if let Some(html) = pdf_html.read().as_ref() {
                        Card {
                            CardHeader {
                                div {
                                    style: "display: flex; justify-content: space-between; align-items: center;",
                                    CardTitle { "PDF Preview" }
                                    div {
                                        style: "display: flex; gap: var(--space-sm);",
                                        Button {
                                            variant: ButtonVariant::Secondary,
                                            onclick: move |_| pdf_html.set(None),
                                            "Close Preview"
                                        }
                                    }
                                }
                            }
                            CardContent {
                                p { class: "text-muted",
                                    style: "margin-bottom: var(--space-md);",
                                    "Use your browser's Print function (Ctrl+P / Cmd+P) to save as PDF."
                                }
                                div {
                                    class: "pdf-preview",
                                    style: "border: 1px solid var(--border); padding: var(--space-lg); background: white; max-height: 600px; overflow-y: auto;",
                                    dangerous_inner_html: "{html}",
                                }
                            }
                        }
                    }
                }},
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Order Not Found" }
                                p { "The order you're looking for doesn't exist in this court district." }
                                Link { to: Route::OrderList {},
                                    Button { "Back to List" }
                                }
                            }
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
        }
    }
}

/// Content tab showing the order details.
#[component]
fn ContentTab(order: JudicialOrderResponse) -> Element {
    let effective_display = order
        .effective_date
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());
    let expiration_display = order
        .expiration_date
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());
    let issued_display = order
        .issued_at
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());
    let signed_display = order
        .signed_at
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());
    let signer_display = order
        .signer_name
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Order Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Title", value: order.title.clone() }
                        DetailItem { label: "Order Type",
                            Badge { variant: type_badge_variant(&order.order_type),
                                "{order.order_type}"
                            }
                        }
                        DetailItem { label: "Status",
                            Badge { variant: status_badge_variant(&order.status),
                                "{order.status}"
                            }
                        }
                        DetailItem { label: "Sealed",
                            if order.is_sealed {
                                Badge { variant: BadgeVariant::Destructive, "Yes" }
                            } else {
                                Badge { variant: BadgeVariant::Secondary, "No" }
                            }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "References" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Case ID", value: order.case_id.clone() }
                        DetailItem { label: "Judge ID", value: order.judge_id.clone() }
                        DetailItem { label: "Signer", value: signer_display }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Dates" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Effective Date", value: effective_display }
                        DetailItem { label: "Expiration Date", value: expiration_display }
                        DetailItem { label: "Signed At", value: signed_display }
                        DetailItem { label: "Issued At", value: issued_display }
                        DetailItem {
                            label: "Created",
                            value: order.created_at.chars().take(10).collect::<String>()
                        }
                        DetailItem {
                            label: "Updated",
                            value: order.updated_at.chars().take(10).collect::<String>()
                        }
                    }
                }
            }
        }

        if !order.content.is_empty() {
            Card {
                CardHeader { CardTitle { "Order Content" } }
                CardContent {
                    pre {
                        class: "order-content-text",
                        style: "white-space: pre-wrap; word-wrap: break-word; font-family: inherit; margin: 0;",
                        "{order.content}"
                    }
                }
            }
        }
    }
}

/// Workflow tab showing the order's status progression and action buttons.
#[component]
fn WorkflowTab(
    order_id: String,
    order: JudicialOrderResponse,
    mut data: Resource<Option<JudicialOrderResponse>>,
) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();
    let mut signing = use_signal(|| false);
    let mut issuing = use_signal(|| false);

    let status = order.status.clone();

    // Determine which workflow actions are available based on current status
    let can_sign = status == "Draft" || status == "Pending Signature";
    let can_issue = status == "Signed";

    let sign_id = order_id.clone();
    let handle_sign = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let oid = sign_id.clone();
        let signer = order
            .signer_name
            .clone()
            .unwrap_or_else(|| "Court Official".to_string());
        spawn(async move {
            signing.set(true);
            match server::api::sign_order_action(court, oid, signer).await {
                Ok(_) => {
                    toast.success("Order signed successfully".to_string(), ToastOptions::new());
                    data.restart();
                }
                Err(e) => {
                    toast.error(format!("{}", e), ToastOptions::new());
                }
            }
            signing.set(false);
        });
    };

    let issue_id = order_id.clone();
    let handle_issue = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let oid = issue_id.clone();
        spawn(async move {
            issuing.set(true);
            match server::api::issue_order_action(court, oid).await {
                Ok(_) => {
                    toast.success("Order issued successfully".to_string(), ToastOptions::new());
                    data.restart();
                }
                Err(e) => {
                    toast.error(format!("{}", e), ToastOptions::new());
                }
            }
            issuing.set(false);
        });
    };

    rsx! {
        Card {
            CardHeader { CardTitle { "Order Workflow" } }
            CardContent {
                DetailList {
                    DetailItem { label: "Current Status",
                        Badge { variant: status_badge_variant(&status), "{status}" }
                    }
                }

                Separator {}

                div { class: "workflow-steps",
                    style: "display: flex; gap: var(--space-md); align-items: center; flex-wrap: wrap; margin-top: var(--space-md);",

                    WorkflowStep {
                        label: "Draft",
                        active: status == "Draft",
                        completed: status != "Draft",
                    }
                    span { class: "workflow-arrow", style: "font-size: 1.2rem; color: var(--muted);", "→" }
                    WorkflowStep {
                        label: "Pending Signature",
                        active: status == "Pending Signature",
                        completed: status == "Signed" || status == "Filed",
                    }
                    span { class: "workflow-arrow", style: "font-size: 1.2rem; color: var(--muted);", "→" }
                    WorkflowStep {
                        label: "Signed",
                        active: status == "Signed",
                        completed: status == "Filed",
                    }
                    span { class: "workflow-arrow", style: "font-size: 1.2rem; color: var(--muted);", "→" }
                    WorkflowStep {
                        label: "Filed",
                        active: status == "Filed",
                        completed: false,
                    }
                }

                Separator {}

                div {
                    style: "display: flex; gap: var(--space-sm); margin-top: var(--space-md);",

                    if can_sign {
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: handle_sign,
                            if *signing.read() { "Signing..." } else { "Sign Order" }
                        }
                    }
                    if can_issue {
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: handle_issue,
                            if *issuing.read() { "Issuing..." } else { "Issue Order" }
                        }
                    }
                    if !can_sign && !can_issue {
                        p { class: "text-muted", "No workflow actions available for this status." }
                    }
                }
            }
        }
    }
}

/// A single step in the workflow visualization.
#[component]
fn WorkflowStep(label: String, active: bool, completed: bool) -> Element {
    let variant = if active {
        BadgeVariant::Primary
    } else if completed {
        BadgeVariant::Secondary
    } else {
        BadgeVariant::Outline
    };

    rsx! {
        Badge { variant: variant, "{label}" }
    }
}

/// Templates tab showing available order templates.
#[component]
fn TemplatesTab() -> Element {
    let ctx = use_context::<CourtContext>();

    let templates = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            match server::api::list_order_templates(court).await {
                Ok(json) => serde_json::from_str::<Vec<OrderTemplateResponse>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        match &*templates.read() {
            Some(Some(list)) if !list.is_empty() => rsx! {
                div { class: "templates-list",
                    for template in list.iter() {
                        TemplateCard { template: template.clone() }
                    }
                }
            },
            Some(_) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No order templates available for this court." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// Individual template card display.
#[component]
fn TemplateCard(template: OrderTemplateResponse) -> Element {
    let description_display = template
        .description
        .clone()
        .unwrap_or_else(|| "No description".to_string());
    let active_variant = if template.is_active {
        BadgeVariant::Primary
    } else {
        BadgeVariant::Outline
    };
    let active_label = if template.is_active { "Active" } else { "Inactive" };

    rsx! {
        Card {
            CardHeader {
                CardTitle { "{template.name}" }
            }
            CardContent {
                DetailList {
                    DetailItem { label: "Order Type",
                        Badge { variant: BadgeVariant::Secondary, "{template.order_type}" }
                    }
                    DetailItem { label: "Status",
                        Badge { variant: active_variant, "{active_label}" }
                    }
                    DetailItem { label: "Description", value: description_display }
                    DetailItem {
                        label: "Created",
                        value: template.created_at.chars().take(10).collect::<String>()
                    }
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
