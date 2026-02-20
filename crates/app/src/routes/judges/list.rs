use dioxus::prelude::*;
use shared_types::JudgeResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Input, PageActions, PageHeader,
    PageTitle, SearchBar, Skeleton,
};
use shared_ui::{HoverCard, HoverCardContent, HoverCardTrigger};

use super::form_sheet::{JudgeFormSheet, FormMode};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn JudgeListPage() -> Element {
    let ctx = use_context::<CourtContext>();

    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);
    let mut show_sheet = use_signal(|| false);

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        async move {
            let result = if q.is_empty() {
                server::api::list_judges(court).await
            } else {
                server::api::search_judges(court, q).await
            };

            match result {
                Ok(json) => serde_json::from_str::<Vec<JudgeResponse>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let handle_search = move |_| {
        search_query.set(search_input.read().clone());
    };

    let handle_clear = move |_| {
        search_input.set(String::new());
        search_query.set(String::new());
    };

    rsx! {
        div { class: "container",
            PageHeader {
                PageTitle { "Judges" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_sheet.set(true),
                        "New Judge"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by name, title, or courtroom...",
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
                Some(Some(judges)) if !judges.is_empty() => rsx! {
                    JudgeTable { judges: judges.clone() }
                },
                Some(_) => rsx! {
                    Card {
                        CardContent {
                            p { "No judges found for this court district." }
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

            JudgeFormSheet {
                mode: FormMode::Create,
                initial: None,
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                on_saved: move |_| data.restart(),
            }
        }
    }
}

#[component]
fn JudgeTable(judges: Vec<JudgeResponse>) -> Element {
    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Name" }
                DataTableColumn { "Title" }
                DataTableColumn { "Status" }
                DataTableColumn { "Courtroom" }
                DataTableColumn { "Caseload" }
            }
            DataTableBody {
                for judge in judges.iter() {
                    JudgeRow { judge: judge.clone() }
                }
            }
        }
    }
}

#[component]
fn JudgeRow(judge: JudgeResponse) -> Element {
    let id = judge.id.clone();
    let name = judge.name.clone();
    let title = judge.title.clone();
    let status = judge.status.clone();
    let courtroom = judge.courtroom.clone().unwrap_or_else(|| "—".to_string());
    let caseload_display = format!("{}/{}", judge.current_caseload, judge.max_caseload);

    let specializations = judge.specializations.join(", ");

    let badge_variant = status_badge_variant(&status);
    let nav_id = id.clone();

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::JudgeDetail { id: nav_id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "judge-name-link", "{name}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{name}" }
                                span { class: "hover-card-username", "{title}" }
                                span { class: "hover-card-id", "Courtroom: {courtroom}" }
                                span { class: "hover-card-id", "Caseload: {caseload_display}" }
                                if !specializations.is_empty() {
                                    span { class: "hover-card-id", "Specializations: {specializations}" }
                                }
                                div { class: "hover-card-meta",
                                    Badge { variant: badge_variant, "{status}" }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{title}" }
            DataTableCell {
                Badge { variant: badge_variant, "{status}" }
            }
            DataTableCell { "{courtroom}" }
            DataTableCell { "{caseload_display}" }
        }
    }
}

fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Senior" => BadgeVariant::Secondary,
        "Inactive" => BadgeVariant::Outline,
        "Retired" => BadgeVariant::Outline,
        "Deceased" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}
