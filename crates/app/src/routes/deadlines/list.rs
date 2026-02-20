use dioxus::prelude::*;
use shared_types::DeadlineResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, FormSelect,
    PageActions, PageHeader, PageTitle, Pagination, SearchBar, Skeleton,
};
use shared_ui::{HoverCard, HoverCardContent, HoverCardTrigger};

use super::form_sheet::{DeadlineFormSheet, FormMode};
use crate::auth::{can, use_user_role, Action};
use crate::auth::use_auth;
use crate::components::scope_toggle::{resolve_scope_filter, ListScope, ScopeToggle};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn DeadlineListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let role = use_user_role();

    let mut offset = use_signal(|| 0i64);
    let mut search_status = use_signal(String::new);
    let limit: i64 = 20;
    let mut show_sheet = use_signal(|| false);

    // Scope toggle: My Items vs All Court (only rendered for Judge/Attorney)
    let scope = use_signal(|| ListScope::MyItems);
    let auth = use_auth();

    let resource_role = role.clone();
    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let st = search_status.read().clone();
        let off = *offset.read();
        let user = auth.current_user.read();
        let linked_attorney = user.as_ref().and_then(|u| u.linked_attorney_id.clone());
        let linked_judge = user.as_ref().and_then(|u| u.linked_judge_id.clone());
        let role_clone = resource_role.clone();
        let scope_filter = resolve_scope_filter(*scope.read(), &role_clone, &linked_attorney, &linked_judge);
        async move {
            // If in "My Items" mode with an attorney_id, use the dedicated endpoint
            if let Some(attorney_id) = scope_filter.attorney_id {
                let status_filter = if st.is_empty() { None } else { Some(st) };
                let deadlines = server::api::list_deadlines_for_attorney(
                    court,
                    attorney_id,
                    status_filter,
                    None, // date_to
                )
                .await
                .unwrap_or_default();

                return Some(shared_types::DeadlineSearchResponse {
                    total: deadlines.len() as i64,
                    deadlines,
                });
            }

            // Default: search all court deadlines
            server::api::search_deadlines(
                court,
                if st.is_empty() { None } else { Some(st) },
                None, // case_id
                None, // date_from
                None, // date_to
                Some(off),
                Some(limit),
            )
            .await
            .ok()
        }
    });

    let handle_clear = move |_| {
        search_status.set(String::new());
        offset.set(0);
    };

    rsx! {
        div { class: "container",
            PageHeader {
                PageTitle { "Deadlines" }
                PageActions {
                    if can(&role, Action::CreateCase) {
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| show_sheet.set(true),
                            "New Deadline"
                        }
                    }
                }
            }

            // My Items / All Court toggle (only for Judge and Attorney roles)
            ScopeToggle { scope: scope }

            SearchBar {
                FormSelect {
                    value: "{search_status}",
                    onchange: move |evt: Event<FormData>| {
                        search_status.set(evt.value().to_string());
                        offset.set(0);
                    },
                    option { value: "", "All Statuses" }
                    option { value: "open", "Open" }
                    option { value: "met", "Met" }
                    option { value: "extended", "Extended" }
                    option { value: "cancelled", "Cancelled" }
                    option { value: "expired", "Expired" }
                }
                if !search_status.read().is_empty() {
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: handle_clear,
                        "Clear Filters"
                    }
                }
            }

            match &*data.read() {
                Some(Some(resp)) => rsx! {
                    DeadlineTable { deadlines: resp.deadlines.clone() }
                    Pagination {
                        total: resp.total,
                        offset: offset,
                        limit: limit,
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No deadlines found for this court district." }
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

            DeadlineFormSheet {
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
fn DeadlineTable(deadlines: Vec<DeadlineResponse>) -> Element {
    if deadlines.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No deadlines found." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Title" }
                DataTableColumn { "Due Date" }
                DataTableColumn { "Rule" }
                DataTableColumn { "Status" }
            }
            DataTableBody {
                for dl in deadlines {
                    DeadlineRow { deadline: dl }
                }
            }
        }
    }
}

#[component]
fn DeadlineRow(deadline: DeadlineResponse) -> Element {
    let id = deadline.id.clone();
    let badge_variant = status_badge_variant(&deadline.status);
    let display_date = format_due_date(&deadline.due_at);
    let rule_display = deadline.rule_code.clone().unwrap_or_default();
    let notes_preview = deadline
        .notes
        .as_deref()
        .unwrap_or("No notes");
    let notes_display = if notes_preview.len() > 100 {
        format!("{}...", &notes_preview[..100])
    } else {
        notes_preview.to_string()
    };

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::DeadlineDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { "{deadline.title}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{deadline.title}" }
                                span { class: "hover-card-username", "Due: {display_date}" }
                                if !rule_display.is_empty() {
                                    span { class: "hover-card-id", "Rule: {rule_display}" }
                                }
                                span { class: "hover-card-id", "{notes_display}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: badge_variant, "{deadline.status}" }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{display_date}" }
            DataTableCell { "{rule_display}" }
            DataTableCell {
                Badge { variant: badge_variant, "{deadline.status}" }
            }
        }
    }
}

fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "open" => BadgeVariant::Primary,
        "met" => BadgeVariant::Secondary,
        "extended" => BadgeVariant::Outline,
        "cancelled" | "expired" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}

fn format_due_date(date_str: &str) -> String {
    crate::format_helpers::format_date_human(date_str)
}
