# Civil Cases Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add full CM/ECF civil case support — `civil_cases` table, NOS codes reference table, polymorphic `case_type` on shared tables, shared types, repo, REST handlers, Typst PDF templates, queue integration, rules engine integration, seed data, and tests.

**Architecture:** Separate `civil_cases` table with JS-44 fields. Shared tables (docket, parties, calendar, etc.) get a `case_type` column for polymorphic references. Civil endpoints mirror criminal patterns. Queue auto-creates items on civil filing/motion. Rules engine fires FRCP deadlines on civil events.

**Tech Stack:** Rust, Axum, sqlx (Postgres), Dioxus 0.7, Typst, shared-ui

**Design Doc:** `docs/plans/2026-02-18-civil-cases-design.md`

---

## Task 1: Create civil_cases and nature_of_suit_codes migrations

**Files:**
- Create: `migrations/20260218000076_create_civil_cases.sql`
- Create: `migrations/20260218000076_create_civil_cases.down.sql`
- Create: `migrations/20260218000077_create_nature_of_suit_codes.sql`
- Create: `migrations/20260218000077_create_nature_of_suit_codes.down.sql`

**Step 1: Write the civil_cases up migration**

Create `migrations/20260218000076_create_civil_cases.sql` with the full table from the design doc. Include indexes:
- `idx_civil_cases_court_status ON civil_cases(court_id, status)`
- `idx_civil_cases_court_judge ON civil_cases(court_id, assigned_judge_id)`
- `idx_civil_cases_nos ON civil_cases(nature_of_suit)`

**Step 2: Write the NOS codes migration**

Create `migrations/20260218000077_create_nature_of_suit_codes.sql` with the reference table and seed all ~75 real JS-44 codes. Use INSERT with ON CONFLICT DO NOTHING.

**Step 3: Write down migrations**

```sql
-- 000076 down
DROP TABLE IF EXISTS civil_cases;

-- 000077 down
DROP TABLE IF EXISTS nature_of_suit_codes;
```

**Step 4: Run migrations**

Run: `sqlx migrate run`
Expected: Both migrations applied.

**Step 5: Commit**

```bash
git add migrations/20260218000076_* migrations/20260218000077_*
git commit -m "feat(civil): add civil_cases table and nature_of_suit_codes reference data"
```

---

## Task 2: Add case_type column to shared tables

**Files:**
- Create: `migrations/20260218000078_add_case_type_to_shared_tables.sql`
- Create: `migrations/20260218000078_add_case_type_to_shared_tables.down.sql`

**Step 1: Write the migration**

For each of these tables, add `case_type` column and drop the FK to criminal_cases:

```sql
-- docket_entries
ALTER TABLE docket_entries DROP CONSTRAINT IF EXISTS docket_entries_case_id_fkey;
ALTER TABLE docket_entries ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('civil', 'criminal'));

-- parties
ALTER TABLE parties DROP CONSTRAINT IF EXISTS parties_case_id_fkey;
ALTER TABLE parties ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('civil', 'criminal'));

-- calendar_events
ALTER TABLE calendar_events DROP CONSTRAINT IF EXISTS calendar_events_case_id_fkey;
ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('civil', 'criminal'));

-- deadlines (uses case_id, check actual FK name)
ALTER TABLE deadlines DROP CONSTRAINT IF EXISTS deadlines_case_id_fkey;
ALTER TABLE deadlines ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('civil', 'criminal'));

-- documents
ALTER TABLE documents DROP CONSTRAINT IF EXISTS documents_case_id_fkey;
ALTER TABLE documents ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('civil', 'criminal'));

-- filings
ALTER TABLE filings DROP CONSTRAINT IF EXISTS filings_case_id_fkey;
ALTER TABLE filings ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('civil', 'criminal'));

-- motions
ALTER TABLE motions DROP CONSTRAINT IF EXISTS motions_case_id_fkey;
ALTER TABLE motions ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('civil', 'criminal'));

-- evidence
ALTER TABLE evidence DROP CONSTRAINT IF EXISTS evidence_case_id_fkey;
ALTER TABLE evidence ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('civil', 'criminal'));

-- judicial_orders
ALTER TABLE judicial_orders DROP CONSTRAINT IF EXISTS judicial_orders_case_id_fkey;
ALTER TABLE judicial_orders ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('civil', 'criminal'));

-- clerk_queue
ALTER TABLE clerk_queue DROP CONSTRAINT IF EXISTS clerk_queue_case_id_fkey;
ALTER TABLE clerk_queue ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('civil', 'criminal'));

-- victims
ALTER TABLE victims DROP CONSTRAINT IF EXISTS victims_case_id_fkey;
ALTER TABLE victims ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('civil', 'criminal'));

-- representations
ALTER TABLE representations DROP CONSTRAINT IF EXISTS representations_case_id_fkey;
ALTER TABLE representations ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('civil', 'criminal'));
```

NOTE: Before writing this migration, check the actual FK constraint names by running:
`psql ... -c "SELECT conname FROM pg_constraint WHERE conrelid = 'docket_entries'::regclass AND contype = 'f';"`

**Step 2: Write the down migration**

Remove `case_type` column from all tables and re-add the FK constraints.

**Step 3: Run migration**

Run: `sqlx migrate run`

**Step 4: Verify existing data unchanged**

Run: `psql ... -c "SELECT case_type, count(*) FROM docket_entries GROUP BY case_type;"`
Expected: All rows are `criminal`.

**Step 5: Run existing test suite**

Run: `cargo test -p tests -- --test-threads=1`
Expected: All 352 tests still pass (default 'criminal' means no existing code breaks).

**Step 6: Commit**

```bash
git add migrations/20260218000078_*
git commit -m "feat(civil): add case_type column to 12 shared tables"
```

---

## Task 3: Add civil case shared types

**Files:**
- Create: `crates/shared-types/src/civil_case.rs`
- Modify: `crates/shared-types/src/lib.rs`

**Step 1: Create civil_case.rs**

Follow the pattern of `case.rs`. Include:
- Constants: `CIVIL_CASE_STATUSES`, `CIVIL_JURISDICTION_BASES`, `CIVIL_JURY_DEMANDS`
- Validation functions: `is_valid_civil_status()`, `is_valid_jurisdiction_basis()`, `is_valid_jury_demand()`
- `CivilCase` struct (DB row, with `#[cfg_attr(feature = "server", derive(sqlx::FromRow))]`)
- `CivilCaseResponse` struct (API response, UUIDs as strings, dates as RFC3339)
- `impl From<CivilCase> for CivilCaseResponse`
- `CreateCivilCaseRequest` — title, nature_of_suit, cause_of_action, jurisdiction_basis, jury_demand, class_action, amount_in_controversy, district_code, etc.
- `CivilCaseSearchParams` — status, nature_of_suit, jurisdiction_basis, class_action, assigned_judge_id, offset, limit
- `UpdateCivilCaseStatusRequest` — status

**Step 2: Register in lib.rs**

Add `pub mod civil_case;` and `pub use civil_case::*;` to lib.rs.

**Step 3: Verify**

Run: `cargo check -p shared-types`

**Step 4: Commit**

```bash
git add crates/shared-types/src/civil_case.rs crates/shared-types/src/lib.rs
git commit -m "feat(civil): add civil case shared types with JS-44 fields"
```

---

## Task 4: Create civil case repo module

**Files:**
- Create: `crates/server/src/repo/civil_case.rs`
- Modify: `crates/server/src/repo/mod.rs`

**Step 1: Write repo module**

Follow the pattern of `repo/case.rs`. Include:
- `generate_case_number()` — format: `{YEAR}-CV-{sequence:05}` (CV not CR)
- `create()` — INSERT with all JS-44 fields
- `find_by_id()` — SELECT by id + court_id
- `search()` — SELECT with optional filters (status, NOS, jurisdiction, class_action, judge), returns `(Vec<CivilCase>, i64)`
- `update_status()` — UPDATE status
- `delete()` — DELETE by id + court_id

**Step 2: Register in repo/mod.rs**

Add `#[cfg(feature = "server")] pub mod civil_case;`

**Step 3: Verify**

Run: `cargo check -p server --features server`

**Step 4: Commit**

```bash
git add crates/server/src/repo/civil_case.rs crates/server/src/repo/mod.rs
git commit -m "feat(civil): add civil case repo with CRUD and search"
```

---

## Task 5: Create civil case REST handlers

**Files:**
- Create: `crates/server/src/rest/civil_case.rs`
- Modify: `crates/server/src/rest/mod.rs`

**Step 1: Write REST handler module**

Follow the pattern of `rest/case.rs`. Endpoints:
- `POST /api/civil-cases` — create with validation (NOS, jurisdiction, jury demand)
- `GET /api/civil-cases` — search with query params
- `GET /api/civil-cases/{id}` — get by ID
- `PATCH /api/civil-cases/{id}` — update
- `DELETE /api/civil-cases/{id}` — delete
- `PATCH /api/civil-cases/{id}/status` — update status
- `GET /api/civil-cases/statistics` — aggregate counts by status, NOS, jurisdiction
- `GET /api/civil-cases/by-judge/{judge_id}` — judge's civil caseload

**Step 2: Register routes in rest/mod.rs**

Add `pub mod civil_case;` and routes in `api_router()`.

**Step 3: Verify**

Run: `cargo check -p server --features server`

**Step 4: Commit**

```bash
git add crates/server/src/rest/civil_case.rs crates/server/src/rest/mod.rs
git commit -m "feat(civil): add civil case REST handlers with 8 endpoints"
```

---

## Task 6: Civil case integration tests

**Files:**
- Create: `crates/tests/src/civil_case_create_tests.rs`
- Create: `crates/tests/src/civil_case_search_tests.rs`
- Create: `crates/tests/src/civil_case_isolation_tests.rs`
- Modify: `crates/tests/src/lib.rs`
- Modify: `crates/tests/src/common.rs` (add civil_cases to TRUNCATE, add helper)

**Step 1: Update common.rs**

Add `civil_cases` and `nature_of_suit_codes` to TRUNCATE. But nature_of_suit_codes is reference data — don't truncate it. Only truncate `civil_cases`.

Add helper:
```rust
pub async fn create_test_civil_case(app: &Router, court: &str, title: &str) -> Value {
    let body = serde_json::json!({
        "title": title,
        "nature_of_suit": "110",
        "cause_of_action": "28 USC 1332",
        "jurisdiction_basis": "diversity",
        "district_code": court,
    });
    let (status, resp) = post_json(app, "/api/civil-cases", &body.to_string(), court).await;
    assert_eq!(status, StatusCode::CREATED);
    resp
}
```

**Step 2: Write create tests (~8 tests)**

- create success with minimal fields
- create with all JS-44 fields (class_action, jury_demand, amount, etc.)
- invalid nature_of_suit returns 400
- invalid jurisdiction_basis returns 400
- empty title returns 400
- case_number auto-generated with CV prefix
- get by ID
- delete

**Step 3: Write search tests (~4 tests)**

- search with status filter
- search with NOS filter
- search with jurisdiction filter
- pagination

**Step 4: Write isolation tests (~3 tests)**

- district9 cannot see district12 civil cases
- civil cases don't appear in criminal case endpoints
- criminal cases don't appear in civil case endpoints

**Step 5: Register in lib.rs**

**Step 6: Run tests**

Run: `cargo test -p tests -- civil_case --test-threads=1`
Expected: All pass.

**Step 7: Commit**

```bash
git add crates/tests/src/civil_case_*.rs crates/tests/src/lib.rs crates/tests/src/common.rs
git commit -m "test(civil): add civil case CRUD, search, and isolation tests"
```

---

## Task 7: Add civil case server functions and UI

**Files:**
- Modify: `crates/server/src/api.rs` (add server functions)
- Modify: `crates/app/src/routes/cases/list.rs` (add civil/criminal toggle)
- Modify: `crates/app/src/routes/mod.rs` (add civil case routes if needed)

**Step 1: Add server functions**

Add to api.rs (follow existing patterns):
- `search_civil_cases(court_id, status, nos, jurisdiction, ...)`
- `get_civil_case(court_id, id)`
- `create_civil_case(court_id, body_json)`
- `update_civil_case_status(court_id, id, status)`

**Step 2: Add civil/criminal toggle to Cases list page**

In `cases/list.rs`, add a tab or filter at the top: "Criminal | Civil | All". When "Civil" is selected, call `search_civil_cases` instead of `search_cases`. Display civil-specific columns (NOS, jurisdiction, jury demand) instead of criminal columns (crime_type).

**Step 3: Verify**

Run: `cargo check -p app`
Run: `dx build --package app --platform web`

**Step 4: Commit**

```bash
git add crates/server/src/api.rs crates/app/src/routes/cases/list.rs
git commit -m "feat(civil): add server functions and cases list civil toggle"
```

---

## Task 8: Typst PDF templates for civil documents

**Files:**
- Create: `templates/js-44-cover-sheet.typ`
- Create: `templates/civil-summons.typ`
- Create: `templates/civil-scheduling-order.typ`
- Create: `templates/civil-judgment.typ`
- Modify: `crates/server/src/rest/pdf.rs` (add civil PDF endpoints)
- Modify: `crates/server/src/rest/mod.rs` (register routes)

**Step 1: Create JS-44 cover sheet template**

Reference the existing `~/Downloads/js-44.typ` file. The template takes: plaintiff, defendant, county, attorneys, basis of jurisdiction, nature of suit code/title, cause of action, jury demand, class action flag, related cases.

**Step 2: Create civil summons template**

Standard federal civil summons: court name, case number, plaintiff, defendant, attorney info, time to answer (21 days).

**Step 3: Create scheduling order and judgment templates**

Follow the pattern of existing `court-order.typ` template.

**Step 4: Add PDF endpoints**

```rust
POST /api/pdf/js44-cover-sheet
POST /api/pdf/civil-summons
POST /api/pdf/civil-scheduling-order
POST /api/pdf/civil-judgment
```

**Step 5: Verify**

Run: `cargo check -p server --features server`

**Step 6: Commit**

```bash
git add templates/js-44-cover-sheet.typ templates/civil-summons.typ templates/civil-scheduling-order.typ templates/civil-judgment.typ crates/server/src/rest/pdf.rs crates/server/src/rest/mod.rs
git commit -m "feat(civil): add Typst PDF templates for JS-44, summons, scheduling order, judgment"
```

---

## Task 9: Queue and rules integration for civil cases

**Files:**
- Modify: `crates/server/src/rest/civil_case.rs` (auto-create queue items)
- Create: `migrations/20260218000079_seed_civil_rules.sql`
- Create: `migrations/20260218000079_seed_civil_rules.down.sql`

**Step 1: Auto-create queue items on civil case filing**

In the `create_civil_case` handler, after successful insert, auto-create a queue item:
```rust
let _ = crate::repo::queue::create(
    &pool, &court.0, "filing", 3,
    &format!("Civil Complaint: {}", case.title),
    Some("New civil complaint requires clerk review"),
    "filing", case.id, Some(case.id), Some(&case.case_number),
    None, None, "review",
).await;
```

**Step 2: Seed civil-specific rules**

Create migration with ~15 FRCP rules:
- FRCP 4(m): Service of process — 90 days
- FRCP 12(a)(1): Answer — 21 days after service
- FRCP 26(f): Discovery conference — 21 days before Rule 16 scheduling order due
- FRCP 16(b): Scheduling order — 8 weeks after defendant served
- FRCP 26(a)(1): Initial disclosures — 14 days after Rule 26(f) conference
- FRCP 26(a)(2): Expert disclosures — 90 days before trial
- FRCP 26(a)(3): Pretrial disclosures — 30 days before trial
- FRCP 56: Summary judgment motion deadline (per local rule)
- Local Rule: Motion response — 14 days
- Local Rule: Reply brief — 7 days
- Local Rule: Dismiss for failure to prosecute — 6 months no activity

Each rule: `source = 'Federal Rules of Civil Procedure'`, `category = 'Deadline'` or `'Filing'`, with conditions JSONB describing the trigger.

**Step 3: Run migration**

**Step 4: Commit**

```bash
git add crates/server/src/rest/civil_case.rs migrations/20260218000079_*
git commit -m "feat(civil): add queue auto-creation and seed FRCP rules"
```

---

## Task 10: Seed realistic civil cases

**Files:**
- Create: `migrations/20260218000080_seed_civil_cases.sql`
- Create: `migrations/20260218000080_seed_civil_cases.down.sql`

**Step 1: Write seed migration**

8 civil cases across both districts with realistic data:

| # | Title | NOS | Jurisdiction | Status | Edge Case |
|---|-------|-----|-------------|--------|-----------|
| 1 | Johnson v. Acme Corp | 442 (Employment) | federal_question | discovery | Class action, $5M controversy |
| 2 | TechStart v. MegaSoft | 830 (Patent) | federal_question | pretrial | Patent infringement, high priority |
| 3 | Smith v. Jones | 190 (Contract) | diversity | filed | $500K demand, jury demand both |
| 4 | Green Earth v. ChemCo | 893 (Environmental) | federal_question | pending | Clean Water Act |
| 5 | Davis v. State Prison | 550 (Prisoner) | federal_question | filed | Pro se, habeas corpus |
| 6 | ACLU v. DHS | 895 (FOIA) | federal_question | judgment_entered | Resolved |
| 7 | Martinez v. City PD | 440 (Civil Rights) | federal_question | trial_ready | 42 USC 1983, jury demand plaintiff |
| 8 | Thompson v. Benefits Inc | 791 (ERISA) | federal_question | settled | Consent to magistrate |

Include parties, docket entries (complaint, answer, scheduling order), and queue items.

**Step 2: Commit**

```bash
git add migrations/20260218000080_*
git commit -m "feat(civil): seed 8 realistic civil cases with parties and docket entries"
```

---

## Task 11: Update Tantivy search index for civil cases

**Files:**
- Modify: `crates/server/src/search.rs`

**Step 1: Add civil cases to build_index**

In the `build_index` function, add a section after criminal cases:

```rust
// Index civil cases
let civil_cases = sqlx::query_as(
    "SELECT id::TEXT, court_id, case_number, title, COALESCE(nature_of_suit, '') FROM civil_cases"
)
.fetch_all(pool).await.unwrap_or_default();

for (id, court_id, case_number, title, nos) in &civil_cases {
    let _ = writer.add_document(doc!(
        search.id => id.as_str(),
        search.entity_type => "civil_case",
        search.title => format!("{} - {}", case_number, title).as_str(),
        search.subtitle => format!("Civil - NOS {}", nos).as_str(),
        search.court_id => court_id.as_str(),
    ));
}
```

**Step 2: Update command palette navigation**

In `command_palette.rs`, add routing for `entity_type == "civil_case"` — navigate to the civil case detail page.

**Step 3: Verify**

Run: `cargo check -p server --features server`
Run: `cargo check -p app`

**Step 4: Commit**

```bash
git add crates/server/src/search.rs crates/app/src/routes/command_palette.rs
git commit -m "feat(civil): add civil cases to Tantivy search index"
```

---

## Task 12: Full verification

**Step 1:** Run `sqlx migrate run`
**Step 2:** Run `cargo sqlx prepare --workspace`
**Step 3:** Run `cargo test -p tests -- --test-threads=1` — all tests pass
**Step 4:** Run `dx build --package app --platform web` — builds clean
**Step 5:** Verify civil case count: `psql ... -c "SELECT count(*) FROM civil_cases;"`
**Step 6:** Verify NOS codes: `psql ... -c "SELECT count(*) FROM nature_of_suit_codes;"`
**Step 7:** Commit sqlx cache

---

## Summary

| Task | What it delivers |
|------|-----------------|
| Task 1 | civil_cases table + NOS codes reference table |
| Task 2 | case_type column on 12 shared tables |
| Task 3 | Civil case shared types (Rust structs, validation) |
| Task 4 | Civil case repo (CRUD + search) |
| Task 5 | Civil case REST handlers (8 endpoints) |
| Task 6 | Integration tests (~15 tests) |
| Task 7 | Server functions + UI toggle on Cases page |
| Task 8 | Typst PDF templates (JS-44, summons, scheduling order, judgment) |
| Task 9 | Queue auto-creation + FRCP rules seed |
| Task 10 | 8 realistic seed civil cases |
| Task 11 | Tantivy search index for civil cases |
| Task 12 | Full verification |
