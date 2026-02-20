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
pub fn PartiesTab(case_id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut show_sheet = use_signal(|| false);
    let mut form_name = use_signal(String::new);
    let mut form_party_type = use_signal(|| "defendant".to_string());
    let mut form_role = use_signal(String::new);

    let case_id_save = case_id.clone();

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move {
            server::api::list_case_parties(court, cid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let cid = case_id_save.clone();
        let name = form_name.read().clone();
        let ptype = form_party_type.read().clone();
        let role = form_role.read().clone();

        spawn(async move {
            if name.trim().is_empty() {
                toast.error("Name is required.".to_string(), ToastOptions::new());
                return;
            }
            let body = serde_json::json!({
                "case_id": cid,
                "name": name.trim(),
                "party_type": ptype,
                "role": role.trim(),
            });
            match server::api::create_party(court, body.to_string()).await {
                Ok(_) => {
                    toast.success("Party added.".to_string(), ToastOptions::new());
                    show_sheet.set(false);
                    form_name.set(String::new());
                    form_role.set(String::new());
                    data.restart();
                }
                Err(e) => toast.error(format!("Error: {e}"), ToastOptions::new()),
            }
        });
    };

    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-md);",
            h3 { "Case Parties" }
            Button {
                variant: ButtonVariant::Primary,
                onclick: move |_| show_sheet.set(true),
                "Add Party"
            }
        }

        match &*data.read() {
            Some(Some(parties)) if !parties.is_empty() => rsx! {
                DataTable {
                    DataTableHeader {
                        DataTableColumn { "Name" }
                        DataTableColumn { "Type" }
                        DataTableColumn { "Role" }
                        DataTableColumn { "Status" }
                    }
                    DataTableBody {
                        for party in parties.iter() {
                            DataTableRow {
                                DataTableCell { {party["name"].as_str().unwrap_or("—")} }
                                DataTableCell {
                                    Badge { variant: BadgeVariant::Secondary,
                                        {party["party_type"].as_str().unwrap_or("—")}
                                    }
                                }
                                DataTableCell { {party["party_role"].as_str().unwrap_or("—")} }
                                DataTableCell {
                                    Badge { variant: BadgeVariant::Primary,
                                        {party["status"].as_str().unwrap_or("active")}
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Some(Some(_)) => rsx! {
                p { class: "empty-state", "No parties added to this case yet." }
            },
            Some(None) => rsx! {
                p { class: "error-state", "Failed to load parties." }
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
                    SheetTitle { "Add Party" }
                    SheetClose { on_close: move |_| show_sheet.set(false) }
                }
                Form {
                    onsubmit: handle_save,
                    div { class: "sheet-form",
                        Input {
                            label: "Name",
                            value: form_name(),
                            on_input: move |e: FormEvent| form_name.set(e.value()),
                            placeholder: "Party name",
                        }
                        FormSelect {
                            label: "Party Type",
                            value: "{form_party_type}",
                            onchange: move |e: Event<FormData>| form_party_type.set(e.value()),
                            option { value: "defendant", "Defendant" }
                            option { value: "prosecution", "Prosecution" }
                            option { value: "witness", "Witness" }
                            option { value: "intervenor", "Intervenor" }
                            option { value: "amicus", "Amicus Curiae" }
                        }
                        Input {
                            label: "Role",
                            value: form_role(),
                            on_input: move |e: FormEvent| form_role.set(e.value()),
                            placeholder: "e.g., Lead Defense Counsel",
                        }
                    }
                    Separator {}
                    SheetFooter {
                        div { class: "sheet-footer-actions",
                            SheetClose { on_close: move |_| show_sheet.set(false) }
                            Button { variant: ButtonVariant::Primary, "Save" }
                        }
                    }
                }
            }
        }
    }
}
