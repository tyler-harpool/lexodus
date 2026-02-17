use dioxus::prelude::*;
use shared_types::{CustodyTransferResponse, EvidenceResponse};
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, Card, CardContent, CardHeader, CardTitle, DetailGrid, DetailItem, DetailList,
    PageActions, PageHeader, PageTitle, Skeleton, TabContent, TabList, TabTrigger, Tabs,
};
use shared_ui::{use_toast, ToastOptions};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn EvidenceDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let evidence_id = id.clone();
    let toast = use_toast();

    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let data = use_resource(move || {
        let court = court_id.clone();
        let eid = evidence_id.clone();
        async move {
            match server::api::get_evidence(court, eid).await {
                Ok(json) => serde_json::from_str::<EvidenceResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let eid = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_evidence(court, eid).await {
                Ok(()) => {
                    toast.success(
                        "Evidence deleted successfully".to_string(),
                        ToastOptions::new(),
                    );
                    let nav = navigator();
                    nav.push(Route::EvidenceList {});
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
                Some(Some(evidence)) => rsx! {
                    PageHeader {
                        PageTitle { "{evidence.description}" }
                        PageActions {
                            Link { to: Route::EvidenceList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                            Button {
                                variant: ButtonVariant::Destructive,
                                onclick: move |_| show_delete_confirm.set(true),
                                "Delete"
                            }
                        }
                    }

                    AlertDialogRoot {
                        open: show_delete_confirm(),
                        on_open_change: move |v| show_delete_confirm.set(v),
                        AlertDialogContent {
                            AlertDialogTitle { "Delete Evidence" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this evidence item? This action cannot be undone."
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

                    Tabs { default_value: "metadata", horizontal: true,
                        TabList {
                            TabTrigger { value: "metadata", index: 0usize, "Metadata" }
                            TabTrigger { value: "custody", index: 1usize, "Custody Transfers" }
                        }
                        TabContent { value: "metadata", index: 0usize,
                            MetadataTab { evidence: evidence.clone() }
                        }
                        TabContent { value: "custody", index: 1usize,
                            CustodyTab { evidence_id: id.clone() }
                        }
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Evidence Not Found" }
                                p { "The evidence item you're looking for doesn't exist in this court district." }
                                Link { to: Route::EvidenceList {},
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

/// Metadata tab showing the evidence item's details.
#[component]
fn MetadataTab(evidence: EvidenceResponse) -> Element {
    let ev = &evidence;
    let seized_date_display = ev
        .seized_date
        .clone()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());
    let seized_by_display = ev
        .seized_by
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let location_display = if ev.location.is_empty() {
        "--".to_string()
    } else {
        ev.location.clone()
    };
    let created_display = ev.created_at.chars().take(10).collect::<String>();

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Evidence Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Description", value: ev.description.clone() }
                        DetailItem { label: "Evidence Type",
                            Badge {
                                variant: evidence_type_badge_variant(&ev.evidence_type),
                                "{ev.evidence_type}"
                            }
                        }
                        DetailItem { label: "Case ID", value: ev.case_id.clone() }
                        DetailItem { label: "Status",
                            if ev.is_sealed {
                                Badge { variant: BadgeVariant::Destructive, "Sealed" }
                            } else {
                                Badge { variant: BadgeVariant::Secondary, "Open" }
                            }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Collection & Storage" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Seized Date", value: seized_date_display }
                        DetailItem { label: "Seized By", value: seized_by_display }
                        DetailItem { label: "Storage Location", value: location_display }
                        DetailItem { label: "Created", value: created_display }
                    }
                }
            }
        }
    }
}

/// Custody Transfers tab listing all chain-of-custody transfers for this evidence.
#[component]
fn CustodyTab(evidence_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let transfers = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let eid = evidence_id.clone();
        async move {
            match server::api::list_custody_transfers(court, eid).await {
                Ok(json) => serde_json::from_str::<Vec<CustodyTransferResponse>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        match &*transfers.read() {
            Some(Some(list)) if !list.is_empty() => rsx! {
                div { class: "charges-list",
                    for transfer in list.iter() {
                        TransferCard { transfer: transfer.clone() }
                    }
                }
            },
            Some(_) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No custody transfers recorded for this evidence item." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// Individual custody transfer card display.
#[component]
fn TransferCard(transfer: CustodyTransferResponse) -> Element {
    let date_display = transfer.date.chars().take(10).collect::<String>();
    let notes_display = transfer
        .notes
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let location_display = if transfer.location.is_empty() {
        "--".to_string()
    } else {
        transfer.location.clone()
    };

    rsx! {
        Card {
            CardHeader {
                CardTitle { "{transfer.transferred_from} → {transfer.transferred_to}" }
            }
            CardContent {
                DetailList {
                    DetailItem { label: "Date", value: date_display }
                    DetailItem { label: "Location", value: location_display }
                    DetailItem { label: "Condition",
                        Badge {
                            variant: condition_badge_variant(&transfer.condition),
                            "{transfer.condition}"
                        }
                    }
                    DetailItem { label: "Notes", value: notes_display }
                }
            }
        }
    }
}

/// Map evidence type to a badge variant.
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

/// Map custody transfer condition to a badge variant.
fn condition_badge_variant(condition: &str) -> BadgeVariant {
    match condition {
        "Excellent" | "Good" => BadgeVariant::Primary,
        "Fair" => BadgeVariant::Secondary,
        "Poor" | "Damaged" => BadgeVariant::Destructive,
        _ => BadgeVariant::Outline,
    }
}
