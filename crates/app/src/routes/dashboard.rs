use crate::tier_gate::TierGate;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdLock, LdLockOpen};
use dioxus_free_icons::Icon;
use server::api::{get_dashboard_stats, get_premium_analytics};
use shared_types::UserTier;
use shared_ui::{
    AspectRatio, Avatar, AvatarFallback, Badge, BadgeVariant, Button, ButtonVariant, Card,
    CardContent, CardDescription, CardHeader, CardTitle, ContentSide, HoverCard, HoverCardContent,
    HoverCardTrigger, Progress, ProgressIndicator, Separator, Skeleton, Tooltip, TooltipContent,
    TooltipTrigger,
};

/// Maximum value for progress bar display.
const PROGRESS_MAX: f64 = 100.0;

/// Number of skeleton placeholders shown while data is loading.
const SKELETON_COUNT: usize = 4;

/// Calculate a percentage from a numerator and denominator, capped at PROGRESS_MAX.
fn calc_percentage(numerator: i64, denominator: i64) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    let pct = (numerator as f64 / denominator as f64) * PROGRESS_MAX;
    pct.min(PROGRESS_MAX)
}

/// Extract the first two characters of a display name, uppercased, for avatar initials.
fn initials_from_name(name: &str) -> String {
    name.chars().take(2).collect::<String>().to_uppercase()
}

/// Dashboard page displaying stats, progress bars, and recent user activity.
#[component]
pub fn Dashboard() -> Element {
    let mut stats_resource = use_server_future(get_dashboard_stats)?;

    let stats_result = stats_resource();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./dashboard.css") }

        div {
            class: "dashboard-page",

            h2 {
                class: "dashboard-title",
                "Dashboard"
            }

            match stats_result {
                None => rsx! { LoadingSkeletons {} },

                Some(Err(err)) => rsx! {
                    Card {
                        CardHeader {
                            CardTitle { "Error" }
                            CardDescription { "Failed to load dashboard data." }
                        }
                        CardContent {
                            p {
                                class: "dashboard-error-text",
                                "{err}"
                            }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| { stats_resource.restart(); },
                                "Retry"
                            }
                        }
                    }
                },

                Some(Ok(stats)) => rsx! {
                    StatsGrid { stats: stats.clone() }
                    ProgressSection { stats: stats.clone() }

                    // Pro tier: analytics section
                    TierGate {
                        required: UserTier::Pro,
                        fallback: rsx! { UpgradePrompt { tier_name: "Pro", feature: "Analytics" } },
                        AnalyticsSection {}
                    }

                    RecentActivity { stats: stats.clone() }

                    // Enterprise tier: admin panel
                    TierGate {
                        required: UserTier::Enterprise,
                        fallback: rsx! { LockedSection { tier_name: "Enterprise", feature: "Admin Panel" } },
                        AdminPanel { total_users: stats.total_users }
                    }
                },
            }
        }
    }
}

/// Grid of skeleton placeholders shown during initial data load.
#[component]
fn LoadingSkeletons() -> Element {
    rsx! {
        div {
            class: "skeleton-grid",
            for _ in 0..SKELETON_COUNT {
                Card {
                    CardHeader {
                        Skeleton { style: "height: 1rem; width: 60%;" }
                    }
                    CardContent {
                        Skeleton { style: "height: 2rem; width: 40%;" }
                    }
                }
            }
        }
    }
}

/// Row of four stat cards displayed in a responsive CSS grid.
#[component]
fn StatsGrid(stats: shared_types::DashboardStats) -> Element {
    let growth_rate = calc_percentage(stats.active_products, stats.total_products);

    rsx! {
        div {
            class: "stats-grid",

            StatCard {
                title: "Total Users",
                value: "{stats.total_users}",
                tooltip_text: "The total number of registered user accounts.",
                badge_label: "live",
            }

            StatCard {
                title: "Total Products",
                value: "{stats.total_products}",
                tooltip_text: "Total products across all categories and statuses.",
            }

            StatCard {
                title: "Active Products",
                value: "{stats.active_products}",
                tooltip_text: "Products currently marked as active in the catalog.",
            }

            StatCard {
                title: "Growth Rate",
                value: "{growth_rate:.1}%",
                tooltip_text: "Percentage of products that are currently active.",
            }
        }
    }
}

/// A single stat card with an optional badge and a tooltip on an info icon.
#[component]
fn StatCard(
    title: String,
    value: String,
    tooltip_text: String,
    #[props(default)] badge_label: Option<String>,
) -> Element {
    rsx! {
        Card {
            CardHeader {
                div {
                    class: "stat-header-row",
                    CardTitle { "{title}" }
                    div {
                        class: "stat-actions",
                        if let Some(label) = &badge_label {
                            Badge { variant: BadgeVariant::Primary, "{label}" }
                        }
                        Tooltip {
                            TooltipTrigger {
                                span {
                                    class: "stat-info-icon",
                                    "?"
                                }
                            }
                            TooltipContent { side: ContentSide::Top, "{tooltip_text}" }
                        }
                    }
                }
            }
            CardContent {
                span {
                    class: "stat-value",
                    "{value}"
                }
            }
        }
    }
}

/// Section with two progress bars: inventory target and active products ratio.
#[component]
fn ProgressSection(stats: shared_types::DashboardStats) -> Element {
    let inventory_pct = calc_percentage(stats.active_products, stats.total_products);
    let active_ratio = calc_percentage(stats.active_products, stats.total_products);

    rsx! {
        Card {
            CardHeader {
                CardTitle { "Progress Overview" }
                CardDescription { "Key inventory and product metrics at a glance." }
            }
            CardContent {
                div {
                    class: "progress-stack",

                    div {
                        class: "progress-row",
                        div {
                            class: "progress-label-row",
                            span {
                                class: "progress-label",
                                "Inventory Target"
                            }
                            span {
                                class: "progress-value",
                                "{stats.active_products} / {stats.total_products}"
                            }
                        }
                        Progress {
                            value: Some(inventory_pct),
                            ProgressIndicator {}
                        }
                    }

                    Separator {}

                    div {
                        class: "progress-row",
                        div {
                            class: "progress-label-row",
                            span {
                                class: "progress-label",
                                "Active Products Ratio"
                            }
                            span {
                                class: "progress-value",
                                "{active_ratio:.1}%"
                            }
                        }
                        Progress {
                            value: Some(active_ratio),
                            ProgressIndicator {}
                        }
                    }
                }
            }
        }
    }
}

/// Card listing recent user activity in a scrollable area with hover cards.
#[component]
fn RecentActivity(stats: shared_types::DashboardStats) -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "Recent Activity" }
                CardDescription { "Newly registered users." }
            }
            CardContent {
                div {
                    for (idx, user) in stats.recent_users.iter().enumerate() {
                        if idx > 0 {
                            Separator {}
                        }
                        UserRow { user: user.clone() }
                    }
                    if stats.recent_users.is_empty() {
                        p {
                            class: "empty-text",
                            "No recent users."
                        }
                    }
                }
            }
        }
    }
}

/// A single user row with avatar, hover card, and user details.
#[component]
fn UserRow(user: shared_types::User) -> Element {
    let fallback_initials = initials_from_name(&user.display_name);

    rsx! {
        div {
            class: "user-row",

            Avatar {
                AvatarFallback { "{fallback_initials}" }
            }

            HoverCard {
                HoverCardTrigger {
                    span {
                        class: "user-name-link",
                        "{user.display_name}"
                    }
                }
                HoverCardContent {
                    div {
                        class: "hover-card-body",

                        div {
                            class: "hover-card-avatar-wrap",
                            AspectRatio {
                                ratio: 1.0,
                                div {
                                    class: "hover-card-avatar-placeholder",
                                    "{fallback_initials}"
                                }
                            }
                        }

                        div {
                            class: "hover-card-details",
                            span {
                                class: "hover-card-name",
                                "{user.display_name}"
                            }
                            span {
                                class: "hover-card-username",
                                "@{user.username}"
                            }
                            span {
                                class: "hover-card-id",
                                "ID: {user.id}"
                            }
                        }
                    }
                }
            }

            div { class: "user-row-spacer" }

            span {
                class: "hide-mobile user-row-username",
                "@{user.username}"
            }
        }
    }
}

/// Premium analytics section — fetches tier-gated data from the server.
#[component]
fn AnalyticsSection() -> Element {
    let analytics = use_server_future(get_premium_analytics)?;
    let result = analytics();

    rsx! {
        Card {
            CardHeader {
                div {
                    class: "stat-header-row",
                    CardTitle { "Analytics" }
                    Badge { variant: BadgeVariant::Secondary, "PRO" }
                }
                CardDescription { "Revenue and category breakdown for Pro users." }
            }
            CardContent {
                match result {
                    None => rsx! {
                        div { class: "analytics-loading",
                            Skeleton { style: "height: 1.5rem; width: 50%;" }
                            Skeleton { style: "height: 1rem; width: 70%; margin-top: 0.5rem;" }
                        }
                    },
                    Some(Err(err)) => rsx! {
                        p { class: "dashboard-error-text", "{err}" }
                    },
                    Some(Ok(data)) => rsx! {
                        div { class: "analytics-grid",
                            div { class: "analytics-metric",
                                span { class: "analytics-metric-label", "Total Revenue" }
                                span { class: "analytics-metric-value", "${data.total_revenue:.2}" }
                            }
                            div { class: "analytics-metric",
                                span { class: "analytics-metric-label", "Avg Price" }
                                span { class: "analytics-metric-value", "${data.avg_product_price:.2}" }
                            }
                            div { class: "analytics-metric",
                                span { class: "analytics-metric-label", "New Users (30d)" }
                                span { class: "analytics-metric-value", "{data.users_last_30_days}" }
                            }
                        }
                        if !data.products_by_category.is_empty() {
                            Separator {}
                            div { class: "analytics-categories",
                                span { class: "analytics-categories-title", "Products by Category" }
                                for cat in data.products_by_category.iter() {
                                    div { class: "analytics-category-row",
                                        span { class: "analytics-category-name", "{cat.category}" }
                                        Badge { variant: BadgeVariant::Primary, "{cat.count}" }
                                    }
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}

/// Elite admin panel — quick user count and navigation link.
#[component]
fn AdminPanel(total_users: i64) -> Element {
    use crate::routes::Route;

    rsx! {
        Card {
            CardHeader {
                div {
                    class: "stat-header-row",
                    CardTitle { "Admin Panel" }
                    Badge { variant: BadgeVariant::Destructive, "ENTERPRISE" }
                }
                CardDescription { "System administration tools for Enterprise users." }
            }
            CardContent {
                div { class: "admin-panel-content",
                    div { class: "admin-stat-row",
                        span { class: "admin-stat-label", "Registered Users" }
                        span { class: "stat-value", "{total_users}" }
                    }
                    Separator {}
                    div { class: "admin-actions",
                        Link { to: Route::Users {},
                            Button { variant: ButtonVariant::Primary, "Manage Users" }
                        }
                        Link { to: Route::Products {},
                            Button { variant: ButtonVariant::Secondary, "Manage Products" }
                        }
                    }
                }
            }
        }
    }
}

/// Upgrade prompt shown to users below the required tier.
#[component]
fn UpgradePrompt(tier_name: String, feature: String) -> Element {
    rsx! {
        div { class: "tier-gate-card tier-gate-upgrade",
            div { class: "tier-gate-icon", Icon::<LdLockOpen> { icon: LdLockOpen, width: 24, height: 24 } }
            h3 { class: "tier-gate-title", "Unlock {feature}" }
            p { class: "tier-gate-description",
                "Upgrade to {tier_name} to access {feature} and more."
            }
        }
    }
}

/// Locked section shown to users below the required tier.
#[component]
fn LockedSection(tier_name: String, feature: String) -> Element {
    rsx! {
        div { class: "tier-gate-card tier-gate-locked",
            div { class: "tier-gate-icon", Icon::<LdLock> { icon: LdLock, width: 24, height: 24 } }
            h3 { class: "tier-gate-title", "{feature}" }
            p { class: "tier-gate-description",
                "This feature requires {tier_name} tier access."
            }
        }
    }
}
