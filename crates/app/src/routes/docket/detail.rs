use dioxus::prelude::*;
use shared_types::{DocketAttachmentResponse, DocketEntryResponse};
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
pub fn DocketDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let entry_id = id.clone();
    let toast = use_toast();

    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let data = use_resource(move || {
        let court = court_id.clone();
        let eid = entry_id.clone();
        async move {
            match server::api::get_docket_entry(court, eid).await {
                Ok(json) => serde_json::from_str::<DocketEntryResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let did = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_docket_entry(court, did).await {
                Ok(()) => {
                    toast.success(
                        "Docket entry deleted successfully".to_string(),
                        ToastOptions::new(),
                    );
                    let nav = navigator();
                    nav.push(Route::DocketList {});
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
                Some(Some(entry)) => rsx! {
                    PageHeader {
                        PageTitle { "Docket Entry #{entry.entry_number}" }
                        PageActions {
                            Link { to: Route::DocketList {},
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
                            AlertDialogTitle { "Delete Docket Entry" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this docket entry? This action cannot be undone."
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
                            TabTrigger { value: "attachments", index: 1usize, "Attachments" }
                            TabTrigger { value: "documents", index: 2usize, "Documents" }
                        }
                        TabContent { value: "metadata", index: 0usize,
                            MetadataTab { entry: entry.clone() }
                        }
                        TabContent { value: "attachments", index: 1usize,
                            AttachmentsTab { entry_id: id.clone() }
                        }
                        TabContent { value: "documents", index: 2usize,
                            DocumentsTab { entry: entry.clone() }
                        }
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Docket Entry Not Found" }
                                p { "The docket entry you're looking for doesn't exist in this court district." }
                                Link { to: Route::DocketList {},
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

/// Metadata tab showing core docket entry fields.
#[component]
fn MetadataTab(entry: DocketEntryResponse) -> Element {
    let date_filed = entry.date_filed.chars().take(10).collect::<String>();
    let date_entered = entry.date_entered.chars().take(10).collect::<String>();
    let filed_by = entry
        .filed_by
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let page_count_display = entry
        .page_count
        .map(|p| p.to_string())
        .unwrap_or_else(|| "--".to_string());
    let related_display = if entry.related_entries.is_empty() {
        "--".to_string()
    } else {
        entry
            .related_entries
            .iter()
            .map(|n| format!("#{}", n))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let service_display = if entry.service_list.is_empty() {
        "--".to_string()
    } else {
        entry.service_list.join(", ")
    };

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Entry Details" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Entry Number", value: format!("#{}", entry.entry_number) }
                        DetailItem { label: "Entry Type",
                            Badge {
                                variant: entry_type_badge_variant(&entry.entry_type),
                                "{format_entry_type(&entry.entry_type)}"
                            }
                        }
                        DetailItem { label: "Description", value: entry.description.clone() }
                        DetailItem { label: "Filed By", value: filed_by }
                        DetailItem { label: "Date Filed", value: date_filed }
                        DetailItem { label: "Date Entered", value: date_entered }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Case & Status" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Case ID", value: entry.case_id.clone() }
                        DetailItem { label: "Sealed",
                            Badge {
                                variant: if entry.is_sealed { BadgeVariant::Destructive } else { BadgeVariant::Secondary },
                                if entry.is_sealed { "Yes" } else { "No" }
                            }
                        }
                        DetailItem { label: "Ex Parte",
                            Badge {
                                variant: if entry.is_ex_parte { BadgeVariant::Destructive } else { BadgeVariant::Secondary },
                                if entry.is_ex_parte { "Yes" } else { "No" }
                            }
                        }
                        DetailItem { label: "Page Count", value: page_count_display }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Related Entries & Service" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Related Entries", value: related_display }
                        DetailItem { label: "Service List", value: service_display }
                    }
                }
            }
        }
    }
}

/// Attachments tab listing file attachments for this docket entry.
#[component]
fn AttachmentsTab(entry_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let attachments = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let eid = entry_id.clone();
        async move {
            match server::api::list_entry_attachments(court, eid).await {
                Ok(json) => serde_json::from_str::<Vec<DocketAttachmentResponse>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        match &*attachments.read() {
            Some(Some(list)) if !list.is_empty() => rsx! {
                div { class: "charges-list",
                    for att in list.iter() {
                        AttachmentCard { attachment: att.clone() }
                    }
                }
            },
            Some(_) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No attachments uploaded for this docket entry." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// Card displaying a single attachment's details.
#[component]
fn AttachmentCard(attachment: DocketAttachmentResponse) -> Element {
    let size_display = format_file_size(attachment.file_size);
    let uploaded_display = attachment
        .uploaded_at
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "Pending".to_string());

    rsx! {
        Card {
            CardHeader {
                CardTitle { "{attachment.filename}" }
            }
            CardContent {
                DetailList {
                    DetailItem { label: "Content Type", value: attachment.content_type.clone() }
                    DetailItem { label: "Size", value: size_display }
                    DetailItem { label: "Uploaded", value: uploaded_display }
                    DetailItem { label: "Sealed",
                        Badge {
                            variant: if attachment.sealed { BadgeVariant::Destructive } else { BadgeVariant::Secondary },
                            if attachment.sealed { "Yes" } else { "No" }
                        }
                    }
                    DetailItem { label: "Encryption", value: attachment.encryption.clone() }
                    if let Some(ref hash) = attachment.sha256 {
                        DetailItem { label: "SHA-256",
                            span { class: "truncate-text", "{hash}" }
                        }
                    }
                }
            }
        }
    }
}

/// Documents tab showing linked document information.
#[component]
fn DocumentsTab(entry: DocketEntryResponse) -> Element {
    match &entry.document_id {
        Some(doc_id) => rsx! {
            Card {
                CardHeader { CardTitle { "Linked Document" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Document ID", value: doc_id.clone() }
                    }
                }
            }
        },
        None => rsx! {
            Card {
                CardContent {
                    p { class: "text-muted", "No documents linked to this docket entry." }
                }
            }
        },
    }
}

/// Map entry type to an appropriate badge variant.
fn entry_type_badge_variant(entry_type: &str) -> BadgeVariant {
    match entry_type {
        "motion" | "response" | "reply" => BadgeVariant::Primary,
        "order" | "minute_order" | "scheduling_order" => BadgeVariant::Secondary,
        "protective_order" | "sealing_order" => BadgeVariant::Destructive,
        "complaint" | "indictment" | "information" | "criminal_complaint" => BadgeVariant::Outline,
        "judgment" | "verdict" | "sentence" => BadgeVariant::Destructive,
        "notice" | "hearing_notice" | "notice_of_appeal" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// Format an entry type slug into a human-readable label.
fn format_entry_type(entry_type: &str) -> String {
    entry_type
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    format!("{}{}", upper, chars.collect::<String>())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format a byte count into a human-readable file size string.
fn format_file_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

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
