use dioxus::prelude::*;
use shared_types::{JudicialOpinionResponse, PaginatedResponse, PaginationMeta};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Input, PageActions, PageHeader,
    PageTitle, SearchBar, Skeleton,
};
use shared_ui::{HoverCard, HoverCardContent, HoverCardTrigger};

use super::form_sheet::{FormMode, OpinionFormSheet};
use crate::auth::{can, use_user_role, Action};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn OpinionListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let role = use_user_role();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);
    let mut show_create = use_signal(|| false);

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let p = *page.read();
        async move {
            let search = if q.is_empty() { None } else { Some(q) };
            server::api::list_all_opinions(court, search, Some(p), Some(20))
                .await
                .ok()
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
                PageTitle { "Opinions" }
                PageActions {
                    if can(&role, Action::DraftOpinion) {
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| show_create.set(true),
                            "New Opinion"
                        }
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by title, case name, or author...",
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
                    OpinionTable { opinions: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No opinions found for this court district." }
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

            OpinionFormSheet {
                mode: FormMode::Create,
                initial: None,
                open: show_create(),
                on_close: move |_| show_create.set(false),
                on_saved: move |_| data.restart(),
            }
        }
    }
}

#[component]
fn OpinionTable(opinions: Vec<JudicialOpinionResponse>) -> Element {
    if opinions.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No opinions found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Title" }
                DataTableColumn { "Case" }
                DataTableColumn { "Author" }
                DataTableColumn { "Type" }
                DataTableColumn { "Status" }
            }
            DataTableBody {
                for opinion in opinions {
                    OpinionRow { opinion: opinion }
                }
            }
        }
    }
}

#[component]
fn OpinionRow(opinion: JudicialOpinionResponse) -> Element {
    let id = opinion.id.clone();
    let status_variant = status_badge_variant(&opinion.status);
    let type_variant = type_badge_variant(&opinion.opinion_type);
    let published_display = opinion
        .published_at
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::OpinionDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{opinion.title}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{opinion.title}" }
                                span { class: "hover-card-id", "Case: {opinion.case_name}" }
                                span { class: "hover-card-id", "Author: {opinion.author_judge_name}" }
                                span { class: "hover-card-id", "Published: {published_display}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: type_variant,
                                        "{opinion.opinion_type}"
                                    }
                                    Badge { variant: status_variant,
                                        "{opinion.status}"
                                    }
                                    if opinion.is_precedential {
                                        Badge { variant: BadgeVariant::Primary, "Precedential" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{opinion.case_name}" }
            DataTableCell { "{opinion.author_judge_name}" }
            DataTableCell {
                Badge { variant: type_variant, "{opinion.opinion_type}" }
            }
            DataTableCell {
                Badge { variant: status_variant, "{opinion.status}" }
            }
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

/// Map opinion status to an appropriate badge variant.
fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Draft" => BadgeVariant::Outline,
        "Under Review" | "Circulated" => BadgeVariant::Secondary,
        "Filed" => BadgeVariant::Primary,
        "Published" => BadgeVariant::Primary,
        "Withdrawn" => BadgeVariant::Destructive,
        "Superseded" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// Map opinion type to an appropriate badge variant.
fn type_badge_variant(opinion_type: &str) -> BadgeVariant {
    match opinion_type {
        "Majority" => BadgeVariant::Primary,
        "Concurrence" => BadgeVariant::Secondary,
        "Dissent" => BadgeVariant::Destructive,
        "Per Curiam" => BadgeVariant::Primary,
        "Memorandum" => BadgeVariant::Outline,
        "En Banc" => BadgeVariant::Primary,
        "Summary" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}
