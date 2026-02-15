pub mod activate;
pub mod attorneys;
pub mod calendar;
pub mod cases;
pub mod compliance;
pub mod dashboard;
pub mod deadlines;
pub mod defendants;
pub mod device_auth;
pub mod docket;
pub mod documents;
pub mod evidence;
pub mod filings;
pub mod forgot_password;
pub mod judges;
pub mod login;
pub mod not_found;
pub mod opinions;
pub mod orders;
pub mod parties;
pub mod privacy;
pub mod products;
pub mod register;
pub mod reset_password;
pub mod rules;
pub mod sentencing;
pub mod service_records;
pub mod settings;
pub mod terms;
pub mod users;
pub mod victims;

use crate::auth::{use_auth, use_sidebar_visibility};
use crate::ProfileState;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdBell, LdBookOpen, LdBriefcase, LdCalendar, LdClock, LdFileText, LdFolder,
    LdLayoutDashboard, LdPackage, LdScale, LdSearch, LdSettings, LdShield, LdUserCheck, LdUsers,
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
    // ── Defendants ──
    #[route("/defendants")]
    DefendantList {},
    #[route("/defendants/:id")]
    DefendantDetail { id: String },
    // ── Parties ──
    #[route("/parties")]
    PartyList {},
    #[route("/parties/:id")]
    PartyDetail { id: String },
    // ── Victims ──
    #[route("/victims")]
    VictimList {},
    #[route("/victims/:id")]
    VictimDetail { id: String },
    // ── Docket ──
    #[route("/docket")]
    DocketList {},
    #[route("/docket/:id")]
    DocketDetail { id: String },
    // ── Filings ──
    #[route("/filings")]
    FilingList {},
    #[route("/filings/:id")]
    FilingDetail { id: String },
    // ── Service Records ──
    #[route("/service-records")]
    ServiceRecordList {},
    #[route("/service-records/:id")]
    ServiceRecordDetail { id: String },
    // ── Orders ──
    #[route("/orders")]
    OrderList {},
    #[route("/orders/:id")]
    OrderDetail { id: String },
    // ── Opinions ──
    #[route("/opinions")]
    OpinionList {},
    #[route("/opinions/:id")]
    OpinionDetail { id: String },
    // ── Evidence ──
    #[route("/evidence")]
    EvidenceList {},
    #[route("/evidence/:id")]
    EvidenceDetail { id: String },
    // ── Documents ──
    #[route("/documents")]
    DocumentList {},
    #[route("/documents/:id")]
    DocumentDetail { id: String },
    // ── Sentencing ──
    #[route("/sentencing")]
    SentencingList {},
    #[route("/sentencing/:id")]
    SentencingDetail { id: String },
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

    let vis = use_sidebar_visibility();

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
        Route::DefendantList {} | Route::DefendantDetail { .. } => "Defendants",
        Route::PartyList {} | Route::PartyDetail { .. } => "Parties",
        Route::VictimList {} | Route::VictimDetail { .. } => "Victims",
        Route::DocketList {} | Route::DocketDetail { .. } => "Docket",
        Route::FilingList {} | Route::FilingDetail { .. } => "Filings",
        Route::ServiceRecordList {} | Route::ServiceRecordDetail { .. } => "Service Records",
        Route::OrderList {} | Route::OrderDetail { .. } => "Orders",
        Route::OpinionList {} | Route::OpinionDetail { .. } => "Opinions",
        Route::EvidenceList {} | Route::EvidenceDetail { .. } => "Evidence",
        Route::DocumentList {} | Route::DocumentDetail { .. } => "Documents",
        Route::SentencingList {} | Route::SentencingDetail { .. } => "Sentencing",
        Route::JudgeList {} | Route::JudgeDetail { .. } => "Judges",
        Route::ComplianceDashboard {} => "Compliance",
        Route::RuleList {} | Route::RuleDetail { .. } => "Rules",
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
                    // ── 1. Core (all roles) ──
                    SidebarGroup {
                        SidebarGroupLabel { "Core" }
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

                    // ── 2. Case Management ──
                    if vis.case_management {
                        SidebarGroup {
                            SidebarGroupLabel { "Case Management" }
                            SidebarGroupContent {
                                SidebarMenu {
                                    SidebarMenuItem {
                                        Link { to: Route::CaseList {},
                                            SidebarMenuButton { active: matches!(route, Route::CaseList {} | Route::CaseCreate {} | Route::CaseDetail { .. }),
                                                Icon::<LdBriefcase> { icon: LdBriefcase, width: 18, height: 18 }
                                                "Cases"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::DefendantList {},
                                            SidebarMenuButton { active: matches!(route, Route::DefendantList {} | Route::DefendantDetail { .. }),
                                                "Defendants"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::PartyList {},
                                            SidebarMenuButton { active: matches!(route, Route::PartyList {} | Route::PartyDetail { .. }),
                                                "Parties"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::VictimList {},
                                            SidebarMenuButton { active: matches!(route, Route::VictimList {} | Route::VictimDetail { .. }),
                                                "Victims"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        SidebarSeparator {}
                    }

                    // ── 3. Court Operations ──
                    if vis.court_operations {
                        SidebarGroup {
                            SidebarGroupLabel { "Court Operations" }
                            SidebarGroupContent {
                                SidebarMenu {
                                    SidebarMenuItem {
                                        Link { to: Route::CalendarList {},
                                            SidebarMenuButton { active: matches!(route, Route::CalendarList {} | Route::CalendarCreate {} | Route::CalendarDetail { .. }),
                                                Icon::<LdCalendar> { icon: LdCalendar, width: 18, height: 18 }
                                                "Calendar"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::DeadlineList {},
                                            SidebarMenuButton { active: matches!(route, Route::DeadlineList {} | Route::DeadlineCreate {} | Route::DeadlineDetail { .. }),
                                                Icon::<LdClock> { icon: LdClock, width: 18, height: 18 }
                                                "Deadlines"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::DocketList {},
                                            SidebarMenuButton { active: matches!(route, Route::DocketList {} | Route::DocketDetail { .. }),
                                                Icon::<LdFileText> { icon: LdFileText, width: 18, height: 18 }
                                                "Docket"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::FilingList {},
                                            SidebarMenuButton { active: matches!(route, Route::FilingList {} | Route::FilingDetail { .. }),
                                                "Filings"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::ServiceRecordList {},
                                            SidebarMenuButton { active: matches!(route, Route::ServiceRecordList {} | Route::ServiceRecordDetail { .. }),
                                                "Service Records"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        SidebarSeparator {}
                    }

                    // ── 4. Legal Documents ──
                    if vis.legal_documents {
                        SidebarGroup {
                            SidebarGroupLabel { "Legal Documents" }
                            SidebarGroupContent {
                                SidebarMenu {
                                    SidebarMenuItem {
                                        Link { to: Route::OrderList {},
                                            SidebarMenuButton { active: matches!(route, Route::OrderList {} | Route::OrderDetail { .. }),
                                                Icon::<LdScale> { icon: LdScale, width: 18, height: 18 }
                                                "Orders"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::OpinionList {},
                                            SidebarMenuButton { active: matches!(route, Route::OpinionList {} | Route::OpinionDetail { .. }),
                                                Icon::<LdBookOpen> { icon: LdBookOpen, width: 18, height: 18 }
                                                "Opinions"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::EvidenceList {},
                                            SidebarMenuButton { active: matches!(route, Route::EvidenceList {} | Route::EvidenceDetail { .. }),
                                                "Evidence"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::DocumentList {},
                                            SidebarMenuButton { active: matches!(route, Route::DocumentList {} | Route::DocumentDetail { .. }),
                                                Icon::<LdFolder> { icon: LdFolder, width: 18, height: 18 }
                                                "Documents"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::SentencingList {},
                                            SidebarMenuButton { active: matches!(route, Route::SentencingList {} | Route::SentencingDetail { .. }),
                                                "Sentencing"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        SidebarSeparator {}
                    }

                    // ── 5. People & Organizations ──
                    if vis.people_orgs {
                        SidebarGroup {
                            SidebarGroupLabel { "People & Organizations" }
                            SidebarGroupContent {
                                SidebarMenu {
                                    SidebarMenuItem {
                                        Link { to: Route::AttorneyList {},
                                            SidebarMenuButton { active: matches!(route, Route::AttorneyList {} | Route::AttorneyCreate {} | Route::AttorneyDetail { .. }),
                                                Icon::<LdUserCheck> { icon: LdUserCheck, width: 18, height: 18 }
                                                "Attorneys"
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        Link { to: Route::JudgeList {},
                                            SidebarMenuButton { active: matches!(route, Route::JudgeList {} | Route::JudgeDetail { .. }),
                                                Icon::<LdScale> { icon: LdScale, width: 18, height: 18 }
                                                "Judges"
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
                                }
                            }
                        }
                        SidebarSeparator {}
                    }

                    // ── 6. Administration ──
                    if vis.administration {
                        SidebarGroup {
                            SidebarGroupLabel { "Administration" }
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
                                                "Rules"
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

// ── New domain route components ──

#[component]
fn DefendantList() -> Element {
    defendants::list::DefendantListPage()
}

#[component]
fn DefendantDetail(id: String) -> Element {
    rsx! { defendants::detail::DefendantDetailPage { id: id } }
}

#[component]
fn PartyList() -> Element {
    parties::list::PartyListPage()
}

#[component]
fn PartyDetail(id: String) -> Element {
    rsx! { parties::detail::PartyDetailPage { id: id } }
}

#[component]
fn VictimList() -> Element {
    victims::list::VictimListPage()
}

#[component]
fn VictimDetail(id: String) -> Element {
    rsx! { victims::detail::VictimDetailPage { id: id } }
}

#[component]
fn DocketList() -> Element {
    docket::list::DocketListPage()
}

#[component]
fn DocketDetail(id: String) -> Element {
    rsx! { docket::detail::DocketDetailPage { id: id } }
}

#[component]
fn FilingList() -> Element {
    filings::list::FilingListPage()
}

#[component]
fn FilingDetail(id: String) -> Element {
    rsx! { filings::detail::FilingDetailPage { id: id } }
}

#[component]
fn ServiceRecordList() -> Element {
    service_records::list::ServiceRecordListPage()
}

#[component]
fn ServiceRecordDetail(id: String) -> Element {
    rsx! { service_records::detail::ServiceRecordDetailPage { id: id } }
}

#[component]
fn OrderList() -> Element {
    orders::list::OrderListPage()
}

#[component]
fn OrderDetail(id: String) -> Element {
    rsx! { orders::detail::OrderDetailPage { id: id } }
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
fn EvidenceList() -> Element {
    evidence::list::EvidenceListPage()
}

#[component]
fn EvidenceDetail(id: String) -> Element {
    rsx! { evidence::detail::EvidenceDetailPage { id: id } }
}

#[component]
fn DocumentList() -> Element {
    documents::list::DocumentListPage()
}

#[component]
fn DocumentDetail(id: String) -> Element {
    rsx! { documents::detail::DocumentDetailPage { id: id } }
}

#[component]
fn SentencingList() -> Element {
    sentencing::list::SentencingListPage()
}

#[component]
fn SentencingDetail(id: String) -> Element {
    rsx! { sentencing::detail::SentencingDetailPage { id: id } }
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
