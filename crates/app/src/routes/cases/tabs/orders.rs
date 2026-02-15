use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant,
    DataTable, DataTableBody, DataTableCell, DataTableColumn, DataTableHeader, DataTableRow,
    Form, FormSelect, Input, Separator,
    Sheet, SheetClose, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle,
    Skeleton,
};
use shared_ui::{use_toast, ToastOptions};

use crate::CourtContext;

#[component]
pub fn OrdersTab(case_id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut show_sheet = use_signal(|| false);
    let mut form_order_type = use_signal(|| "scheduling_order".to_string());
    let mut form_title = use_signal(String::new);
    let mut form_template_id = use_signal(String::new);

    let case_id_save = case_id.clone();

    let mut orders_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move {
            server::api::list_orders_by_case(court, cid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    let templates_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            server::api::list_active_order_templates(court)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let cid = case_id_save.clone();
        let otype = form_order_type.read().clone();
        let title = form_title.read().clone();
        let template = form_template_id.read().clone();

        spawn(async move {
            if title.trim().is_empty() {
                toast.error("Title is required.".to_string(), ToastOptions::new());
                return;
            }
            let mut body = serde_json::json!({
                "case_id": cid,
                "order_type": otype,
                "title": title.trim(),
            });
            if !template.is_empty() {
                body["template_id"] = serde_json::Value::String(template);
            }
            match server::api::create_order(court, body.to_string()).await {
                Ok(_) => {
                    toast.success("Order drafted.".to_string(), ToastOptions::new());
                    show_sheet.set(false);
                    form_title.set(String::new());
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
            Button {
                variant: ButtonVariant::Primary,
                onclick: move |_| show_sheet.set(true),
                "Draft Order"
            }
        }

        match &*orders_data.read() {
            Some(Some(orders)) if !orders.is_empty() => rsx! {
                DataTable {
                    DataTableHeader {
                        DataTableColumn { "Title" }
                        DataTableColumn { "Order Type" }
                        DataTableColumn { "Date Issued" }
                        DataTableColumn { "Status" }
                    }
                    DataTableBody {
                        for order in orders.iter() {
                            DataTableRow {
                                DataTableCell { {order["title"].as_str().unwrap_or("—")} }
                                DataTableCell {
                                    Badge { variant: BadgeVariant::Secondary,
                                        {order["order_type"].as_str().unwrap_or("—").replace('_', " ")}
                                    }
                                }
                                DataTableCell {
                                    {order["date_issued"].as_str().map(|d| &d[..10]).unwrap_or("—")}
                                }
                                DataTableCell {
                                    Badge { variant: BadgeVariant::Primary,
                                        {order["status"].as_str().unwrap_or("draft").replace('_', " ")}
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
                            option { value: "scheduling_order", "Scheduling Order" }
                            option { value: "protective_order", "Protective Order" }
                            option { value: "sealing_order", "Sealing Order" }
                            option { value: "minute_order", "Minute Order" }
                            option { value: "standing_order", "Standing Order" }
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
                                                value: tpl["id"].as_str().unwrap_or(""),
                                                {tpl["name"].as_str().unwrap_or("Unnamed")}
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
