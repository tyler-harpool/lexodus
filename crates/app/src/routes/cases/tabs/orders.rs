use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant,
    DataTable, DataTableBody, DataTableCell, DataTableColumn, DataTableHeader, DataTableRow,
    Form, FormSelect, Input, Separator,
    Sheet, SheetClose, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle,
    Skeleton,
};
use shared_ui::{use_toast, ToastOptions};

use crate::auth::{can, use_user_role, Action};
use crate::CourtContext;

#[component]
pub fn OrdersTab(case_id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();
    let role = use_user_role();

    let mut show_sheet = use_signal(|| false);
    let mut form_order_type = use_signal(|| "Scheduling".to_string());
    let mut form_title = use_signal(String::new);
    let mut form_content = use_signal(String::new);
    let mut form_judge_id = use_signal(String::new);
    let mut form_judge_name = use_signal(String::new);
    let mut form_template_id = use_signal(String::new);

    // Fetch the case assignment to get the assigned judge ID and name
    let case_id_for_judge = case_id.clone();
    let _case_judge = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id_for_judge.clone();
        async move {
            if let Ok(assignments) = server::api::list_case_assignments(court, cid).await {
                if let Some(a) = assignments.first() {
                    form_judge_id.set(a.judge_id.clone());
                    if let Some(ref name) = a.judge_name {
                        form_judge_name.set(name.clone());
                    }
                }
            }
        }
    });

    let case_id_save = case_id.clone();

    let mut orders_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move {
            server::api::list_orders_by_case(court, cid)
                .await
                .ok()
        }
    });

    let templates_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            server::api::list_active_order_templates(court)
                .await
                .ok()
        }
    });

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let cid = case_id_save.clone();
        let otype = form_order_type.read().clone();
        let title = form_title.read().clone();

        spawn(async move {
            if title.trim().is_empty() {
                toast.error("Title is required.".to_string(), ToastOptions::new());
                return;
            }
            let judge = form_judge_id.read().clone();
            if judge.is_empty() {
                toast.error("No judge assigned to this case.".to_string(), ToastOptions::new());
                return;
            }
            let content = form_content.read().clone();
            let case_uuid = match uuid::Uuid::parse_str(&cid) {
                Ok(u) => u,
                Err(_) => {
                    toast.error("Invalid case ID.".to_string(), ToastOptions::new());
                    return;
                }
            };
            let judge_uuid = match uuid::Uuid::parse_str(&judge) {
                Ok(u) => u,
                Err(_) => {
                    toast.error("Invalid judge ID.".to_string(), ToastOptions::new());
                    return;
                }
            };
            let req = shared_types::CreateJudicialOrderRequest {
                case_id: case_uuid,
                judge_id: judge_uuid,
                order_type: otype,
                title: title.trim().to_string(),
                content: if content.trim().is_empty() {
                    "Draft order content pending.".to_string()
                } else {
                    content.trim().to_string()
                },
                status: None,
                is_sealed: None,
                effective_date: None,
                expiration_date: None,
                related_motions: Vec::new(),
            };
            match server::api::create_order(court, req).await {
                Ok(_) => {
                    toast.success("Order drafted.".to_string(), ToastOptions::new());
                    show_sheet.set(false);
                    form_title.set(String::new());
                    form_content.set(String::new());
                    orders_data.restart();
                }
                Err(e) => toast.error(format!("Error: {e}"), ToastOptions::new()),
            }
        });
    };

    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-md);",
            h3 { "Court Orders" }
            if can(&role, Action::EditCase) {
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| show_sheet.set(true),
                    "Draft Order"
                }
            }
        }

        match &*orders_data.read() {
            Some(Some(orders)) if !orders.is_empty() => rsx! {
                DataTable {
                    DataTableHeader {
                        DataTableColumn { "Title" }
                        DataTableColumn { "Judge" }
                        DataTableColumn { "Order Type" }
                        DataTableColumn { "Date" }
                        DataTableColumn { "Status" }
                    }
                    DataTableBody {
                        for order in orders.iter() {
                            {
                                let date_display = order.issued_at.as_deref()
                                    .unwrap_or(&order.created_at);
                                let date_short = date_display.get(..10).unwrap_or(date_display);
                                rsx! {
                                    DataTableRow {
                                        DataTableCell { {order.title.clone()} }
                                        DataTableCell {
                                            {order.judge_name.as_deref().unwrap_or("\u{2014}")}
                                        }
                                        DataTableCell {
                                            Badge { variant: BadgeVariant::Secondary,
                                                {order.order_type.replace('_', " ")}
                                            }
                                        }
                                        DataTableCell {
                                            {date_short.to_string()}
                                        }
                                        DataTableCell {
                                            Badge { variant: BadgeVariant::Primary,
                                                {order.status.replace('_', " ")}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Some(Some(_)) => rsx! {
                p { class: "empty-state", "No orders for this case yet." }
            },
            Some(None) => rsx! {
                p { class: "error-state", "Failed to load orders." }
            },
            None => rsx! {
                Skeleton { style: "width: 100%; height: 200px" }
            },
        }

        Sheet {
            open: show_sheet(),
            on_close: move |_| show_sheet.set(false),
            side: SheetSide::Right,
            SheetContent {
                SheetHeader {
                    SheetTitle { "Draft Order" }
                    SheetClose { on_close: move |_| show_sheet.set(false) }
                }
                Form {
                    onsubmit: handle_save,
                    div { class: "sheet-form",
                        Input {
                            label: "Title",
                            value: form_title(),
                            on_input: move |e: FormEvent| form_title.set(e.value()),
                            placeholder: "Order title",
                        }
                        FormSelect {
                            label: "Order Type",
                            value: "{form_order_type}",
                            onchange: move |e: Event<FormData>| form_order_type.set(e.value()),
                            option { value: "Scheduling", "Scheduling Order" }
                            option { value: "Protective", "Protective Order" }
                            option { value: "Sealing", "Sealing Order" }
                            option { value: "Procedural", "Minute Order" }
                            option { value: "Standing", "Standing Order" }
                            option { value: "Discovery", "Discovery Order" }
                            option { value: "Dismissal", "Dismissal Order" }
                            option { value: "Sentencing", "Sentencing Order" }
                            option { value: "Detention", "Detention Order" }
                            option { value: "Release", "Release Order" }
                            option { value: "Restraining", "Restraining Order" }
                            option { value: "Contempt", "Contempt Order" }
                            option { value: "Other", "Other" }
                        }

                        // Judge (auto-populated from case assignment)
                        if !form_judge_id.read().is_empty() {
                            div { class: "form-field",
                                label { class: "form-label", "Assigned Judge" }
                                p { class: "form-static-value",
                                    if form_judge_name.read().is_empty() {
                                        "{form_judge_id}"
                                    } else {
                                        "{form_judge_name}"
                                    }
                                }
                            }
                        }

                        // Order content
                        div { class: "form-field",
                            label { class: "form-label", "Content" }
                            textarea {
                                class: "input",
                                rows: 4,
                                placeholder: "Order content (optional — defaults to draft placeholder)",
                                value: "{form_content}",
                                oninput: move |e: Event<FormData>| form_content.set(e.value()),
                            }
                        }

                        // Template selector (loaded from server)
                        {
                            match &*templates_data.read() {
                                Some(Some(templates)) if !templates.is_empty() => rsx! {
                                    FormSelect {
                                        label: "Template (optional)",
                                        value: "{form_template_id}",
                                        onchange: move |e: Event<FormData>| form_template_id.set(e.value()),
                                        option { value: "", "No template" }
                                        for tpl in templates.iter() {
                                            option {
                                                value: "{tpl.id}",
                                                {tpl.name.clone()}
                                            }
                                        }
                                    }
                                },
                                _ => rsx! {},
                            }
                        }
                    }
                    Separator {}
                    SheetFooter {
                        div { class: "sheet-footer-actions",
                            SheetClose { on_close: move |_| show_sheet.set(false) }
                            Button { variant: ButtonVariant::Primary, "Create Order" }
                        }
                    }
                }
            }
        }
    }
}
