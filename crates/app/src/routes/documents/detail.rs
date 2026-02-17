use dioxus::prelude::*;
use shared_types::{DocumentEventResponse, DocumentResponse};
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, Card, CardContent, CardHeader, CardTitle, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, DetailGrid, DetailItem,
    DetailList, Input, PageActions, PageHeader, PageTitle, Skeleton, TabContent, TabList,
    TabTrigger, Tabs,
};
use shared_ui::{use_toast, ToastOptions};

use crate::auth::{can, use_user_role, Action};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn DocumentDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let document_id = id.clone();

    let role = use_user_role();
    let toast = use_toast();
    let mut show_strike_confirm = use_signal(|| false);
    let mut striking = use_signal(|| false);

    let mut data = use_resource(move || {
        let court = court_id.clone();
        let did = document_id.clone();
        async move {
            match server::api::get_document_by_id(court, did).await {
                Ok(json) => serde_json::from_str::<DocumentResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(doc)) => {
                    let strike_doc_id = doc.id.clone();
                    let handle_strike = move |_: MouseEvent| {
                        let court = ctx.court_id.read().clone();
                        let did = strike_doc_id.clone();
                        spawn(async move {
                            striking.set(true);
                            match server::api::strike_document_action(court, did).await {
                                Ok(_) => {
                                    toast.success(
                                        "Document stricken from record".to_string(),
                                        ToastOptions::new(),
                                    );
                                    show_strike_confirm.set(false);
                                    data.restart();
                                }
                                Err(e) => {
                                    toast.error(format!("{}", e), ToastOptions::new());
                                    striking.set(false);
                                    show_strike_confirm.set(false);
                                }
                            }
                        });
                    };

                    rsx! {
                    PageHeader {
                        PageTitle { "{doc.title}" }
                        PageActions {
                            Link { to: Route::DocumentList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                            if can(&role, Action::Seal) && !doc.is_stricken {
                                Button {
                                    variant: ButtonVariant::Destructive,
                                    onclick: move |_| show_strike_confirm.set(true),
                                    "Strike"
                                }
                            }
                        }
                    }

                    AlertDialogRoot {
                        open: show_strike_confirm(),
                        on_open_change: move |v| show_strike_confirm.set(v),
                        AlertDialogContent {
                            AlertDialogTitle { "Strike Document" }
                            AlertDialogDescription {
                                "This will permanently mark this document as stricken from the record. This action cannot be undone."
                            }
                            AlertDialogActions {
                                AlertDialogCancel { "Cancel" }
                                AlertDialogAction {
                                    on_click: handle_strike,
                                    if *striking.read() { "Striking..." } else { "Strike" }
                                }
                            }
                        }
                    }

                    if doc.is_stricken {
                        Card {
                            CardContent {
                                div { class: "empty-state",
                                    Badge { variant: BadgeVariant::Destructive, "Stricken" }
                                    p { "This document has been stricken from the record." }
                                }
                            }
                        }
                    }

                    Tabs { default_value: "metadata", horizontal: true,
                        TabList {
                            TabTrigger { value: "metadata", index: 0usize, "Metadata" }
                            TabTrigger { value: "sealing", index: 1usize, "Sealing" }
                            TabTrigger { value: "events", index: 2usize, "Events" }
                        }
                        TabContent { value: "metadata", index: 0usize,
                            MetadataTab { document: doc.clone() }
                        }
                        TabContent { value: "sealing", index: 1usize,
                            SealingTab { document: doc.clone(), on_refresh: move || data.restart() }
                        }
                        TabContent { value: "events", index: 2usize,
                            EventsTab { document_id: doc.id.clone() }
                        }
                    }
                }},
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Document Not Found" }
                                p { "The document you're looking for doesn't exist in this court district." }
                                Link { to: Route::DocumentList {},
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

/// Metadata tab showing document file info and identifiers.
#[component]
fn MetadataTab(document: DocumentResponse) -> Element {
    let doc = &document;
    let filed_date = doc.created_at.chars().take(10).collect::<String>();
    let file_size_display = format_file_size(doc.file_size);
    let source_att = doc
        .source_attachment_id
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let replaced_by = doc
        .replaced_by_document_id
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Document Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Title", value: doc.title.clone() }
                        DetailItem { label: "Type",
                            Badge {
                                variant: doc_type_badge_variant(&doc.document_type),
                                "{doc.document_type}"
                            }
                        }
                        DetailItem { label: "Case ID", value: doc.case_id.clone() }
                        DetailItem { label: "Filed Date", value: filed_date }
                        DetailItem { label: "Uploaded By", value: doc.uploaded_by.clone() }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "File Details" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Content Type", value: doc.content_type.clone() }
                        DetailItem { label: "File Size", value: file_size_display }
                        DetailItem { label: "Checksum", value: doc.checksum.clone() }
                        DetailItem { label: "Storage Key", value: doc.storage_key.clone() }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "References" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Document ID", value: doc.id.clone() }
                        DetailItem { label: "Source Attachment", value: source_att }
                        DetailItem { label: "Replaced By", value: replaced_by }
                        DetailItem { label: "Stricken",
                            Badge {
                                variant: if doc.is_stricken { BadgeVariant::Destructive } else { BadgeVariant::Secondary },
                                if doc.is_stricken { "Yes" } else { "No" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Sealing tab showing seal status and seal/unseal actions.
#[component]
fn SealingTab(document: DocumentResponse, on_refresh: EventHandler<()>) -> Element {
    let ctx = use_context::<CourtContext>();
    let role = use_user_role();
    let toast = use_toast();

    let mut show_seal_confirm = use_signal(|| false);
    let mut show_unseal_confirm = use_signal(|| false);
    let mut sealing = use_signal(|| false);

    // Seal form fields
    let mut seal_level = use_signal(|| "SealedCourtOnly".to_string());
    let mut seal_reason = use_signal(String::new);

    let doc_id = document.id.clone();
    let is_sealed = document.is_sealed;

    let seal_doc_id = doc_id.clone();
    let handle_seal = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let did = seal_doc_id.clone();
        let level = seal_level.read().clone();
        let reason = seal_reason.read().clone();
        let on_refresh = on_refresh.clone();
        spawn(async move {
            sealing.set(true);
            match server::api::seal_document_action(court, did, level, reason, None).await {
                Ok(_) => {
                    toast.success("Document sealed successfully".to_string(), ToastOptions::new());
                    show_seal_confirm.set(false);
                    on_refresh.call(());
                }
                Err(e) => {
                    toast.error(format!("{}", e), ToastOptions::new());
                }
            }
            sealing.set(false);
        });
    };

    let unseal_doc_id = doc_id.clone();
    let handle_unseal = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let did = unseal_doc_id.clone();
        let on_refresh = on_refresh.clone();
        spawn(async move {
            sealing.set(true);
            match server::api::unseal_document_action(court, did).await {
                Ok(_) => {
                    toast.success(
                        "Document unsealed successfully".to_string(),
                        ToastOptions::new(),
                    );
                    show_unseal_confirm.set(false);
                    on_refresh.call(());
                }
                Err(e) => {
                    toast.error(format!("{}", e), ToastOptions::new());
                }
            }
            sealing.set(false);
        });
    };

    let seal_reason_display = document
        .seal_reason_code
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let seal_motion_display = document
        .seal_motion_id
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Sealing Status" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Sealed",
                            Badge {
                                variant: if is_sealed { BadgeVariant::Destructive } else { BadgeVariant::Secondary },
                                if is_sealed { "Yes" } else { "No" }
                            }
                        }
                        DetailItem { label: "Sealing Level",
                            Badge {
                                variant: sealing_level_variant(&document.sealing_level),
                                "{document.sealing_level}"
                            }
                        }
                        DetailItem { label: "Seal Reason", value: seal_reason_display }
                        DetailItem { label: "Seal Motion ID", value: seal_motion_display }
                    }
                }
            }

            if can(&role, Action::Seal) {
                Card {
                    CardHeader { CardTitle { "Actions" } }
                    CardContent {
                        div { class: "sheet-form",
                            if !is_sealed {
                                label { class: "input-label", "Sealing Level" }
                                select {
                                    class: "input",
                                    value: seal_level.read().clone(),
                                    onchange: move |e: FormEvent| seal_level.set(e.value().to_string()),
                                    option { value: "SealedCourtOnly", "Court Only" }
                                    option { value: "SealedCaseParticipants", "Case Participants" }
                                    option { value: "SealedAttorneysOnly", "Attorneys Only" }
                                }
                                Input {
                                    label: "Reason Code",
                                    value: seal_reason.read().clone(),
                                    on_input: move |e: FormEvent| seal_reason.set(e.value().to_string()),
                                    placeholder: "e.g., JuvenileRecord, TradeSecret",
                                }
                                Button {
                                    variant: ButtonVariant::Destructive,
                                    onclick: move |_| show_seal_confirm.set(true),
                                    "Seal Document"
                                }
                            } else {
                                p { class: "text-muted", "This document is currently sealed." }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    onclick: move |_| show_unseal_confirm.set(true),
                                    "Unseal Document"
                                }
                            }
                        }
                    }
                }
            }
        }

        // Seal confirmation dialog
        AlertDialogRoot {
            open: show_seal_confirm(),
            on_open_change: move |v| show_seal_confirm.set(v),
            AlertDialogContent {
                AlertDialogTitle { "Seal Document" }
                AlertDialogDescription {
                    "Are you sure you want to seal this document? It will be restricted from public view."
                }
                AlertDialogActions {
                    AlertDialogCancel { "Cancel" }
                    AlertDialogAction {
                        on_click: handle_seal,
                        if *sealing.read() { "Sealing..." } else { "Seal" }
                    }
                }
            }
        }

        // Unseal confirmation dialog
        AlertDialogRoot {
            open: show_unseal_confirm(),
            on_open_change: move |v| show_unseal_confirm.set(v),
            AlertDialogContent {
                AlertDialogTitle { "Unseal Document" }
                AlertDialogDescription {
                    "Are you sure you want to unseal this document? It will become publicly accessible."
                }
                AlertDialogActions {
                    AlertDialogCancel { "Cancel" }
                    AlertDialogAction {
                        on_click: handle_unseal,
                        if *sealing.read() { "Unsealing..." } else { "Unseal" }
                    }
                }
            }
        }
    }
}

/// Events tab listing the audit trail for this document.
#[component]
fn EventsTab(document_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let events = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let did = document_id.clone();
        async move {
            match server::api::list_document_events_action(court, did).await {
                Ok(json) => serde_json::from_str::<Vec<DocumentEventResponse>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        match &*events.read() {
            Some(Some(list)) if !list.is_empty() => rsx! {
                DataTable {
                    DataTableHeader {
                        DataTableColumn { "Event Type" }
                        DataTableColumn { "Actor" }
                        DataTableColumn { "Date" }
                        DataTableColumn { "Details" }
                    }
                    DataTableBody {
                        for event in list.iter() {
                            EventRow { event: event.clone() }
                        }
                    }
                }
            },
            Some(_) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No events recorded for this document." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// A single event row in the events table.
#[component]
fn EventRow(event: DocumentEventResponse) -> Element {
    let event_date = event.created_at.chars().take(10).collect::<String>();
    let event_variant = event_type_badge_variant(&event.event_type);
    let detail_str = if event.detail.is_null() {
        "--".to_string()
    } else {
        serde_json::to_string(&event.detail).unwrap_or_else(|_| "--".to_string())
    };
    // Truncate long detail strings for the table
    let detail_display = if detail_str.len() > 60 {
        format!("{}...", &detail_str[..60])
    } else {
        detail_str
    };

    rsx! {
        DataTableRow {
            DataTableCell {
                Badge { variant: event_variant, "{event.event_type}" }
            }
            DataTableCell { "{event.actor}" }
            DataTableCell { "{event_date}" }
            DataTableCell { "{detail_display}" }
        }
    }
}

/// Map document type to a badge variant.
fn doc_type_badge_variant(doc_type: &str) -> BadgeVariant {
    match doc_type {
        "Order" | "Judgment" | "Verdict" => BadgeVariant::Primary,
        "Motion" | "Brief" | "Memorandum" => BadgeVariant::Secondary,
        "Indictment" | "Warrant" | "Subpoena" => BadgeVariant::Destructive,
        "Exhibit" | "Transcript" => BadgeVariant::Outline,
        _ => BadgeVariant::Outline,
    }
}

/// Map sealing level to a badge variant.
fn sealing_level_variant(level: &str) -> BadgeVariant {
    match level {
        "Public" => BadgeVariant::Secondary,
        "SealedCourtOnly" => BadgeVariant::Destructive,
        "SealedCaseParticipants" => BadgeVariant::Primary,
        "SealedAttorneysOnly" => BadgeVariant::Outline,
        _ => BadgeVariant::Outline,
    }
}

/// Map event type to a badge variant.
fn event_type_badge_variant(event_type: &str) -> BadgeVariant {
    match event_type {
        "seal" => BadgeVariant::Destructive,
        "unseal" => BadgeVariant::Secondary,
        "strike" => BadgeVariant::Destructive,
        "replace" => BadgeVariant::Primary,
        _ => BadgeVariant::Outline,
    }
}

/// Format a file size in bytes to a human-readable string.
fn format_file_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = 1024 * KB;
    const GB: i64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
