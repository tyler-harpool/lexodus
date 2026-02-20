# Tab Consolidation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Restructure 10 case detail tabs into 5 (Overview, Docket, Parties, Scheduling, Sentencing) with subtabs for Docket and Scheduling.

**Architecture:** Extract the 2,100-line docket system from `detail.rs` into `tabs/docket.rs`. Merge Overview + CaseInfo into a single Overview tab. Create two subtab container components for Docket (Entries/Orders/Evidence) and Scheduling (Calendar/Deadlines/Speedy Trial). Rewire `detail.rs` to 5 main tabs.

**Tech Stack:** Dioxus 0.7, shared-ui `Tabs`/`TabList`/`TabTrigger`/`TabContent` components (nested for subtabs).

---

### Task 1: Extract DocketTab and all helpers from detail.rs into tabs/docket.rs

This is the biggest task. Move lines 310–2357 of `detail.rs` (DocketTab, EventComposer, DocketEntryForm, FilingForm, DocketTable, DocketRow, EntryDetailPanel, FilesSection, FilingNoticeSection, ServiceSection, ServiceRecordTableRow, ServiceRecordForm, ServiceRecordRow, AttachmentRow, DocumentActionsSection, NefModal) plus helpers (`format_file_size`, `mime_from_filename`) into a new file.

**Files:**
- Create: `crates/app/src/routes/cases/tabs/docket.rs`
- Modify: `crates/app/src/routes/cases/tabs/mod.rs` — add `pub mod docket;`
- Modify: `crates/app/src/routes/cases/detail.rs` — remove extracted code

**Step 1: Create `tabs/docket.rs`**

Create the new file with all docket components extracted from `detail.rs`. The file needs its own imports since it's a new module. Make `DocketTab` public (`pub fn DocketTab`).

Required imports for `docket.rs`:
```rust
use dioxus::prelude::*;
use shared_types::{
    DocketAttachmentResponse, DocketEntryResponse, DocketSearchResponse,
    DocumentResponse, NefResponse, ServiceRecordResponse, UserRole, VALID_DOCUMENT_TYPES,
};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardHeader,
    DataTable, DataTableBody, DataTableCell, DataTableColumn, DataTableHeader, DataTableRow,
    FormSelect, Input, Separator, Skeleton, Textarea,
};
use shared_ui::{use_toast, ToastOptions};

use crate::auth::{use_user_role, UserRole as _};
use crate::CourtContext;
```

The following components move to this file (all `fn`, make `DocketTab` `pub fn`, rest stay private `fn`):
- `pub fn DocketTab(case_id: String)` — lines 310–383
- `fn EventComposer(...)` — lines 388–830
- `fn NefModal(...)` — lines 833–855
- `fn DocketEntryForm(...)` — lines 856–976
- `fn FilingForm(...)` — lines 979–1223
- `fn DocketTable(...)` — lines 1224–1270
- `fn DocketRow(...)` — lines 1271–1295
- `fn EntryDetailPanel(...)` — lines 1299–1475
- `fn FilesSection(...)` — lines 1478–1538
- `fn FilingNoticeSection(...)` — lines 1541–1604
- `fn ServiceSection(...)` — lines 1607–1705
- `fn ServiceRecordTableRow(...)` — lines 1709–1759
- `fn ServiceRecordForm(...)` — lines 1762–1909
- `fn ServiceRecordRow(...)` — lines 1912–1970
- `fn AttachmentRow(...)` — lines 1974–2063
- `fn DocumentActionsSection(...)` — lines 2067–2357
- `fn format_file_size(...)` — lines 2359–2367
- `fn mime_from_filename(...)` — lines 2369–2384

**Step 2: Update `tabs/mod.rs`**

Add `pub mod docket;` to the module declarations.

**Step 3: Remove extracted code from `detail.rs`**

Delete lines 310–2384 from `detail.rs` (everything from `fn DocketTab` through `fn mime_from_filename`). Keep `CaseInfoDisplay` (lines 209–307) for now — it will be merged in Task 2. Keep the remaining helpers at the bottom (`status_badge_variant`, `priority_badge_variant`, `format_date`).

Update `detail.rs` imports:
- Remove docket-specific types: `DocketAttachmentResponse`, `DocketEntryResponse`, `DocketSearchResponse`, `DocumentResponse`, `NefResponse`, `ServiceRecordResponse`, `VALID_DOCUMENT_TYPES`
- Remove docket-specific UI components no longer used in detail.rs: `DataTable*`, `FormSelect`, `Input`, `Textarea`
- Add docket tab import: `use super::tabs::docket::DocketTab;`
- The `TabContent { value: "docket" }` in CaseDetailView already points to `DocketTab { case_id }` — no change needed there since `DocketTab` was local before and now comes from the import.

Wait — the `DocketTab` was an inline `fn` in `detail.rs`, not imported from tabs. The existing tab structure (line 179) calls `DocketTab { case_id: id.clone() }` which referenced the local `fn DocketTab`. After extraction, it needs to be imported. But that import line already imports other tabs from `super::tabs::*` — just add `docket::DocketTab` to that import block.

**Step 4: Compile check**

Run: `cargo check -p app`
Expected: Clean compilation (possibly with pre-existing warnings).

**Step 5: Commit**

```
git add crates/app/src/routes/cases/tabs/docket.rs crates/app/src/routes/cases/tabs/mod.rs crates/app/src/routes/cases/detail.rs
git commit -m "refactor: extract DocketTab and helpers from detail.rs into tabs/docket.rs"
```

---

### Task 2: Merge Overview + CaseInfo into unified OverviewTab

Combine the current OverviewTab (case summary, judge, activity) with CaseInfoDisplay metadata (case number, status, type, district, priority, opened/closed dates, sealed status) into one component.

**Files:**
- Modify: `crates/app/src/routes/cases/tabs/overview.rs`
- Modify: `crates/app/src/routes/cases/detail.rs` — remove CaseInfoDisplay, update OverviewTab call

**Step 1: Rewrite `tabs/overview.rs`**

The merged OverviewTab takes `case_item: CaseResponse` and `case_id: String` as props (instead of individual string props). It combines:

1. **Case Details card** — case number, crime type, district, location, status badge, priority badge, sealed badge (from CaseInfoDisplay)
2. **Timing & Assignment card** — opened date, assigned judge with tooltip, closed date, updated date (from CaseInfoDisplay)
3. **Description card** — if description is non-empty (from CaseInfoDisplay)
4. **Recent Activity card** — timeline fetched from `get_case_timeline` (from current OverviewTab)

New imports needed:
```rust
use shared_types::CaseResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle,
    DetailFooter, DetailGrid, DetailItem, DetailList,
    Separator, Skeleton, Tooltip, TooltipContent, TooltipTrigger,
};
```

New signature:
```rust
#[component]
pub fn OverviewTab(case_item: CaseResponse) -> Element {
```

The case_id is available as `case_item.id`.

Copy the `status_badge_variant`, `priority_badge_variant`, and `format_date` helpers into `overview.rs` (or keep them in `detail.rs` and make them `pub(super)` — but since they're small, duplicating into overview is cleaner).

**Step 2: Update detail.rs OverviewTab call**

Change from:
```rust
TabContent { value: "overview", index: 0usize,
    OverviewTab {
        case_id: id.clone(),
        title: case_item.title.clone(),
        case_number: case_item.case_number.clone(),
        status: case_item.status.clone(),
        crime_type: case_item.crime_type.clone(),
        district: case_item.district_code.clone(),
        priority: case_item.priority.clone(),
        description: case_item.description.clone(),
    }
}
```

To:
```rust
TabContent { value: "overview", index: 0usize,
    OverviewTab { case_item: case_item.clone() }
}
```

**Step 3: Remove CaseInfoDisplay from detail.rs**

Delete the entire `CaseInfoDisplay` component (lines ~209–307) and its TabContent entry. Also remove the "info" TabTrigger.

**Step 4: Remove `overview.css` stylesheet if content was only used by old Overview**

Check if `overview.css` styles (`case-overview`, `overview-grid`, `overview-item`, `overview-label`, `overview-value`, `overview-description`) should be kept or if the merged version uses DetailGrid/DetailItem components instead. If we use the DetailGrid pattern from CaseInfoDisplay, the CSS file can be deleted. If we keep the card-based layout, keep the CSS.

Recommendation: Use the `DetailGrid` + `DetailList` + `DetailItem` pattern from CaseInfoDisplay for metadata, plus the card-based layout for judge + activity. This gives a clean, structured look. Keep `overview.css` for the activity timeline styling.

**Step 5: Compile check**

Run: `cargo check -p app`

**Step 6: Commit**

```
git add crates/app/src/routes/cases/tabs/overview.rs crates/app/src/routes/cases/detail.rs
git commit -m "refactor: merge Overview and CaseInfo into unified OverviewTab"
```

---

### Task 3: Create Scheduling subtab container

**Files:**
- Create: `crates/app/src/routes/cases/tabs/scheduling.rs`
- Modify: `crates/app/src/routes/cases/tabs/mod.rs` — add `pub mod scheduling;`

**Step 1: Create `tabs/scheduling.rs`**

```rust
use dioxus::prelude::*;
use shared_ui::components::{TabContent, TabList, TabTrigger, Tabs};

use super::calendar_tab::CalendarTab;
use super::deadlines::DeadlinesTab;
use super::speedy_trial::SpeedyTrialTab;

#[component]
pub fn SchedulingTab(case_id: String) -> Element {
    rsx! {
        Tabs { default_value: "calendar", horizontal: true,
            TabList {
                TabTrigger { value: "calendar", index: 0usize, "Calendar" }
                TabTrigger { value: "deadlines", index: 1usize, "Deadlines" }
                TabTrigger { value: "speedy-trial", index: 2usize, "Speedy Trial" }
            }
            TabContent { value: "calendar", index: 0usize,
                CalendarTab { case_id: case_id.clone() }
            }
            TabContent { value: "deadlines", index: 1usize,
                DeadlinesTab { case_id: case_id.clone() }
            }
            TabContent { value: "speedy-trial", index: 2usize,
                SpeedyTrialTab { case_id: case_id.clone() }
            }
        }
    }
}
```

**Step 2: Update `tabs/mod.rs`**

Add `pub mod scheduling;`

**Step 3: Compile check**

Run: `cargo check -p app`

**Step 4: Commit**

```
git add crates/app/src/routes/cases/tabs/scheduling.rs crates/app/src/routes/cases/tabs/mod.rs
git commit -m "feat: add SchedulingTab container with Calendar/Deadlines/Speedy Trial subtabs"
```

---

### Task 4: Create Docket subtab container

**Files:**
- Create: `crates/app/src/routes/cases/tabs/docket_container.rs`
- Modify: `crates/app/src/routes/cases/tabs/mod.rs` — add `pub mod docket_container;`

**Step 1: Create `tabs/docket_container.rs`**

```rust
use dioxus::prelude::*;
use shared_ui::components::{TabContent, TabList, TabTrigger, Tabs};

use super::docket::DocketTab;
use super::evidence::EvidenceTab;
use super::orders::OrdersTab;

#[component]
pub fn DocketContainerTab(case_id: String) -> Element {
    rsx! {
        Tabs { default_value: "entries", horizontal: true,
            TabList {
                TabTrigger { value: "entries", index: 0usize, "Entries" }
                TabTrigger { value: "orders", index: 1usize, "Orders" }
                TabTrigger { value: "evidence", index: 2usize, "Evidence" }
            }
            TabContent { value: "entries", index: 0usize,
                DocketTab { case_id: case_id.clone() }
            }
            TabContent { value: "orders", index: 1usize,
                OrdersTab { case_id: case_id.clone() }
            }
            TabContent { value: "evidence", index: 2usize,
                EvidenceTab { case_id: case_id.clone() }
            }
        }
    }
}
```

**Step 2: Update `tabs/mod.rs`**

Add `pub mod docket_container;`

**Step 3: Compile check**

Run: `cargo check -p app`

**Step 4: Commit**

```
git add crates/app/src/routes/cases/tabs/docket_container.rs crates/app/src/routes/cases/tabs/mod.rs
git commit -m "feat: add DocketContainerTab with Entries/Orders/Evidence subtabs"
```

---

### Task 5: Rewire detail.rs to 5 main tabs

**Files:**
- Modify: `crates/app/src/routes/cases/detail.rs`

**Step 1: Update imports in detail.rs**

Replace the tab imports:
```rust
use super::tabs::{
    calendar_tab::CalendarTab, deadlines::DeadlinesTab, evidence::EvidenceTab, orders::OrdersTab,
    overview::OverviewTab, parties::PartiesTab, sentencing::SentencingTab,
    speedy_trial::SpeedyTrialTab,
};
```

With:
```rust
use super::tabs::{
    docket_container::DocketContainerTab,
    overview::OverviewTab,
    parties::PartiesTab,
    scheduling::SchedulingTab,
    sentencing::SentencingTab,
};
```

Also trim the shared_types import to only what detail.rs still needs (just `CaseResponse` and `UserRole`):
```rust
use shared_types::{CaseResponse, UserRole};
```

Trim the shared_ui import to only what's still used in the page header, delete confirmation, and tab shell:
```rust
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
    Button, ButtonVariant, Card, CardContent,
    PageActions, PageHeader, PageTitle, Skeleton,
    TabContent, TabList, TabTrigger, Tabs,
};
```

**Step 2: Replace 10-tab TabList with 5-tab TabList**

In `CaseDetailView`, replace the Tabs block (lines ~151–203) with:
```rust
Tabs { default_value: "overview", horizontal: true,
    TabList {
        TabTrigger { value: "overview", index: 0usize, "Overview" }
        TabTrigger { value: "docket", index: 1usize, "Docket" }
        TabTrigger { value: "parties", index: 2usize, "Parties" }
        TabTrigger { value: "scheduling", index: 3usize, "Scheduling" }
        TabTrigger { value: "sentencing", index: 4usize, "Sentencing" }
    }
    TabContent { value: "overview", index: 0usize,
        OverviewTab { case_item: case_item.clone() }
    }
    TabContent { value: "docket", index: 1usize,
        DocketContainerTab { case_id: id.clone() }
    }
    TabContent { value: "parties", index: 2usize,
        PartiesTab { case_id: id.clone() }
    }
    TabContent { value: "scheduling", index: 3usize,
        SchedulingTab { case_id: id.clone() }
    }
    TabContent { value: "sentencing", index: 4usize,
        SentencingTab { case_id: id.clone() }
    }
}
```

**Step 3: Remove unused helpers from detail.rs**

If `status_badge_variant`, `priority_badge_variant`, and `format_date` are no longer used in detail.rs (they were used by CaseInfoDisplay which is now gone), delete them. They should now live in `overview.rs`.

**Step 4: Compile check**

Run: `cargo check -p app`

**Step 5: Run tests**

Run: `cargo test -p tests -- --test-threads=1`
Expected: All 388 tests pass (this is a UI-only refactor, no server changes).

**Step 6: Commit**

```
git add crates/app/src/routes/cases/detail.rs
git commit -m "refactor: rewire case detail to 5-tab layout (Overview, Docket, Parties, Scheduling, Sentencing)"
```

---

### Task 6: Verify and clean up

**Step 1: Check for dead code warnings**

Run: `cargo check -p app 2>&1 | grep warning`

Fix any unused import or dead code warnings introduced by the refactor.

**Step 2: Verify line counts**

- `detail.rs` should be ~200 lines (down from 2,413)
- `tabs/docket.rs` should be ~2,100 lines
- `tabs/overview.rs` should be ~250 lines
- `tabs/scheduling.rs` should be ~30 lines
- `tabs/docket_container.rs` should be ~30 lines

**Step 3: Final compile + test**

Run: `cargo check -p app && cargo test -p tests -- --test-threads=1`

**Step 4: Commit any cleanup**

```
git add -u
git commit -m "chore: clean up unused imports and dead code after tab consolidation"
```
