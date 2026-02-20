use dioxus::prelude::*;
use shared_types::{CustodyTransferResponse, EvidenceResponse};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardHeader,
    DataTable, DataTableBody, DataTableCell, DataTableColumn, DataTableHeader, DataTableRow,
    Form, FormSelect, Input, Separator,
    Sheet, SheetClose, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle,
    Skeleton,
};
use shared_ui::{use_toast, ToastOptions};

use crate::CourtContext;

#[component]
pub fn EvidenceTab(case_id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut show_create_sheet = use_signal(|| false);
    let mut show_custody_sheet = use_signal(|| false);
    let mut selected_evidence_id = use_signal(String::new);
    let mut custody_chain = use_signal(|| Option::<Vec<CustodyTransferResponse>>::None);

    let mut form_description = use_signal(String::new);
    let mut form_evidence_type = use_signal(|| "Documentary".to_string());
    let mut form_location = use_signal(String::new);

    let case_id_save = case_id.clone();
    let case_id_custody = case_id.clone();

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move {
            server::api::list_evidence_by_case(court, cid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<EvidenceResponse>>(&json).ok())
        }
    });

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let cid = case_id_save.clone();
        let desc = form_description.read().clone();
        let etype = form_evidence_type.read().clone();
        let loc = form_location.read().clone();

        spawn(async move {
            if desc.trim().is_empty() {
                toast.error("Description is required.".to_string(), ToastOptions::new());
                return;
            }
            let body = serde_json::json!({
                "case_id": cid,
                "description": desc.trim(),
                "evidence_type": etype,
                "location": loc.trim(),
            });
            match server::api::create_evidence(court, body.to_string()).await {
                Ok(_) => {
                    toast.success("Evidence added.".to_string(), ToastOptions::new());
                    show_create_sheet.set(false);
                    form_description.set(String::new());
                    form_location.set(String::new());
                    data.restart();
                }
                Err(e) => toast.error(format!("Error: {e}"), ToastOptions::new()),
            }
        });
    };

    // view_custody is inlined into onclick handlers to avoid FnMut/move issues
    let _ = &case_id_custody;

    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-md);",
            h3 { "Evidence" }
            Button {
                variant: ButtonVariant::Primary,
                onclick: move |_| show_create_sheet.set(true),
                "Add Evidence"
            }
        }

        match &*data.read() {
            Some(Some(items)) if !items.is_empty() => rsx! {
                DataTable {
                    DataTableHeader {
                        DataTableColumn { "Description" }
                        DataTableColumn { "Type" }
                        DataTableColumn { "Location" }
                        DataTableColumn { "Sealed" }
                        DataTableColumn { "Actions" }
                    }
                    DataTableBody {
                        for item in items.iter() {
                            {
                                let eid = item.id.clone();
                                rsx! {
                                    DataTableRow {
                                        DataTableCell { {item.description.clone()} }
                                        DataTableCell {
                                            Badge { variant: BadgeVariant::Secondary,
                                                {item.evidence_type.clone()}
                                            }
                                        }
                                        DataTableCell { {item.location.clone()} }
                                        DataTableCell {
                                            if item.is_sealed {
                                                Badge { variant: BadgeVariant::Destructive, "Sealed" }
                                            } else {
                                                Badge { variant: BadgeVariant::Secondary, "Open" }
                                            }
                                        }
                                        DataTableCell {
                                            Button {
                                                variant: ButtonVariant::Outline,
                                                onclick: {
                                                    let eid = eid.clone();
                                                    move |_| {
                                                        let court = ctx.court_id.read().clone();
                                                        let evidence_id = eid.clone();
                                                        selected_evidence_id.set(evidence_id.clone());
                                                        custody_chain.set(None);
                                                        show_custody_sheet.set(true);
                                                        spawn(async move {
                                                            match server::api::list_custody_transfers(court, evidence_id).await {
                                                                Ok(json) => {
                                                                    let transfers = serde_json::from_str::<Vec<CustodyTransferResponse>>(&json)
                                                                        .unwrap_or_default();
                                                                    custody_chain.set(Some(transfers));
                                                                }
                                                                Err(_) => custody_chain.set(Some(Vec::new())),
                                                            }
                                                        });
                                                    }
                                                },
                                                "Custody Chain"
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
                p { class: "empty-state", "No evidence recorded for this case." }
            },
            Some(None) => rsx! {
                p { class: "error-state", "Failed to load evidence." }
            },
            None => rsx! {
                Skeleton { style: "width: 100%; height: 200px" }
            },
        }

        // Create Evidence Sheet
        Sheet {
            open: show_create_sheet(),
            on_close: move |_| show_create_sheet.set(false),
            side: SheetSide::Right,
            SheetContent {
                SheetHeader {
                    SheetTitle { "Add Evidence" }
                    SheetClose { on_close: move |_| show_create_sheet.set(false) }
                }
                Form {
                    onsubmit: handle_save,
                    div { class: "sheet-form",
                        Input {
                            label: "Description",
                            value: form_description(),
                            on_input: move |e: FormEvent| form_description.set(e.value()),
                            placeholder: "Evidence description",
                        }
                        FormSelect {
                            label: "Evidence Type",
                            value: "{form_evidence_type}",
                            onchange: move |e: Event<FormData>| form_evidence_type.set(e.value()),
                            option { value: "Physical", "Physical" }
                            option { value: "Documentary", "Documentary" }
                            option { value: "Digital", "Digital" }
                            option { value: "Testimonial", "Testimonial" }
                            option { value: "Demonstrative", "Demonstrative" }
                            option { value: "Forensic", "Forensic" }
                            option { value: "Other", "Other" }
                        }
                        Input {
                            label: "Storage Location",
                            value: form_location(),
                            on_input: move |e: FormEvent| form_location.set(e.value()),
                            placeholder: "e.g., Evidence Locker A-12",
                        }
                    }
                    Separator {}
                    SheetFooter {
                        div { class: "sheet-footer-actions",
                            SheetClose { on_close: move |_| show_create_sheet.set(false) }
                            Button { variant: ButtonVariant::Primary, "Save" }
                        }
                    }
                }
            }
        }

        // Custody Chain Sheet
        Sheet {
            open: show_custody_sheet(),
            on_close: move |_| show_custody_sheet.set(false),
            side: SheetSide::Right,
            SheetContent {
                SheetHeader {
                    SheetTitle { "Chain of Custody" }
                    SheetClose { on_close: move |_| show_custody_sheet.set(false) }
                }
                div { style: "padding: var(--space-md);",
                    match &*custody_chain.read() {
                        Some(transfers) if !transfers.is_empty() => rsx! {
                            for (i, t) in transfers.iter().enumerate() {
                                {
                                    let date_short = t.date.get(..10).unwrap_or(&t.date);
                                    rsx! {
                                        Card {
                                            CardHeader {
                                                "Transfer #{i + 1}"
                                            }
                                            CardContent {
                                                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-sm);",
                                                    div {
                                                        span { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted);", "From" }
                                                        p { {t.transferred_from.clone()} }
                                                    }
                                                    div {
                                                        span { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted);", "To" }
                                                        p { {t.transferred_to.clone()} }
                                                    }
                                                    div {
                                                        span { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted);", "Date" }
                                                        p { {date_short.to_string()} }
                                                    }
                                                    div {
                                                        span { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted);", "Condition" }
                                                        p { {t.condition.clone()} }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(_) => rsx! {
                            p { class: "empty-state", "No custody transfers recorded." }
                        },
                        None => rsx! {
                            Skeleton { style: "width: 100%; height: 100px" }
                        },
                    }
                }
            }
        }
    }
}
