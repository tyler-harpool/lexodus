use dioxus::prelude::*;
use shared_types::{CaseResponse, CaseSearchResponse};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, FormSelect, Input, PageActions,
    PageHeader, PageTitle, Pagination, SearchBar, Skeleton,
};
use shared_ui::{HoverCard, HoverCardContent, HoverCardTrigger};

use super::form_sheet::{CaseFormSheet, FormMode};
use crate::auth::{can, use_user_role, Action};
use crate::auth::use_auth;
use crate::components::scope_toggle::{resolve_scope_filter, ListScope, ScopeToggle};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn CaseListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let role = use_user_role();

    let mut offset = use_signal(|| 0i64);
    let mut case_type = use_signal(|| "criminal".to_string());
    let mut filter_status = use_signal(String::new);
    let mut filter_crime_type = use_signal(String::new);
    let mut search_query = use_signal(String::new);
    let limit: i64 = 20;

    // Scope toggle: My Items vs All Court (only rendered for Judge/Attorney)
    let scope = use_signal(|| ListScope::MyItems);
    let auth = use_auth();

    // Sheet state for creating cases
    let mut show_sheet = use_signal(|| false);

    let resource_role = role.clone();
    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let ct = case_type.read().clone();
        let st = filter_status.read().clone();
        let crime = filter_crime_type.read().clone();
        let q = search_query.read().clone();
        let off = *offset.read();
        let user = auth.current_user.read();
        let linked_attorney = user.as_ref().and_then(|u| u.linked_attorney_id.clone());
        let linked_judge = user.as_ref().and_then(|u| u.linked_judge_id.clone());
        let role_clone = resource_role.clone();
        let scope_filter = resolve_scope_filter(*scope.read(), &role_clone, &linked_attorney, &linked_judge);
        async move {
            // If in "My Items" mode with an attorney_id, use the dedicated endpoint
            if let Some(attorney_id) = scope_filter.attorney_id {
                let cases = server::api::list_cases_for_attorney(
                    court.clone(),
                    attorney_id,
                )
                .await
                .unwrap_or_default();

                return Some(CaseSearchResponse {
                    total: cases.len() as i64,
                    cases,
                });
            }

            // For judge scope, pass assigned_judge_id filter to the search endpoints
            let judge_filter = scope_filter.judge_id.clone();

            if ct == "civil" {
                let result = server::api::search_civil_cases(
                    court,
                    if st.is_empty() { None } else { Some(st) },
                    None, // nature_of_suit
                    if crime.is_empty() { None } else { Some(crime) },
                    None, // class_action
                    judge_filter, // assigned_judge_id
                    if q.is_empty() { None } else { Some(q) },
                    Some(off),
                    Some(limit),
                )
                .await;

                match result {
                    Ok(civil_resp) => {
                        // Map civil cases into the unified CaseSearchResponse format
                        // so the same table component can render both types
                        let cases = civil_resp
                            .cases
                            .into_iter()
                            .map(|c| CaseResponse {
                                id: c.id,
                                case_number: c.case_number,
                                title: c.title,
                                description: c.description,
                                case_type: "civil".to_string(),
                                crime_type: c.nature_of_suit,
                                status: c.status,
                                priority: c.priority,
                                assigned_judge_id: c.assigned_judge_id,
                                district_code: c.district_code,
                                location: c.location,
                                is_sealed: c.is_sealed,
                                sealed_by: c.sealed_by,
                                sealed_date: c.sealed_date,
                                seal_reason: c.seal_reason,
                                opened_at: c.opened_at,
                                updated_at: c.updated_at,
                                closed_at: c.closed_at,
                                jurisdiction_basis: Some(c.jurisdiction_basis),
                                jury_demand: Some(c.jury_demand),
                                class_action: Some(c.class_action),
                                amount_in_controversy: c.amount_in_controversy,
                                consent_to_magistrate: Some(c.consent_to_magistrate),
                                pro_se: Some(c.pro_se),
                            })
                            .collect();
                        Some(CaseSearchResponse {
                            cases,
                            total: civil_resp.total,
                        })
                    }
                    Err(_) => None,
                }
            } else {
                // Criminal case search (original behavior)
                let result = server::api::search_cases(
                    court,
                    if st.is_empty() { None } else { Some(st) },
                    if crime.is_empty() { None } else { Some(crime) },
                    None, // priority
                    if q.is_empty() { None } else { Some(q) },
                    Some(off),
                    Some(limit),
                )
                .await;

                result.ok()
            }
        }
    });

    // Clearing filters resets the type-specific filter and search, but preserves case_type
    let handle_clear = move |_| {
        filter_status.set(String::new());
        filter_crime_type.set(String::new());
        search_query.set(String::new());
        offset.set(0);
    };

    let has_filters = !filter_status.read().is_empty()
        || !filter_crime_type.read().is_empty()
        || !search_query.read().is_empty();

    let is_civil = *case_type.read() == "civil";

    rsx! {
        div { class: "container",
            PageHeader {
                PageTitle { "Cases" }
                PageActions {
                    if can(&role, Action::CreateCase) {
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| show_sheet.set(true),
                            if is_civil { "New Civil Case" } else { "New Case" }
                        }
                    }
                }
            }

            // My Items / All Court toggle (only for Judge and Attorney roles)
            ScopeToggle { scope: scope }

            // Case type toggle (criminal / civil)
            div { class: "case-type-toggle",
                Button {
                    variant: if !is_civil { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                    onclick: move |_| {
                        case_type.set("criminal".to_string());
                        filter_status.set(String::new());
                        filter_crime_type.set(String::new());
                        offset.set(0);
                    },
                    "Criminal"
                }
                Button {
                    variant: if is_civil { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                    onclick: move |_| {
                        case_type.set("civil".to_string());
                        filter_status.set(String::new());
                        filter_crime_type.set(String::new());
                        offset.set(0);
                    },
                    "Civil"
                }
            }

            SearchBar {
                Input {
                    value: search_query.read().clone(),
                    placeholder: "Search by title or case number...",
                    label: "",
                    on_input: move |evt: FormEvent| {
                        search_query.set(evt.value().to_string());
                        offset.set(0);
                    },
                }

                // Status filter: different options for criminal vs civil
                if is_civil {
                    FormSelect {
                        value: "{filter_status}",
                        onchange: move |evt: Event<FormData>| {
                            filter_status.set(evt.value().to_string());
                            offset.set(0);
                        },
                        option { value: "", "All Statuses" }
                        option { value: "filed", "Filed" }
                        option { value: "pending", "Pending" }
                        option { value: "discovery", "Discovery" }
                        option { value: "pretrial", "Pretrial" }
                        option { value: "trial_ready", "Trial Ready" }
                        option { value: "in_trial", "In Trial" }
                        option { value: "settled", "Settled" }
                        option { value: "judgment_entered", "Judgment Entered" }
                        option { value: "on_appeal", "On Appeal" }
                        option { value: "closed", "Closed" }
                        option { value: "dismissed", "Dismissed" }
                        option { value: "transferred", "Transferred" }
                    }
                } else {
                    FormSelect {
                        value: "{filter_status}",
                        onchange: move |evt: Event<FormData>| {
                            filter_status.set(evt.value().to_string());
                            offset.set(0);
                        },
                        option { value: "", "All Statuses" }
                        option { value: "filed", "Filed" }
                        option { value: "arraigned", "Arraigned" }
                        option { value: "discovery", "Discovery" }
                        option { value: "pretrial_motions", "Pretrial Motions" }
                        option { value: "plea_negotiations", "Plea Negotiations" }
                        option { value: "trial_ready", "Trial Ready" }
                        option { value: "in_trial", "In Trial" }
                        option { value: "awaiting_sentencing", "Awaiting Sentencing" }
                        option { value: "sentenced", "Sentenced" }
                        option { value: "dismissed", "Dismissed" }
                        option { value: "on_appeal", "On Appeal" }
                    }
                }

                // Type-specific secondary filter: crime type for criminal, jurisdiction for civil
                if is_civil {
                    FormSelect {
                        value: "{filter_crime_type}",
                        onchange: move |evt: Event<FormData>| {
                            filter_crime_type.set(evt.value().to_string());
                            offset.set(0);
                        },
                        option { value: "", "All Jurisdictions" }
                        option { value: "federal_question", "Federal Question" }
                        option { value: "diversity", "Diversity" }
                        option { value: "us_government_plaintiff", "US Gov. Plaintiff" }
                        option { value: "us_government_defendant", "US Gov. Defendant" }
                    }
                } else {
                    FormSelect {
                        value: "{filter_crime_type}",
                        onchange: move |evt: Event<FormData>| {
                            filter_crime_type.set(evt.value().to_string());
                            offset.set(0);
                        },
                        option { value: "", "All Crime Types" }
                        option { value: "fraud", "Fraud" }
                        option { value: "drug_offense", "Drug Offense" }
                        option { value: "racketeering", "Racketeering" }
                        option { value: "cybercrime", "Cybercrime" }
                        option { value: "tax_offense", "Tax Offense" }
                        option { value: "money_laundering", "Money Laundering" }
                        option { value: "immigration", "Immigration" }
                        option { value: "firearms", "Firearms" }
                        option { value: "other", "Other" }
                    }
                }

                if has_filters {
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: handle_clear,
                        "Clear Filters"
                    }
                }
            }

            match &*data.read() {
                Some(Some(resp)) => rsx! {
                    CaseTable { cases: resp.cases.clone(), is_civil: is_civil }
                    Pagination {
                        total: resp.total,
                        offset: offset,
                        limit: limit,
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No cases found for this court district." }
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

            CaseFormSheet {
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
fn CaseTable(cases: Vec<CaseResponse>, is_civil: bool) -> Element {
    if cases.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No cases found." }
                }
            }
        };
    }

    // The "Type" column label changes based on case type
    let type_label = if is_civil { "Nature of Suit" } else { "Crime Type" };

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Case Number" }
                DataTableColumn { "Title" }
                DataTableColumn { "{type_label}" }
                DataTableColumn { "Status" }
                DataTableColumn { "Priority" }
                DataTableColumn { "Opened" }
            }
            DataTableBody {
                for c in cases {
                    CaseRow { case_item: c }
                }
            }
        }
    }
}

#[component]
fn CaseRow(case_item: CaseResponse) -> Element {
    let id = case_item.id.clone();
    let status_variant = status_badge_variant(&case_item.status);
    let priority_variant = priority_badge_variant(&case_item.priority);
    let display_date = format_date(&case_item.opened_at);
    let display_type = format_crime_type(&case_item.crime_type);
    let display_status = format_status(&case_item.status);
    let desc_preview = if case_item.description.len() > 80 {
        format!("{}...", &case_item.description[..80])
    } else if case_item.description.is_empty() {
        "No description".to_string()
    } else {
        case_item.description.clone()
    };

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::CaseDetail { id: id.clone(), tab: None });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "case-number-link", "{case_item.case_number}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{case_item.title}" }
                                span { class: "hover-card-username", "{case_item.case_number}" }
                                span { class: "hover-card-id", "{desc_preview}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: status_variant, "{display_status}" }
                                    Badge { variant: priority_variant, "{case_item.priority}" }
                                    if case_item.is_sealed {
                                        Badge { variant: BadgeVariant::Destructive, "SEALED" }
                                    }
                                }
                                if !case_item.location.is_empty() {
                                    span { class: "hover-card-id", "Location: {case_item.location}" }
                                }
                                span { class: "hover-card-id", "District: {case_item.district_code}" }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{case_item.title}" }
            DataTableCell { "{display_type}" }
            DataTableCell {
                Badge { variant: status_variant, "{display_status}" }
            }
            DataTableCell {
                Badge { variant: priority_variant, "{case_item.priority}" }
            }
            DataTableCell { "{display_date}" }
        }
    }
}

fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        // Criminal statuses
        "filed" => BadgeVariant::Primary,
        "arraigned" | "discovery" | "pretrial_motions" | "plea_negotiations" => {
            BadgeVariant::Secondary
        }
        "trial_ready" | "in_trial" => BadgeVariant::Outline,
        "awaiting_sentencing" | "sentenced" => BadgeVariant::Secondary,
        "dismissed" | "on_appeal" => BadgeVariant::Destructive,
        // Civil statuses
        "pending" | "pretrial" => BadgeVariant::Secondary,
        "settled" | "judgment_entered" | "closed" => BadgeVariant::Outline,
        "transferred" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}

fn priority_badge_variant(priority: &str) -> BadgeVariant {
    match priority {
        "low" => BadgeVariant::Secondary,
        "medium" => BadgeVariant::Outline,
        "high" => BadgeVariant::Primary,
        "critical" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}

fn format_date(date_str: &str) -> String {
    if date_str.len() >= 10 {
        date_str[..10].to_string()
    } else {
        date_str.to_string()
    }
}

fn format_crime_type(ct: &str) -> String {
    ct.replace('_', " ")
}

fn format_status(s: &str) -> String {
    s.replace('_', " ")
}
