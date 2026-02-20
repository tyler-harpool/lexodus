# Role-Based UX Overhaul — Design Document

## Problem

Lexodus has functional CRUD pages but no role-awareness. Every user sees every button, every list shows all court items, and judge/attorney dashboards are placeholders with aggregate counts and dead buttons. The system doesn't answer "what do I need to do right now?" for any role except clerks (who have a working queue).

## Product Shape

The system is converging on three role-first workstation loops:

- **Clerk:** Queue → Claim → Work the case → Complete → Next
- **Judge:** Sign orders → Run hearings → Rule on motions
- **Attorney:** Don't miss deadlines → Show up → Track activity

This design makes those loops real.

## Design Principles

- **Clerk-first.** Clerks do 80% of the daily work. Their experience (queue dashboard) is already the most complete. Judges and attorneys get functional but secondary experiences.
- **One axis of change per PR.** UI-only PRs don't touch the server. Server PRs don't move UI components. When unavoidable, isolate in a tight-scope PR.
- **Dashboards are query products.** They only feel actionable if the backend returns what the row needs (case_number, judge_name, timestamps). UUID soup is not acceptable.

## Current State Assessment

### What exists and works

| Capability | Status |
|------------|--------|
| `UserRole` enum (Admin, Clerk, Judge, Attorney, Public) | Implemented |
| `use_user_role()` reactive hook | Implemented |
| `can(role, action)` permission check | Implemented but **never called** |
| `SidebarVisibility` per role | Implemented |
| Clerk dashboard (queue) | Fully functional |
| `search_calendar_events(judge_id)` | Ready |
| `list_orders_by_judge()` with judge_name | Ready |
| Motion struct with status/ruling_date/ruling_text | Ready |
| `representation::list_active_by_attorney()` repo | Ready (no API wrapper) |
| Typed server function returns | Just completed (194 functions) |

### What's missing

| Gap | Impact |
|-----|--------|
| `can()` never called in any UI page | All users see all buttons |
| Calendar/docket/order responses have `case_id` not `case_number` | Dashboard rows show UUIDs instead of case numbers |
| No API for attorney's cases | Can't build attorney dashboard |
| No `attorney_id` filter on search_deadlines/search_calendar | Can't filter "my" items for attorneys |
| No deep-linking to case detail tabs | Dashboard items can't link to specific tab |
| Judge/attorney dashboards are placeholder cards + dead buttons | No actionable information |
| 4 deprecated create pages still routable | Duplicate flows, confusing |

---

## Phased Implementation

### Phase 1: Permission Gating + Route Cleanup (UI-only PR)

**Goal:** Wire existing `can()` into every page. Delete deprecated routes. Zero backend changes.

#### 1a. Expand Action Enum

Replace the current broad actions with entity-scoped actions:

```rust
pub enum Action {
    // Court record management (clerk/admin)
    ManageAttorneys,
    ManageJudges,
    ManageRules,

    // Case workflow
    CreateCase,         // clerk/admin
    EditCase,           // clerk/admin/judge
    DeleteCase,         // admin only

    // Docket & Filing
    CreateDocketEntry,  // clerk/admin
    FileFiling,         // attorney/clerk/admin

    // Judicial actions
    SignOrder,          // judge/admin
    IssueOrder,         // clerk/admin
    DraftOpinion,       // judge/admin

    // Document control
    SealDocument,       // judge/clerk/admin
    StrikeDocument,     // judge/clerk/admin

    // Evidence & Sentencing
    ManageEvidence,     // clerk/admin
    EnterSentencing,    // judge/clerk/admin

    // Universal
    GeneratePdf,        // all except public
}
```

#### 1b. Gate Buttons Across All Pages

Every action button gets wrapped:

```rust
let role = use_user_role();
if can(&role, Action::CreateCase) {
    Button { onclick: open_form_sheet, "New Case" }
}
```

**Pages requiring gating:**

| Page | Buttons | Required Action |
|------|---------|----------------|
| Case List | "New Case" | `CreateCase` |
| Case Detail | "Edit", "Delete" | `EditCase`, `DeleteCase` |
| Case > Docket tab | "New Entry", "File" | `CreateDocketEntry`, `FileFiling` |
| Case > Orders tab | "Draft Order", "Sign", "Issue" | `EditCase`, `SignOrder`, `IssueOrder` |
| Case > Sentencing tab | "Enter Sentencing" | `EnterSentencing` |
| Case > Evidence tab | "Add Evidence" | `ManageEvidence` |
| Case > Parties tab | "Add Party" | `CreateCase` |
| Attorney List/Detail | "New", "Edit", "Delete" | `ManageAttorneys` |
| Judge List/Detail | "New", "Edit", "Delete" | `ManageJudges` |
| Calendar List | "Schedule Event" | `CreateCase` |
| Deadline List | "New Deadline" | `CreateCase` |
| Opinion List | "New Opinion" | `DraftOpinion` |
| Rules List | "New Rule" | `ManageRules` |

#### 1c. Delete Deprecated Create Routes

Remove routes and files:
- `/cases/new` → `cases/create.rs`
- `/attorneys/new` → `attorneys/create.rs`
- `/calendar/new` → `calendar/create.rs`
- `/deadlines/new` → `deadlines/create.rs`

Remove from `Route` enum and delete the 4 files.

#### 1d. Role-Adaptive Sidebar Label

| Role | Sidebar Label | Route |
|------|--------------|-------|
| Admin/Clerk | "Queue" | `/` (ClerkDashboard) |
| Judge | "Dashboard" | `/` (JudgeDashboard) |
| Attorney | "Dashboard" | `/` (AttorneyDashboard) |

#### 1e. Add Identity Links to AuthUser

Add to `AuthUser` in shared-types:
- `linked_judge_id: Option<String>` — populated at login if user maps to a judge record
- `linked_attorney_id: Option<String>` — populated at login if user maps to an attorney record

Lookup logic: match by email address against judges/attorneys table during auth check.

---

### Phase 2: Case Number Enrichment (small server PR)

**Goal:** Add `case_number` to response types that only have `case_id`. Follow the existing pattern from `order.rs` where `judge_name` is already populated via LEFT JOIN.

**Types to enrich:**

| Type | New Field | JOIN Source |
|------|-----------|------------|
| `CalendarEntryResponse` | `case_number: Option<String>` | LEFT JOIN `criminal_cases` / `civil_cases` on `case_id` |
| `JudicialOrderResponse` | `case_number: Option<String>` | Same pattern |
| `DocketEntryResponse` | `case_number: Option<String>` | Same pattern |

**SQL pattern (already used for judge_name in orders):**

```sql
SELECT o.*, j.name as judge_name,
       COALESCE(cc.case_number, cv.case_number) as case_number
FROM judicial_orders o
LEFT JOIN judges j ON o.judge_id = j.id
LEFT JOIN criminal_cases cc ON o.case_id = cc.id
LEFT JOIN civil_cases cv ON o.case_id = cv.id
WHERE o.court_id = $1
```

---

### Phase 3: Judge Dashboard MVP

**Goal:** Replace placeholder cards with actionable lists. Uses existing endpoints + Phase 2 enrichment.

#### Section 1: Orders Pending Signature

- **Source:** `list_orders_by_judge(court, judge_id)` + client-side filter `status == "pending_signature"`
- **Row:** Case number, order title, submitted date, "Review & Sign" link
- **Click:** Navigate to `/cases/{case_id}?tab=orders` (Phase 5) or `/cases/{case_id}` for now
- **Empty state:** "No orders awaiting your signature"

#### Section 2: Upcoming Hearings (Next 7 Days)

- **Source:** `search_calendar_events(court, judge_id, date_from=today, date_to=today+7)`
- **Row:** Date/time, case number (from Phase 2), event type, courtroom
- **Click:** Navigate to `/cases/{case_id}`
- **Empty state:** "No hearings scheduled this week"

#### Section 3: Pending Motions

**New server function required:** `list_pending_motions_for_judge(court, judge_id)`

**Business rule:** Motions WHERE `status = 'Pending'` AND the case is assigned to this judge (JOIN `case_assignments`).

**Query:**
```sql
SELECT m.*, cc.case_number
FROM motions m
JOIN case_assignments ca ON m.case_id = ca.case_id AND ca.judge_id = $2
LEFT JOIN criminal_cases cc ON m.case_id = cc.id
WHERE m.court_id = $1
  AND m.status = 'Pending'
ORDER BY m.filed_date ASC
```

- **Row:** Case number, motion type, filed date, filed by
- **Click:** Navigate to `/cases/{case_id}`
- **Empty state:** "No pending motions"

---

### Phase 4: Attorney Dashboard + "My Items" Toggle

**Goal:** Make the attorney experience functional. Requires new server endpoints.

#### New Server Endpoints

1. **`list_cases_for_attorney(court, attorney_id)`**
   - Wraps existing `representation::list_active_by_attorney()` repo function
   - Returns `Vec<CaseResponse>` with case_number included

2. **`search_deadlines` + `attorney_id` param**
   - JOIN: `deadlines → case_id → representations WHERE attorney_id = $N`
   - Returns only deadlines on cases where this attorney has active representation

3. **`list_calendar_events_for_attorney(court, attorney_id)`**
   - JOIN: `calendar_events → case_id → representations WHERE attorney_id = $N`
   - Returns events on cases where this attorney has active representation

#### Attorney Dashboard Sections

1. **Filing Deadlines (Next 14 Days)**
   - Source: `search_deadlines(court, attorney_id, status=open, date_to=today+14)`
   - Row: Due date, title, case number, days remaining (red < 3, yellow < 7)
   - Empty state: "No upcoming deadlines"

2. **Upcoming Appearances**
   - Source: `list_calendar_events_for_attorney(court, attorney_id, date_from=today)`
   - Row: Date/time, case number, event type, courtroom, judge name
   - Empty state: "No appearances scheduled"

3. **Recent Docket Activity (Last 7 Days)**
   - Source: New endpoint or filter on `search_docket_entries` scoped to attorney's cases
   - Row: Date, case number, entry type, filed by
   - Empty state: "No recent activity on your cases"

#### "My Items / All Court" Toggle

A reusable `MyItemsToggle` component:

- Renders only for Judge and Attorney roles
- Defaults to "My Items"
- Placed at the top of: Cases list, Deadlines list, Calendar list
- When active, passes `judge_id` or `attorney_id` to search functions
- Toggle state stored in signal (per-session, resets on refresh)

---

### Phase 5: Tab Deep-Linking (UI-only)

**Goal:** Dashboard items can link directly to a specific case detail tab.

Add `?tab=` query param support to CaseDetail:

```rust
#[route("/cases/:id?:tab")]
CaseDetail { id: String, tab: Option<String> },
```

Tab component reads the query param as `default_value`:

```rust
let default_tab = tab.unwrap_or_else(|| "overview".to_string());
Tabs { default_value: default_tab, horizontal: true, ... }
```

Dashboard links then use: `/cases/{case_id}?tab=orders`, `/cases/{case_id}?tab=docket`, etc.

---

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Phase 2 JOINs add query overhead | Same pattern already used for judge_name in orders — proven at scale |
| "Pending Motions" business rule is heuristic | Motion table has explicit `status` field — no heuristic needed |
| Attorney→cases JOIN is expensive | Representations table already indexed on `attorney_id` |
| Deep-linking breaks existing tab state | Default to "overview" when no param — backward compatible |
| `can()` expansion could miss edge cases | Start permissive, tighten — same approach used for v1 |

## Success Criteria

- Judge logs in, sees orders to sign, hearings this week, pending motions — all clickable
- Attorney logs in, sees filing deadlines with countdown, upcoming appearances — all clickable
- No user sees buttons they can't use
- Clerk experience unchanged (queue dashboard already works)
- Deprecated create pages gone, no dead routes
- Zero runtime permission errors (compiler catches type mismatches, `can()` catches role mismatches)
