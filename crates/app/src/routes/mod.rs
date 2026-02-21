pub mod activate;
pub mod attorneys;
pub mod calendar;
pub mod cases;
pub mod command_palette;
pub mod compliance;
pub mod dashboard;
pub mod deadlines;
pub mod device_auth;
pub mod district_picker;
pub mod forgot_password;
pub mod judges;
pub mod login;
pub mod not_found;
pub mod opinions;
pub mod privacy;
pub mod register;
pub mod reset_password;
pub mod rules;
pub mod search;
pub mod settings;
pub mod terms;
pub mod users;

use crate::auth::{use_auth, use_sidebar_visibility, use_user_role};
use shared_types::UserRole;
use crate::{CourtContext, ProfileState};
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdBell, LdBookOpen, LdBriefcase, LdCalendar, LdClock, LdLayoutDashboard, LdSearch,
    LdSettings, LdShield, LdUsers,
};
use dioxus_free_icons::Icon;
use shared_types::{FeatureFlags, UserTier};
use shared_ui::{
    Avatar, AvatarFallback, AvatarImage, Badge, BadgeVariant, DropdownMenu, DropdownMenuContent,
    DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger, Navbar, Separator, Sidebar,
    SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupContent, SidebarGroupLabel,
    SidebarHeader, SidebarInset, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider,
    SidebarRail, SidebarSeparator, SidebarTrigger, Switch, SwitchThumb,
};

use activate::Activate;
use dashboard::Dashboard;
use device_auth::DeviceAuth;
use forgot_password::ForgotPassword;
use login::Login;
use not_found::NotFound;
use privacy::Privacy;
use register::Register;
use reset_password::ResetPassword;
use settings::Settings;
use terms::Terms;
use users::Users;

/// Application routes.
#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/login?:redirect")]
    Login { redirect: Option<String> },
    #[route("/register")]
    Register {},
    #[route("/forgot-password")]
    ForgotPassword {},
    #[route("/reset-password?:token")]
    ResetPassword { token: Option<String> },
    #[route("/privacy")]
    Privacy {},
    #[route("/terms")]
    Terms {},
    #[route("/activate?:code")]
    Activate { code: Option<String> },
    #[route("/device-auth")]
    DeviceAuth {},
    #[layout(AuthGuard)]
    #[layout(AppLayout)]
    #[route("/")]
    Dashboard {},
    #[route("/users")]
    Users {},
    #[route("/settings/?:billing&:verified")]
    Settings {
        billing: Option<String>,
        verified: Option<String>,
    },
    // Court domain routes
    #[route("/attorneys")]
    AttorneyList {},
    #[route("/attorneys/:id")]
    AttorneyDetail { id: String },
    #[route("/calendar")]
    CalendarList {},
    #[route("/calendar/:id")]
    CalendarDetail { id: String },
    #[route("/cases")]
    CaseList {},
    #[route("/cases/:id?:tab")]
    CaseDetail { id: String, tab: Option<String> },
    #[route("/deadlines")]
    DeadlineList {},
    #[route("/deadlines/:id")]
    DeadlineDetail { id: String },
    // ── Opinions ──
    #[route("/opinions")]
    OpinionList {},
    #[route("/opinions/:id")]
    OpinionDetail { id: String },
    // ── Judges ──
    #[route("/judges")]
    JudgeList {},
    #[route("/judges/:id")]
    JudgeDetail { id: String },
    // ── Compliance ──
    #[route("/compliance")]
    ComplianceDashboard {},
    // ── Rules ──
    #[route("/rules")]
    RuleList {},
    #[route("/rules/:id")]
    RuleDetail { id: String },
    // ── Search ──
    #[route("/search")]
    AdvancedSearch {},
    #[end_layout]
    #[end_layout]
    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

/// Auth guard layout -- redirects to /login if not authenticated.
///
/// Uses `use_server_future` with `?` to propagate suspension properly.
/// During SSR the component suspends until the auth check completes, then
/// Dioxus re-renders with the resolved data embedded in the HTML.
/// During hydration the embedded data is available immediately.
/// A `SuspenseBoundary` in `App` catches the suspension and shows a spinner.
#[component]
fn AuthGuard() -> Element {
    let mut auth = use_auth();

    // `?` propagates RenderError during suspension so Dioxus knows to
    // re-render this component when the server future resolves.
    let resource = use_server_future(move || async move { server::api::get_current_user().await })?;

    // Clone the result out of the resource guard to avoid lifetime issues.
    let result = resource.read().as_ref().cloned();

    match result {
        Some(Ok(Some(user))) => {
            if !auth.is_authenticated() {
                auth.set_user(user.clone());
                // Auto-select an accessible district if the current one is not in the user's court_roles
                let mut ctx = use_context::<CourtContext>();
                let current = ctx.court_id.read().clone();
                if user.role != "admin" && !user.court_roles.contains_key(&current) {
                    if let Some(first_court) = user.court_roles.keys().next() {
                        ctx.court_id.set(first_court.clone());
                    }
                }
            }
            rsx! { Outlet::<Route> {} }
        }
        Some(Ok(None)) | Some(Err(_)) => {
            auth.clear_auth();
            navigator().push(Route::Login { redirect: None });
            rsx! {
                div { class: "auth-guard-loading",
                    p { "Redirecting to login..." }
                }
            }
        }
        None => {
            rsx! {
                div { class: "auth-guard-loading",
                    p { "Loading..." }
                }
            }
        }
    }
}

/// Main app layout with sidebar and top navbar.
#[component]
fn AppLayout() -> Element {
    let route: Route = use_route();
    let profile: ProfileState = use_context();
    let flags: FeatureFlags = use_context();
    let mut auth = use_auth();

    let vis = use_sidebar_visibility();
    let mut show_palette = use_signal(|| false);

    let mut theme_state = use_context_provider(|| shared_ui::theme::ThemeState {
        family: Signal::new("cyberpunk".to_string()),
        is_dark: Signal::new(true),
    });

    let role = use_user_role();
    let page_title = match &route {
        Route::Dashboard {} => match role {
            UserRole::Admin | UserRole::Clerk => "Queue",
            _ => "Dashboard",
        },
        Route::Users {} => "Users",
        Route::Settings { .. } => "Settings",
        Route::AttorneyList {} | Route::AttorneyDetail { .. } => "Attorneys",
        Route::CalendarList {} | Route::CalendarDetail { .. } => "Calendar",
        Route::CaseList {} | Route::CaseDetail { .. } => "Cases",
        Route::DeadlineList {} | Route::DeadlineDetail { .. } => "Deadlines",
        Route::OpinionList {} | Route::OpinionDetail { .. } => "Opinions",
        Route::JudgeList {} | Route::JudgeDetail { .. } => "Judges",
        Route::ComplianceDashboard {} => "Compliance",
        Route::RuleList {} | Route::RuleDetail { .. } => "Rules",
        Route::AdvancedSearch {} => "Search",
        Route::Login { .. }
        | Route::Register {}
        | Route::ForgotPassword {}
        | Route::ResetPassword { .. }
        | Route::Activate { .. }
        | Route::DeviceAuth {} => "Auth",
        Route::Privacy {} | Route::Terms {} => "Legal",
        _ => "",
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./layout.css") }

        // Global keyboard listener: Cmd+K (Mac) / Ctrl+K (Win/Linux) toggles the palette.
        // tabindex="0" ensures the wrapper div is focusable and receives key events.
        div {
            tabindex: 0,
            style: "outline: none;",
            onkeydown: move |e: KeyboardEvent| {
                let key = e.key();
                let mods = e.modifiers();
                // Check for Cmd+K (Meta on Mac) or Ctrl+K (Control on Win/Linux)
                let is_cmd_k = matches!(key, Key::Character(ref c) if c == "k")
                    && (mods.contains(Modifiers::META) || mods.contains(Modifiers::CONTROL));
                if is_cmd_k {
                    e.prevent_default();
                    show_palette.toggle();
                }
            },

        SidebarProvider { default_open: false,
            command_palette::CommandPalette { show: show_palette }
            if flags.stripe {
                crate::billing_listener::BillingListener {}
            }
            Sidebar {
                SidebarHeader {
                    div {
                        class: "sidebar-brand",
                        span {
                            class: "sidebar-brand-name",
                            "Lexodus"
                        }
                    }
                    district_picker::DistrictPicker {}
                }

                SidebarSeparator {}

                SidebarContent {
                    // ── 1. My Work ──
                    if vis.work {
                        SidebarGroup {
                            SidebarGroupLabel { "My Work" }
                            SidebarGroupContent {
                                SidebarMenu {
                                    SidebarMenuItem {
                                        Link { to: Route::Dashboard {},
                                            SidebarMenuButton { active: matches!(route, Route::Dashboard {}),
                                                Icon::<LdLayoutDashboard> { icon: LdLayoutDashboard, width: 18, height: 18 }
                                                {match role {
                                                    UserRole::Admin | UserRole::Clerk => "Queue",
                                                    _ => "Dashboard",
                                                }}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        SidebarSeparator {}
                    }

                    // ── 2. Court ──
                    if vis.court {
                        SidebarGroup {
                            SidebarGroupLabel { "Court" }
                            SidebarGroupContent {
                                SidebarMenu {
                                    SidebarMenuItem {
                                        Link { to: Route::CaseList {},
                                            SidebarMenuButton { active: matches!(route, Route::CaseList {} | Route::CaseDetail { .. }),
                                                Icon::<LdBriefcase> { icon: LdBriefcase, width: 18, height: 18 }
                                                "Cases"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::CalendarList {},
                                            SidebarMenuButton { active: matches!(route, Route::CalendarList {} | Route::CalendarDetail { .. }),
                                                Icon::<LdCalendar> { icon: LdCalendar, width: 18, height: 18 }
                                                "Schedule"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::DeadlineList {},
                                            SidebarMenuButton { active: matches!(route, Route::DeadlineList {} | Route::DeadlineDetail { .. }),
                                                Icon::<LdClock> { icon: LdClock, width: 18, height: 18 }
                                                "Deadlines"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        SidebarSeparator {}
                    }

                    // ── 3. Admin ──
                    if vis.admin {
                        SidebarGroup {
                            SidebarGroupLabel { "Admin" }
                            SidebarGroupContent {
                                SidebarMenu {
                                    SidebarMenuItem {
                                        Link { to: Route::ComplianceDashboard {},
                                            SidebarMenuButton { active: matches!(route, Route::ComplianceDashboard {}),
                                                Icon::<LdShield> { icon: LdShield, width: 18, height: 18 }
                                                "Compliance"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::RuleList {},
                                            SidebarMenuButton { active: matches!(route, Route::RuleList {} | Route::RuleDetail { .. }),
                                                Icon::<LdBookOpen> { icon: LdBookOpen, width: 18, height: 18 }
                                                "Rules"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::Users {},
                                            SidebarMenuButton { active: matches!(route, Route::Users {}),
                                                Icon::<LdUsers> { icon: LdUsers, width: 18, height: 18 }
                                                "Users"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::Settings { billing: None, verified: None },
                                            SidebarMenuButton { active: matches!(route, Route::Settings { .. }),
                                                Icon::<LdSettings> { icon: LdSettings, width: 18, height: 18 }
                                                "Settings"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                SidebarFooter {
                    TierBadge {}
                    div {
                        class: "sidebar-footer-row",
                        span {
                            class: "sidebar-footer-label",
                            "Light Mode"
                        }
                        Switch {
                            checked: (theme_state.is_dark)(),
                            on_checked_change: move |checked: bool| {
                                theme_state.is_dark.set(checked);
                                theme_state.apply();
                            },
                            SwitchThumb {}
                        }
                    }
                }

                SidebarRail {}
            }

            SidebarInset {
                // Top navbar
                Navbar {
                    div {
                        class: "navbar-bar",

                        SidebarTrigger {
                            span { class: "navbar-trigger-icon", "\u{2630}" }
                        }

                        Separator { horizontal: false }

                        span {
                            class: "navbar-title",
                            "{page_title}"
                        }

                        // Spacer
                        div { class: "navbar-spacer" }

                        // Search / command palette
                        button {
                            class: "navbar-notification-bell",
                            title: "Search (Cmd+K)",
                            onclick: move |_| show_palette.toggle(),
                            Icon::<LdSearch> { icon: LdSearch, width: 20, height: 20 }
                        }

                        // Notification bell
                        button {
                            class: "navbar-notification-bell",
                            title: "Notifications",
                            onclick: move |_| {
                                // TODO: Toggle notification panel
                            },
                            Icon::<LdBell> { icon: LdBell, width: 20, height: 20 }
                        }

                        // User dropdown
                        DropdownMenu {
                            DropdownMenuTrigger {
                                Avatar {
                                    if let Some(url) = profile.avatar_url.read().as_ref() {
                                        AvatarImage { src: url.clone() }
                                    }
                                    AvatarFallback {
                                        {profile.display_name.read().split_whitespace().filter_map(|w| w.chars().next()).take(2).collect::<String>().to_uppercase()}
                                    }
                                }
                            }
                            DropdownMenuContent {
                                DropdownMenuItem::<String> {
                                    value: "profile".to_string(),
                                    index: 0usize,
                                    on_select: move |_: String| {
                                        navigator().push(Route::Settings { billing: None, verified: None });
                                    },
                                    "Profile"
                                }
                                DropdownMenuSeparator {}
                                DropdownMenuItem::<String> {
                                    value: "docs".to_string(),
                                    index: 1usize,
                                    div {
                                        onclick: move |_| {
                                            navigator().push(
                                                NavigationTarget::<Route>::External(
                                                    "/docs".to_string(),
                                                ),
                                            );
                                        },
                                        class: "dropdown-docs-link",
                                        "API Docs"
                                    }
                                }
                                DropdownMenuSeparator {}
                                DropdownMenuItem::<String> {
                                    value: "logout".to_string(),
                                    index: 2usize,
                                    on_select: move |_: String| {
                                        spawn(async move {
                                            let _ = server::api::logout().await;
                                        });
                                        auth.clear_auth();
                                        navigator().push(Route::Login { redirect: None });
                                    },
                                    "Sign Out"
                                }
                            }
                        }
                    }
                }

                // Page content
                div {
                    class: "page-content",
                    Outlet::<Route> {}
                }
            }
        }
        } // close global keyboard listener wrapper div
    }
}

// Court domain route components

#[component]
fn AttorneyList() -> Element {
    attorneys::list::AttorneyListPage()
}

#[component]
fn AttorneyDetail(id: String) -> Element {
    rsx! { attorneys::detail::AttorneyDetailPage { id: id } }
}

#[component]
fn CalendarList() -> Element {
    calendar::list::CalendarListPage()
}

#[component]
fn CalendarDetail(id: String) -> Element {
    rsx! { calendar::detail::CalendarDetailPage { id: id } }
}

#[component]
fn CaseList() -> Element {
    cases::list::CaseListPage()
}

#[component]
fn CaseDetail(id: String, tab: Option<String>) -> Element {
    rsx! { cases::detail::CaseDetailPage { id: id, tab: tab.unwrap_or_default() } }
}

#[component]
fn DeadlineList() -> Element {
    deadlines::list::DeadlineListPage()
}

#[component]
fn DeadlineDetail(id: String) -> Element {
    rsx! { deadlines::detail::DeadlineDetailPage { id: id } }
}

#[component]
fn OpinionList() -> Element {
    opinions::list::OpinionListPage()
}

#[component]
fn OpinionDetail(id: String) -> Element {
    rsx! { opinions::detail::OpinionDetailPage { id: id } }
}

#[component]
fn JudgeList() -> Element {
    judges::list::JudgeListPage()
}

#[component]
fn JudgeDetail(id: String) -> Element {
    rsx! { judges::detail::JudgeDetailPage { id: id } }
}

#[component]
fn ComplianceDashboard() -> Element {
    compliance::ComplianceDashboardPage()
}

#[component]
fn RuleList() -> Element {
    rules::list::RuleListPage()
}

#[component]
fn RuleDetail(id: String) -> Element {
    rsx! { rules::detail::RuleDetailPage { id: id } }
}

#[component]
fn AdvancedSearch() -> Element {
    search::advanced::AdvancedSearchPage()
}

/// Displays the selected court's tier as a badge in the sidebar footer.
#[component]
fn TierBadge() -> Element {
    let ctx = use_context::<CourtContext>();
    let tier = ctx.court_tier.read().clone();

    let (variant, label) = match tier {
        UserTier::Free => (BadgeVariant::Secondary, "FREE"),
        UserTier::Pro => (BadgeVariant::Primary, "PRO"),
        UserTier::Enterprise => (BadgeVariant::Destructive, "ENTERPRISE"),
    };

    rsx! {
        div { class: "sidebar-footer-row sidebar-tier-row",
            span { class: "sidebar-footer-label", "Court Tier" }
            Badge { variant: variant, "{label}" }
        }
    }
}
