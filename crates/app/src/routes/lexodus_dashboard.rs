use dioxus::prelude::*;
use shared_types::{AttorneyResponse, CalendarSearchResponse, PaginatedResponse};
use shared_ui::components::{Card, CardContent, CardHeader, PageHeader, PageTitle, Skeleton};

use crate::CourtContext;

#[component]
pub fn DashboardPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let attorney_count = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            let result = server::api::list_attorneys(court, Some(1), Some(1)).await;
            match result {
                Ok(json) => serde_json::from_str::<PaginatedResponse<AttorneyResponse>>(&json)
                    .ok()
                    .map(|r| r.meta.total),
                Err(_) => None,
            }
        }
    });

    let calendar_count = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            let result = server::api::search_calendar_events(
                court,
                None, None, None, None, None, None,
                Some(0),
                Some(1),
            )
            .await;
            match result {
                Ok(json) => serde_json::from_str::<CalendarSearchResponse>(&json)
                    .ok()
                    .map(|r| r.total),
                Err(_) => None,
            }
        }
    });

    rsx! {
        div { class: "container",
            PageHeader {
                PageTitle { "Dashboard" }
            }

            div { class: "dashboard-stats",
                StatCard {
                    title: "Attorneys",
                    value: attorney_count.read().clone().flatten(),
                }
                StatCard {
                    title: "Calendar Events",
                    value: calendar_count.read().clone().flatten(),
                }
            }
        }
    }
}

#[component]
fn StatCard(title: &'static str, value: Option<i64>) -> Element {
    rsx! {
        Card {
            CardHeader {
                span { class: "stat-label", "{title}" }
            }
            CardContent {
                match value {
                    Some(count) => rsx! {
                        span { class: "stat-number", "{count}" }
                    },
                    None => rsx! {
                        Skeleton {}
                    },
                }
            }
        }
    }
}
