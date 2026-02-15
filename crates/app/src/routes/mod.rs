pub mod activate;
pub mod attorneys;
pub mod calendar;
pub mod cases;
pub mod dashboard;
pub mod deadlines;
pub mod device_auth;
pub mod forgot_password;
pub mod login;
pub mod not_found;
pub mod privacy;
pub mod products;
pub mod register;
pub mod reset_password;
pub mod settings;
pub mod terms;
pub mod users;

use crate::auth::use_auth;
use crate::ProfileState;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdLayoutDashboard, LdPackage, LdSettings, LdUsers};
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
use products::Products;
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
    #[route("/products")]
    Products {},
    #[route("/settings/?:billing&:verified")]
    Settings {
        billing: Option<String>,
        verified: Option<String>,
    },
    // Court domain routes
    #[route("/attorneys")]
    AttorneyList {},
    #[route("/attorneys/new")]
    AttorneyCreate {},
    #[route("/attorneys/:id")]
    AttorneyDetail { id: String },
    #[route("/calendar")]
    CalendarList {},
    #[route("/calendar/new")]
    CalendarCreate {},
    #[route("/calendar/:id")]
    CalendarDetail { id: String },
    #[route("/cases")]
    CaseList {},
    #[route("/cases/new")]
    CaseCreate {},
    #[route("/cases/:id")]
    CaseDetail { id: String },
    #[route("/deadlines")]
    DeadlineList {},
    #[route("/deadlines/new")]
    DeadlineCreate {},
    #[route("/deadlines/:id")]
    DeadlineDetail { id: String },
    #[end_layout]
    #[end_layout]
    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

/// Auth guard layout — redirects to /login if not authenticated.
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
                auth.set_user(user);
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

    let mut theme_state = use_context_provider(|| shared_ui::theme::ThemeState {
        family: Signal::new("cyberpunk".to_string()),
        is_dark: Signal::new(true),
    });

    let page_title = match &route {
        Route::Dashboard {} => "Dashboard",
        Route::Users {} => "Users",
        Route::Products {} => "Products",
        Route::Settings { .. } => "Settings",
        Route::AttorneyList {} | Route::AttorneyCreate {} | Route::AttorneyDetail { .. } => "Attorneys",
        Route::CalendarList {} | Route::CalendarCreate {} | Route::CalendarDetail { .. } => "Calendar",
        Route::CaseList {} | Route::CaseCreate {} | Route::CaseDetail { .. } => "Cases",
        Route::DeadlineList {} | Route::DeadlineCreate {} | Route::DeadlineDetail { .. } => "Deadlines",
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

        SidebarProvider { default_open: false,
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
                }

                SidebarSeparator {}

                SidebarContent {
                    SidebarGroup {
                        SidebarGroupLabel { "Navigation" }
                        SidebarGroupContent {
                            SidebarMenu {
                                SidebarMenuItem {
                                    Link { to: Route::Dashboard {},
                                        SidebarMenuButton { active: matches!(route, Route::Dashboard {}),
                                            Icon::<LdLayoutDashboard> { icon: LdLayoutDashboard, width: 18, height: 18 }
                                            "Dashboard"
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
                                    Link { to: Route::Products {},
                                        SidebarMenuButton { active: matches!(route, Route::Products {}),
                                            Icon::<LdPackage> { icon: LdPackage, width: 18, height: 18 }
                                            "Products"
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

                    SidebarSeparator {}

                    SidebarGroup {
                        SidebarGroupLabel { "Court Management" }
                        SidebarGroupContent {
                            SidebarMenu {
                                SidebarMenuItem {
                                    Link { to: Route::AttorneyList {},
                                        SidebarMenuButton { active: matches!(route, Route::AttorneyList {} | Route::AttorneyCreate {} | Route::AttorneyDetail { .. }),
                                            "Attorneys"
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    Link { to: Route::CaseList {},
                                        SidebarMenuButton { active: matches!(route, Route::CaseList {} | Route::CaseCreate {} | Route::CaseDetail { .. }),
                                            "Cases"
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    Link { to: Route::CalendarList {},
                                        SidebarMenuButton { active: matches!(route, Route::CalendarList {} | Route::CalendarCreate {} | Route::CalendarDetail { .. }),
                                            "Calendar"
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    Link { to: Route::DeadlineList {},
                                        SidebarMenuButton { active: matches!(route, Route::DeadlineList {} | Route::DeadlineCreate {} | Route::DeadlineDetail { .. }),
                                            "Deadlines"
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
    }
}

// Court domain route components

#[component]
fn AttorneyList() -> Element {
    attorneys::list::AttorneyListPage()
}

#[component]
fn AttorneyCreate() -> Element {
    attorneys::create::AttorneyCreatePage()
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
fn CalendarCreate() -> Element {
    calendar::create::CalendarCreatePage()
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
fn CaseCreate() -> Element {
    cases::create::CaseCreatePage()
}

#[component]
fn CaseDetail(id: String) -> Element {
    rsx! { cases::detail::CaseDetailPage { id: id } }
}

#[component]
fn DeadlineList() -> Element {
    deadlines::list::DeadlineListPage()
}

#[component]
fn DeadlineCreate() -> Element {
    deadlines::create::DeadlineCreatePage()
}

#[component]
fn DeadlineDetail(id: String) -> Element {
    rsx! { deadlines::detail::DeadlineDetailPage { id: id } }
}

/// Displays the current user's tier as a badge in the sidebar footer.
#[component]
fn TierBadge() -> Element {
    let auth = use_auth();
    let tier = use_memo(move || {
        auth.current_user
            .read()
            .as_ref()
            .map(|u| u.tier.clone())
            .unwrap_or(UserTier::Free)
    });

    let (variant, label) = match tier() {
        UserTier::Free => (BadgeVariant::Secondary, "FREE"),
        UserTier::Pro => (BadgeVariant::Primary, "PRO"),
        UserTier::Enterprise => (BadgeVariant::Destructive, "ENTERPRISE"),
    };

    rsx! {
        div { class: "sidebar-footer-row sidebar-tier-row",
            span { class: "sidebar-footer-label", "Tier" }
            Badge { variant: variant, "{label}" }
        }
    }
}
