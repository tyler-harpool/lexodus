# UI Phase 2: Server Function Wrappers + Case Hub Tabs — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Expose ALL missing REST API endpoints as Dioxus server functions in `api.rs`, then flesh out the 7 stub case hub tabs with real DataTables, Sheets, and API integration.

**Architecture:** Server functions follow the established pattern: `get_db().await` → call repo function → serialize to JSON string. Each returns `Result<String, ServerFnError>`. The case hub tabs use `use_resource()` for data fetching, `DataTable` for display, and `Sheet` for create/edit forms.

**Tech Stack:** Dioxus server functions, sqlx repos, serde_json, shared-ui components

**Reference:** Design at `docs/plans/2026-02-15-lexodus-ui-ux-design.md`, Phase 1 at `docs/plans/2026-02-15-ui-phase1-infrastructure.md`

---

## Part A: Server Function Wrappers

### Server Function Pattern Reference

All server functions live in `crates/server/src/api.rs` and follow these 4 patterns:

**Pattern 1 — LIST (with court_id):**
```rust
pub async fn list_defendants(court_id: String, case_id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid = Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = defendant::list_by_case(pool, &court_id, case_uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&rows).unwrap_or_default())
}
```

**Pattern 2 — GET (single by ID):**
```rust
pub async fn get_defendant(court_id: String, id: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = defendant::find_by_id(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}
```

**Pattern 3 — CREATE (JSON body):**
```rust
pub async fn create_defendant(court_id: String, body: String) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;

    let pool = get_db().await;
    let req: shared_types::CreateDefendantRequest = serde_json::from_str(&body)
        .map_err(|e| ServerFnError::new(format!("Invalid request: {}", e)))?;
    let row = defendant::create(pool, &court_id, req).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(serde_json::to_string(&row).unwrap_or_default())
}
```

**Pattern 4 — DELETE:**
```rust
pub async fn delete_defendant(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::defendant;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    defendant::delete(pool, &court_id, uuid).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}
```

---

### Task 1: Defendant Server Functions

**File:** Modify `crates/server/src/api.rs`

Add these 5 functions (use patterns above — `crate::repo::defendant`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_defendants` | `court_id, case_id` | `defendant::list_by_case(pool, &court_id, case_uuid)` | `String` (JSON array) |
| `get_defendant` | `court_id, id` | `defendant::find_by_id(pool, &court_id, uuid)` | `String` |
| `create_defendant` | `court_id, body` | `defendant::create(pool, &court_id, req)` | `String` |
| `update_defendant` | `court_id, id, body` | `defendant::update(pool, &court_id, uuid, req)` | `String` |
| `delete_defendant` | `court_id, id` | `defendant::delete(pool, &court_id, uuid)` | `()` |

**Step 1:** Write all 5 functions following the patterns above.

**Step 2:** Run `cargo check -p server --features server` — verify compiles.

**Step 3:** Commit: `git commit -m "feat(api): add defendant server function wrappers"`

---

### Task 2: Party Server Functions

**File:** Modify `crates/server/src/api.rs`

Add these functions (use `crate::repo::party`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `create_party` | `court_id, body` | `party::create(pool, &court_id, req)` | `String` |
| `get_party` | `court_id, id` | `party::find_by_id(pool, &court_id, uuid)` | `String` |
| `update_party` | `court_id, id, body` | `party::update(pool, &court_id, uuid, req)` | `String` |
| `delete_party` | `court_id, id` | `party::delete(pool, &court_id, uuid)` | `()` |
| `list_parties_by_case` | `court_id, case_id` | `party::list_by_case(pool, &court_id, case_uuid)` | `String` |
| `list_unrepresented_parties` | `court_id` | `party::list_unrepresented(pool, &court_id)` | `String` |
| `list_parties_by_attorney` | `court_id, attorney_id` | `party::list_by_attorney(pool, &court_id, att_uuid)` | `String` |

Note: `list_case_parties` already exists in api.rs — it calls `party::list_full_by_case`. Keep it. The new `list_parties_by_case` calls the simpler `party::list_by_case`.

**Step 1:** Write all 7 functions.
**Step 2:** `cargo check -p server --features server`
**Step 3:** Commit: `git commit -m "feat(api): add party server function wrappers"`

---

### Task 3: Evidence + Custody Transfer Server Functions

**File:** Modify `crates/server/src/api.rs`

Evidence (`crate::repo::evidence`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_evidence_by_case` | `court_id, case_id` | `evidence::list_by_case(pool, &court_id, case_uuid)` | `String` |
| `get_evidence` | `court_id, id` | `evidence::find_by_id(pool, &court_id, uuid)` | `String` |
| `create_evidence` | `court_id, body` | `evidence::create(pool, &court_id, req)` | `String` |
| `update_evidence` | `court_id, id, body` | `evidence::update(pool, &court_id, uuid, req)` | `String` |
| `delete_evidence` | `court_id, id` | `evidence::delete(pool, &court_id, uuid)` | `()` |

Custody Transfers (`crate::repo::custody_transfer`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_custody_transfers` | `court_id, evidence_id` | `custody_transfer::list_by_evidence(pool, &court_id, ev_uuid)` | `String` |
| `create_custody_transfer` | `court_id, body` | `custody_transfer::create(pool, &court_id, req)` | `String` |

**Step 1:** Write all 7 functions.
**Step 2:** `cargo check -p server --features server`
**Step 3:** Commit: `git commit -m "feat(api): add evidence and custody transfer server function wrappers"`

---

### Task 4: Order + Order Template Server Functions

**File:** Modify `crates/server/src/api.rs`

Orders (`crate::repo::order`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_orders_by_case` | `court_id, case_id` | `order::list_by_case(pool, &court_id, case_uuid)` | `String` |
| `list_orders_by_judge` | `court_id, judge_id` | `order::list_by_judge(pool, &court_id, judge_uuid)` | `String` |
| `get_order` | `court_id, id` | `order::find_by_id(pool, &court_id, uuid)` | `String` |
| `create_order` | `court_id, body` | `order::create(pool, &court_id, req)` | `String` |
| `update_order` | `court_id, id, body` | `order::update(pool, &court_id, uuid, req)` | `String` |
| `delete_order` | `court_id, id` | `order::delete(pool, &court_id, uuid)` | `()` |

Order Templates (`crate::repo::order_template`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_order_templates` | `court_id` | `order_template::list_all(pool, &court_id)` | `String` |
| `list_active_order_templates` | `court_id` | `order_template::list_active(pool, &court_id)` | `String` |
| `get_order_template` | `court_id, id` | `order_template::find_by_id(pool, &court_id, uuid)` | `String` |
| `create_order_template` | `court_id, body` | `order_template::create(pool, &court_id, req)` | `String` |
| `update_order_template` | `court_id, id, body` | `order_template::update(pool, &court_id, uuid, req)` | `String` |
| `delete_order_template` | `court_id, id` | `order_template::delete(pool, &court_id, uuid)` | `()` |

**Step 1:** Write all 12 functions.
**Step 2:** `cargo check -p server --features server`
**Step 3:** Commit: `git commit -m "feat(api): add order and order template server function wrappers"`

---

### Task 5: Sentencing Server Functions

**File:** Modify `crates/server/src/api.rs`

Sentencing (`crate::repo::sentencing`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_sentencing_by_case` | `court_id, case_id` | `sentencing::list_by_case(pool, &court_id, case_uuid)` | `String` |
| `list_sentencing_by_defendant` | `court_id, defendant_id` | `sentencing::list_by_defendant(pool, &court_id, def_uuid)` | `String` |
| `get_sentencing` | `court_id, id` | `sentencing::find_by_id(pool, &court_id, uuid)` | `String` |
| `create_sentencing` | `court_id, body` | `sentencing::create(pool, &court_id, req)` | `String` |
| `update_sentencing` | `court_id, id, body` | `sentencing::update(pool, &court_id, uuid, req)` | `String` |
| `delete_sentencing` | `court_id, id` | `sentencing::delete(pool, &court_id, uuid)` | `()` |

Sentencing Conditions (`crate::repo::sentencing_condition`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_sentencing_conditions` | `court_id, sentencing_id` | `sentencing_condition::list_by_sentencing(pool, &court_id, s_uuid)` | `String` |
| `create_sentencing_condition` | `court_id, body` | `sentencing_condition::create(pool, &court_id, req)` | `String` |

**Step 1:** Write all 8 functions.
**Step 2:** `cargo check -p server --features server`
**Step 3:** Commit: `git commit -m "feat(api): add sentencing server function wrappers"`

---

### Task 6: Speedy Trial Server Functions

**File:** Modify `crates/server/src/api.rs`

Speedy Trial (`crate::repo::speedy_trial`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `get_speedy_trial` | `court_id, case_id` | `speedy_trial::find_by_case_id(pool, &court_id, case_uuid)` | `String` |
| `start_speedy_trial` | `court_id, body` | `speedy_trial::create(pool, &court_id, req)` | `String` |
| `update_speedy_trial` | `court_id, case_id, body` | `speedy_trial::update(pool, &court_id, case_uuid, req)` | `String` |
| `list_speedy_trial_delays` | `court_id, case_id` | `speedy_trial::list_delays_by_case(pool, &court_id, case_uuid)` | `String` |
| `create_speedy_trial_delay` | `court_id, body` | `speedy_trial::create_delay(pool, &court_id, req)` | `String` |
| `delete_speedy_trial_delay` | `court_id, id` | `speedy_trial::delete_delay(pool, &court_id, uuid)` | `()` |

**Step 1:** Write all 6 functions.
**Step 2:** `cargo check -p server --features server`
**Step 3:** Commit: `git commit -m "feat(api): add speedy trial server function wrappers"`

---

### Task 7: Extension + Reminder Server Functions

**File:** Modify `crates/server/src/api.rs`

Extensions (`crate::repo::extension_request`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_extensions_by_deadline` | `court_id, deadline_id` | `extension_request::list_by_deadline(pool, &court_id, dl_uuid)` | `String` |
| `get_extension` | `court_id, id` | `extension_request::find_by_id(pool, &court_id, uuid)` | `String` |
| `create_extension_request` | `court_id, body` | `extension_request::create(pool, &court_id, req)` | `String` |
| `list_pending_extensions` | `court_id` | `extension_request::list_pending(pool, &court_id)` | `String` |
| `rule_on_extension` | `court_id, id, body` | `extension_request::update_ruling(pool, &court_id, uuid, req)` | `String` |

Reminders (`crate::repo::deadline_reminder`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_reminders_by_deadline` | `court_id, deadline_id` | `deadline_reminder::list_by_deadline(pool, &court_id, dl_uuid)` | `String` |
| `list_pending_reminders` | `court_id` | `deadline_reminder::list_pending(pool, &court_id)` | `String` |
| `send_reminder` | `court_id, body` | `deadline_reminder::send(pool, &court_id, req)` | `String` |
| `acknowledge_reminder` | `court_id, id` | `deadline_reminder::acknowledge(pool, &court_id, uuid)` | `String` |

**Step 1:** Write all 9 functions.
**Step 2:** `cargo check -p server --features server`
**Step 3:** Commit: `git commit -m "feat(api): add extension and reminder server function wrappers"`

---

### Task 8: Judge Server Functions

**File:** Modify `crates/server/src/api.rs`

Judges (`crate::repo::judge`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_judges` | `court_id` | `judge::list_by_court(pool, &court_id)` | `String` |
| `search_judges` | `court_id, query` | `judge::search(pool, &court_id, &query)` | `String` |
| `get_judge` | `court_id, id` | `judge::find_by_id(pool, &court_id, uuid)` | `String` |
| `create_judge` | `court_id, body` | `judge::create(pool, &court_id, req)` | `String` |
| `update_judge` | `court_id, id, body` | `judge::update(pool, &court_id, uuid, req)` | `String` |
| `delete_judge` | `court_id, id` | `judge::delete(pool, &court_id, uuid)` | `()` |

Judge Conflicts (`crate::repo::judge_conflict`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_judge_conflicts` | `court_id, judge_id` | `judge_conflict::list_by_judge(pool, &court_id, j_uuid)` | `String` |
| `create_judge_conflict` | `court_id, body` | `judge_conflict::create(pool, &court_id, req)` | `String` |
| `delete_judge_conflict` | `court_id, id` | `judge_conflict::delete(pool, &court_id, uuid)` | `()` |

Case Assignments (`crate::repo::case_assignment`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_case_assignments` | `court_id, case_id` | `case_assignment::list_by_case(pool, &court_id, case_uuid)` | `String` |
| `create_case_assignment` | `court_id, body` | `case_assignment::create(pool, &court_id, req)` | `String` |
| `delete_case_assignment` | `court_id, id` | `case_assignment::delete(pool, &court_id, uuid)` | `()` |

Recusals (`crate::repo::recusal_motion`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `create_recusal` | `court_id, body` | `recusal_motion::create(pool, &court_id, req)` | `String` |
| `list_pending_recusals` | `court_id` | `recusal_motion::list_pending(pool, &court_id)` | `String` |
| `rule_on_recusal` | `court_id, id, body` | `recusal_motion::update_ruling(pool, &court_id, uuid, req)` | `String` |

**Step 1:** Write all 15 functions.
**Step 2:** `cargo check -p server --features server`
**Step 3:** Commit: `git commit -m "feat(api): add judge, conflict, assignment, and recusal server function wrappers"`

---

### Task 9: Opinion Server Functions

**File:** Modify `crates/server/src/api.rs`

Opinions (`crate::repo::opinion`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_opinions_by_case` | `court_id, case_id` | `opinion::list_by_case(pool, &court_id, case_uuid)` | `String` |
| `list_opinions_by_judge` | `court_id, judge_id` | `opinion::list_by_judge(pool, &court_id, j_uuid)` | `String` |
| `search_opinions` | `court_id, query, offset, limit` | `opinion::search(pool, &court_id, &query, offset, limit)` | `String` |
| `get_opinion` | `court_id, id` | `opinion::find_by_id(pool, &court_id, uuid)` | `String` |
| `create_opinion` | `court_id, body` | `opinion::create(pool, &court_id, req)` | `String` |
| `update_opinion` | `court_id, id, body` | `opinion::update(pool, &court_id, uuid, req)` | `String` |
| `delete_opinion` | `court_id, id` | `opinion::delete(pool, &court_id, uuid)` | `()` |

Opinion Drafts (`crate::repo::opinion_draft`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_opinion_drafts` | `court_id, opinion_id` | `opinion_draft::list_by_opinion(pool, &court_id, op_uuid)` | `String` |
| `create_opinion_draft` | `court_id, body` | `opinion_draft::create(pool, &court_id, req)` | `String` |
| `get_current_opinion_draft` | `court_id, opinion_id` | `opinion_draft::find_current(pool, &court_id, op_uuid)` | `String` |

Opinion Votes (`crate::repo::opinion_vote`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_opinion_votes` | `court_id, opinion_id` | `opinion_vote::list_by_opinion(pool, &court_id, op_uuid)` | `String` |
| `create_opinion_vote` | `court_id, body` | `opinion_vote::create(pool, &court_id, req)` | `String` |

Headnotes (`crate::repo::headnote`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_headnotes` | `court_id, opinion_id` | `headnote::list_by_opinion(pool, &court_id, op_uuid)` | `String` |
| `create_headnote` | `court_id, body` | `headnote::create(pool, &court_id, req)` | `String` |

**Step 1:** Write all 14 functions.
**Step 2:** `cargo check -p server --features server`
**Step 3:** Commit: `git commit -m "feat(api): add opinion, draft, vote, and headnote server function wrappers"`

---

### Task 10: Remaining Domain Server Functions

**File:** Modify `crates/server/src/api.rs`

Victims (`crate::repo::victim`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_victims_by_case` | `court_id, case_id` | `victim::list_by_case(pool, &court_id, case_uuid)` | `String` |
| `create_victim` | `court_id, body` | `victim::create(pool, &court_id, req)` | `String` |
| `get_victim` | `court_id, id` | `victim::find_by_id(pool, &court_id, uuid)` | `String` |
| `delete_victim` | `court_id, id` | `victim::delete(pool, &court_id, uuid)` | `()` |

Representations (`crate::repo::representation`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_representations_by_case` | `court_id, case_id` | `representation::list_by_case(pool, &court_id, case_uuid)` | `String` |
| `list_active_representations` | `court_id, attorney_id` | `representation::list_active_by_attorney(pool, &court_id, att_uuid)` | `String` |
| `get_representation` | `court_id, id` | `representation::find_by_id(pool, &court_id, uuid)` | `String` |
| `create_representation` | `court_id, body` | `representation::create(pool, &court_id, req)` | `String` |
| `end_representation` | `court_id, id` | `representation::end_representation(pool, &court_id, uuid)` | `String` |

Charges (`crate::repo::charge`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_charges_by_defendant` | `court_id, defendant_id` | `charge::list_by_defendant(pool, &court_id, def_uuid)` | `String` |
| `get_charge` | `court_id, id` | `charge::find_by_id(pool, &court_id, uuid)` | `String` |
| `create_charge` | `court_id, body` | `charge::create(pool, &court_id, req)` | `String` |
| `update_charge` | `court_id, id, body` | `charge::update(pool, &court_id, uuid, req)` | `String` |
| `delete_charge` | `court_id, id` | `charge::delete(pool, &court_id, uuid)` | `()` |

Motions (`crate::repo::motion`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_motions_by_case` | `court_id, case_id` | `motion::list_by_case(pool, &court_id, case_uuid)` | `String` |
| `get_motion` | `court_id, id` | `motion::find_by_id(pool, &court_id, uuid)` | `String` |
| `create_motion` | `court_id, body` | `motion::create(pool, &court_id, req)` | `String` |
| `update_motion` | `court_id, id, body` | `motion::update(pool, &court_id, uuid, req)` | `String` |
| `delete_motion` | `court_id, id` | `motion::delete(pool, &court_id, uuid)` | `()` |

Case Notes (`crate::repo::case_note`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_case_notes` | `court_id, case_id` | `case_note::list_by_case(pool, &court_id, case_uuid)` | `String` |
| `get_case_note` | `court_id, id` | `case_note::find_by_id(pool, &court_id, uuid)` | `String` |
| `create_case_note` | `court_id, body` | `case_note::create(pool, &court_id, req)` | `String` |
| `update_case_note` | `court_id, id, body` | `case_note::update(pool, &court_id, uuid, req)` | `String` |
| `delete_case_note` | `court_id, id` | `case_note::delete(pool, &court_id, uuid)` | `()` |

Rules (`crate::repo::rule`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `list_rules` | `court_id` | `rule::list_all(pool, &court_id)` | `String` |
| `get_rule` | `court_id, id` | `rule::find_by_id(pool, &court_id, uuid)` | `String` |
| `create_rule` | `court_id, body` | `rule::create(pool, &court_id, req)` | `String` |
| `update_rule` | `court_id, id, body` | `rule::update(pool, &court_id, uuid, req)` | `String` |
| `delete_rule` | `court_id, id` | `rule::delete(pool, &court_id, uuid)` | `()` |

Conflict Checks (`crate::repo::conflict_check`):

| Function | Params | Repo Call | Return |
|----------|--------|-----------|--------|
| `run_conflict_check` | `court_id, body` | `conflict_check::run_check(pool, &court_id, req)` | `String` |
| `list_conflicts_by_attorney` | `court_id, attorney_id` | `conflict_check::list_by_attorney(pool, &court_id, att_uuid)` | `String` |

**Step 1:** Write all 30 functions.
**Step 2:** `cargo check -p server --features server`
**Step 3:** Commit: `git commit -m "feat(api): add victim, representation, charge, motion, case note, rule, and conflict check server function wrappers"`

---

### Task 10B: Verify ALL server functions compile

**Step 1:** Run full workspace check:
```bash
cargo check --workspace
```
Expected: Clean compilation.

**Step 2:** Count total server functions:
```bash
grep -c "pub async fn" crates/server/src/api.rs
```
Expected: ~170+ functions (71 existing + ~100 new).

**Step 3:** Commit if any fixes needed.

---

## Part B: Flesh Out Case Hub Tabs

### Task 11: Parties Tab — Full DataTable + Create Sheet

**Files:**
- Modify: `crates/app/src/routes/cases/tabs/parties.rs`

Replace the stub with a full implementation:

```rust
use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardHeader,
    DataTable, DataTableBody, DataTableCell, DataTableColumn, DataTableHeader, DataTableRow,
    Form, Input, Label, FormSelect, PageActions,
    Sheet, SheetClose, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle,
    Skeleton, Separator,
};
use shared_ui::{use_toast, ToastOptions};
use crate::CourtContext;

#[component]
pub fn PartiesTab(case_id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut show_sheet = use_signal(|| false);
    let mut form_name = use_signal(String::new);
    let mut form_party_type = use_signal(|| "defendant".to_string());
    let mut form_role = use_signal(String::new);

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move {
            server::api::list_case_parties(court, cid).await.ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        let name = form_name.read().clone();
        let ptype = form_party_type.read().clone();
        let role = form_role.read().clone();

        spawn(async move {
            if name.trim().is_empty() {
                toast.error("Name is required.".to_string(), ToastOptions::new());
                return;
            }
            let body = serde_json::json!({
                "case_id": cid,
                "name": name.trim(),
                "party_type": ptype,
                "role": role.trim(),
            });
            match server::api::create_party(court, body.to_string()).await {
                Ok(_) => {
                    toast.success("Party added.".to_string(), ToastOptions::new());
                    show_sheet.set(false);
                    form_name.set(String::new());
                    form_role.set(String::new());
                    data.restart();
                }
                Err(e) => toast.error(format!("Error: {e}"), ToastOptions::new()),
            }
        });
    };

    rsx! {
        div { class: "tab-header",
            h3 { "Case Parties" }
            Button {
                variant: ButtonVariant::Primary,
                onclick: move |_| show_sheet.set(true),
                "Add Party"
            }
        }

        match &*data.read() {
            Some(Some(parties)) if !parties.is_empty() => rsx! {
                DataTable {
                    DataTableHeader {
                        DataTableColumn { "Name" }
                        DataTableColumn { "Type" }
                        DataTableColumn { "Role" }
                        DataTableColumn { "Status" }
                    }
                    DataTableBody {
                        for party in parties.iter() {
                            DataTableRow {
                                DataTableCell { {party["name"].as_str().unwrap_or("—")} }
                                DataTableCell {
                                    Badge { variant: BadgeVariant::Secondary,
                                        {party["party_type"].as_str().unwrap_or("—")}
                                    }
                                }
                                DataTableCell { {party["role"].as_str().unwrap_or("—")} }
                                DataTableCell {
                                    Badge { variant: BadgeVariant::Primary,
                                        {party["status"].as_str().unwrap_or("active")}
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Some(Some(_)) => rsx! {
                p { class: "empty-state", "No parties added to this case yet." }
            },
            Some(None) => rsx! {
                p { class: "error-state", "Failed to load parties." }
            },
            None => rsx! {
                Skeleton { width: "100%", height: "200px" }
            },
        }

        // Create Party Sheet
        Sheet { open: show_sheet(), side: SheetSide::Right,
            SheetHeader {
                SheetTitle { "Add Party" }
            }
            SheetContent {
                Form { onsubmit: handle_save,
                    Label { "Name" }
                    Input {
                        value: "{form_name}",
                        oninput: move |e: FormEvent| form_name.set(e.value()),
                        placeholder: "Party name",
                    }
                    Label { "Party Type" }
                    FormSelect {
                        value: "{form_party_type}",
                        onchange: move |e: FormEvent| form_party_type.set(e.value()),
                        option { value: "defendant", "Defendant" }
                        option { value: "prosecution", "Prosecution" }
                        option { value: "witness", "Witness" }
                        option { value: "intervenor", "Intervenor" }
                        option { value: "amicus", "Amicus Curiae" }
                    }
                    Label { "Role" }
                    Input {
                        value: "{form_role}",
                        oninput: move |e: FormEvent| form_role.set(e.value()),
                        placeholder: "e.g., Lead Defense Counsel",
                    }
                }
            }
            SheetFooter {
                SheetClose {
                    Button { variant: ButtonVariant::Secondary, "Cancel" }
                }
                Button { variant: ButtonVariant::Primary, onclick: handle_save, "Save" }
            }
        }
    }
}
```

**Step 1:** Replace the stub file with the full implementation above.
**Step 2:** `cargo check -p app --features server`
**Step 3:** Commit: `git commit -m "feat(ui): flesh out case hub Parties tab with DataTable and create sheet"`

---

### Task 12: Deadlines Tab — Full DataTable with Urgency Colors

**File:** Modify `crates/app/src/routes/cases/tabs/deadlines.rs`

Replace stub with full implementation using `search_deadlines` server function (already exists) filtered by case_id. Display urgency badges (red for overdue, yellow for <3 days, green for >3 days). Add "Request Extension" and "Set Reminder" buttons that open sheets calling `create_extension_request` and `send_reminder`.

Follow the same pattern as Task 11: `use_resource` → DataTable → Sheet. The deadline DataTable columns are: Type, Due Date, Status (countdown badge), Assigned To.

**Step 1:** Write the full component.
**Step 2:** `cargo check -p app --features server`
**Step 3:** Commit: `git commit -m "feat(ui): flesh out case hub Deadlines tab with urgency badges"`

---

### Task 13: Orders Tab — DataTable + Template Selector

**File:** Modify `crates/app/src/routes/cases/tabs/orders.rs`

Replace stub with full implementation using `list_orders_by_case`. DataTable columns: Order Type, Date Issued, Judge, Status. "Draft Order" button opens sheet with template selector (fetched via `list_active_order_templates`), then calls `create_order`.

**Step 1:** Write the full component.
**Step 2:** `cargo check -p app --features server`
**Step 3:** Commit: `git commit -m "feat(ui): flesh out case hub Orders tab with template selector"`

---

### Task 14: Evidence Tab — DataTable + Custody Chain

**File:** Modify `crates/app/src/routes/cases/tabs/evidence.rs`

Replace stub with full implementation using `list_evidence_by_case`. DataTable columns: Exhibit #, Description, Type, Custody Status. "Add Evidence" sheet calls `create_evidence`. Clicking a row shows custody chain via `list_custody_transfers`.

**Step 1:** Write the full component.
**Step 2:** `cargo check -p app --features server`
**Step 3:** Commit: `git commit -m "feat(ui): flesh out case hub Evidence tab with custody chain"`

---

### Task 15: Sentencing Tab — Summary + Guidelines

**File:** Modify `crates/app/src/routes/cases/tabs/sentencing.rs`

Replace stub with implementation using `list_sentencing_by_case`. Shows sentencing summary card if exists, with key fields (offense level, criminal history, guidelines range, sentence imposed). "Prepare Sentencing" button opens create sheet calling `create_sentencing`. Display sentencing conditions via `list_sentencing_conditions`.

**Step 1:** Write the full component.
**Step 2:** `cargo check -p app --features server`
**Step 3:** Commit: `git commit -m "feat(ui): flesh out case hub Sentencing tab"`

---

### Task 16: Calendar Tab — Mini Calendar + Events

**File:** Modify `crates/app/src/routes/cases/tabs/calendar_tab.rs`

Replace stub with implementation using `search_calendar_events` filtered by case_id. DataTable columns: Date, Event Type, Courtroom, Status. "Schedule Event" sheet calls `schedule_calendar_event`.

**Step 1:** Write the full component.
**Step 2:** `cargo check -p app --features server`
**Step 3:** Commit: `git commit -m "feat(ui): flesh out case hub Calendar tab"`

---

### Task 17: Speedy Trial Tab — Timeline Visualization

**File:** Modify `crates/app/src/routes/cases/tabs/speedy_trial.rs`

Replace stub with implementation using `get_speedy_trial` and `list_speedy_trial_delays`. Display:
- Clock status badge (Running / Tolled / Expired)
- Days elapsed counter
- Delay/exclusion periods list as DataTable (reason, start date, end date, days excluded)
- "Add Exclusion" sheet calls `create_speedy_trial_delay`
- Visual progress bar showing days used out of 70-day limit

**Step 1:** Write the full component.
**Step 2:** `cargo check -p app --features server`
**Step 3:** Commit: `git commit -m "feat(ui): flesh out case hub Speedy Trial tab with timeline"`

---

### Task 18: Overview Tab — Enhanced with Real Data

**File:** Modify `crates/app/src/routes/cases/tabs/overview.rs`

Enhance the existing overview tab to show:
- Case info grid (already there)
- Assigned judge card (fetch via `list_case_assignments`)
- Recent activity feed (fetch via `get_case_timeline` — already exists as server function)
- Quick stats cards: # parties, # pending deadlines, # docket entries

**Step 1:** Enhance the component with resource fetches.
**Step 2:** `cargo check -p app --features server`
**Step 3:** Commit: `git commit -m "feat(ui): enhance case hub Overview tab with real data"`

---

### Task 19: Final Verification

**Step 1:** Run full workspace check:
```bash
cargo check --workspace
```
Expected: Clean compilation.

**Step 2:** Run existing tests to verify no regressions:
```bash
cargo test --workspace
```

**Step 3:** Count total server functions added:
```bash
grep -c "pub async fn" crates/server/src/api.rs
```
Expected: ~170+

**Step 4:** Final commit if any fixes needed.

---

## Summary

After completing Tasks 1-19:

**Part A (Tasks 1-10B):** ~100 new server function wrappers covering:
- Defendants (5), Parties (7), Evidence (7), Orders (12), Sentencing (8)
- Speedy Trial (6), Extensions (5), Reminders (4), Judges (15), Opinions (14)
- Victims (4), Representations (5), Charges (5), Motions (5), Case Notes (5)
- Rules (5), Conflict Checks (2)

**Part B (Tasks 11-18):** 7 case hub tabs fleshed out with real DataTables, Sheets, and API integration:
- Parties, Deadlines, Orders, Evidence, Sentencing, Calendar, Speedy Trial + enhanced Overview

## Next Plans

- **Phase 3:** Judge detail hub (7 tabs) + Attorney detail hub (7 tabs)
- **Phase 4:** Standalone domain pages (Docket, Filings, Documents, Opinions, etc.)
- **Phase 5:** Workflow Wizards (New Case Filing, Sentencing Prep, Attorney Onboarding, etc.)
- **Phase 6:** Administration pages + Command Palette entity search
