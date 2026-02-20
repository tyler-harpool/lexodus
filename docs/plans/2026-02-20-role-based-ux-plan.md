# Role-Based UX Overhaul — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make every page role-aware — gate actions by permission, filter lists to "my items" for judges/attorneys, and replace placeholder dashboards with actionable work lists.

**Architecture:** Wire the existing `can(role, action)` function into every page (it's implemented but never called). Add `case_number` to response types via LEFT JOINs (proven pattern from `order.rs`). Rebuild judge/attorney dashboards with real filtered queries. Add "My Items / All Court" toggle for list pages.

**Tech Stack:** Dioxus 0.7 (Rust), Axum, PostgreSQL, sqlx, shared-types crate

---

## Phase 1: Permission Gating + Route Cleanup (UI-only)

### Task 1: Expand Action Enum + Permission Matrix

**Files:**
- Modify: `crates/app/src/auth.rs:119-143`

**Step 1: Replace the Action enum and can() function**

Replace lines 119-143 of `crates/app/src/auth.rs` with:

```rust
/// Actions that can be role-gated in the UI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    // Court record management (clerk/admin)
    ManageAttorneys,
    ManageJudges,
    ManageRules,

    // Case workflow
    CreateCase,
    EditCase,
    DeleteCase,

    // Docket & Filing
    CreateDocketEntry,
    FileFiling,

    // Judicial actions
    SignOrder,
    IssueOrder,
    DraftOpinion,

    // Document control
    SealDocument,
    StrikeDocument,

    // Evidence & Sentencing
    ManageEvidence,
    EnterSentencing,

    // Universal
    GeneratePdf,
}

/// Check if a role is permitted to perform an action.
pub fn can(role: &UserRole, action: Action) -> bool {
    match action {
        // Clerk/Admin only
        Action::ManageAttorneys | Action::ManageJudges | Action::ManageRules => {
            matches!(role, UserRole::Clerk | UserRole::Admin)
        }
        Action::CreateCase | Action::CreateDocketEntry => {
            matches!(role, UserRole::Clerk | UserRole::Admin)
        }
        // Admin only
        Action::DeleteCase => matches!(role, UserRole::Admin),
        // Clerk/Admin/Judge
        Action::EditCase | Action::EnterSentencing => {
            matches!(role, UserRole::Clerk | UserRole::Judge | UserRole::Admin)
        }
        // Attorney/Clerk/Admin
        Action::FileFiling => {
            matches!(role, UserRole::Attorney | UserRole::Clerk | UserRole::Admin)
        }
        // Judge/Admin
        Action::SignOrder | Action::DraftOpinion => {
            matches!(role, UserRole::Judge | UserRole::Admin)
        }
        // Clerk/Admin
        Action::IssueOrder => matches!(role, UserRole::Clerk | UserRole::Admin),
        // Judge/Clerk/Admin
        Action::SealDocument | Action::StrikeDocument => {
            matches!(role, UserRole::Judge | UserRole::Clerk | UserRole::Admin)
        }
        Action::ManageEvidence => {
            matches!(role, UserRole::Clerk | UserRole::Admin)
        }
        // All except public
        Action::GeneratePdf => !matches!(role, UserRole::Public),
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check -p app`
Expected: PASS (no other files reference the old Action variants yet)

**Step 3: Commit**

```bash
git add crates/app/src/auth.rs
git commit -m "feat: expand Action enum with entity-scoped permissions"
```

---

### Task 2: Gate Buttons on List Pages

**Files:**
- Modify: `crates/app/src/routes/cases/list.rs`
- Modify: `crates/app/src/routes/attorneys/list.rs`
- Modify: `crates/app/src/routes/judges/list.rs`
- Modify: `crates/app/src/routes/calendar/list.rs`
- Modify: `crates/app/src/routes/deadlines/list.rs`
- Modify: `crates/app/src/routes/opinions/list.rs`
- Modify: `crates/app/src/routes/rules/list.rs`

**Step 1: Add permission gating to each list page**

For each file, add these imports at the top (if not present):

```rust
use crate::auth::{can, use_user_role, Action};
```

Then add `let role = use_user_role();` near the top of the component function, and wrap action buttons with `if can(&role, Action::X)`.

**Pattern for each page:**

| File | Find button | Wrap with |
|------|------------|-----------|
| `cases/list.rs` | "New Case" Button | `if can(&role, Action::CreateCase)` |
| `attorneys/list.rs` | "New Attorney" Button | `if can(&role, Action::ManageAttorneys)` |
| `judges/list.rs` | "New Judge" Button | `if can(&role, Action::ManageJudges)` |
| `calendar/list.rs` | "Schedule Event" Button | `if can(&role, Action::CreateCase)` |
| `deadlines/list.rs` | "New Deadline" Button | `if can(&role, Action::CreateCase)` |
| `opinions/list.rs` | "New Opinion" Button | `if can(&role, Action::DraftOpinion)` |
| `rules/list.rs` | "New Rule" Button | `if can(&role, Action::ManageRules)` |

**Example transformation (cases/list.rs):**

Before:
```rust
PageActions {
    Button { onclick: move |_| show_sheet.set(true), "New Case" }
}
```

After:
```rust
PageActions {
    if can(&role, Action::CreateCase) {
        Button { onclick: move |_| show_sheet.set(true), "New Case" }
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check -p app`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/app/src/routes/cases/list.rs crates/app/src/routes/attorneys/list.rs crates/app/src/routes/judges/list.rs crates/app/src/routes/calendar/list.rs crates/app/src/routes/deadlines/list.rs crates/app/src/routes/opinions/list.rs crates/app/src/routes/rules/list.rs
git commit -m "feat: gate list page action buttons by role"
```

---

### Task 3: Gate Buttons on Detail/Tab Pages

**Files:**
- Modify: `crates/app/src/routes/cases/detail.rs`
- Modify: `crates/app/src/routes/cases/tabs/docket.rs`
- Modify: `crates/app/src/routes/cases/tabs/orders.rs`
- Modify: `crates/app/src/routes/cases/tabs/evidence.rs`
- Modify: `crates/app/src/routes/cases/tabs/parties.rs`
- Modify: `crates/app/src/routes/cases/tabs/sentencing.rs`
- Modify: `crates/app/src/routes/attorneys/detail.rs`
- Modify: `crates/app/src/routes/judges/detail.rs`

**Step 1: Gate buttons on case detail and tabs**

Same pattern as Task 2. Add imports, get role, wrap buttons:

| File | Button | Action |
|------|--------|--------|
| `cases/detail.rs` | Edit button | `EditCase` |
| `cases/detail.rs` | Delete button | `DeleteCase` |
| `cases/tabs/docket.rs` | New Entry button | `CreateDocketEntry` |
| `cases/tabs/docket.rs` | File button | `FileFiling` |
| `cases/tabs/orders.rs` | Draft Order button | `EditCase` |
| `cases/tabs/orders.rs` | Sign button | `SignOrder` |
| `cases/tabs/orders.rs` | Issue button | `IssueOrder` |
| `cases/tabs/evidence.rs` | Add Evidence button | `ManageEvidence` |
| `cases/tabs/parties.rs` | Add Party button | `CreateCase` |
| `cases/tabs/sentencing.rs` | Enter Sentencing button | `EnterSentencing` |
| `attorneys/detail.rs` | Edit/Delete buttons | `ManageAttorneys` |
| `judges/detail.rs` | Edit/Delete buttons | `ManageJudges` |

**Step 2: Verify compilation**

Run: `cargo check -p app`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/app/src/routes/cases/ crates/app/src/routes/attorneys/detail.rs crates/app/src/routes/judges/detail.rs
git commit -m "feat: gate detail/tab page action buttons by role"
```

---

### Task 4: Delete Deprecated Create Routes

**Files:**
- Delete: `crates/app/src/routes/cases/create.rs`
- Delete: `crates/app/src/routes/attorneys/create.rs`
- Delete: `crates/app/src/routes/calendar/create.rs`
- Delete: `crates/app/src/routes/deadlines/create.rs`
- Modify: `crates/app/src/routes/mod.rs` (remove route variants + component functions)
- Modify: `crates/app/src/routes/cases/mod.rs` (remove `pub mod create;`)
- Modify: `crates/app/src/routes/attorneys/mod.rs` (remove `pub mod create;`)
- Modify: `crates/app/src/routes/calendar/mod.rs` (remove `pub mod create;`)
- Modify: `crates/app/src/routes/deadlines/mod.rs` (remove `pub mod create;`)

**Step 1: Remove route variants from Route enum**

In `crates/app/src/routes/mod.rs`, delete these lines:
- `#[route("/attorneys/new")] AttorneyCreate {},`
- `#[route("/calendar/new")] CalendarCreate {},`
- `#[route("/cases/new")] CaseCreate {},`
- `#[route("/deadlines/new")] DeadlineCreate {},`

Also delete the component wrapper functions:
- `fn AttorneyCreate() -> Element`
- `fn CalendarCreate() -> Element`
- `fn CaseCreate() -> Element`
- `fn DeadlineCreate() -> Element`

Remove them from the `page_title` match arm as well.

Remove the `use` import for these if present.

**Step 2: Remove module declarations**

In each `mod.rs` file, remove the `pub mod create;` line:
- `crates/app/src/routes/cases/mod.rs`
- `crates/app/src/routes/attorneys/mod.rs`
- `crates/app/src/routes/calendar/mod.rs`
- `crates/app/src/routes/deadlines/mod.rs`

**Step 3: Delete the create files**

Delete these 4 files:
- `crates/app/src/routes/cases/create.rs`
- `crates/app/src/routes/attorneys/create.rs`
- `crates/app/src/routes/calendar/create.rs`
- `crates/app/src/routes/deadlines/create.rs`

**Step 4: Verify compilation**

Run: `cargo check -p app`
Expected: PASS (form sheets handle all create flows now)

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: remove deprecated standalone create pages"
```

---

### Task 5: Role-Adaptive Sidebar Label

**Files:**
- Modify: `crates/app/src/routes/mod.rs:262-278` (sidebar "My Work" section)

**Step 1: Change sidebar label based on role**

In the AppLayout component, the sidebar currently shows "Queue" for the dashboard link. Change it to be role-adaptive:

```rust
// In the My Work section, replace the static "Queue" label:
// Before:
SidebarMenuButton { active: matches!(route, Route::Dashboard {}),
    Icon::<LdLayoutDashboard> { ... }
    "Queue"
}

// After:
SidebarMenuButton { active: matches!(route, Route::Dashboard {}),
    Icon::<LdLayoutDashboard> { ... }
    {match use_user_role() {
        UserRole::Admin | UserRole::Clerk => "Queue",
        _ => "Dashboard",
    }}
}
```

Also update `page_title` mapping (line 202):
```rust
Route::Dashboard {} => match use_user_role() {
    UserRole::Admin | UserRole::Clerk => "Queue",
    _ => "Dashboard",
},
```

**Step 2: Verify compilation**

Run: `cargo check -p app`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/app/src/routes/mod.rs
git commit -m "feat: role-adaptive sidebar label (Queue vs Dashboard)"
```

---

### Task 6: Add Identity Links to AuthUser

**Files:**
- Modify: `crates/shared-types/src/models.rs` (AuthUser struct)
- Modify: `crates/server/src/api/auth.rs` (fetch_auth_user function)

**Step 1: Add fields to AuthUser**

In `crates/shared-types/src/models.rs`, add to the AuthUser struct (after `preferred_court_id`):

```rust
    /// UUID of the judge record linked to this user (matched by email).
    #[serde(default)]
    pub linked_judge_id: Option<String>,
    /// UUID of the attorney record linked to this user (matched by email).
    #[serde(default)]
    pub linked_attorney_id: Option<String>,
```

**Step 2: Populate during auth check**

In `crates/server/src/api/auth.rs`, in the `fetch_auth_user` function, after loading court_tiers but before building AuthUser, add lookups:

```rust
    // Resolve linked judge/attorney by email
    let linked_judge_id: Option<String> = sqlx::query_scalar(
        "SELECT id::TEXT FROM judges WHERE court_id = ANY($1) AND email = $2 LIMIT 1"
    )
    .bind(&court_ids)
    .bind(&u.email)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let linked_attorney_id: Option<String> = sqlx::query_scalar(
        "SELECT id::TEXT FROM attorneys WHERE court_id = ANY($1) AND email = $2 LIMIT 1"
    )
    .bind(&court_ids)
    .bind(&u.email)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
```

Then add the fields to the AuthUser construction:

```rust
    linked_judge_id,
    linked_attorney_id,
```

**Step 3: Verify compilation**

Run: `cargo check -p server -p app`
Expected: PASS

**Step 4: Write test**

Add to `crates/tests/src/lib.rs` or a new test file:

```rust
#[tokio::test]
async fn test_linked_judge_id_populated_for_judge_user() {
    let (app, pool, _lock) = test_app().await;
    // Create a judge with a known email
    let judge_id = create_test_judge(&pool, "district9", "Judge Test").await;
    // Update judge email to match test user email
    sqlx::query("UPDATE judges SET email = 'test@example.com' WHERE id = $1")
        .bind(judge_id)
        .execute(&pool)
        .await
        .unwrap();
    // Call get_current_user and verify linked_judge_id is populated
    // (depends on auth flow — may need to simulate authenticated request)
}
```

**Step 5: Verify tests**

Run: `cargo test -p tests -- --test-threads=1`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/shared-types/src/models.rs crates/server/src/api/auth.rs crates/tests/src/
git commit -m "feat: add linked_judge_id and linked_attorney_id to AuthUser"
```

---

## Phase 2: Case Number Enrichment (Server PR)

### Task 7: Add case_number to CalendarEntryResponse

**Files:**
- Modify: `crates/shared-types/src/calendar.rs` (CalendarEntryResponse + CalendarEvent)
- Modify: `crates/server/src/repo/calendar.rs` (search + list queries)

**Step 1: Add case_number field to shared types**

In `crates/shared-types/src/calendar.rs`, add to `CalendarEvent` struct:

```rust
    pub case_number: Option<String>,
```

And to `CalendarEntryResponse`:

```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,
```

Update the `From<CalendarEvent> for CalendarEntryResponse` impl to include:

```rust
    case_number: e.case_number,
```

**Step 2: Add LEFT JOIN to calendar repo queries**

In `crates/server/src/repo/calendar.rs`, find the search query and add:

```sql
LEFT JOIN criminal_cases cc ON ce.case_id = cc.id
LEFT JOIN civil_cases cv ON ce.case_id = cv.id
```

And add to SELECT:

```sql
COALESCE(cc.case_number, cv.case_number) as case_number
```

Apply the same pattern to all calendar queries that return CalendarEvent (search, list_by_case, get_by_id, etc.).

**Step 3: Verify compilation**

Run: `cargo check -p server -p app`
Expected: PASS

**Step 4: Run tests**

Run: `cargo test -p tests -- --test-threads=1 calendar`
Expected: PASS (existing tests should still work — case_number is nullable)

**Step 5: Commit**

```bash
git add crates/shared-types/src/calendar.rs crates/server/src/repo/calendar.rs
git commit -m "feat: add case_number to calendar event responses via LEFT JOIN"
```

---

### Task 8: Add case_number to JudicialOrderResponse

**Files:**
- Modify: `crates/shared-types/src/order.rs` (JudicialOrder + JudicialOrderResponse)
- Modify: `crates/server/src/repo/order.rs` (queries)

**Step 1: Add case_number field**

Same pattern as Task 7. Add `case_number: Option<String>` to `JudicialOrder` and `JudicialOrderResponse`, update the `From` impl.

**Step 2: Add LEFT JOIN to order repo queries**

The order repo already has a LEFT JOIN for judge_name. Add the case_number JOINs:

```sql
LEFT JOIN criminal_cases cc ON o.case_id = cc.id
LEFT JOIN civil_cases cv ON o.case_id = cv.id
```

Add to SELECT: `COALESCE(cc.case_number, cv.case_number) as case_number`

**Step 3: Verify and test**

Run: `cargo check -p server -p app && cargo test -p tests -- --test-threads=1 order`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/shared-types/src/order.rs crates/server/src/repo/order.rs
git commit -m "feat: add case_number to order responses via LEFT JOIN"
```

---

### Task 9: Add case_number to DocketEntryResponse

**Files:**
- Modify: `crates/shared-types/src/docket.rs` (DocketEntry + DocketEntryResponse)
- Modify: `crates/server/src/repo/docket.rs` (queries)

**Step 1-4:** Same pattern as Tasks 7-8.

**Commit:**

```bash
git add crates/shared-types/src/docket.rs crates/server/src/repo/docket.rs
git commit -m "feat: add case_number to docket entry responses via LEFT JOIN"
```

---

## Phase 3: Judge Dashboard MVP

### Task 10: Create Pending Motions Server Function + Repo Query

**Files:**
- Create: `crates/server/src/repo/motion.rs` (if not exists — check first)
- Modify: `crates/server/src/repo/mod.rs` (add module)
- Modify: `crates/server/src/api/judge.rs` (add server function)
- Modify: `crates/shared-types/src/case.rs` (add MotionResponse if missing)

**Step 1: Add MotionResponse to shared-types (if missing)**

Check if `MotionResponse` exists in `crates/shared-types/src/case.rs`. If not, add:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MotionResponse {
    pub id: String,
    pub case_id: String,
    pub case_number: Option<String>,
    pub motion_type: String,
    pub filed_by: String,
    pub description: String,
    pub filed_date: String,
    pub status: String,
    pub ruling_date: Option<String>,
    pub ruling_text: Option<String>,
}

impl From<Motion> for MotionResponse {
    fn from(m: Motion) -> Self {
        Self {
            id: m.id.to_string(),
            case_id: m.case_id.to_string(),
            case_number: None, // populated by query
            motion_type: m.motion_type,
            filed_by: m.filed_by,
            description: m.description,
            filed_date: m.filed_date.to_rfc3339(),
            status: m.status,
            ruling_date: m.ruling_date.map(|d| d.to_rfc3339()),
            ruling_text: m.ruling_text,
        }
    }
}
```

**Step 2: Create repo function for pending motions**

In `crates/server/src/repo/motion.rs` (create if needed):

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;
use shared_types::Motion;

pub async fn list_pending_for_judge(
    pool: &PgPool,
    court_id: &str,
    judge_id: Uuid,
) -> Result<Vec<Motion>, AppError> {
    let rows = sqlx::query_as!(
        Motion,
        r#"
        SELECT m.id, m.court_id, m.case_id, m.motion_type, m.filed_by,
               m.description, m.filed_date, m.status,
               m.ruling_date, m.ruling_text,
               COALESCE(cc.case_number, cv.case_number) as "case_number?"
        FROM motions m
        JOIN case_assignments ca ON m.case_id = ca.case_id
             AND ca.judge_id = $2 AND ca.court_id = $1
        LEFT JOIN criminal_cases cc ON m.case_id = cc.id
        LEFT JOIN civil_cases cv ON m.case_id = cv.id
        WHERE m.court_id = $1
          AND m.status = 'Pending'
        ORDER BY m.filed_date ASC
        "#,
        court_id,
        judge_id,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows)
}
```

Note: The Motion struct may need a `case_number` field added for this query. Adapt accordingly.

**Step 3: Add server function**

In `crates/server/src/api/judge.rs`, add:

```rust
#[server]
pub async fn list_pending_motions_for_judge(
    court_id: String,
    judge_id: String,
) -> Result<Vec<shared_types::MotionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let judge_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| ServerFnError::new("Invalid judge UUID"))?;

    let rows = motion::list_pending_for_judge(pool, &court_id, judge_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows.into_iter().map(shared_types::MotionResponse::from).collect())
}
```

**Step 4: Write test**

Add test in `crates/tests/src/`:

```rust
#[tokio::test]
async fn test_list_pending_motions_for_judge() {
    let (app, pool, _lock) = test_app().await;
    let court = "district9";
    let judge_id = create_test_judge(&pool, court, "Motion Judge").await;
    let case_id = create_test_case(&pool, court, "9:26-cr-00099").await;
    // Assign judge to case
    // Create a pending motion
    // Call the endpoint
    // Assert the motion appears
}
```

**Step 5: Verify**

Run: `cargo check -p server -p app && cargo test -p tests -- --test-threads=1 motion`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/server/src/repo/motion.rs crates/server/src/repo/mod.rs crates/server/src/api/judge.rs crates/shared-types/src/case.rs crates/tests/src/
git commit -m "feat: add list_pending_motions_for_judge endpoint"
```

---

### Task 11: Rebuild Judge Dashboard UI

**Files:**
- Modify: `crates/app/src/routes/dashboard/judge.rs` (complete rewrite)

**Step 1: Rewrite JudgeDashboard component**

Replace the entire file with an actionable dashboard using 3 sections:

1. **Orders Pending Signature** — `list_orders_by_judge()` filtered client-side to `status == "pending_signature"`
2. **Upcoming Hearings (7 days)** — `search_calendar_events(judge_id, date_from, date_to)`
3. **Pending Motions** — `list_pending_motions_for_judge()`

Each section: Card with header, data table rows, click navigates to case detail, empty state message.

Get `judge_id` from `use_auth().current_user.read().as_ref().and_then(|u| u.linked_judge_id.clone())`.

**Step 2: Verify compilation**

Run: `cargo check -p app`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/app/src/routes/dashboard/judge.rs
git commit -m "feat: rebuild judge dashboard with actionable work lists"
```

---

## Phase 4: Attorney Dashboard + My Items Toggle

### Task 12: Add Attorney Server Endpoints

**Files:**
- Modify: `crates/server/src/api/party.rs` or create `crates/server/src/api/attorney.rs` (if appropriate)
- Modify: `crates/server/src/repo/representation.rs` (may need new queries)
- Modify: `crates/server/src/repo/calendar.rs` (add attorney filter)
- Modify: `crates/server/src/repo/deadline.rs` (add attorney filter)

**Step 1: Create list_cases_for_attorney server function**

Wraps existing `representation::list_active_by_attorney()`:

```rust
#[server]
pub async fn list_cases_for_attorney(
    court_id: String,
    attorney_id: String,
) -> Result<Vec<shared_types::CaseResponse>, ServerFnError> {
    // Get active representations, extract case_ids, fetch cases
}
```

**Step 2: Add attorney_id filter to search_deadlines**

Add optional `attorney_id: Option<String>` param. When provided, JOIN through representations:

```sql
JOIN representations r ON d.case_id = r.case_id
    AND r.attorney_id = $N AND r.status = 'Active'
```

**Step 3: Create list_calendar_events_for_attorney**

New server function that JOINs calendar_events through representations.

**Step 4: Write tests for all 3 endpoints**

**Step 5: Verify**

Run: `cargo check -p server -p app && cargo test -p tests -- --test-threads=1`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/server/src/ crates/shared-types/src/ crates/tests/src/
git commit -m "feat: add attorney-scoped endpoints (cases, deadlines, calendar)"
```

---

### Task 13: Rebuild Attorney Dashboard UI

**Files:**
- Modify: `crates/app/src/routes/dashboard/attorney.rs` (complete rewrite)

**Step 1: Rewrite AttorneyDashboard component**

3 sections:
1. **Filing Deadlines (14 days)** — `search_deadlines(attorney_id, status=open, date_to=today+14)`
   - Color-code: red < 3 days, yellow < 7 days, green > 7 days
2. **Upcoming Appearances** — `list_calendar_events_for_attorney(attorney_id, date_from=today)`
3. **Recent Docket Activity** — docket entries on attorney's cases from last 7 days

Get `attorney_id` from `use_auth().current_user.read().as_ref().and_then(|u| u.linked_attorney_id.clone())`.

**Step 2: Verify and commit**

```bash
git add crates/app/src/routes/dashboard/attorney.rs
git commit -m "feat: rebuild attorney dashboard with deadline countdown and activity feed"
```

---

### Task 14: My Items / All Court Toggle Component

**Files:**
- Create: `crates/app/src/components/my_items_toggle.rs` (or add to existing components)
- Modify: `crates/app/src/routes/cases/list.rs`
- Modify: `crates/app/src/routes/deadlines/list.rs`
- Modify: `crates/app/src/routes/calendar/list.rs`

**Step 1: Create MyItemsToggle component**

```rust
#[component]
pub fn MyItemsToggle(
    my_active: Signal<bool>,
) -> Element {
    let role = use_user_role();

    // Only render for Judge and Attorney
    if !matches!(role, UserRole::Judge | UserRole::Attorney) {
        return rsx! {};
    }

    rsx! {
        div { class: "my-items-toggle",
            button {
                class: if (my_active)() { "toggle-btn active" } else { "toggle-btn" },
                onclick: move |_| my_active.set(true),
                "My Items"
            }
            button {
                class: if !(my_active)() { "toggle-btn active" } else { "toggle-btn" },
                onclick: move |_| my_active.set(false),
                "All Court"
            }
        }
    }
}
```

**Step 2: Wire into list pages**

In each list page, add a signal `let mut my_items = use_signal(|| true);` and pass it to the toggle. When `my_items` is true, pass the user's `linked_judge_id` or `linked_attorney_id` to the search function.

**Step 3: Verify and commit**

```bash
git add crates/app/src/
git commit -m "feat: add My Items / All Court toggle to list pages"
```

---

## Phase 5: Tab Deep-Linking

### Task 15: Add Query Param Support to Case Detail Tabs

**Files:**
- Modify: `crates/app/src/routes/mod.rs` (Route enum)
- Modify: `crates/app/src/routes/cases/detail.rs` (tab default_value)

**Step 1: Update Route enum**

Change:
```rust
#[route("/cases/:id")]
CaseDetail { id: String },
```
To:
```rust
#[route("/cases/:id?:tab")]
CaseDetail { id: String, tab: Option<String> },
```

**Step 2: Update CaseDetail component wrapper**

```rust
#[component]
fn CaseDetail(id: String, tab: Option<String>) -> Element {
    rsx! { cases::detail::CaseDetailPage { id: id, tab: tab } }
}
```

**Step 3: Update CaseDetailPage to accept tab prop**

In `crates/app/src/routes/cases/detail.rs`, add `tab: Option<String>` to props and use it:

```rust
#[component]
pub fn CaseDetailPage(id: String, tab: Option<String>) -> Element {
    let default_tab = tab.unwrap_or_else(|| "overview".to_string());
    // ...
    Tabs { default_value: default_tab, horizontal: true, ... }
}
```

**Step 4: Update dashboard links to use tab param**

In judge dashboard: `Link { to: Route::CaseDetail { id: case_id, tab: Some("orders".to_string()) } }`

**Step 5: Verify compilation**

Run: `cargo check -p app`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/app/src/routes/mod.rs crates/app/src/routes/cases/detail.rs crates/app/src/routes/dashboard/
git commit -m "feat: add tab deep-linking to case detail via query param"
```

---

## Verification Checklist

After all tasks complete:

```bash
# Full compilation
cargo check -p server -p app

# Full test suite
cargo test -p tests -- --test-threads=1

# Verify no deprecated routes remain
grep -rn "CaseCreate\|AttorneyCreate\|CalendarCreate\|DeadlineCreate" crates/app/src/routes/mod.rs
# Expected: 0 matches

# Verify can() is used
grep -rn "can(&role" crates/app/src/routes/ | wc -l
# Expected: 15+ matches

# Verify no more placeholder dashboards
grep -rn "quick-action-btn\|No items pending\|No pending filings" crates/app/src/routes/dashboard/
# Expected: 0 matches
```
