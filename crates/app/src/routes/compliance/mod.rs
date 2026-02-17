use dioxus::prelude::*;
use shared_types::{CalendarSearchResponse, CaseSearchResponse, DeadlineSearchResponse, RuleResponse};
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, PageHeader, PageTitle, Separator,
    Skeleton,
};

use crate::CourtContext;

/// Number of skeleton cards shown during initial load per section.
const SKELETON_CARD_COUNT: usize = 4;

// ── Main component ──────────────────────────────────────────────────

/// Compliance dashboard that aggregates metrics from cases, deadlines,
/// calendar events, and rules for the currently selected court.
#[component]
pub fn ComplianceDashboardPage() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./compliance.css") }

        PageHeader {
            PageTitle { "Compliance Dashboard" }
        }

        div { class: "compliance-page",
            CaseStatusSection {}
            Separator {}
            DeadlineComplianceSection {}
            Separator {}
            CalendarSection {}
            Separator {}
            RulesSummarySection {}
        }
    }
}

// ── Section 1: Case Status Overview ─────────────────────────────────

#[component]
fn CaseStatusSection() -> Element {
    let ctx = use_context::<CourtContext>();

    let data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            // Fetch all cases with a high limit to get the total count.
            // We use limit=1 and read the `total` field for each query.
            let total = fetch_case_total(&court, None).await;
            let awaiting_sentencing =
                fetch_case_total(&court, Some("awaiting_sentencing")).await;
            let on_appeal = fetch_case_total(&court, Some("on_appeal")).await;
            let dismissed = fetch_case_total(&court, Some("dismissed")).await;
            let sentenced = fetch_case_total(&court, Some("sentenced")).await;

            let active = total - dismissed - sentenced;

            CaseMetrics {
                total,
                active,
                awaiting_sentencing,
                on_appeal,
            }
        }
    });

    rsx! {
        h3 { class: "compliance-section-title", "Case Status Overview" }

        match &*data.read() {
            Some(metrics) => rsx! {
                div { class: "compliance-stats-grid",
                    MetricCard { value: metrics.total, label: "Total Cases" }
                    MetricCard {
                        value: metrics.active,
                        label: "Active Cases",
                        badge_label: "active",
                        badge_variant: BadgeVariant::Primary,
                    }
                    MetricCard {
                        value: metrics.awaiting_sentencing,
                        label: "Awaiting Sentencing",
                        badge_label: "pending",
                        badge_variant: BadgeVariant::Secondary,
                    }
                    MetricCard {
                        value: metrics.on_appeal,
                        label: "On Appeal",
                        badge_label: "appeal",
                        badge_variant: BadgeVariant::Outline,
                    }
                }
            },
            None => rsx! { SkeletonGrid {} },
        }
    }
}

// ── Section 2: Deadline Compliance ──────────────────────────────────

#[component]
fn DeadlineComplianceSection() -> Element {
    let ctx = use_context::<CourtContext>();

    let data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            let total = fetch_deadline_total(&court, None).await;
            let open = fetch_deadline_total(&court, Some("open")).await;
            let expired = fetch_deadline_total(&court, Some("expired")).await;
            let met = fetch_deadline_total(&court, Some("met")).await;
            let extended = fetch_deadline_total(&court, Some("extended")).await;

            DeadlineMetrics {
                total,
                open,
                expired,
                met,
                extended,
            }
        }
    });

    rsx! {
        h3 { class: "compliance-section-title", "Deadline Compliance" }

        match &*data.read() {
            Some(metrics) => rsx! {
                div { class: "compliance-stats-grid",
                    MetricCard { value: metrics.total, label: "Total Deadlines" }
                    MetricCard {
                        value: metrics.expired,
                        label: "Expired",
                        badge_label: "overdue",
                        badge_variant: BadgeVariant::Destructive,
                    }
                    MetricCard {
                        value: metrics.open,
                        label: "Open",
                        badge_label: "pending",
                        badge_variant: BadgeVariant::Secondary,
                    }
                    MetricCard {
                        value: metrics.met,
                        label: "Met",
                        badge_label: "complete",
                        badge_variant: BadgeVariant::Primary,
                    }
                    MetricCard {
                        value: metrics.extended,
                        label: "Extended",
                        badge_label: "extended",
                        badge_variant: BadgeVariant::Outline,
                    }
                }

                if metrics.total > 0 {
                    ComplianceRate {
                        met: metrics.met,
                        total: metrics.total,
                    }
                }
            },
            None => rsx! { SkeletonGrid {} },
        }
    }
}

// ── Section 3: Calendar Overview ────────────────────────────────────

#[component]
fn CalendarSection() -> Element {
    let ctx = use_context::<CourtContext>();

    let data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            let total = fetch_calendar_total(&court, None).await;
            let scheduled = fetch_calendar_total(&court, Some("scheduled")).await;
            let completed = fetch_calendar_total(&court, Some("completed")).await;
            let cancelled = fetch_calendar_total(&court, Some("cancelled")).await;

            CalendarMetrics {
                total,
                scheduled,
                completed,
                cancelled,
            }
        }
    });

    rsx! {
        h3 { class: "compliance-section-title", "Calendar Overview" }

        match &*data.read() {
            Some(metrics) => rsx! {
                div { class: "compliance-stats-grid",
                    MetricCard { value: metrics.total, label: "Total Events" }
                    MetricCard {
                        value: metrics.scheduled,
                        label: "Scheduled",
                        badge_label: "upcoming",
                        badge_variant: BadgeVariant::Primary,
                    }
                    MetricCard {
                        value: metrics.completed,
                        label: "Completed",
                        badge_label: "done",
                        badge_variant: BadgeVariant::Secondary,
                    }
                    MetricCard {
                        value: metrics.cancelled,
                        label: "Cancelled",
                        badge_label: "cancelled",
                        badge_variant: BadgeVariant::Destructive,
                    }
                }
            },
            None => rsx! { SkeletonGrid {} },
        }
    }
}

// ── Section 4: Rules Summary ────────────────────────────────────────

#[component]
fn RulesSummarySection() -> Element {
    let ctx = use_context::<CourtContext>();

    let data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            let result = server::api::list_rules(court).await;
            match result {
                Ok(json) => {
                    let rules: Vec<RuleResponse> =
                        serde_json::from_str(&json).unwrap_or_default();
                    let total = rules.len() as i64;
                    let active = rules.iter().filter(|r| r.status == "active").count() as i64;

                    // Aggregate rule counts by source.
                    let mut source_counts: Vec<(String, i64)> = Vec::new();
                    for rule in &rules {
                        if let Some(entry) = source_counts.iter_mut().find(|(s, _)| *s == rule.source) {
                            entry.1 += 1;
                        } else {
                            source_counts.push((rule.source.clone(), 1));
                        }
                    }
                    source_counts.sort_by(|a, b| b.1.cmp(&a.1));

                    Some(RulesMetrics {
                        total,
                        active,
                        source_counts,
                    })
                }
                Err(_) => None,
            }
        }
    });

    rsx! {
        h3 { class: "compliance-section-title", "Rules Summary" }

        match &*data.read() {
            Some(Some(metrics)) => rsx! {
                div { class: "compliance-stats-grid compliance-stats-grid-narrow",
                    MetricCard { value: metrics.total, label: "Total Rules" }
                    MetricCard {
                        value: metrics.active,
                        label: "Active Rules",
                        badge_label: "active",
                        badge_variant: BadgeVariant::Primary,
                    }
                }

                if !metrics.source_counts.is_empty() {
                    Card {
                        CardHeader {
                            CardTitle { "Rules by Source" }
                        }
                        CardContent {
                            div { class: "source-distribution",
                                for (source, count) in metrics.source_counts.iter() {
                                    div { class: "source-row",
                                        span { class: "source-name", "{source}" }
                                        Badge { variant: BadgeVariant::Secondary, "{count}" }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Some(None) => rsx! {
                Card {
                    CardContent {
                        p { class: "compliance-empty-text", "Unable to load rules data." }
                    }
                }
            },
            None => rsx! { SkeletonGrid {} },
        }
    }
}

// ── Reusable sub-components ─────────────────────────────────────────

/// A single metric card displaying a large number, a label, and an optional badge.
#[component]
fn MetricCard(
    value: i64,
    label: String,
    #[props(default)] badge_label: Option<String>,
    #[props(default)] badge_variant: Option<BadgeVariant>,
) -> Element {
    rsx! {
        Card {
            CardContent {
                div { class: "metric-card",
                    h2 { class: "metric-value", "{value}" }
                    p { class: "metric-label", "{label}" }
                    if let Some(bl) = &badge_label {
                        Badge {
                            variant: badge_variant.unwrap_or(BadgeVariant::Secondary),
                            "{bl}"
                        }
                    }
                }
            }
        }
    }
}

/// Displays a compliance rate percentage based on met vs total deadlines.
#[component]
fn ComplianceRate(met: i64, total: i64) -> Element {
    let rate = if total > 0 {
        (met as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let variant = if rate >= 80.0 {
        BadgeVariant::Primary
    } else if rate >= 50.0 {
        BadgeVariant::Secondary
    } else {
        BadgeVariant::Destructive
    };

    rsx! {
        Card {
            CardContent {
                div { class: "compliance-rate",
                    span { class: "compliance-rate-value", "{rate:.1}%" }
                    span { class: "compliance-rate-label", "Deadline Compliance Rate" }
                    Badge { variant: variant,
                        if rate >= 80.0 { "Good" } else if rate >= 50.0 { "Fair" } else { "Needs Attention" }
                    }
                }
            }
        }
    }
}

/// Skeleton loading grid matching the 4-column metric layout.
#[component]
fn SkeletonGrid() -> Element {
    rsx! {
        div { class: "compliance-stats-grid",
            for _ in 0..SKELETON_CARD_COUNT {
                Card {
                    CardContent {
                        Skeleton { width: "100%", height: "80px" }
                    }
                }
            }
        }
    }
}

// ── Data types ──────────────────────────────────────────────────────

struct CaseMetrics {
    total: i64,
    active: i64,
    awaiting_sentencing: i64,
    on_appeal: i64,
}

struct DeadlineMetrics {
    total: i64,
    open: i64,
    expired: i64,
    met: i64,
    extended: i64,
}

struct CalendarMetrics {
    total: i64,
    scheduled: i64,
    completed: i64,
    cancelled: i64,
}

struct RulesMetrics {
    total: i64,
    active: i64,
    source_counts: Vec<(String, i64)>,
}

// ── Data-fetching helpers ───────────────────────────────────────────

/// Fetch the total count of cases matching an optional status filter.
/// Uses limit=1 to minimise data transfer; only the `total` field matters.
async fn fetch_case_total(court: &str, status: Option<&str>) -> i64 {
    let result = server::api::search_cases(
        court.to_string(),
        status.map(|s| s.to_string()),
        None,
        None,
        None,
        Some(0),
        Some(1),
    )
    .await;

    result
        .ok()
        .and_then(|json| serde_json::from_str::<CaseSearchResponse>(&json).ok())
        .map(|r| r.total)
        .unwrap_or(0)
}

/// Fetch the total count of deadlines matching an optional status filter.
async fn fetch_deadline_total(court: &str, status: Option<&str>) -> i64 {
    let result = server::api::search_deadlines(
        court.to_string(),
        status.map(|s| s.to_string()),
        None,
        None,
        None,
        Some(0),
        Some(1),
    )
    .await;

    result
        .ok()
        .and_then(|json| serde_json::from_str::<DeadlineSearchResponse>(&json).ok())
        .map(|r| r.total)
        .unwrap_or(0)
}

/// Fetch the total count of calendar events matching an optional status filter.
async fn fetch_calendar_total(court: &str, status: Option<&str>) -> i64 {
    let result = server::api::search_calendar_events(
        court.to_string(),
        None,
        None,
        None,
        status.map(|s| s.to_string()),
        None,
        None,
        Some(0),
        Some(1),
    )
    .await;

    result
        .ok()
        .and_then(|json| serde_json::from_str::<CalendarSearchResponse>(&json).ok())
        .map(|r| r.total)
        .unwrap_or(0)
}
