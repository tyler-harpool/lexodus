# UI Phase 3: Attorney Detail Hub + Judge Detail Hub — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the two secondary hub screens — a 7-tab Attorney Detail and a 7-tab Judge Detail — plus their supporting server functions and the Judge List page. Transform the flat-card attorney detail page into a tabbed hub matching the case detail pattern.

**Architecture:** Both hubs follow the proven case detail pattern: `Tabs { default_value, horizontal: true }` with `TabList` + `TabTrigger` + `TabContent`. Each tab lives in its own file under a `tabs/` subdirectory. Tabs use `use_resource()` for async data, `DataTable` for lists, `Sheet` for create forms. Server functions follow the established pattern: `get_db().await` → call repo → serialize to JSON string.

**Tech Stack:** Dioxus (Rust), shared-ui components (Tabs, DataTable, Sheet, Card, Badge), server functions calling repo layer, serde_json

**Reference:** Design at `docs/plans/2026-02-15-lexodus-ui-ux-design.md` (Section 5: Secondary Hub Screens), Phase 1 at `docs/plans/2026-02-15-ui-phase1-infrastructure.md`, Phase 2 at `docs/plans/2026-02-15-ui-phase2-server-functions-case-hub.md`

**Current State:**
- Attorney detail (`crates/app/src/routes/attorneys/detail.rs`): 216-line flat card layout (Basic Info, Contact, Address, Practice Details). Needs conversion to 7-tab hub.
- Judge detail (`crates/app/src/routes/judges/detail.rs`): Stub placeholder. Needs full 7-tab hub.
- Judge list (`crates/app/src/routes/judges/list.rs`): Stub placeholder. Needs DataTable + create sheet.
- Case detail hub (`crates/app/src/routes/cases/detail.rs`): Proven 10-tab pattern to follow.
- Existing server functions: Attorney CRUD (6), Judge CRUD (6+), Opinions (14), Representations (5), Case Assignments (3), Conflicts (3), Recusals (3). ZERO for: bar_admission, federal_admission, cja_appointment, pro_hac_vice, discipline, practice_area, ecf_registration.

---

## Part A: Attorney Sub-Domain Server Functions

### Task 1: Bar Admission + Federal Admission Server Functions

**File:** Modify `crates/server/src/api.rs`

**Step 1: Write bar admission server functions**

Append these 4 functions to `api.rs`:

```rust
// ── Bar Admissions ──────────────────────────────────────────

pub async fn list_bar_admissions(court_id: String, attorney_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::bar_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = bar_admission::list_by_attorney(pool, &court_id, att_uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

pub async fn create_bar_admission(court_id: String, attorney_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::bar_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let req: shared_types::CreateBarAdmissionRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = bar_admission::create(pool, &court_id, att_uuid, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

pub async fn delete_bar_admission(court_id: String, attorney_id: String, state: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::bar_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    bar_admission::delete_by_state(pool, &court_id, att_uuid, &state).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}
```

**Step 2: Write federal admission server functions**

Append these 3 functions:

```rust
// ── Federal Admissions ──────────────────────────────────────

pub async fn list_federal_admissions(court_id: String, attorney_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::federal_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = federal_admission::list_by_attorney(pool, &court_id, att_uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

pub async fn create_federal_admission(court_id: String, attorney_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::federal_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let req: shared_types::CreateFederalAdmissionRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = federal_admission::create(pool, &court_id, att_uuid, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

pub async fn delete_federal_admission(court_id: String, attorney_id: String, court_name: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::federal_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    federal_admission::delete_by_court_name(pool, &court_id, att_uuid, &court_name).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}
```

**Step 3:** Run `cargo check -p server --features server` — verify compiles.

**Step 4:** Commit: `git commit -m "feat(api): add bar admission and federal admission server functions"`

---

### Task 2: CJA Appointment Server Functions

**File:** Modify `crates/server/src/api.rs`

**Step 1: Write CJA appointment server functions**

```rust
// ── CJA Appointments ────────────────────────────────────────

pub async fn list_cja_appointments(court_id: String, attorney_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::cja_appointment;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = cja_appointment::list_by_attorney(pool, &court_id, att_uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

pub async fn create_cja_appointment(court_id: String, attorney_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::cja_appointment;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let req: shared_types::CreateCjaAppointmentRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = cja_appointment::create(pool, &court_id, att_uuid, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

pub async fn list_pending_cja_vouchers(court_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::cja_appointment;

    let pool = get_db().await;
    let rows = cja_appointment::list_pending_vouchers(pool, &court_id).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}
```

**Step 2:** Run `cargo check -p server --features server`

**Step 3:** Commit: `git commit -m "feat(api): add CJA appointment server functions"`

---

### Task 3: Pro Hac Vice Server Functions

**File:** Modify `crates/server/src/api.rs`

**Step 1: Write pro hac vice server functions**

```rust
// ── Pro Hac Vice ────────────────────────────────────────────

pub async fn list_pro_hac_vice(court_id: String, attorney_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::pro_hac_vice;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = pro_hac_vice::list_by_attorney(pool, &court_id, att_uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

pub async fn create_pro_hac_vice(court_id: String, attorney_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::pro_hac_vice;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let req: shared_types::CreateProHacViceRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = pro_hac_vice::create(pool, &court_id, att_uuid, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

pub async fn update_pro_hac_vice_status(court_id: String, attorney_id: String, case_id: String, new_status: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::pro_hac_vice;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let case_uuid = Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let row = pro_hac_vice::update_status(pool, &court_id, att_uuid, case_uuid, &new_status).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("PHV record not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}
```

**Step 2:** Run `cargo check -p server --features server`

**Step 3:** Commit: `git commit -m "feat(api): add pro hac vice server functions"`

---

### Task 4: Discipline + Practice Area Server Functions

**File:** Modify `crates/server/src/api.rs`

**Step 1: Write discipline server functions**

```rust
// ── Discipline Records ──────────────────────────────────────

pub async fn list_discipline_records(court_id: String, attorney_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::discipline;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = discipline::list_by_attorney(pool, &court_id, att_uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

pub async fn create_discipline_record(court_id: String, attorney_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::discipline;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let req: shared_types::CreateDisciplineRecordRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = discipline::create(pool, &court_id, att_uuid, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}
```

**Step 2: Write practice area server functions**

```rust
// ── Practice Areas ──────────────────────────────────────────

pub async fn list_practice_areas(court_id: String, attorney_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::practice_area;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = practice_area::list_by_attorney(pool, &court_id, att_uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}

pub async fn add_practice_area(court_id: String, attorney_id: String, area: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::practice_area;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = practice_area::add(pool, &court_id, att_uuid, &area).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

pub async fn remove_practice_area(court_id: String, attorney_id: String, area: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::practice_area;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    practice_area::remove(pool, &court_id, att_uuid, &area).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}
```

**Step 3:** Run `cargo check -p server --features server`

**Step 4:** Commit: `git commit -m "feat(api): add discipline and practice area server functions"`

---

### Task 5: ECF Registration Server Functions

**File:** Modify `crates/server/src/api.rs`

**Step 1: Write ECF registration server functions**

```rust
// ── ECF Registration ────────────────────────────────────────

pub async fn get_ecf_registration(court_id: String, attorney_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::ecf_registration;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = ecf_registration::find_by_attorney(pool, &court_id, att_uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

pub async fn upsert_ecf_registration(court_id: String, attorney_id: String, status: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::ecf_registration;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = ecf_registration::upsert(pool, &court_id, att_uuid, &status).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}

pub async fn revoke_ecf_registration(court_id: String, attorney_id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::ecf_registration;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id).map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    ecf_registration::revoke(pool, &court_id, att_uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}
```

**Step 2:** Run `cargo check -p server --features server`

**Step 3:** Commit: `git commit -m "feat(api): add ECF registration server functions"`

---

### Task 6: Judge Sub-Domain Server Functions (list_by_judge gaps)

**File:** Modify `crates/server/src/repo/case_assignment.rs` AND `crates/server/src/api.rs`

The existing `list_case_assignments` server function filters by `case_id`, but the judge caseload tab needs to filter by `judge_id`. We need a new repo function and server function.

**Step 1: Add `list_by_judge` to case_assignment repo**

Add to `crates/server/src/repo/case_assignment.rs`:

```rust
pub async fn list_by_judge(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Uuid,
) -> Result<Vec<CaseAssignment>, AppError> {
    let rows = sqlx::query_as::<_, CaseAssignment>(
        "SELECT * FROM case_assignments WHERE court_id = $1 AND judge_id = $2 ORDER BY assigned_date DESC"
    )
    .bind(court_id)
    .bind(judge_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

**Step 2: Add server function wrapper to api.rs**

```rust
pub async fn list_assignments_by_judge(court_id: String, judge_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_assignment;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid = Uuid::parse_str(&judge_id).map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let rows = case_assignment::list_by_judge(pool, &court_id, j_uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}
```

**Step 3: Add `list_recusals_by_judge` server function** (repo function already exists)

```rust
pub async fn list_recusals_by_judge(court_id: String, judge_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::recusal_motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid = Uuid::parse_str(&judge_id).map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let rows = recusal_motion::list_by_judge(pool, &court_id, j_uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}
```

**Step 4:** Run `cargo check -p server --features server`

**Step 5:** Commit: `git commit -m "feat(api): add list_by_judge for case assignments and recusals"`

---

### Task 7: Verify All New Server Functions Compile

**Step 1:** Run full workspace check:
```bash
cargo check --workspace
```
Expected: Clean compilation.

**Step 2:** Count server functions added in Part A:
```bash
grep -c "pub async fn" crates/server/src/api.rs
```
Expected: ~195+ (previous ~170 + ~25 new).

**Step 3:** Commit if any fixes needed.

---

## Part B: Attorney Detail Hub (7 Tabs)

### Task 8: Create Attorney Tabs Directory + Module

**Files:**
- Create: `crates/app/src/routes/attorneys/tabs/mod.rs`
- Create: `crates/app/src/routes/attorneys/tabs/profile.rs` (empty stub)
- Create: `crates/app/src/routes/attorneys/tabs/admissions.rs` (empty stub)
- Create: `crates/app/src/routes/attorneys/tabs/cja.rs` (empty stub)
- Create: `crates/app/src/routes/attorneys/tabs/cases.rs` (empty stub)
- Create: `crates/app/src/routes/attorneys/tabs/metrics.rs` (empty stub)
- Create: `crates/app/src/routes/attorneys/tabs/discipline.rs` (empty stub)
- Create: `crates/app/src/routes/attorneys/tabs/pro_hac_vice.rs` (empty stub)
- Modify: `crates/app/src/routes/attorneys/mod.rs`

**Step 1: Create tabs/mod.rs**

```rust
pub mod admissions;
pub mod cases;
pub mod cja;
pub mod discipline;
pub mod metrics;
pub mod pro_hac_vice;
pub mod profile;
```

**Step 2: Create stub files** for each tab — each should contain:

```rust
use dioxus::prelude::*;

#[component]
pub fn [TabName]Tab(attorney_id: String) -> Element {
    rsx! { p { "[Tab Name] — loading..." } }
}
```

Where `[TabName]` is: `Profile`, `Admissions`, `Cja`, `AttorneyCases`, `AttorneyMetrics`, `Discipline`, `ProHacVice`

**Step 3: Update attorneys/mod.rs** to include the tabs module:

```rust
pub mod list;
pub mod create;
pub mod detail;
pub mod tabs;
```

**Step 4:** Run `cargo check -p app --features server`

**Step 5:** Commit: `git commit -m "feat(ui): scaffold attorney detail 7-tab directory structure"`

---

### Task 9: Attorney Profile Tab

**File:** Create `crates/app/src/routes/attorneys/tabs/profile.rs`

Extract the current flat-card content from `detail.rs` into this tab component. This tab shows: Basic Information card, Contact card, Address card, Practice Details card, plus practice areas list and ECF registration status.

```rust
use dioxus::prelude::*;
use shared_types::AttorneyResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardHeader, CardTitle,
    DataTable, DataTableBody, DataTableCell, DataTableColumn, DataTableHeader, DataTableRow,
    DetailGrid, DetailItem, DetailList, Skeleton,
};
use crate::CourtContext;

#[component]
pub fn ProfileTab(attorney: AttorneyResponse, attorney_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let practice_areas = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let aid = attorney_id.clone();
        async move {
            server::api::list_practice_areas(court, aid).await.ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    let ecf = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let aid = attorney_id.clone();
        async move {
            server::api::get_ecf_registration(court, aid).await.ok()
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
        }
    });

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Basic Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Bar Number", value: attorney.bar_number.clone() }
                        DetailItem { label: "First Name", value: attorney.first_name.clone() }
                        DetailItem { label: "Last Name", value: attorney.last_name.clone() }
                        if let Some(mid) = &attorney.middle_name {
                            DetailItem { label: "Middle Name", value: mid.clone() }
                        }
                        if let Some(firm) = &attorney.firm_name {
                            DetailItem { label: "Firm", value: firm.clone() }
                        }
                        DetailItem { label: "Status",
                            Badge {
                                variant: status_badge_variant(&attorney.status),
                                "{attorney.status}"
                            }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Contact" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Email", value: attorney.email.clone() }
                        DetailItem { label: "Phone", value: attorney.phone.clone() }
                        if let Some(fax) = &attorney.fax {
                            DetailItem { label: "Fax", value: fax.clone() }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Address" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Street", value: attorney.address.street1.clone() }
                        if let Some(s2) = &attorney.address.street2 {
                            DetailItem { label: "Street 2", value: s2.clone() }
                        }
                        DetailItem { label: "City", value: attorney.address.city.clone() }
                        DetailItem { label: "State", value: attorney.address.state.clone() }
                        DetailItem { label: "ZIP", value: attorney.address.zip_code.clone() }
                        DetailItem { label: "Country", value: attorney.address.country.clone() }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Practice Areas" } }
                CardContent {
                    match &*practice_areas.read() {
                        Some(Some(areas)) if !areas.is_empty() => rsx! {
                            div { class: "badge-group",
                                for area in areas.iter() {
                                    Badge { variant: BadgeVariant::Secondary,
                                        {area["area"].as_str().unwrap_or("—")}
                                    }
                                }
                            }
                        },
                        Some(_) => rsx! { p { class: "text-muted", "No practice areas listed." } },
                        None => rsx! { Skeleton {} },
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "ECF Registration" } }
                CardContent {
                    match &*ecf.read() {
                        Some(Some(reg)) => rsx! {
                            DetailList {
                                DetailItem { label: "Status",
                                    Badge { variant: BadgeVariant::Primary,
                                        {reg["status"].as_str().unwrap_or("unknown")}
                                    }
                                }
                                DetailItem {
                                    label: "Registered",
                                    value: reg["registration_date"].as_str().unwrap_or("—").to_string()
                                }
                            }
                        },
                        Some(None) => rsx! { p { class: "text-muted", "Not registered for ECF." } },
                        None => rsx! { Skeleton {} },
                    }
                }
            }
        }
    }
}

fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Inactive" => BadgeVariant::Secondary,
        "Suspended" => BadgeVariant::Destructive,
        "Retired" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}
```

**Step 1:** Write the profile tab file.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement attorney Profile tab with practice areas and ECF status"`

---

### Task 10: Attorney Admissions Tab (Bar + Federal)

**File:** Create `crates/app/src/routes/attorneys/tabs/admissions.rs`

Two DataTables side-by-side: state bar admissions and federal court admissions. Each has an "Add" button that opens a Sheet.

```rust
use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardHeader, CardTitle,
    DataTable, DataTableBody, DataTableCell, DataTableColumn, DataTableHeader, DataTableRow,
    Form, Input, FormSelect,
    Sheet, SheetClose, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle,
    Skeleton,
};
use shared_ui::{use_toast, ToastOptions};
use crate::CourtContext;

#[component]
pub fn AdmissionsTab(attorney_id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // ── State Bar Admissions ──
    let mut show_bar_sheet = use_signal(|| false);
    let mut bar_state = use_signal(String::new);
    let mut bar_number = use_signal(String::new);

    let aid_bar = attorney_id.clone();
    let mut bar_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let aid = aid_bar.clone();
        async move {
            server::api::list_bar_admissions(court, aid).await.ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    let aid_bar_save = attorney_id.clone();
    let handle_bar_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let aid = aid_bar_save.clone();
        let st = bar_state.read().clone();
        let bn = bar_number.read().clone();
        spawn(async move {
            let body = serde_json::json!({ "state": st, "bar_number": bn });
            match server::api::create_bar_admission(court, aid, body.to_string()).await {
                Ok(_) => {
                    toast.success("Bar admission added.".to_string(), ToastOptions::new());
                    show_bar_sheet.set(false);
                    bar_state.set(String::new());
                    bar_number.set(String::new());
                    bar_data.restart();
                }
                Err(e) => toast.error(format!("Error: {e}"), ToastOptions::new()),
            }
        });
    };

    // ── Federal Court Admissions ──
    let mut show_fed_sheet = use_signal(|| false);
    let mut fed_court_name = use_signal(String::new);

    let aid_fed = attorney_id.clone();
    let mut fed_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let aid = aid_fed.clone();
        async move {
            server::api::list_federal_admissions(court, aid).await.ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    let aid_fed_save = attorney_id.clone();
    let handle_fed_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let aid = aid_fed_save.clone();
        let cn = fed_court_name.read().clone();
        spawn(async move {
            let body = serde_json::json!({ "court_name": cn });
            match server::api::create_federal_admission(court, aid, body.to_string()).await {
                Ok(_) => {
                    toast.success("Federal admission added.".to_string(), ToastOptions::new());
                    show_fed_sheet.set(false);
                    fed_court_name.set(String::new());
                    fed_data.restart();
                }
                Err(e) => toast.error(format!("Error: {e}"), ToastOptions::new()),
            }
        });
    };

    rsx! {
        // State Bar Admissions Section
        div { class: "tab-section",
            div { class: "tab-header",
                h3 { "State Bar Admissions" }
                Button { variant: ButtonVariant::Primary, onclick: move |_| show_bar_sheet.set(true), "Add Admission" }
            }
            match &*bar_data.read() {
                Some(Some(rows)) if !rows.is_empty() => rsx! {
                    DataTable {
                        DataTableHeader {
                            DataTableColumn { "State" }
                            DataTableColumn { "Bar Number" }
                            DataTableColumn { "Admission Date" }
                            DataTableColumn { "Status" }
                        }
                        DataTableBody {
                            for row in rows.iter() {
                                DataTableRow {
                                    DataTableCell { {row["state"].as_str().unwrap_or("—")} }
                                    DataTableCell { {row["bar_number"].as_str().unwrap_or("—")} }
                                    DataTableCell { {row["admission_date"].as_str().unwrap_or("—").chars().take(10).collect::<String>()} }
                                    DataTableCell {
                                        Badge { variant: BadgeVariant::Primary,
                                            {row["status"].as_str().unwrap_or("active")}
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Some(_) => rsx! { p { class: "empty-state", "No state bar admissions on file." } },
                None => rsx! { Skeleton { width: "100%", height: "120px" } },
            }
        }

        // Federal Court Admissions Section
        div { class: "tab-section",
            div { class: "tab-header",
                h3 { "Federal Court Admissions" }
                Button { variant: ButtonVariant::Primary, onclick: move |_| show_fed_sheet.set(true), "Add Admission" }
            }
            match &*fed_data.read() {
                Some(Some(rows)) if !rows.is_empty() => rsx! {
                    DataTable {
                        DataTableHeader {
                            DataTableColumn { "Court" }
                            DataTableColumn { "Admission Date" }
                            DataTableColumn { "Status" }
                        }
                        DataTableBody {
                            for row in rows.iter() {
                                DataTableRow {
                                    DataTableCell { {row["court_name"].as_str().unwrap_or("—")} }
                                    DataTableCell { {row["admission_date"].as_str().unwrap_or("—").chars().take(10).collect::<String>()} }
                                    DataTableCell {
                                        Badge { variant: BadgeVariant::Primary,
                                            {row["status"].as_str().unwrap_or("active")}
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Some(_) => rsx! { p { class: "empty-state", "No federal court admissions on file." } },
                None => rsx! { Skeleton { width: "100%", height: "120px" } },
            }
        }

        // Bar Admission Sheet
        Sheet { open: show_bar_sheet(), side: SheetSide::Right,
            SheetHeader { SheetTitle { "Add Bar Admission" } }
            SheetContent {
                Form { onsubmit: handle_bar_save,
                    Input { value: "{bar_state}", oninput: move |e: FormEvent| bar_state.set(e.value()), placeholder: "State (e.g., CA, NY)" }
                    Input { value: "{bar_number}", oninput: move |e: FormEvent| bar_number.set(e.value()), placeholder: "Bar Number" }
                }
            }
            SheetFooter {
                SheetClose { Button { variant: ButtonVariant::Secondary, "Cancel" } }
                Button { variant: ButtonVariant::Primary, onclick: handle_bar_save, "Save" }
            }
        }

        // Federal Admission Sheet
        Sheet { open: show_fed_sheet(), side: SheetSide::Right,
            SheetHeader { SheetTitle { "Add Federal Admission" } }
            SheetContent {
                Form { onsubmit: handle_fed_save,
                    Input { value: "{fed_court_name}", oninput: move |e: FormEvent| fed_court_name.set(e.value()), placeholder: "Federal Court Name" }
                }
            }
            SheetFooter {
                SheetClose { Button { variant: ButtonVariant::Secondary, "Cancel" } }
                Button { variant: ButtonVariant::Primary, onclick: handle_fed_save, "Save" }
            }
        }
    }
}
```

**Step 1:** Write the admissions tab file.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement attorney Admissions tab with bar and federal admissions"`

---

### Task 11: Attorney CJA Tab

**File:** Create `crates/app/src/routes/attorneys/tabs/cja.rs`

DataTable showing CJA appointments (case link, appointment date, termination date, voucher status, amount). "Add Appointment" sheet calls `create_cja_appointment`.

Follow the same DataTable + Sheet pattern as the Admissions tab. Columns: Case ID, Appointment Date, Termination Date, Voucher Status (badge), Voucher Amount.

**Step 1:** Write the CJA tab file following the Admissions tab pattern.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement attorney CJA tab with appointment management"`

---

### Task 12: Attorney Cases Tab

**File:** Create `crates/app/src/routes/attorneys/tabs/cases.rs`

DataTable showing active representations via `list_active_representations` server function (already exists). Columns: Case ID, Role, Start Date, Status. Click a case row navigates to case detail via `Route::CaseDetail`.

**Step 1:** Write the cases tab file. Use `list_active_representations(court_id, attorney_id)`.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement attorney Cases tab with active representations"`

---

### Task 13: Attorney Metrics Tab

**File:** Create `crates/app/src/routes/attorneys/tabs/metrics.rs`

Display stats cards from the `AttorneyResponse` data already loaded in the parent (no additional API calls needed). Cards: Cases Handled, Win Rate, Avg Case Duration, Languages Spoken, CJA Panel Status.

```rust
use dioxus::prelude::*;
use shared_types::AttorneyResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DetailGrid, DetailItem, DetailList,
};

#[component]
pub fn AttorneyMetricsTab(attorney: AttorneyResponse) -> Element {
    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Caseload" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Cases Handled", value: attorney.cases_handled.to_string() }
                        DetailItem { label: "CJA Panel Member",
                            Badge {
                                variant: if attorney.cja_panel_member { BadgeVariant::Primary } else { BadgeVariant::Secondary },
                                if attorney.cja_panel_member { "Yes" } else { "No" }
                            }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Performance" } }
                CardContent {
                    DetailList {
                        if let Some(wr) = attorney.win_rate_percentage {
                            DetailItem { label: "Win Rate", value: format!("{:.1}%", wr) }
                        }
                        if let Some(dur) = attorney.avg_case_duration_days {
                            DetailItem { label: "Avg Case Duration", value: format!("{} days", dur) }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Languages" } }
                CardContent {
                    if !attorney.languages_spoken.is_empty() {
                        div { class: "badge-group",
                            for lang in attorney.languages_spoken.iter() {
                                Badge { variant: BadgeVariant::Outline, "{lang}" }
                            }
                        }
                    } else {
                        p { class: "text-muted", "No languages listed." }
                    }
                }
            }
        }
    }
}
```

**Step 1:** Write the metrics tab file.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement attorney Metrics tab with performance stats"`

---

### Task 14: Attorney Discipline Tab

**File:** Create `crates/app/src/routes/attorneys/tabs/discipline.rs`

DataTable showing discipline records via `list_discipline_records`. Columns: Action Type, Jurisdiction, Date, Effective Date, End Date, Description. "Add Record" sheet calls `create_discipline_record`.

Follow the DataTable + Sheet pattern. Use destructive badge variant for active discipline actions.

**Step 1:** Write the discipline tab file.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement attorney Discipline tab"`

---

### Task 15: Attorney Pro Hac Vice Tab

**File:** Create `crates/app/src/routes/attorneys/tabs/pro_hac_vice.rs`

DataTable showing PHV applications via `list_pro_hac_vice`. Columns: Case ID, Sponsoring Attorney, Status (badge), Admission Date, Expiration Date. "New Application" sheet calls `create_pro_hac_vice`.

**Step 1:** Write the PHV tab file.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement attorney Pro Hac Vice tab"`

---

### Task 16: Rewrite Attorney Detail as 7-Tab Hub

**File:** Modify `crates/app/src/routes/attorneys/detail.rs`

Replace the flat-card layout with a tabbed hub following the case detail pattern. Keep the PageHeader and delete dialog. Add `Tabs` with 7 tabs. Import from the tabs module.

```rust
use dioxus::prelude::*;
use shared_types::AttorneyResponse;
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, PageActions, PageHeader, PageTitle, Skeleton,
    TabContent, TabList, TabTrigger, Tabs,
};

use super::tabs::{
    admissions::AdmissionsTab,
    cases::AttorneyCasesTab,
    cja::CjaTab,
    discipline::DisciplineTab,
    metrics::AttorneyMetricsTab,
    pro_hac_vice::ProHacViceTab,
    profile::ProfileTab,
};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn AttorneyDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let attorney_id = id.clone();

    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let data = use_resource(move || {
        let court = court_id.clone();
        let aid = attorney_id.clone();
        async move {
            match server::api::get_attorney(court, aid).await {
                Ok(json) => serde_json::from_str::<AttorneyResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let aid = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_attorney(court, aid).await {
                Ok(()) => navigator().push(Route::AttorneyList {}),
                Err(_) => {
                    deleting.set(false);
                    show_delete_confirm.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(att)) => rsx! {
                    PageHeader {
                        PageTitle { "{att.last_name}, {att.first_name}" }
                        PageActions {
                            Link { to: Route::AttorneyList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                            Badge { variant: status_badge_variant(&att.status), "{att.status}" }
                            Button {
                                variant: ButtonVariant::Destructive,
                                onclick: move |_| show_delete_confirm.set(true),
                                "Delete"
                            }
                        }
                    }

                    AlertDialogRoot {
                        open: show_delete_confirm(),
                        on_open_change: move |v| show_delete_confirm.set(v),
                        AlertDialogContent {
                            AlertDialogTitle { "Delete Attorney" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this attorney? This action cannot be undone."
                            }
                            AlertDialogActions {
                                AlertDialogCancel { "Cancel" }
                                AlertDialogAction {
                                    on_click: handle_delete,
                                    if *deleting.read() { "Deleting..." } else { "Delete" }
                                }
                            }
                        }
                    }

                    Tabs { default_value: "profile", horizontal: true,
                        TabList {
                            TabTrigger { value: "profile", index: 0usize, "Profile" }
                            TabTrigger { value: "admissions", index: 1usize, "Admissions" }
                            TabTrigger { value: "cja", index: 2usize, "CJA" }
                            TabTrigger { value: "cases", index: 3usize, "Cases" }
                            TabTrigger { value: "metrics", index: 4usize, "Metrics" }
                            TabTrigger { value: "discipline", index: 5usize, "Discipline" }
                            TabTrigger { value: "phv", index: 6usize, "Pro Hac Vice" }
                        }
                        TabContent { value: "profile", index: 0usize,
                            ProfileTab { attorney: att.clone(), attorney_id: id.clone() }
                        }
                        TabContent { value: "admissions", index: 1usize,
                            AdmissionsTab { attorney_id: id.clone() }
                        }
                        TabContent { value: "cja", index: 2usize,
                            CjaTab { attorney_id: id.clone() }
                        }
                        TabContent { value: "cases", index: 3usize,
                            AttorneyCasesTab { attorney_id: id.clone() }
                        }
                        TabContent { value: "metrics", index: 4usize,
                            AttorneyMetricsTab { attorney: att.clone() }
                        }
                        TabContent { value: "discipline", index: 5usize,
                            DisciplineTab { attorney_id: id.clone() }
                        }
                        TabContent { value: "phv", index: 6usize,
                            ProHacViceTab { attorney_id: id.clone() }
                        }
                    }
                },
                Some(None) => rsx! {
                    PageHeader {
                        PageTitle { "Attorney Not Found" }
                        PageActions {
                            Link { to: Route::AttorneyList {},
                                Button { "Back to List" }
                            }
                        }
                    }
                },
                None => rsx! {
                    div { class: "loading", Skeleton {} Skeleton {} Skeleton {} }
                },
            }
        }
    }
}

fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Inactive" => BadgeVariant::Secondary,
        "Suspended" => BadgeVariant::Destructive,
        "Retired" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}
```

**Step 1:** Replace `crates/app/src/routes/attorneys/detail.rs` with the tabbed hub above.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): convert attorney detail page to 7-tab hub"`

---

## Part C: Judge List + Detail Hub

### Task 17: Judge List Page

**File:** Modify `crates/app/src/routes/judges/list.rs`

Replace stub with full DataTable + search + create Sheet. Uses `list_judges` and `search_judges` server functions (both already exist).

DataTable columns: Name, Title, District, Status (badge), Courtroom, Caseload (current/max). "Add Judge" button opens create Sheet calling `create_judge`. Search bar filters via `search_judges`.

Follow the attorney list page pattern at `crates/app/src/routes/attorneys/list.rs`.

**Step 1:** Write the full judge list page with DataTable, search, and create Sheet.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement judge list page with DataTable and create sheet"`

---

### Task 18: Create Judge Tabs Directory + Module

**Files:**
- Create: `crates/app/src/routes/judges/tabs/mod.rs`
- Create stubs for: `profile.rs`, `caseload.rs`, `calendar.rs`, `opinions.rs`, `conflicts.rs`, `workload.rs`, `vacation.rs`
- Modify: `crates/app/src/routes/judges/mod.rs`

**Step 1: Create tabs/mod.rs**

```rust
pub mod calendar;
pub mod caseload;
pub mod conflicts;
pub mod opinions;
pub mod profile;
pub mod vacation;
pub mod workload;
```

**Step 2: Create stub files** for each tab (same pattern as Task 8).

**Step 3: Update judges/mod.rs**:

```rust
pub mod list;
pub mod detail;
pub mod tabs;
```

**Step 4:** Run `cargo check -p app --features server`

**Step 5:** Commit: `git commit -m "feat(ui): scaffold judge detail 7-tab directory structure"`

---

### Task 19: Judge Profile Tab

**File:** Create `crates/app/src/routes/judges/tabs/profile.rs`

Show judge info cards: Bio (name, title, district, status, appointed date, senior status date), Chambers (courtroom, specializations), Caseload (current/max with progress indicator).

Uses `get_judge` data passed down from parent — no additional API calls needed.

```rust
use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle,
    DetailGrid, DetailItem, DetailList,
};

#[component]
pub fn JudgeProfileTab(judge: serde_json::Value) -> Element {
    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Biographical Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Name", value: judge["name"].as_str().unwrap_or("—").to_string() }
                        DetailItem { label: "Title", value: judge["title"].as_str().unwrap_or("—").to_string() }
                        DetailItem { label: "District", value: judge["district"].as_str().unwrap_or("—").to_string() }
                        DetailItem { label: "Status",
                            Badge {
                                variant: match judge["status"].as_str().unwrap_or("") {
                                    "Active" => BadgeVariant::Primary,
                                    "Senior" => BadgeVariant::Secondary,
                                    "Retired" => BadgeVariant::Outline,
                                    _ => BadgeVariant::Secondary,
                                },
                                {judge["status"].as_str().unwrap_or("—")}
                            }
                        }
                        if let Some(d) = judge["appointed_date"].as_str() {
                            DetailItem { label: "Appointed", value: d.chars().take(10).collect::<String>() }
                        }
                        if let Some(d) = judge["senior_status_date"].as_str() {
                            DetailItem { label: "Senior Status", value: d.chars().take(10).collect::<String>() }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Chambers" } }
                CardContent {
                    DetailList {
                        if let Some(cr) = judge["courtroom"].as_str() {
                            DetailItem { label: "Courtroom", value: cr.to_string() }
                        }
                        DetailItem {
                            label: "Specializations",
                            value: judge["specializations"].as_array()
                                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                                .unwrap_or_else(|| "None".to_string())
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Caseload" } }
                CardContent {
                    DetailList {
                        DetailItem {
                            label: "Current / Max",
                            value: format!("{} / {}",
                                judge["current_caseload"].as_i64().unwrap_or(0),
                                judge["max_caseload"].as_i64().unwrap_or(0)
                            )
                        }
                    }
                }
            }
        }
    }
}
```

**Step 1:** Write the profile tab file.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement judge Profile tab"`

---

### Task 20: Judge Caseload Tab

**File:** Create `crates/app/src/routes/judges/tabs/caseload.rs`

DataTable showing assigned cases via `list_assignments_by_judge` (added in Task 6). Columns: Case ID, Assignment Type, Assigned Date, Reason.

**Step 1:** Write the caseload tab using `list_assignments_by_judge(court_id, judge_id)`.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement judge Caseload tab with case assignments"`

---

### Task 21: Judge Calendar Tab

**File:** Create `crates/app/src/routes/judges/tabs/calendar.rs`

DataTable showing judge's calendar events via `search_calendar_events` filtered by judge_id. Columns: Date, Event Type, Case, Courtroom, Status. Reuse the existing server function which already accepts `judge_id: Option<String>`.

**Step 1:** Write the calendar tab filtering by judge_id.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement judge Calendar tab"`

---

### Task 22: Judge Opinions Tab

**File:** Create `crates/app/src/routes/judges/tabs/opinions.rs`

DataTable showing authored opinions via `list_opinions_by_judge` (already exists). Columns: Title, Case Name, Type, Status (badge: Draft/Published), Filed Date. Click navigates to opinion detail (future).

**Step 1:** Write the opinions tab using `list_opinions_by_judge(court_id, judge_id)`.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement judge Opinions tab"`

---

### Task 23: Judge Conflicts Tab

**File:** Create `crates/app/src/routes/judges/tabs/conflicts.rs`

Two sections: Conflicts DataTable via `list_judge_conflicts` (columns: Party/Firm/Corp, Conflict Type, Start Date, End Date, Notes) + "Add Conflict" Sheet. Recusals DataTable via `list_recusals_by_judge` (columns: Case, Filed By, Reason, Status badge, Ruling Date). "File Recusal" Sheet calling `create_recusal`.

**Step 1:** Write the conflicts tab with both DataTables and both Sheets.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement judge Conflicts tab with recusals"`

---

### Task 24: Judge Workload Tab

**File:** Create `crates/app/src/routes/judges/tabs/workload.rs`

Stats cards derived from judge data (current_caseload, max_caseload) plus counts from `list_assignments_by_judge` and `list_opinions_by_judge`. Cards: Active Cases, Caseload Utilization (%), Opinions Authored, Pending Recusals.

**Step 1:** Write the workload tab with stats cards.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement judge Workload tab with stats"`

---

### Task 25: Judge Vacation Tab

**File:** Create `crates/app/src/routes/judges/tabs/vacation.rs`

No vacation-specific types or repos exist yet. Implement as a placeholder card with a message that vacation scheduling is planned, or filter calendar events where event_type = "Vacation" or "Out of Office".

```rust
use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader, CardTitle};

#[component]
pub fn VacationTab(judge_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { CardTitle { "Vacation Schedule" } }
            CardContent {
                p { class: "text-muted",
                    "Vacation schedule management is planned for a future release. "
                    "Use the Calendar tab to view scheduled time off."
                }
            }
        }
    }
}
```

**Step 1:** Write the vacation tab placeholder.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): add judge Vacation tab placeholder"`

---

### Task 26: Rewrite Judge Detail as 7-Tab Hub

**File:** Modify `crates/app/src/routes/judges/detail.rs`

Replace the stub with a full tabbed hub. Fetch judge data via `get_judge`, display PageHeader with name and status badge, delete dialog, and 7-tab Tabs component.

```rust
use dioxus::prelude::*;
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, PageActions, PageHeader, PageTitle, Skeleton,
    TabContent, TabList, TabTrigger, Tabs,
};

use super::tabs::{
    calendar::JudgeCalendarTab,
    caseload::CaseloadTab,
    conflicts::ConflictsTab,
    opinions::OpinionsTab,
    profile::JudgeProfileTab,
    vacation::VacationTab,
    workload::WorkloadTab,
};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn JudgeDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let judge_id = id.clone();

    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let data = use_resource(move || {
        let court = court_id.clone();
        let jid = judge_id.clone();
        async move {
            match server::api::get_judge(court, jid).await {
                Ok(json) => serde_json::from_str::<serde_json::Value>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let jid = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_judge(court, jid).await {
                Ok(()) => navigator().push(Route::JudgeList {}),
                Err(_) => {
                    deleting.set(false);
                    show_delete_confirm.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(judge)) => rsx! {
                    PageHeader {
                        PageTitle { {judge["name"].as_str().unwrap_or("Judge")} }
                        PageActions {
                            Link { to: Route::JudgeList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                            Badge {
                                variant: match judge["status"].as_str().unwrap_or("") {
                                    "Active" => BadgeVariant::Primary,
                                    "Senior" => BadgeVariant::Secondary,
                                    "Retired" => BadgeVariant::Outline,
                                    _ => BadgeVariant::Secondary,
                                },
                                {judge["status"].as_str().unwrap_or("—")}
                            }
                            Button {
                                variant: ButtonVariant::Destructive,
                                onclick: move |_| show_delete_confirm.set(true),
                                "Delete"
                            }
                        }
                    }

                    AlertDialogRoot {
                        open: show_delete_confirm(),
                        on_open_change: move |v| show_delete_confirm.set(v),
                        AlertDialogContent {
                            AlertDialogTitle { "Delete Judge" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this judge record? This action cannot be undone."
                            }
                            AlertDialogActions {
                                AlertDialogCancel { "Cancel" }
                                AlertDialogAction {
                                    on_click: handle_delete,
                                    if *deleting.read() { "Deleting..." } else { "Delete" }
                                }
                            }
                        }
                    }

                    Tabs { default_value: "profile", horizontal: true,
                        TabList {
                            TabTrigger { value: "profile", index: 0usize, "Profile" }
                            TabTrigger { value: "caseload", index: 1usize, "Caseload" }
                            TabTrigger { value: "calendar", index: 2usize, "Calendar" }
                            TabTrigger { value: "opinions", index: 3usize, "Opinions" }
                            TabTrigger { value: "conflicts", index: 4usize, "Conflicts" }
                            TabTrigger { value: "workload", index: 5usize, "Workload" }
                            TabTrigger { value: "vacation", index: 6usize, "Vacation" }
                        }
                        TabContent { value: "profile", index: 0usize,
                            JudgeProfileTab { judge: judge.clone() }
                        }
                        TabContent { value: "caseload", index: 1usize,
                            CaseloadTab { judge_id: id.clone() }
                        }
                        TabContent { value: "calendar", index: 2usize,
                            JudgeCalendarTab { judge_id: id.clone() }
                        }
                        TabContent { value: "opinions", index: 3usize,
                            OpinionsTab { judge_id: id.clone() }
                        }
                        TabContent { value: "conflicts", index: 4usize,
                            ConflictsTab { judge_id: id.clone() }
                        }
                        TabContent { value: "workload", index: 5usize,
                            WorkloadTab { judge_id: id.clone() }
                        }
                        TabContent { value: "vacation", index: 6usize,
                            VacationTab { judge_id: id.clone() }
                        }
                    }
                },
                Some(None) => rsx! {
                    PageHeader {
                        PageTitle { "Judge Not Found" }
                        PageActions {
                            Link { to: Route::JudgeList {},
                                Button { "Back to List" }
                            }
                        }
                    }
                },
                None => rsx! {
                    div { class: "loading", Skeleton {} Skeleton {} Skeleton {} }
                },
            }
        }
    }
}
```

**Step 1:** Replace `crates/app/src/routes/judges/detail.rs` with the tabbed hub above.

**Step 2:** Run `cargo check -p app --features server`

**Step 3:** Commit: `git commit -m "feat(ui): implement judge detail page as 7-tab hub"`

---

## Part D: Verification

### Task 27: Full Build Verification

**Step 1:** Run full workspace check:
```bash
cargo check --workspace
```
Expected: Clean compilation.

**Step 2:** Run existing tests:
```bash
cargo test --workspace
```
Expected: All existing tests pass.

**Step 3:** Count total server functions:
```bash
grep -c "pub async fn" crates/server/src/api.rs
```
Expected: ~195+ (previous ~170 + ~25 new from Part A).

**Step 4:** Verify tab file counts:
```bash
ls crates/app/src/routes/attorneys/tabs/ | wc -l
ls crates/app/src/routes/judges/tabs/ | wc -l
```
Expected: 8 attorney tab files (7 + mod.rs), 8 judge tab files (7 + mod.rs).

**Step 5:** Final commit if any fixes needed.

---

## Summary

After completing Tasks 1-27:

**Part A (Tasks 1-7):** ~25 new server functions for attorney sub-domains:
- Bar Admissions (3), Federal Admissions (3), CJA Appointments (3), Pro Hac Vice (3)
- Discipline Records (2), Practice Areas (3), ECF Registration (3)
- Judge: list_assignments_by_judge (1), list_recusals_by_judge (1) + new repo function

**Part B (Tasks 8-16):** Attorney Detail Hub with 7 tabs:
- Profile (basic info, contact, address, practice areas, ECF status)
- Admissions (bar + federal DataTables with create sheets)
- CJA (appointments DataTable with create sheet)
- Cases (active representations DataTable)
- Metrics (stats cards from attorney data)
- Discipline (records DataTable with create sheet)
- Pro Hac Vice (applications DataTable with create sheet)

**Part C (Tasks 17-26):** Judge List Page + Judge Detail Hub with 7 tabs:
- List page with DataTable, search, and create sheet
- Profile (bio, chambers, caseload)
- Caseload (assigned cases DataTable)
- Calendar (judge's events DataTable)
- Opinions (authored opinions DataTable)
- Conflicts (conflicts + recusals DataTables with create sheets)
- Workload (stats cards)
- Vacation (placeholder)

**Part D (Task 27):** Full workspace verification.

## Next Plans

- **Phase 4:** Standalone domain pages (Docket, Filings, Documents, Opinions, Orders, etc.)
- **Phase 5:** Workflow Wizards (New Case Filing, Sentencing Prep, Attorney Onboarding, etc.)
- **Phase 6:** Administration pages + Command Palette entity search
