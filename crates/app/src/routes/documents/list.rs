use dioxus::prelude::*;
use shared_types::{DocumentResponse, PaginatedResponse, PaginationMeta};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Input, PageHeader, PageTitle,
    SearchBar, Skeleton,
};
use shared_ui::{HoverCard, HoverCardContent, HoverCardTrigger};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn DocumentListPage() -> Element {
    let ctx = use_context::<CourtContext>();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);

    let data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let p = *page.read();
        async move {
            let search = if q.is_empty() { None } else { Some(q) };
            match server::api::list_all_documents(court, search, Some(p), Some(20)).await {
                Ok(json) => {
                    serde_json::from_str::<PaginatedResponse<DocumentResponse>>(&json).ok()
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
                PageTitle { "Documents" }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by document title...",
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
                    DocumentTable { documents: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No documents found for this court district." }
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

#[component]
fn DocumentTable(documents: Vec<DocumentResponse>) -> Element {
    if documents.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No documents found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Title" }
                DataTableColumn { "Type" }
                DataTableColumn { "Case" }
                DataTableColumn { "Sealed" }
                DataTableColumn { "Filed Date" }
            }
            DataTableBody {
                for doc in documents {
                    DocumentRow { document: doc }
                }
            }
        }
    }
}

#[component]
fn DocumentRow(document: DocumentResponse) -> Element {
    let id = document.id.clone();
    let type_variant = doc_type_badge_variant(&document.document_type);
    let sealed_variant = if document.is_sealed {
        BadgeVariant::Destructive
    } else {
        BadgeVariant::Secondary
    };
    let sealed_label = if document.is_sealed { "Sealed" } else { "Public" };
    let filed_date = document
        .created_at
        .chars()
        .take(10)
        .collect::<String>();
    let case_id_short = if document.case_id.len() > 8 {
        format!("{}...", &document.case_id[..8])
    } else {
        document.case_id.clone()
    };
    let file_size_display = format_file_size(document.file_size);

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::DocumentDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{document.title}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{document.title}" }
                                span { class: "hover-card-id", "Type: {document.document_type}" }
                                span { class: "hover-card-id", "Size: {file_size_display}" }
                                span { class: "hover-card-id", "Case: {document.case_id}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: type_variant,
                                        "{document.document_type}"
                                    }
                                    Badge { variant: sealed_variant,
                                        "{sealed_label}"
                                    }
                                    if document.is_stricken {
                                        Badge { variant: BadgeVariant::Destructive,
                                            "Stricken"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell {
                Badge { variant: type_variant, "{document.document_type}" }
            }
            DataTableCell { "{case_id_short}" }
            DataTableCell {
                Badge { variant: sealed_variant, "{sealed_label}" }
            }
            DataTableCell { "{filed_date}" }
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
