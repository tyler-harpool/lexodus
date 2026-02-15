use dioxus::prelude::*;
use shared_ui::components::{
    Card, CardContent, CardHeader, PageHeader, PageTitle, Skeleton,
};

use crate::CourtContext;

#[component]
pub fn JudgeDashboard() -> Element {
    let ctx = use_context::<CourtContext>();
    let court = ctx.court_id.read().clone();

    let stats = use_resource(move || {
        let court = court.clone();
        async move {
            let cases_result = server::api::search_cases(
                court.clone(),
                None,
                None,
                None,
                None,
                Some(0),
                Some(1),
            )
            .await;
            let active_cases = cases_result
                .ok()
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                .and_then(|v| v["pagination"]["total"].as_i64())
                .unwrap_or(0);

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
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                .and_then(|v| v["pagination"]["total"].as_i64())
                .unwrap_or(0);

            (active_cases, upcoming_deadlines)
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./judge.css") }
        PageHeader {
            PageTitle { "Judicial Dashboard" }
        }

        match &*stats.read() {
            Some((active_cases, upcoming_deadlines)) => rsx! {
                div { class: "judge-stats-grid",
                    Card {
                        CardHeader { "My Caseload" }
                        CardContent {
                            span { class: "stat-value", "{active_cases}" }
                            span { class: "stat-label", "Active Cases" }
                        }
                    }
                    Card {
                        CardHeader { "Upcoming Deadlines" }
                        CardContent {
                            span { class: "stat-value", "{upcoming_deadlines}" }
                            span { class: "stat-label", "Due This Week" }
                        }
                    }
                    Card {
                        CardHeader { "Pending Motions" }
                        CardContent {
                            span { class: "stat-value", "\u{2014}" }
                            span { class: "stat-label", "Awaiting Ruling" }
                        }
                    }
                    Card {
                        CardHeader { "Opinion Drafts" }
                        CardContent {
                            span { class: "stat-value", "\u{2014}" }
                            span { class: "stat-label", "In Progress" }
                        }
                    }
                }

                div { class: "judge-quick-actions",
                    h3 { "Quick Actions" }
                    div { class: "quick-action-grid",
                        button { class: "quick-action-btn", "Draft Opinion" }
                        button { class: "quick-action-btn", "Issue Order" }
                        button { class: "quick-action-btn", "Review Motion" }
                        button { class: "quick-action-btn", "View Calendar" }
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
