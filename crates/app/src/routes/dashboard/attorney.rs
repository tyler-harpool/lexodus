use dioxus::prelude::*;
use shared_types::{CalendarSearchResponse, DeadlineSearchResponse};
use shared_ui::components::{Card, CardContent, CardHeader, PageHeader, PageTitle, Skeleton};

use crate::CourtContext;

#[component]
pub fn AttorneyDashboard() -> Element {
    let ctx = use_context::<CourtContext>();
    let court = ctx.court_id.read().clone();

    let stats = use_resource(move || {
        let court = court.clone();
        async move {
            let deadlines_result = server::api::search_deadlines(
                court.clone(),
                None,
                None,
                None,
                None,
                Some(0),
                Some(5),
            )
            .await;
            let upcoming_deadlines = deadlines_result
                .ok()
                .and_then(|json| serde_json::from_str::<DeadlineSearchResponse>(&json).ok())
                .map(|r| r.total)
                .unwrap_or(0);

            let calendar_result = server::api::search_calendar_events(
                court.clone(),
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0),
                Some(5),
            )
            .await;
            let upcoming_events = calendar_result
                .ok()
                .and_then(|json| serde_json::from_str::<CalendarSearchResponse>(&json).ok())
                .map(|r| r.total)
                .unwrap_or(0);

            (upcoming_deadlines, upcoming_events)
        }
    });

    rsx! {
        PageHeader {
            PageTitle { "Attorney Dashboard" }
        }

        match &*stats.read() {
            Some((deadlines, events)) => rsx! {
                div { class: "judge-stats-grid",
                    Card {
                        CardHeader { "Upcoming Deadlines" }
                        CardContent {
                            span { class: "stat-value", "{deadlines}" }
                            span { class: "stat-label", "Filing Deadlines" }
                        }
                    }
                    Card {
                        CardHeader { "Court Appearances" }
                        CardContent {
                            span { class: "stat-value", "{events}" }
                            span { class: "stat-label", "Scheduled" }
                        }
                    }
                    Card {
                        CardHeader { "My Cases" }
                        CardContent {
                            span { class: "stat-value", "\u{2014}" }
                            span { class: "stat-label", "Active Representations" }
                        }
                    }
                    Card {
                        CardHeader { "Recent Filings" }
                        CardContent {
                            span { class: "stat-value", "\u{2014}" }
                            span { class: "stat-label", "New Docket Activity" }
                        }
                    }
                }

                div { class: "judge-quick-actions",
                    h3 { "Quick Actions" }
                    div { class: "quick-action-grid",
                        button { class: "quick-action-btn", "File Document" }
                        button { class: "quick-action-btn", "Request Extension" }
                        button { class: "quick-action-btn", "Check Deadlines" }
                        button { class: "quick-action-btn", "View Calendar" }
                    }
                }

                // My filings queue placeholder
                Card {
                    CardHeader { "My Filings" }
                    CardContent {
                        div { class: "clerk-empty-state",
                            p { style: "font-size: var(--font-size-lg); font-weight: 600; margin-bottom: var(--space-xs);",
                                "No pending filings"
                            }
                            p { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted);",
                                "Track your filed documents here."
                            }
                        }
                    }
                }
            },
            None => rsx! {
                div { class: "judge-stats-grid",
                    for _ in 0..4 {
                        Card {
                            CardContent { Skeleton { width: "100%", height: "60px" } }
                        }
                    }
                }
            },
        }
    }
}
