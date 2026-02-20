use dioxus::prelude::*;
use shared_types::{CalendarEntryResponse, CaseResponse, DeadlineResponse, UserRole};
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, PageHeader, PageTitle, Skeleton,
};

use crate::auth::{use_auth, use_user_role};
use crate::routes::Route;
use crate::CourtContext;

/// Number of days to look ahead for deadlines.
const DEADLINE_LOOKAHEAD_DAYS: i64 = 14;

/// Calculate the number of days remaining until a deadline.
/// Returns None if the date string cannot be parsed.
fn days_remaining(due_at: &str) -> Option<i64> {
    let now = chrono::Utc::now();
    let due = chrono::DateTime::parse_from_rfc3339(due_at).ok()?;
    let diff = due.signed_duration_since(now);
    Some(diff.num_days())
}

/// Map days remaining to a badge variant for urgency color-coding.
fn urgency_variant(days: i64) -> BadgeVariant {
    if days < 3 {
        BadgeVariant::Destructive
    } else if days < 7 {
        BadgeVariant::Outline
    } else {
        BadgeVariant::Secondary
    }
}

/// Map days remaining to a human-readable label.
fn urgency_label(days: i64) -> String {
    if days < 0 {
        format!("{} days overdue", days.abs())
    } else if days == 0 {
        "Due today".to_string()
    } else if days == 1 {
        "Due tomorrow".to_string()
    } else {
        format!("{days} days left")
    }
}

/// Map case status to a badge variant.
fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "filed" | "pending" => BadgeVariant::Primary,
        "arraigned" | "discovery" | "pretrial_motions" | "plea_negotiations" | "pretrial" => {
            BadgeVariant::Secondary
        }
        "trial_ready" | "in_trial" => BadgeVariant::Outline,
        "dismissed" | "on_appeal" | "transferred" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}

/// Map case priority to a badge variant.
fn priority_badge_variant(priority: &str) -> BadgeVariant {
    match priority {
        "low" => BadgeVariant::Secondary,
        "medium" => BadgeVariant::Outline,
        "high" => BadgeVariant::Primary,
        "critical" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}

/// Attorney dashboard showing filing deadlines, upcoming appearances, and active cases.
///
/// Requires the logged-in user to have a linked attorney record (linked_attorney_id).
/// If not linked, displays a prompt to contact the administrator.
#[component]
pub fn AttorneyDashboard() -> Element {
    let ctx = use_context::<CourtContext>();
    let auth = use_auth();
    let role = use_user_role();

    // Only attorneys see this dashboard
    if role != UserRole::Attorney {
        return rsx! {
            PageHeader { PageTitle { "Dashboard" } }
            Card {
                CardContent {
                    p { "This dashboard is for attorney accounts." }
                }
            }
        };
    }

    let attorney_id = auth
        .current_user
        .read()
        .as_ref()
        .and_then(|u| u.linked_attorney_id.clone());

    if attorney_id.is_none() {
        return rsx! {
            document::Link { rel: "stylesheet", href: asset!("./attorney.css") }
            PageHeader { PageTitle { "Attorney Dashboard" } }
            Card {
                CardContent {
                    div { class: "attorney-not-linked",
                        p { class: "attorney-not-linked-title",
                            "Account Not Linked"
                        }
                        p { class: "attorney-not-linked-description",
                            "Your account is not linked to an attorney record. Contact your administrator."
                        }
                    }
                }
            }
        };
    }

    let attorney_id = attorney_id.unwrap();
    let court = ctx.court_id.read().clone();

    // Compute the deadline cutoff date (today + DEADLINE_LOOKAHEAD_DAYS)
    let cutoff = chrono::Utc::now()
        + chrono::Duration::days(DEADLINE_LOOKAHEAD_DAYS);
    let cutoff_rfc3339 = cutoff.to_rfc3339();

    // Fetch the "today" timestamp for calendar events (only future appearances)
    let today_rfc3339 = chrono::Utc::now().to_rfc3339();

    // Fetch all three data sources in parallel via a single resource
    let data = use_resource(move || {
        let court = court.clone();
        let atty_id = attorney_id.clone();
        let cutoff = cutoff_rfc3339.clone();
        let today = today_rfc3339.clone();
        async move {
            let deadlines = server::api::list_deadlines_for_attorney(
                court.clone(),
                atty_id.clone(),
                Some("open".to_string()),
                Some(cutoff),
            )
            .await
            .unwrap_or_default();

            let events = server::api::list_calendar_events_for_attorney(
                court.clone(),
                atty_id.clone(),
                Some(today),
            )
            .await
            .unwrap_or_default();

            let cases = server::api::list_cases_for_attorney(
                court.clone(),
                atty_id.clone(),
            )
            .await
            .unwrap_or_default();

            (deadlines, events, cases)
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./attorney.css") }
        PageHeader { PageTitle { "Attorney Dashboard" } }

        match &*data.read() {
            Some((deadlines, events, cases)) => rsx! {
                div { class: "attorney-dashboard-grid",
                    // Section 1: Filing Deadlines (Next 14 Days)
                    DeadlinesSection { deadlines: deadlines.clone() }

                    // Section 2: Upcoming Appearances
                    AppearancesSection { events: events.clone() }

                    // Section 3: My Cases
                    CasesSection { cases: cases.clone() }
                }
            },
            None => rsx! {
                div { class: "attorney-dashboard-grid",
                    Card {
                        CardContent { Skeleton { style: "height: 200px; width: 100%;" } }
                    }
                    Card {
                        CardContent { Skeleton { style: "height: 200px; width: 100%;" } }
                    }
                    Card {
                        CardContent { Skeleton { style: "height: 200px; width: 100%;" } }
                    }
                }
            },
        }
    }
}

/// Filing deadlines section showing the next 14 days of upcoming deadlines.
#[component]
fn DeadlinesSection(deadlines: Vec<DeadlineResponse>) -> Element {
    let nav = use_navigator();

    rsx! {
        Card {
            CardHeader {
                CardTitle { "Filing Deadlines (Next 14 Days)" }
            }
            CardContent {
                if deadlines.is_empty() {
                    div { class: "attorney-empty-state",
                        p { class: "attorney-empty-title", "No upcoming deadlines" }
                    }
                } else {
                    div { class: "attorney-deadline-list",
                        for dl in deadlines.iter() {
                            {
                                let id = dl.id.clone();
                                let days = days_remaining(&dl.due_at).unwrap_or(0);
                                let variant = urgency_variant(days);
                                let label = urgency_label(days);
                                let display_date = crate::format_helpers::format_date_human(&dl.due_at);
                                let case_id_display = dl.case_id.as_deref().unwrap_or("N/A");

                                rsx! {
                                    div {
                                        class: "attorney-deadline-item",
                                        onclick: move |_| {
                                            nav.push(Route::DeadlineDetail { id: id.clone() });
                                        },
                                        div { class: "attorney-deadline-item-main",
                                            span { class: "attorney-deadline-item-title", "{dl.title}" }
                                            div { class: "attorney-deadline-item-meta",
                                                span { "{display_date}" }
                                                span { class: "attorney-deadline-item-dot", "\u{00B7}" }
                                                span { "Case: {case_id_display}" }
                                            }
                                        }
                                        div { class: "attorney-days-badge",
                                            Badge { variant: variant, "{label}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Upcoming court appearances section.
#[component]
fn AppearancesSection(events: Vec<CalendarEntryResponse>) -> Element {
    let nav = use_navigator();

    rsx! {
        Card {
            CardHeader {
                CardTitle { "Upcoming Appearances" }
            }
            CardContent {
                if events.is_empty() {
                    div { class: "attorney-empty-state",
                        p { class: "attorney-empty-title", "No appearances scheduled" }
                    }
                } else {
                    div { class: "attorney-appearance-list",
                        for event in events.iter() {
                            {
                                let id = event.case_id.clone();
                                let display_date = crate::format_helpers::format_datetime_human(&event.scheduled_date);
                                let display_type = crate::format_helpers::format_snake_case_title(&event.event_type);
                                let case_number = event.case_number.as_deref().unwrap_or("N/A");

                                rsx! {
                                    div {
                                        class: "attorney-appearance-item",
                                        onclick: move |_| {
                                            nav.push(Route::CaseDetail { id: id.clone(), tab: Some("scheduling".to_string()) });
                                        },
                                        div { class: "attorney-appearance-item-main",
                                            span { class: "attorney-appearance-item-type", "{display_type}" }
                                            div { class: "attorney-appearance-item-meta",
                                                span { "{display_date}" }
                                                span { class: "attorney-deadline-item-dot", "\u{00B7}" }
                                                span { "Case: {case_number}" }
                                                span { class: "attorney-deadline-item-dot", "\u{00B7}" }
                                                span { "Room: {event.courtroom}" }
                                            }
                                        }
                                        Badge { variant: BadgeVariant::Primary, "{event.status}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Active cases section.
#[component]
fn CasesSection(cases: Vec<CaseResponse>) -> Element {
    let nav = use_navigator();

    rsx! {
        Card {
            CardHeader {
                CardTitle { "My Cases" }
            }
            CardContent {
                if cases.is_empty() {
                    div { class: "attorney-empty-state",
                        p { class: "attorney-empty-title", "No active cases" }
                    }
                } else {
                    div { class: "attorney-case-list",
                        for case_item in cases.iter() {
                            {
                                let id = case_item.id.clone();
                                let status_variant = status_badge_variant(&case_item.status);
                                let priority_variant = priority_badge_variant(&case_item.priority);
                                let display_status = case_item.status.replace('_', " ");

                                rsx! {
                                    div {
                                        class: "attorney-case-item",
                                        onclick: move |_| {
                                            nav.push(Route::CaseDetail { id: id.clone(), tab: None });
                                        },
                                        div { class: "attorney-case-item-main",
                                            span { class: "attorney-case-item-number", "{case_item.case_number}" }
                                            div { class: "attorney-case-item-title", "{case_item.title}" }
                                        }
                                        div { class: "attorney-case-item-badges",
                                            Badge { variant: status_variant, "{display_status}" }
                                            Badge { variant: priority_variant, "{case_item.priority}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
