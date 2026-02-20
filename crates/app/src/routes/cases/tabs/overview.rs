use dioxus::prelude::*;
use shared_types::CaseResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DetailFooter, DetailGrid,
    DetailItem, DetailList, Skeleton, Tooltip, TooltipContent, TooltipTrigger,
};

use crate::CourtContext;

#[component]
pub fn OverviewTab(case_item: CaseResponse) -> Element {
    let ctx = use_context::<CourtContext>();

    let case_id = case_item.id.clone();
    let case_id_timeline = case_item.id.clone();

    // Fetch assigned judge
    let judge_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move {
            server::api::list_case_assignments(court, cid)
                .await
                .ok()
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
        }
    });

    let status_variant = status_badge_variant(&case_item.status);
    let priority_variant = priority_badge_variant(&case_item.priority);
    let display_status = case_item.status.replace('_', " ");
    let display_type = case_item.crime_type.replace('_', " ");
    let display_opened = format_date(&case_item.opened_at);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./overview.css") }

        // Case Details + Timing & Assignment grid
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Case Details" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Case Number", value: case_item.case_number.clone() }
                        DetailItem { label: "Crime Type", value: display_type }
                        DetailItem { label: "District Code", value: case_item.district_code.clone() }
                        if !case_item.location.is_empty() {
                            DetailItem { label: "Location", value: case_item.location.clone() }
                        }
                        DetailItem { label: "Status",
                            Badge { variant: status_variant, "{display_status}" }
                        }
                        DetailItem { label: "Priority",
                            Badge { variant: priority_variant, "{case_item.priority}" }
                        }
                        if case_item.is_sealed {
                            DetailItem { label: "Sealed",
                                Badge { variant: BadgeVariant::Destructive, "SEALED" }
                            }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Timing & Assignment" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Opened", value: display_opened }
                        match &*judge_data.read() {
                            Some(Some(assignments)) if !assignments.is_empty() => {
                                let a = &assignments[0];
                                let judge_display = a.judge_name.as_deref()
                                    .unwrap_or(&a.judge_id);
                                let assigned_date_short = a.assigned_date.get(..10)
                                    .unwrap_or(&a.assigned_date);
                                rsx! {
                                    DetailItem { label: "Assigned Judge",
                                        Tooltip {
                                            TooltipTrigger { "{judge_display}" }
                                            TooltipContent {
                                                "{a.judge_id}"
                                            }
                                        }
                                    }
                                    DetailItem { label: "Assigned Date",
                                        {assigned_date_short.to_string()}
                                    }
                                }
                            },
                            Some(Some(_)) => rsx! {},
                            Some(None) => rsx! {},
                            None => rsx! {
                                DetailItem { label: "Assigned Judge",
                                    Skeleton { style: "width: 120px; height: 20px" }
                                }
                            },
                        }
                        if let Some(ref closed) = case_item.closed_at {
                            DetailItem { label: "Closed", value: format_date(closed) }
                        }
                        DetailItem { label: "Updated", value: format_date(&case_item.updated_at) }
                    }
                }
            }

            if !case_item.description.is_empty() {
                Card {
                    CardHeader { CardTitle { "Description" } }
                    CardContent {
                        p { "{case_item.description}" }
                    }
                }
            }
        }

        DetailFooter {
            span { "ID: {case_item.id}" }
        }

        // Recent Activity
        Card {
            CardHeader { "Recent Activity" }
            CardContent {
                match &*timeline_data.read() {
                    Some(Some(resp)) if !resp.entries.is_empty() => {
                        let entries = &resp.entries;
                        rsx! {
                            for evt in entries.iter() {
                                div { style: "display: flex; gap: var(--space-md); padding: var(--space-sm) 0; border-bottom: 1px solid var(--color-border);",
                                    div { style: "min-width: 80px; color: var(--color-on-surface-muted); font-size: var(--font-size-sm);",
                                        {format_date(&evt.timestamp)}
                                    }
                                    div {
                                        span { style: "font-weight: 500;",
                                            {evt.entry_type.replace('_', " ")}
                                        }
                                        p { style: "color: var(--color-on-surface-muted); font-size: var(--font-size-sm); margin-top: 2px;",
                                            "{evt.summary}"
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

pub fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "filed" => BadgeVariant::Primary,
        "arraigned" | "discovery" | "pretrial_motions" | "plea_negotiations" => BadgeVariant::Secondary,
        "trial_ready" | "in_trial" => BadgeVariant::Outline,
        "awaiting_sentencing" | "sentenced" => BadgeVariant::Secondary,
        "dismissed" | "on_appeal" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}

pub fn priority_badge_variant(priority: &str) -> BadgeVariant {
    match priority {
        "low" => BadgeVariant::Secondary,
        "medium" => BadgeVariant::Outline,
        "high" => BadgeVariant::Primary,
        "critical" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}

pub fn format_date(date_str: &str) -> String {
    crate::format_helpers::format_date_human(date_str)
}
