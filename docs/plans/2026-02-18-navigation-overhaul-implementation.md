# Navigation Overhaul + Global Search Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the 20+ item database-admin sidebar with an 11-item workflow-first sidebar, delete pages that duplicate case detail tabs, and add a Tantivy-powered Cmd+K command palette for global search.

**Architecture:** Three phases — (1) delete duplicate-only pages and restructure sidebar, (2) add Tantivy search index built at server startup, (3) build the Cmd+K command palette UI. Case detail tabs become the sole way to access case-specific data. Pages with cross-case value (Calendar, Deadlines, Compliance, Rules, Opinions, Users) are kept.

**Tech Stack:** Dioxus 0.7, Axum, Tantivy 0.22, shared-ui components

**Design Doc:** `docs/plans/2026-02-18-navigation-overhaul-design.md`

---

## Audit Results

**KEEP (cross-case value):** Attorneys, Calendar, Deadlines, Compliance, Rules, Opinions, Users, Settings, Judges, Cases, Dashboard

**DELETE (pure case-tab duplicates):** Defendants, Parties, Victims, Evidence, Documents, Docket, Filings, Sentencing, Orders, Service Records, Products

---

## Task 1: Delete standalone page directories that duplicate case tabs

**Files:**
- Delete: `crates/app/src/routes/defendants/` (entire directory)
- Delete: `crates/app/src/routes/parties/` (entire directory)
- Delete: `crates/app/src/routes/victims/` (entire directory)
- Delete: `crates/app/src/routes/docket/` (entire directory)
- Delete: `crates/app/src/routes/filings/` (entire directory)
- Delete: `crates/app/src/routes/service_records/` (entire directory)
- Delete: `crates/app/src/routes/orders/` (entire directory)
- Delete: `crates/app/src/routes/evidence/` (entire directory)
- Delete: `crates/app/src/routes/documents/` (entire directory)
- Delete: `crates/app/src/routes/sentencing/` (entire directory)
- Delete: `crates/app/src/routes/products.rs`

**Step 1: Delete all directories and files**

```bash
rm -rf crates/app/src/routes/defendants
rm -rf crates/app/src/routes/parties
rm -rf crates/app/src/routes/victims
rm -rf crates/app/src/routes/docket
rm -rf crates/app/src/routes/filings
rm -rf crates/app/src/routes/service_records
rm -rf crates/app/src/routes/orders
rm -rf crates/app/src/routes/evidence
rm -rf crates/app/src/routes/documents
rm -rf crates/app/src/routes/sentencing
rm crates/app/src/routes/products.rs
```

**Step 2: Commit deletions**

```bash
git add -A crates/app/src/routes/
git commit -m "refactor: delete standalone pages that duplicate case detail tabs"
```

---

## Task 2: Clean up routes/mod.rs — remove dead modules, routes, handlers, and restructure sidebar

**Files:**
- Modify: `crates/app/src/routes/mod.rs`
- Modify: `crates/app/src/auth.rs` (SidebarVisibility)

**Step 1: Remove pub mod declarations for deleted pages**

Delete these from the top of mod.rs:
```rust
pub mod defendants;
pub mod docket;
pub mod documents;
pub mod evidence;
pub mod filings;
pub mod orders;
pub mod parties;
pub mod products;
pub mod sentencing;
pub mod service_records;
pub mod victims;
```

Keep these (all have cross-case value):
```rust
pub mod attorneys;
pub mod calendar;
pub mod cases;
pub mod compliance;
pub mod dashboard;
pub mod deadlines;
pub mod judges;
pub mod opinions;
pub mod rules;
pub mod settings;
pub mod users;
```

**Step 2: Remove Route enum variants for deleted pages**

Delete from the `#[derive(Routable)]` enum:
- `DefendantList`, `DefendantDetail`
- `PartyList`, `PartyDetail`
- `VictimList`, `VictimDetail`
- `DocketList`, `DocketDetail`
- `FilingList`, `FilingDetail`
- `ServiceRecordList`, `ServiceRecordDetail`
- `OrderList`, `OrderDetail`
- `EvidenceList`, `EvidenceDetail`
- `DocumentList`, `DocumentDetail`
- `SentencingList`, `SentencingDetail`
- `Products`

Keep these routes (cross-case pages):
- `Dashboard`, `Settings`
- `CaseList`, `CaseCreate`, `CaseDetail`
- `CalendarList`, `CalendarCreate`, `CalendarDetail`
- `DeadlineList`, `DeadlineCreate`, `DeadlineDetail`
- `AttorneyList`, `AttorneyDetail`
- `JudgeList`, `JudgeDetail`
- `OpinionList`, `OpinionDetail`
- `ComplianceDashboard`
- `RuleList`, `RuleDetail`
- `Users`

**Step 3: Remove page_title match arms for deleted routes**

Delete all match arms referencing removed route variants.

**Step 4: Remove route handler functions for deleted pages**

Delete `fn DefendantList()`, `fn DefendantDetail()`, `fn PartyList()`, etc. handler functions at the bottom of mod.rs.

**Step 5: Restructure sidebar to 4 groups / 11 items**

Replace the current 6-group sidebar with:

```
WORK
  Queue                → Route::Queue (rename from Dashboard, path=/queue)
  Cases                → Route::CaseList
  Calendar             → Route::CalendarList
  Deadlines            → Route::DeadlineList

PEOPLE
  Attorneys            → Route::AttorneyList
  Judges               → Route::JudgeList

LEGAL
  Opinions             → Route::OpinionList

ADMIN
  Compliance           → Route::ComplianceDashboard
  Rules                → Route::RuleList
  Users                → Route::Users
  Settings             → Route::Settings
```

Update the `SidebarVisibility` struct in `crates/app/src/auth.rs` to match new groups:
```rust
pub struct SidebarVisibility {
    pub work: bool,          // Queue, Cases, Calendar, Deadlines
    pub people: bool,        // Attorneys, Judges
    pub legal: bool,         // Opinions
    pub admin: bool,         // Compliance, Rules, Users, Settings
}
```

Role visibility:
- Admin/Clerk: all true
- Judge: work=true, people=false, legal=true, admin=false
- Attorney: work=true, people=false, legal=true, admin=false
- Public: work=true (Cases only), people=false, legal=false, admin=false

**Step 6: Verify compilation**

Run: `cargo check -p app`
Expected: May have errors from references to deleted routes in other files.

**Step 7: Commit**

```bash
git add crates/app/src/routes/mod.rs crates/app/src/auth.rs
git commit -m "refactor: restructure sidebar to 11 workflow-first items, remove dead routes"
```

---

## Task 3: Fix remaining compilation errors

**Files:**
- Modify: Various files that reference deleted routes

**Step 1: Find all references to deleted routes**

Run: `cargo check -p app 2>&1 | grep "error"` to find broken references.

Common fixes:
- Links in case detail tabs pointing to standalone pages (e.g., `Route::OrderList`) → remove or change to stay within case detail
- Navigation actions using deleted route variants → replace with case-context navigation
- Any `use` imports referencing deleted modules → remove

**Step 2: Fix all references**

For each broken reference, remove the dead link or replace with case-context navigation.

**Step 3: Verify clean compilation**

Run: `cargo check -p app`
Run: `dx build --package app --platform web`
Expected: Builds successfully.

**Step 4: Commit**

```bash
git add -A
git commit -m "fix: resolve all references to deleted routes"
```

---

## Task 4: Add Tantivy dependency and search index module

**Files:**
- Modify: `crates/server/Cargo.toml`
- Create: `crates/server/src/search.rs`
- Modify: `crates/server/src/lib.rs`

**Step 1: Add tantivy to server Cargo.toml**

Add to `[dependencies]`:
```toml
tantivy = { version = "0.22", optional = true }
```

Add to `[features]` under `server = [...]`:
```toml
"dep:tantivy",
```

**Step 2: Create search index module**

Create `crates/server/src/search.rs` with:

- `SearchIndex` struct holding a Tantivy in-RAM index with fields: id, entity_type, title, subtitle, court_id
- `SearchIndex::new()` — creates schema and in-RAM index
- `SearchIndex::search(query, court_id, limit)` — full-text search filtered by court, returns `Vec<SearchResult>`
- `SearchResult` struct: `{ id, entity_type, title, subtitle }`
- `build_index(pool, search)` — async function that queries DB for cases, attorneys, judges and indexes them

Index ALL court entities:
- Cases: `title = "{case_number} - {title}"`, `subtitle = "{crime_type}"`
- Attorneys: `title = "{first_name} {last_name}"`, `subtitle = "{bar_number} - {firm_name}"`
- Judges: `title = "{name}"`, `subtitle = "{title}"`
- Docket entries: `title = "Dkt #{entry_number}: {description}"`, `subtitle = "{entry_type} - {case_number}"`
- Calendar events: `title = "{event_type}: {description}"`, `subtitle = "{case_title}"`
- Deadlines: `title = "{title}"`, `subtitle = "{status} - {case_title}"`
- Orders: `title = "{title}"`, `subtitle = "{order_type} - {case_title}"`
- Opinions: `title = "{title}"`, `subtitle = "{author}"`

**Step 3: Register module in lib.rs**

Add `#[cfg(feature = "server")] pub mod search;` to `crates/server/src/lib.rs`.

**Step 4: Verify compilation**

Run: `cargo check -p server --features server`

**Step 5: Commit**

```bash
git add crates/server/Cargo.toml crates/server/src/search.rs crates/server/src/lib.rs
git commit -m "feat(search): add Tantivy search index module with case/attorney/judge indexing"
```

---

## Task 5: Wire search index into AppState and add server function

**Files:**
- Modify: `crates/server/src/db.rs` (add SearchIndex to AppState)
- Modify: `crates/server/src/api.rs` (add search server function)
- Modify: server startup (build index after migrations)

**Step 1: Add SearchIndex to AppState**

Add `search: Arc<search::SearchIndex>` to AppState. Initialize during startup, call `search::build_index(&pool, &search_index).await` after migrations run.

**Step 2: Add search server function**

Add to api.rs:
```rust
#[server]
pub async fn global_search(
    court_id: String,
    query: String,
    limit: Option<usize>,
) -> Result<String, ServerFnError> {
    // Get SearchIndex from app state, search, return JSON results
}
```

**Step 3: Verify compilation**

Run: `cargo check -p server --features server`

**Step 4: Commit**

```bash
git add crates/server/src/db.rs crates/server/src/api.rs
git commit -m "feat(search): wire Tantivy index into AppState and add global_search server function"
```

---

## Task 6: Build the Cmd+K command palette UI component

**Files:**
- Create: `crates/app/src/routes/command_palette.rs`
- Create: `crates/app/src/routes/command_palette.css`
- Modify: `crates/app/src/routes/mod.rs` (integrate into layout)

**Step 1: Create the command palette component**

A component that:
1. Invisible overlay until activated
2. Activates on Cmd+K / Ctrl+K OR click search icon in header
3. Modal with search input at top
4. Empty state: "Recent" section (last 5 viewed items)
5. On typing: debounce 200ms, call `server::api::global_search(court, query)`
6. Results grouped by entity_type: Cases, Attorneys, Judges
7. Keyboard nav: arrow keys, Enter selects, Escape closes
8. Click result → navigate: cases → `/cases/{id}`, attorneys → `/attorneys/{id}`, judges → `/judges/{id}`

**Step 2: Create CSS for overlay, modal, results**

**Step 3: Integrate into main layout in mod.rs**

Wire search icon click and Cmd+K global key listener.

**Step 4: Verify**

Run: `cargo check -p app` and `dx build --package app --platform web`

**Step 5: Commit**

```bash
git add crates/app/src/routes/command_palette.rs crates/app/src/routes/command_palette.css crates/app/src/routes/mod.rs
git commit -m "feat(ui): add Cmd+K command palette with Tantivy-powered global search"
```

---

## Task 7: Full verification

**Step 1:** Run `cargo test -p tests -- --test-threads=1` — all tests pass
**Step 2:** Run `dx build --package app --platform web` — builds clean
**Step 3:** Manual smoke test:
1. Sidebar shows 11 items in 4 groups
2. Queue shows clerk work queue with seed data
3. Cases list works, click case → detail with all tabs
4. Calendar, Deadlines, Opinions — cross-case views work
5. Cmd+K opens palette, search "Rodriguez" → case result, click → navigates
6. Escape closes palette
7. Attorneys, Judges standalone pages work

**Step 4:** Commit any fixes

---

## Summary

| Task | What it delivers |
|------|-----------------|
| Task 1 | Delete 11 standalone page directories/files (case-tab duplicates only) |
| Task 2 | Restructure mod.rs: 11-item sidebar in 4 groups, remove dead routes/handlers |
| Task 3 | Fix compilation errors from deleted references |
| Task 4 | Tantivy search index module (cases + attorneys + judges) |
| Task 5 | Wire search into AppState + server function |
| Task 6 | Cmd+K command palette UI component |
| Task 7 | Full verification |
