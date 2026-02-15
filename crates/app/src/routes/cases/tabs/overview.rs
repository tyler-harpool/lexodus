use dioxus::prelude::*;
use shared_ui::components::{Badge, BadgeVariant, Card, CardContent, CardHeader, Separator, Skeleton};

use crate::CourtContext;

#[component]
pub fn OverviewTab(
    case_id: String,
    title: String,
    case_number: String,
    status: String,
    crime_type: String,
    district: String,
    priority: String,
    description: String,
) -> Element {
    let ctx = use_context::<CourtContext>();

    let case_id_timeline = case_id.clone();

    // Fetch assigned judge
    let judge_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move {
            server::api::list_case_assignments(court, cid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    // Fetch recent timeline activity
    let timeline_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id_timeline.clone();
        async move {
            server::api::get_case_timeline(court, cid, Some(0), Some(5))
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    let status_variant = match status.as_str() {
        "filed" | "arraigned" => BadgeVariant::Primary,
        "dismissed" | "sentenced" => BadgeVariant::Secondary,
        "in_trial" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    };
    let priority_variant = match priority.as_str() {
        "high" | "critical" => BadgeVariant::Destructive,
        "medium" => BadgeVariant::Primary,
        _ => BadgeVariant::Secondary,
    };
    let display_status = status.replace('_', " ");
    let display_type = crime_type.replace('_', " ");

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./overview.css") }
        div { class: "case-overview",
            // Case Information Card
            Card {
                CardHeader { "Case Information" }
                CardContent {
                    div { class: "overview-grid",
                        div { class: "overview-item",
                            span { class: "overview-label", "Case Number" }
                            span { class: "overview-value", "{case_number}" }
                        }
                        div { class: "overview-item",
                            span { class: "overview-label", "Status" }
                            Badge { variant: status_variant, "{display_status}" }
                        }
                        div { class: "overview-item",
                            span { class: "overview-label", "Crime Type" }
                            span { class: "overview-value", "{display_type}" }
                        }
                        div { class: "overview-item",
                            span { class: "overview-label", "District" }
                            span { class: "overview-value", "{district}" }
                        }
                        div { class: "overview-item",
                            span { class: "overview-label", "Priority" }
                            Badge { variant: priority_variant, "{priority}" }
                        }
                    }

                    if !description.is_empty() {
                        Separator {}
                        div { class: "overview-description",
                            h4 { "Description" }
                            p { "{description}" }
                        }
                    }
                }
            }

            // Assigned Judge Card
            Card {
                CardHeader { "Assigned Judge" }
                CardContent {
                    match &*judge_data.read() {
                        Some(Some(assignments)) if !assignments.is_empty() => {
                            let a = &assignments[0];
                            rsx! {
                                div { class: "overview-grid",
                                    div { class: "overview-item",
                                        span { class: "overview-label", "Judge ID" }
                                        span { class: "overview-value",
                                            {a["judge_id"].as_str().unwrap_or("—")}
                                        }
                                    }
                                    div { class: "overview-item",
                                        span { class: "overview-label", "Assignment Type" }
                                        span { class: "overview-value",
                                            {a["assignment_type"].as_str().unwrap_or("primary").replace('_', " ")}
                                        }
                                    }
                                    if let Some(date) = a["assigned_date"].as_str() {
                                        div { class: "overview-item",
                                            span { class: "overview-label", "Assigned Date" }
                                            span { class: "overview-value",
                                                {if date.len() >= 10 { &date[..10] } else { date }}
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Some(_)) => rsx! {
                            p { style: "color: var(--color-on-surface-muted);", "No judge assigned yet." }
                        },
                        Some(None) => rsx! {
                            p { style: "color: var(--color-on-surface-muted);", "Could not load assignment info." }
                        },
                        None => rsx! {
                            Skeleton { style: "width: 100%; height: 60px" }
                        },
                    }
                }
            }

            // Recent Activity
            Card {
                CardHeader { "Recent Activity" }
                CardContent {
                    match &*timeline_data.read() {
                        Some(Some(events)) if !events.is_empty() => rsx! {
                            for evt in events.iter() {
                                div { style: "display: flex; gap: var(--space-md); padding: var(--space-sm) 0; border-bottom: 1px solid var(--color-border);",
                                    div { style: "min-width: 80px; color: var(--color-on-surface-muted); font-size: var(--font-size-sm);",
                                        {evt["timestamp"].as_str().map(|d| if d.len() >= 10 { &d[..10] } else { d }).unwrap_or("—")}
                                    }
                                    div {
                                        span { style: "font-weight: 500;",
                                            {evt["event_type"].as_str().unwrap_or("Event").replace('_', " ")}
                                        }
                                        if let Some(desc) = evt["description"].as_str() {
                                            p { style: "color: var(--color-on-surface-muted); font-size: var(--font-size-sm); margin-top: 2px;",
                                                "{desc}"
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Some(_)) => rsx! {
                            p { style: "color: var(--color-on-surface-muted);", "No recent activity." }
                        },
                        Some(None) => rsx! {
                            p { style: "color: var(--color-on-surface-muted);", "Activity timeline unavailable." }
                        },
                        None => rsx! {
                            Skeleton { style: "width: 100%; height: 100px" }
                        },
                    }
                }
            }
        }
    }
}
