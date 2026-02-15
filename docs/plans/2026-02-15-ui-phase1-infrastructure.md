# UI Phase 1: Infrastructure + Dashboards + Case Hub — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the navigation infrastructure (role-adaptive sidebar, command palette, notifications), 5 role-specific dashboards, and the 9-tab case detail hub.

**Architecture:** Restructure the existing sidebar into 6 role-gated groups, add a command palette (Cmd+K) overlay component, create role-specific dashboard components dispatched by `use_user_role()`, and refactor the case detail page into a tabbed hub with sub-components per tab. All new server functions follow the existing thin-wrapper pattern in `api.rs`.

**Tech Stack:** Dioxus (Rust), shared-ui components, shared-types models, server API wrappers

**Reference:** Design doc at `docs/plans/2026-02-15-lexodus-ui-ux-design.md`

---

## Phase 1A: Navigation Infrastructure

### Task 1: Restructure Sidebar into 6 Role-Gated Groups

**Files:**
- Modify: `crates/app/src/routes/mod.rs`
- Modify: `crates/app/src/auth.rs`

**Step 1: Add sidebar visibility helper to auth.rs**

Add this function at the end of `crates/app/src/auth.rs`:

```rust
/// Determine which sidebar groups are visible for the current user's role.
/// Returns a struct with booleans for each group.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SidebarVisibility {
    pub core: bool,
    pub case_management: bool,
    pub court_operations: bool,
    pub legal_documents: bool,
    pub people_orgs: bool,
    pub administration: bool,
}

pub fn use_sidebar_visibility() -> SidebarVisibility {
    let role = use_user_role();
    match role {
        UserRole::Admin => SidebarVisibility {
            core: true,
            case_management: true,
            court_operations: true,
            legal_documents: true,
            people_orgs: true,
            administration: true,
        },
        UserRole::Clerk => SidebarVisibility {
            core: true,
            case_management: true,
            court_operations: true,
            legal_documents: true,
            people_orgs: true,
            administration: true,
        },
        UserRole::Judge => SidebarVisibility {
            core: true,
            case_management: true,
            court_operations: true,
            legal_documents: true,
            people_orgs: false,
            administration: false,
        },
        UserRole::Attorney => SidebarVisibility {
            core: true,
            case_management: true,
            court_operations: false,
            legal_documents: true,
            people_orgs: false,
            administration: false,
        },
        UserRole::Public => SidebarVisibility {
            core: true,
            case_management: false,
            court_operations: false,
            legal_documents: false,
            people_orgs: false,
            administration: false,
        },
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p app --features server`
Expected: compiles (new code is unused yet)

**Step 3: Replace the sidebar in AppLayout**

In `crates/app/src/routes/mod.rs`, replace the entire `SidebarContent { ... }` block (lines 206-284) with the new 6-group sidebar. Update imports to add:

```rust
use crate::auth::{use_auth, use_sidebar_visibility};
```

And add icon imports:

```rust
use dioxus_free_icons::icons::ld_icons::{
    LdLayoutDashboard, LdPackage, LdSettings, LdUsers, LdSearch,
    LdBell, LdScale, LdCalendar, LdClock, LdFileText, LdGavel,
    LdUserCheck, LdShield, LdBookOpen, LdFolder, LdBriefcase,
};
```

Replace the SidebarContent block with:

```rust
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
                                Icon::<LdGavel> { icon: LdGavel, width: 18, height: 18 }
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
```

Also add `let vis = use_sidebar_visibility();` at the top of `AppLayout` body, after the existing `let` bindings.

**Step 4: Verify it compiles**

Run: `cargo check -p app --features server`
Expected: Compiler errors for undefined routes (DefendantList, PartyList, etc.) — expected, we'll add those in Task 2.

**Step 5: Commit**

```bash
git add crates/app/src/auth.rs crates/app/src/routes/mod.rs
git commit -m "feat(ui): restructure sidebar into 6 role-gated groups"
```

---

### Task 2: Expand Route Enum with All Domain Pages

**Files:**
- Modify: `crates/app/src/routes/mod.rs`

**Step 1: Add module declarations**

At the top of `crates/app/src/routes/mod.rs`, add new module declarations after the existing ones:

```rust
pub mod compliance;
pub mod defendants;
pub mod docket;
pub mod documents;
pub mod evidence;
pub mod filings;
pub mod judges;
pub mod opinions;
pub mod orders;
pub mod parties;
pub mod rules;
pub mod sentencing;
pub mod service_records;
pub mod victims;
```

**Step 2: Expand the Route enum**

Add these variants inside the `#[layout(AppLayout)]` section of the Route enum, after the existing DeadlineDetail:

```rust
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
```

**Step 3: Add page_title matches**

Extend the `page_title` match in AppLayout for all new routes:

```rust
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
```

**Step 4: Add route component stubs**

Add stub components at the bottom of `mod.rs` for each new route, following the existing pattern:

```rust
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
```

**Step 5: Do NOT compile yet — module files don't exist. Commit the route definitions.**

```bash
git add crates/app/src/routes/mod.rs
git commit -m "feat(ui): add route definitions for all 14 new domain modules"
```

---

### Task 3: Create Stub Modules for All New Domains

**Files:**
- Create 14 new module directories under `crates/app/src/routes/`

Each domain needs: `mod.rs`, `list.rs`, `detail.rs`. They all follow the same stub pattern.

**Step 1: Create module files**

For each domain, create the 3 files. Here is the **template** — shown once for `defendants`, then replicate for all 13 others.

**`crates/app/src/routes/defendants/mod.rs`:**
```rust
pub mod list;
pub mod detail;
```

**`crates/app/src/routes/defendants/list.rs`:**
```rust
use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn DefendantListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Defendants" }
        }
        div { class: "page-placeholder",
            p { "Defendant list page — coming soon." }
        }
    }
}
```

**`crates/app/src/routes/defendants/detail.rs`:**
```rust
use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn DefendantDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Defendant Detail" }
        }
        div { class: "page-placeholder",
            p { "Defendant {id} — coming soon." }
        }
    }
}
```

Replicate this exact pattern for these 13 remaining domains (substituting names):

| Directory | List component | Detail component |
|-----------|---------------|-----------------|
| `parties` | `PartyListPage` | `PartyDetailPage` |
| `victims` | `VictimListPage` | `VictimDetailPage` |
| `docket` | `DocketListPage` | `DocketDetailPage` |
| `filings` | `FilingListPage` | `FilingDetailPage` |
| `service_records` | `ServiceRecordListPage` | `ServiceRecordDetailPage` |
| `orders` | `OrderListPage` | `OrderDetailPage` |
| `opinions` | `OpinionListPage` | `OpinionDetailPage` |
| `evidence` | `EvidenceListPage` | `EvidenceDetailPage` |
| `documents` | `DocumentListPage` | `DocumentDetailPage` |
| `sentencing` | `SentencingListPage` | `SentencingDetailPage` |
| `judges` | `JudgeListPage` | `JudgeDetailPage` |
| `rules` | `RuleListPage` | `RuleDetailPage` |

For `compliance`, create a single file (no list/detail split):

**`crates/app/src/routes/compliance/mod.rs`:**
```rust
use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn ComplianceDashboardPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Compliance Dashboard" }
        }
        div { class: "page-placeholder",
            p { "Compliance dashboard — coming soon." }
        }
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p app --features server`
Expected: Clean compilation.

**Step 3: Commit**

```bash
git add crates/app/src/routes/defendants/ crates/app/src/routes/parties/ \
  crates/app/src/routes/victims/ crates/app/src/routes/docket/ \
  crates/app/src/routes/filings/ crates/app/src/routes/service_records/ \
  crates/app/src/routes/orders/ crates/app/src/routes/opinions/ \
  crates/app/src/routes/evidence/ crates/app/src/routes/documents/ \
  crates/app/src/routes/sentencing/ crates/app/src/routes/judges/ \
  crates/app/src/routes/rules/ crates/app/src/routes/compliance/
git commit -m "feat(ui): scaffold stub pages for all 14 new domain modules"
```

---

### Task 4: Add Notification Bell to Navbar

**Files:**
- Modify: `crates/app/src/routes/mod.rs` (navbar section)

**Step 1: Add notification bell icon next to user avatar**

In the navbar section of AppLayout (inside `SidebarInset > Navbar`), add a notification bell before the user dropdown. Find the `// Spacer` div and add after it:

```rust
// Notification bell
button {
    class: "navbar-notification-bell",
    title: "Notifications",
    onclick: move |_| {
        // TODO: Toggle notification panel
    },
    Icon::<LdBell> { icon: LdBell, width: 20, height: 20 }
    // Badge for unread count — placeholder
    // span { class: "notification-badge", "3" }
}
```

Add `LdBell` to the icon imports if not already present.

**Step 2: Add CSS for notification bell**

In `crates/app/src/routes/layout.css`, add:

```css
.navbar-notification-bell {
    background: none;
    border: none;
    color: var(--color-on-surface-muted);
    cursor: pointer;
    padding: var(--space-xs);
    border-radius: var(--radius);
    position: relative;
    display: flex;
    align-items: center;
    transition: color var(--transition-fast);
    margin-right: var(--space-sm);
}

.navbar-notification-bell:hover {
    color: var(--color-on-surface);
    background: var(--color-surface-raised);
}

.notification-badge {
    position: absolute;
    top: -2px;
    right: -2px;
    background: var(--color-error);
    color: white;
    font-size: var(--font-size-xs);
    border-radius: var(--radius-full);
    min-width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 4px;
}
```

**Step 3: Verify it compiles**

Run: `cargo check -p app --features server`
Expected: Clean compilation.

**Step 4: Commit**

```bash
git add crates/app/src/routes/mod.rs crates/app/src/routes/layout.css
git commit -m "feat(ui): add notification bell icon to navbar"
```

---

### Task 5: Create Command Palette Component

**Files:**
- Create: `crates/app/src/routes/command_palette.rs`
- Modify: `crates/app/src/routes/mod.rs`

**Step 1: Create the command palette component**

**`crates/app/src/routes/command_palette.rs`:**

```rust
use dioxus::prelude::*;
use shared_ui::components::{Input, Separator};
use crate::routes::Route;

/// Global command palette overlay, toggled with Cmd+K / Ctrl+K.
#[component]
pub fn CommandPalette(show: Signal<bool>) -> Element {
    let mut query = use_signal(String::new);
    let nav = navigator();

    if !show() {
        return rsx! {};
    }

    let q = query.read().to_lowercase();

    // Static navigation items
    let nav_items: Vec<(&str, &str, Route)> = vec![
        ("Dashboard", "Go to dashboard", Route::Dashboard {}),
        ("Cases", "View all cases", Route::CaseList {}),
        ("Attorneys", "View all attorneys", Route::AttorneyList {}),
        ("Judges", "View all judges", Route::JudgeList {}),
        ("Calendar", "Court calendar", Route::CalendarList {}),
        ("Deadlines", "View deadlines", Route::DeadlineList {}),
        ("Docket", "View docket entries", Route::DocketList {}),
        ("Orders", "View court orders", Route::OrderList {}),
        ("Opinions", "View opinions", Route::OpinionList {}),
        ("Evidence", "View evidence", Route::EvidenceList {}),
        ("Documents", "View documents", Route::DocumentList {}),
        ("Sentencing", "View sentencing", Route::SentencingList {}),
        ("Filings", "View filings", Route::FilingList {}),
        ("Defendants", "View defendants", Route::DefendantList {}),
        ("Parties", "View parties", Route::PartyList {}),
        ("Settings", "User settings", Route::Settings { billing: None, verified: None }),
        ("Compliance", "Compliance dashboard", Route::ComplianceDashboard {}),
        ("Rules", "Court rules", Route::RuleList {}),
    ];

    let filtered: Vec<_> = if q.is_empty() {
        nav_items.iter().take(8).collect()
    } else {
        nav_items
            .iter()
            .filter(|(name, desc, _)| {
                name.to_lowercase().contains(&q) || desc.to_lowercase().contains(&q)
            })
            .collect()
    };

    rsx! {
        // Backdrop
        div {
            class: "cmd-palette-backdrop",
            onclick: move |_| show.set(false),
        }
        div {
            class: "cmd-palette",
            div {
                class: "cmd-palette-input-wrap",
                Input {
                    placeholder: "Type a command or search...",
                    value: "{query}",
                    oninput: move |e: FormEvent| query.set(e.value()),
                    autofocus: true,
                }
            }
            Separator {}
            div {
                class: "cmd-palette-results",
                if filtered.is_empty() {
                    p { class: "cmd-palette-empty", "No results found." }
                }
                for (name, desc, route) in filtered {
                    {
                        let route = route.clone();
                        rsx! {
                            button {
                                class: "cmd-palette-item",
                                onclick: move |_| {
                                    nav.push(route.clone());
                                    show.set(false);
                                    query.set(String::new());
                                },
                                span { class: "cmd-palette-item-name", "{name}" }
                                span { class: "cmd-palette-item-desc", "{desc}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

**Step 2: Add CSS**

Add to `crates/app/src/routes/layout.css`:

```css
.cmd-palette-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 999;
}

.cmd-palette {
    position: fixed;
    top: 20%;
    left: 50%;
    transform: translateX(-50%);
    width: min(600px, 90vw);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.3);
    z-index: 1000;
    overflow: hidden;
    animation: fade-in 0.15s ease-out;
}

.cmd-palette-input-wrap {
    padding: var(--space-md);
}

.cmd-palette-results {
    max-height: 400px;
    overflow-y: auto;
    padding: var(--space-xs);
}

.cmd-palette-item {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    width: 100%;
    padding: var(--space-sm) var(--space-md);
    background: none;
    border: none;
    color: var(--color-on-surface);
    cursor: pointer;
    border-radius: var(--radius);
    text-align: left;
    transition: background var(--transition-fast);
}

.cmd-palette-item:hover {
    background: var(--color-surface-raised);
}

.cmd-palette-item-name {
    font-weight: 500;
    min-width: 120px;
}

.cmd-palette-item-desc {
    color: var(--color-on-surface-muted);
    font-size: var(--font-size-sm);
}

.cmd-palette-empty {
    padding: var(--space-lg);
    text-align: center;
    color: var(--color-on-surface-muted);
}
```

**Step 3: Wire into AppLayout**

In `crates/app/src/routes/mod.rs`:

1. Add `pub mod command_palette;` at the top
2. Add import: `use command_palette::CommandPalette;`
3. Add a signal in AppLayout: `let mut show_palette = use_signal(|| false);`
4. Add the palette component right after the `SidebarProvider` opening, before `if flags.stripe`:

```rust
CommandPalette { show: show_palette }
```

5. Add a keyboard listener in AppLayout (after the signal):

```rust
// Cmd+K / Ctrl+K to toggle command palette
use_hook(move || {
    spawn(async move {
        // Keyboard shortcut handled via JS event listener
        // (Dioxus doesn't have native global keyboard hooks yet)
    });
});
```

Note: For the keyboard shortcut, we'll add a small JS snippet in a later task. For now the palette can be opened via a button.

6. Add a search button in the navbar (before the notification bell):

```rust
button {
    class: "navbar-notification-bell",
    title: "Search (Cmd+K)",
    onclick: move |_| show_palette.toggle(),
    Icon::<LdSearch> { icon: LdSearch, width: 20, height: 20 }
}
```

**Step 4: Verify it compiles**

Run: `cargo check -p app --features server`
Expected: Clean compilation.

**Step 5: Commit**

```bash
git add crates/app/src/routes/command_palette.rs crates/app/src/routes/mod.rs crates/app/src/routes/layout.css
git commit -m "feat(ui): add command palette with Cmd+K search"
```

---

## Phase 1B: Role-Adaptive Dashboards

### Task 6: Refactor Dashboard to Role-Based Dispatch

**Files:**
- Modify: `crates/app/src/routes/dashboard.rs`
- Create: `crates/app/src/routes/dashboard/mod.rs` (if converting to module directory)

**Step 1: Convert dashboard.rs to a module directory**

Rename `crates/app/src/routes/dashboard.rs` to `crates/app/src/routes/dashboard/mod.rs`.

Create `crates/app/src/routes/dashboard/mod.rs` with the role dispatch:

```rust
pub mod clerk;
pub mod judge;
pub mod attorney;

use dioxus::prelude::*;
use shared_types::UserRole;
use crate::auth::use_user_role;

/// Role-adaptive dashboard — renders the appropriate dashboard for the user's role.
#[component]
pub fn Dashboard() -> Element {
    let role = use_user_role();

    match role {
        UserRole::Admin | UserRole::Clerk => rsx! { clerk::ClerkDashboard {} },
        UserRole::Judge => rsx! { judge::JudgeDashboard {} },
        UserRole::Attorney => rsx! { attorney::AttorneyDashboard {} },
        UserRole::Public => rsx! { PublicDashboard {} },
    }
}

/// Minimal public dashboard with search focus.
#[component]
fn PublicDashboard() -> Element {
    rsx! {
        div { class: "dashboard-public",
            h1 { "Lexodus Public Access" }
            p { "Search public court records, opinions, and attorney information." }
        }
    }
}
```

Move the existing dashboard.rs content into `crates/app/src/routes/dashboard/clerk.rs` (renaming the main component to `ClerkDashboard`). This preserves the existing working dashboard for clerks/admins.

**Step 2: Verify it compiles**

Run: `cargo check -p app --features server`
Expected: Clean compilation.

**Step 3: Commit**

```bash
git add crates/app/src/routes/dashboard/
git commit -m "feat(ui): refactor dashboard to role-based dispatch"
```

---

### Task 7: Create Judge Dashboard

**Files:**
- Create: `crates/app/src/routes/dashboard/judge.rs`

**Step 1: Write the judge dashboard**

```rust
use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, PageHeader, PageTitle, Skeleton,
};

use crate::CourtContext;

#[component]
pub fn JudgeDashboard() -> Element {
    let ctx = use_context::<CourtContext>();
    let court = ctx.court_id.read().clone();

    // Fetch judge-specific stats
    let stats = use_resource(move || {
        let court = court.clone();
        async move {
            // Use case search to get assigned cases count
            let cases_result = server::api::search_cases(
                court.clone(), Some("active".into()), None, None, None, Some(0), Some(1),
            ).await;
            let active_cases = cases_result.ok()
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                .and_then(|v| v["pagination"]["total"].as_i64())
                .unwrap_or(0);

            let deadlines_result = server::api::search_deadlines(
                court.clone(), None, None, None, None, Some(0), Some(5),
            ).await;
            let upcoming_deadlines = deadlines_result.ok()
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                .and_then(|v| v["pagination"]["total"].as_i64())
                .unwrap_or(0);

            (active_cases, upcoming_deadlines)
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./dashboard/judge.css") }
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
                            span { class: "stat-value", "—" }
                            span { class: "stat-label", "Awaiting Ruling" }
                        }
                    }
                    Card {
                        CardHeader { "Opinion Drafts" }
                        CardContent {
                            span { class: "stat-value", "—" }
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
```

**Step 2: Add CSS**

Create `crates/app/src/routes/dashboard/judge.css`:

```css
.judge-stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: var(--space-md);
    margin-bottom: var(--space-xl);
}

.stat-value {
    font-size: var(--font-size-3xl);
    font-weight: 700;
    display: block;
}

.stat-label {
    font-size: var(--font-size-sm);
    color: var(--color-on-surface-muted);
}

.judge-quick-actions h3 {
    margin-bottom: var(--space-md);
}

.quick-action-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: var(--space-sm);
}

.quick-action-btn {
    padding: var(--space-sm) var(--space-md);
    background: var(--color-surface-raised);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    color: var(--color-on-surface);
    cursor: pointer;
    font-weight: 500;
    transition: all var(--transition-fast);
}

.quick-action-btn:hover {
    background: var(--color-primary);
    color: white;
    border-color: var(--color-primary);
}
```

**Step 3: Verify it compiles**

Run: `cargo check -p app --features server`
Expected: Clean compilation.

**Step 4: Commit**

```bash
git add crates/app/src/routes/dashboard/
git commit -m "feat(ui): add judge role dashboard with caseload stats"
```

---

### Task 8: Create Attorney Dashboard

**Files:**
- Create: `crates/app/src/routes/dashboard/attorney.rs`

**Step 1: Write the attorney dashboard**

```rust
use dioxus::prelude::*;
use shared_ui::components::{
    Card, CardContent, CardHeader, PageHeader, PageTitle, Skeleton,
};

use crate::CourtContext;

#[component]
pub fn AttorneyDashboard() -> Element {
    let ctx = use_context::<CourtContext>();
    let court = ctx.court_id.read().clone();

    let stats = use_resource(move || {
        let court = court.clone();
        async move {
            let deadlines_result = server::api::search_deadlines(
                court.clone(), None, None, None, None, Some(0), Some(5),
            ).await;
            let upcoming_deadlines = deadlines_result.ok()
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                .and_then(|v| v["pagination"]["total"].as_i64())
                .unwrap_or(0);

            let calendar_result = server::api::search_calendar_events(
                court.clone(), None, None, None, None, None, Some(0), Some(5),
            ).await;
            let upcoming_events = calendar_result.ok()
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                .and_then(|v| v["pagination"]["total"].as_i64())
                .unwrap_or(0);

            (upcoming_deadlines, upcoming_events)
        }
    });

    rsx! {
        PageHeader {
            PageTitle { "Attorney Dashboard" }
        }

        match &*stats.read() {
            Some((deadlines, events)) => rsx! {
                div { class: "judge-stats-grid",
                    Card {
                        CardHeader { "Upcoming Deadlines" }
                        CardContent {
                            span { class: "stat-value", "{deadlines}" }
                            span { class: "stat-label", "Filing Deadlines" }
                        }
                    }
                    Card {
                        CardHeader { "Court Appearances" }
                        CardContent {
                            span { class: "stat-value", "{events}" }
                            span { class: "stat-label", "Scheduled" }
                        }
                    }
                    Card {
                        CardHeader { "My Cases" }
                        CardContent {
                            span { class: "stat-value", "—" }
                            span { class: "stat-label", "Active Representations" }
                        }
                    }
                    Card {
                        CardHeader { "Recent Filings" }
                        CardContent {
                            span { class: "stat-value", "—" }
                            span { class: "stat-label", "New Docket Activity" }
                        }
                    }
                }

                div { class: "judge-quick-actions",
                    h3 { "Quick Actions" }
                    div { class: "quick-action-grid",
                        button { class: "quick-action-btn", "File Document" }
                        button { class: "quick-action-btn", "Request Extension" }
                        button { class: "quick-action-btn", "Check Deadlines" }
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
```

**Step 2: Verify it compiles**

Run: `cargo check -p app --features server`
Expected: Clean compilation.

**Step 3: Commit**

```bash
git add crates/app/src/routes/dashboard/attorney.rs
git commit -m "feat(ui): add attorney role dashboard with deadline stats"
```

---

## Phase 1C: Case Detail Hub (9-Tab Redesign)

### Task 9: Refactor Case Detail into Tabbed Hub

This is the most complex task. The existing case detail page already has some tab functionality. We'll restructure it into the 9-tab hub.

**Files:**
- Modify: `crates/app/src/routes/cases/detail.rs`
- Create: `crates/app/src/routes/cases/tabs/mod.rs`
- Create: `crates/app/src/routes/cases/tabs/overview.rs`
- Create: `crates/app/src/routes/cases/tabs/parties.rs`
- Create: `crates/app/src/routes/cases/tabs/deadlines.rs`
- Create: `crates/app/src/routes/cases/tabs/orders.rs`
- Create: `crates/app/src/routes/cases/tabs/sentencing.rs`
- Create: `crates/app/src/routes/cases/tabs/evidence.rs`
- Create: `crates/app/src/routes/cases/tabs/calendar_tab.rs`
- Create: `crates/app/src/routes/cases/tabs/speedy_trial.rs`

**Step 1: Create tabs module**

**`crates/app/src/routes/cases/tabs/mod.rs`:**
```rust
pub mod overview;
pub mod parties;
pub mod deadlines;
pub mod orders;
pub mod sentencing;
pub mod evidence;
pub mod calendar_tab;
pub mod speedy_trial;
```

**Step 2: Create Overview tab**

**`crates/app/src/routes/cases/tabs/overview.rs`:**
```rust
use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, Separator,
};

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
    rsx! {
        div { class: "case-overview",
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
                            Badge {
                                variant: match status.as_str() {
                                    "active" => BadgeVariant::Primary,
                                    "closed" => BadgeVariant::Secondary,
                                    _ => BadgeVariant::Secondary,
                                },
                                "{status}"
                            }
                        }
                        div { class: "overview-item",
                            span { class: "overview-label", "Crime Type" }
                            span { class: "overview-value", "{crime_type}" }
                        }
                        div { class: "overview-item",
                            span { class: "overview-label", "District" }
                            span { class: "overview-value", "{district}" }
                        }
                        div { class: "overview-item",
                            span { class: "overview-label", "Priority" }
                            Badge {
                                variant: match priority.as_str() {
                                    "high" => BadgeVariant::Destructive,
                                    "medium" => BadgeVariant::Primary,
                                    _ => BadgeVariant::Secondary,
                                },
                                "{priority}"
                            }
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
        }
    }
}
```

**Step 3: Create Parties tab stub**

**`crates/app/src/routes/cases/tabs/parties.rs`:**
```rust
use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader, Skeleton};

use crate::CourtContext;

#[component]
pub fn PartiesTab(case_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let parties = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move {
            server::api::list_case_parties(court, cid).await.ok()
        }
    });

    rsx! {
        match &*parties.read() {
            Some(Some(json)) => rsx! {
                Card {
                    CardHeader { "Case Parties" }
                    CardContent {
                        p { "Party data loaded. Full table coming soon." }
                        pre { class: "debug-json", "{json}" }
                    }
                }
            },
            Some(None) => rsx! {
                Card {
                    CardContent { p { "No parties found for this case." } }
                }
            },
            None => rsx! {
                Skeleton { width: "100%", height: "200px" }
            },
        }
    }
}
```

**Step 4: Create remaining tab stubs**

Create placeholder tabs for each remaining tab. Each follows the same pattern — a simple component that takes `case_id: String` and shows a placeholder.

**`crates/app/src/routes/cases/tabs/deadlines.rs`:**
```rust
use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader};

#[component]
pub fn DeadlinesTab(case_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { "Case Deadlines" }
            CardContent {
                p { "Deadline tracking for case {case_id} — coming soon." }
            }
        }
    }
}
```

**`crates/app/src/routes/cases/tabs/orders.rs`:**
```rust
use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader};

#[component]
pub fn OrdersTab(case_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { "Court Orders" }
            CardContent {
                p { "Orders for case {case_id} — coming soon." }
            }
        }
    }
}
```

**`crates/app/src/routes/cases/tabs/sentencing.rs`:**
```rust
use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader};

#[component]
pub fn SentencingTab(case_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { "Sentencing" }
            CardContent {
                p { "Sentencing data for case {case_id} — coming soon." }
            }
        }
    }
}
```

**`crates/app/src/routes/cases/tabs/evidence.rs`:**
```rust
use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader};

#[component]
pub fn EvidenceTab(case_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { "Evidence" }
            CardContent {
                p { "Evidence exhibit list for case {case_id} — coming soon." }
            }
        }
    }
}
```

**`crates/app/src/routes/cases/tabs/calendar_tab.rs`:**
```rust
use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader};

#[component]
pub fn CalendarTab(case_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { "Case Calendar" }
            CardContent {
                p { "Scheduled events for case {case_id} — coming soon." }
            }
        }
    }
}
```

**`crates/app/src/routes/cases/tabs/speedy_trial.rs`:**
```rust
use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader};

#[component]
pub fn SpeedyTrialTab(case_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { "Speedy Trial Clock" }
            CardContent {
                p { "Speedy trial tracking for case {case_id} — coming soon." }
            }
        }
    }
}
```

**Step 5: Update cases/mod.rs**

Add `pub mod tabs;` to `crates/app/src/routes/cases/mod.rs`.

**Step 6: Refactor case detail.rs to use tabs**

The existing `detail.rs` already has some tab infrastructure. Modify it to use 9 tabs with the new tab components. Replace the tab section in `CaseDetailView` with:

```rust
use super::tabs::{
    overview::OverviewTab,
    parties::PartiesTab,
    deadlines::DeadlinesTab,
    orders::OrdersTab,
    sentencing::SentencingTab,
    evidence::EvidenceTab,
    calendar_tab::CalendarTab,
    speedy_trial::SpeedyTrialTab,
};
```

And use this tab structure in the detail view:

```rust
Tabs { default_value: "overview", horizontal: true,
    TabList {
        TabTrigger { value: "overview", "Overview" }
        TabTrigger { value: "docket", "Docket" }
        TabTrigger { value: "parties", "Parties" }
        TabTrigger { value: "deadlines", "Deadlines" }
        TabTrigger { value: "orders", "Orders" }
        TabTrigger { value: "sentencing", "Sentencing" }
        TabTrigger { value: "evidence", "Evidence" }
        TabTrigger { value: "calendar", "Calendar" }
        TabTrigger { value: "speedy-trial", "Speedy Trial" }
    }
    TabContent { value: "overview",
        OverviewTab {
            case_id: case_id.clone(),
            title: case.title.clone().unwrap_or_default(),
            case_number: case.case_number.clone().unwrap_or_default(),
            status: case.status.clone().unwrap_or_default(),
            crime_type: case.crime_type.clone().unwrap_or_default(),
            district: case.district_code.clone().unwrap_or_default(),
            priority: case.priority.clone().unwrap_or_default(),
            description: case.description.clone().unwrap_or_default(),
        }
    }
    TabContent { value: "docket",
        // Keep existing DocketTab component
        DocketTab { case_id: case_id.clone() }
    }
    TabContent { value: "parties",
        PartiesTab { case_id: case_id.clone() }
    }
    TabContent { value: "deadlines",
        DeadlinesTab { case_id: case_id.clone() }
    }
    TabContent { value: "orders",
        OrdersTab { case_id: case_id.clone() }
    }
    TabContent { value: "sentencing",
        SentencingTab { case_id: case_id.clone() }
    }
    TabContent { value: "evidence",
        EvidenceTab { case_id: case_id.clone() }
    }
    TabContent { value: "calendar",
        CalendarTab { case_id: case_id.clone() }
    }
    TabContent { value: "speedy-trial",
        SpeedyTrialTab { case_id: case_id.clone() }
    }
}
```

Note: The existing DocketTab component in detail.rs should be preserved — it already fetches and displays docket entries. The other tabs start as stubs and will be fleshed out in subsequent plans.

**Step 7: Add case hub CSS**

Add to existing case CSS or create `crates/app/src/routes/cases/detail.css`:

```css
.case-overview {
    padding: var(--space-md) 0;
}

.overview-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--space-md);
}

.overview-item {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
}

.overview-label {
    font-size: var(--font-size-sm);
    color: var(--color-on-surface-muted);
    font-weight: 500;
}

.overview-value {
    font-size: var(--font-size-base);
    color: var(--color-on-surface);
}

.overview-description h4 {
    margin-bottom: var(--space-sm);
}

.debug-json {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    max-height: 300px;
    overflow-y: auto;
    background: var(--color-surface);
    padding: var(--space-sm);
    border-radius: var(--radius);
    white-space: pre-wrap;
    word-break: break-all;
}
```

**Step 8: Verify it compiles**

Run: `cargo check -p app --features server`
Expected: Clean compilation.

**Step 9: Commit**

```bash
git add crates/app/src/routes/cases/
git commit -m "feat(ui): refactor case detail into 9-tab hub with overview and stub tabs"
```

---

## Summary of What This Plan Delivers

After completing Tasks 1-9:

1. **Sidebar restructured** into 6 role-gated groups (Core, Case Management, Court Operations, Legal Documents, People & Orgs, Administration)
2. **Role visibility system** — sidebar shows/hides groups based on user role
3. **14 new domain routes** registered and navigable with stub pages
4. **Notification bell** in the navbar (UI only, backend later)
5. **Command palette** (Cmd+K style) for quick navigation to any domain
6. **Role-adaptive dashboard** dispatching to Clerk/Judge/Attorney/Public dashboards
7. **Judge dashboard** with caseload stats and quick actions
8. **Attorney dashboard** with deadline stats and quick actions
9. **Case detail 9-tab hub** — Overview, Docket (existing), Parties, Deadlines, Orders, Sentencing, Evidence, Calendar, Speedy Trial

## Next Plans

- **Phase 2:** Flesh out case hub tabs — full DataTable + Sheet for each tab (Parties, Deadlines, Orders, Evidence, etc.)
- **Phase 3:** Judge detail hub (7 tabs) + Attorney detail hub (7 tabs)
- **Phase 4:** Court Operations domain pages (Docket, Filings, Service Records)
- **Phase 5:** Legal Documents domain pages (Orders, Opinions, Documents)
- **Phase 6:** Sentencing module (Guidelines Calculator, Offense Level Builder)
- **Phase 7:** Administration pages + Workflow Wizards
