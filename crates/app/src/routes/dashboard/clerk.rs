use dioxus::prelude::*;
use shared_types::QueueItemResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardDescription, CardHeader,
    CardTitle, PageHeader, PageTitle, Separator, Skeleton,
};

use crate::routes::Route;
use crate::CourtContext;

/// Priority level to badge variant mapping.
fn priority_badge(priority: i32) -> (BadgeVariant, &'static str) {
    match priority {
        1 => (BadgeVariant::Destructive, "Critical"),
        2 => (BadgeVariant::Outline, "High"),
        3 => (BadgeVariant::Secondary, "Normal"),
        _ => (BadgeVariant::Primary, "Low"),
    }
}

/// Human-readable label for a pipeline step.
fn step_label(step: &str) -> &str {
    match step {
        "review" => "Review",
        "docket" => "Docket",
        "nef" => "NEF",
        "route_judge" => "Route Judge",
        "serve" => "Serve",
        "completed" => "Completed",
        _ => step,
    }
}

/// Clerk dashboard displaying the filing work queue.
#[component]
pub fn ClerkDashboard() -> Element {
    let ctx = use_context::<CourtContext>();
    let court = ctx.court_id.read().clone();

    // Filter signals
    let mut status_filter = use_signal(|| "pending".to_string());
    let mut type_filter = use_signal(|| String::new());
    let mut priority_filter = use_signal(|| None::<i32>);

    let stats = use_resource(move || {
        let court = court.clone();
        async move {
            server::api::get_queue_stats(court, None)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<shared_types::QueueStats>(&json).ok())
        }
    });

    let court_for_items = ctx.court_id.read().clone();
    let items = use_resource(move || {
        let court = court_for_items.clone();
        let status = status_filter.read().clone();
        let qtype = type_filter.read().clone();
        let pri = *priority_filter.read();
        async move {
            let status_opt = if status.is_empty() { None } else { Some(status) };
            let type_opt = if qtype.is_empty() { None } else { Some(qtype) };
            server::api::search_queue(court, status_opt, type_opt, pri, None, None, None, Some(50))
                .await
                .ok()
                .and_then(|json| {
                    serde_json::from_str::<shared_types::QueueSearchResponse>(&json).ok()
                })
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./clerk.css") }
        PageHeader {
            PageTitle { "Clerk Work Queue" }
        }

        // Stats cards
        match &*stats.read() {
            Some(Some(s)) => rsx! {
                div { class: "clerk-stats-grid",
                    StatCard { label: "Pending", value: s.pending_count, variant: BadgeVariant::Destructive }
                    StatCard { label: "My Items", value: s.my_count, variant: BadgeVariant::Primary }
                    StatCard { label: "Today", value: s.today_count, variant: BadgeVariant::Secondary }
                    StatCard { label: "Urgent", value: s.urgent_count, variant: BadgeVariant::Outline }
                }
            },
            _ => rsx! {
                div { class: "clerk-stats-grid",
                    for _ in 0..4 {
                        Card {
                            CardContent {
                                Skeleton { style: "height: 2.5rem; width: 100%;" }
                            }
                        }
                    }
                }
            },
        }

        // Filter bar
        Card {
            CardContent {
                div { class: "clerk-filter-bar",
                    div { class: "clerk-filter-group",
                        label { class: "clerk-filter-label", "Status" }
                        select {
                            class: "input clerk-filter-select",
                            value: "{status_filter}",
                            onchange: move |e| status_filter.set(e.value()),
                            option { value: "", "All" }
                            option { value: "pending", "Pending" }
                            option { value: "in_review", "In Review" }
                            option { value: "processing", "Processing" }
                            option { value: "completed", "Completed" }
                            option { value: "rejected", "Rejected" }
                        }
                    }
                    div { class: "clerk-filter-group",
                        label { class: "clerk-filter-label", "Type" }
                        select {
                            class: "input clerk-filter-select",
                            value: "{type_filter}",
                            onchange: move |e| type_filter.set(e.value()),
                            option { value: "", "All" }
                            option { value: "filing", "Filing" }
                            option { value: "motion", "Motion" }
                            option { value: "order", "Order" }
                            option { value: "deadline_alert", "Deadline Alert" }
                            option { value: "general", "General" }
                        }
                    }
                    div { class: "clerk-filter-group",
                        label { class: "clerk-filter-label", "Priority" }
                        select {
                            class: "input clerk-filter-select",
                            value: match *priority_filter.read() {
                                Some(p) => format!("{p}"),
                                None => String::new(),
                            },
                            onchange: move |e: Event<FormData>| {
                                let val = e.value();
                                if val.is_empty() {
                                    priority_filter.set(None);
                                } else if let Ok(p) = val.parse::<i32>() {
                                    priority_filter.set(Some(p));
                                }
                            },
                            option { value: "", "All" }
                            option { value: "1", "Critical" }
                            option { value: "2", "High" }
                            option { value: "3", "Normal" }
                            option { value: "4", "Low" }
                        }
                    }
                }
            }
        }

        // Queue items list
        match &*items.read() {
            Some(Some(data)) => rsx! {
                Card {
                    CardHeader {
                        CardTitle { "Queue Items" }
                        CardDescription { "{data.total} items" }
                    }
                    CardContent {
                        if data.items.is_empty() {
                            div { class: "clerk-empty-state",
                                p { class: "clerk-empty-title", "No items in queue" }
                                p { class: "clerk-empty-description", "New filings and motions will appear here automatically." }
                            }
                        } else {
                            div { class: "clerk-queue-list",
                                for item in data.items.iter() {
                                    QueueItemRow { item: item.clone() }
                                }
                            }
                        }
                    }
                }
            },
            Some(None) => rsx! {
                Card {
                    CardContent {
                        p { class: "clerk-empty-title", "Failed to load queue items." }
                    }
                }
            },
            None => rsx! {
                Card {
                    CardContent {
                        for _ in 0..5 {
                            Skeleton { style: "height: 3rem; width: 100%; margin-bottom: 0.5rem;" }
                        }
                    }
                }
            },
        }
    }
}

/// A single stat card.
#[component]
fn StatCard(label: String, value: i64, variant: BadgeVariant) -> Element {
    rsx! {
        Card {
            CardContent {
                div { class: "clerk-stat-card",
                    span { class: "clerk-stat-value", "{value}" }
                    Badge { variant: variant, "{label}" }
                }
            }
        }
    }
}

/// A single queue item row in the list.
#[component]
fn QueueItemRow(item: QueueItemResponse) -> Element {
    let nav = use_navigator();
    let (badge_variant, badge_label) = priority_badge(item.priority);
    let step = step_label(&item.current_step);

    let case_id = item.case_id.clone();
    let _queue_id = item.id.clone();

    rsx! {
        div { class: "clerk-queue-item",
            div { class: "clerk-queue-item-main",
                div { class: "clerk-queue-item-header",
                    span { class: "clerk-queue-item-title", "{item.title}" }
                    Badge { variant: badge_variant, "{badge_label}" }
                    Badge { variant: BadgeVariant::Secondary, "{item.queue_type}" }
                }
                div { class: "clerk-queue-item-meta",
                    span { class: "clerk-queue-item-step", "Step: {step}" }
                    if let Some(ref cn) = item.case_number {
                        Separator {}
                        span { class: "clerk-queue-item-case", "Case: {cn}" }
                    }
                    Separator {}
                    span { class: "clerk-queue-item-status", "{item.status}" }
                }
            }
            div { class: "clerk-queue-item-actions",
                if item.status == "pending" {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| {
                            if let Some(ref cid) = case_id {
                                nav.push(Route::CaseDetail { id: cid.clone() });
                            }
                        },
                        "Claim"
                    }
                } else if item.status == "in_review" || item.status == "processing" {
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            if let Some(ref cid) = case_id {
                                nav.push(Route::CaseDetail { id: cid.clone() });
                            }
                        },
                        "Continue"
                    }
                }
            }
        }
    }
}
