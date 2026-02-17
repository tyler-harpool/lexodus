# Lexodus UI Delivery Plan: Edit System + Workflows + PDF

**Date:** 2026-02-16
**Status:** Design — awaiting approval
**Context:** 380+ REST endpoints exist. UI has List/Detail/Create for most modules but almost no Edit/Update, no Judge CRUD, no workflow actions (sign/issue/serve), no PDF generation.

---

## Goals

1. Ship a reusable Edit Sheet system across all 15+ modules
2. Close the Judge CRUD gap (only entity with no Create UI)
3. Make Orders operational (sign → issue → serve lifecycle)
4. Deliver single-order PDF generation from detail pages
5. Establish role gating infrastructure so the UI doesn't rot

## Non-goals (first 90 days)

| Excluded | Reason |
|----------|--------|
| Autosave | Judicial records need intentional saves |
| Filing/NEF workflow UI | Complex validation + upload flow; own design phase |
| Speedy trial calculator | Display exists; interactive calculator is separate feature |
| Compliance/reminders sending | Read-only dashboard exists; admin feature, not day-1 |
| Conflict check UI | Specialized workflow; core CRUD matters more now |
| Representation management | Downstream of party/attorney edit working first |
| Signature upload/management | Use seed data for v1; formalize later |
| Feature flag / config override UI | Internal tooling; use API directly |
| Batch operations UI | Bulk update UX is complex; single-record edit first |
| Advanced Select (searchable) | Native `<select>` works; polish not function |
| Mobile/responsive | Desktop-first for court clerk workflows |
| Inline table editing | Edit happens on detail pages via sheets |
| Opinion draft collaboration | Rich feature; own design phase |
| TODOs UI | Nice-to-have, not core CM/ECF |
| Batch PDF generation | v2; start with single-order generation |

---

## 1) Top 4 Initiatives

### Initiative 1: Reusable Edit Sheet System

**Why now:** 13+ modules have zero edit UI. Every record with a typo requires delete + recreate. This is the highest-friction gap and the force multiplier — once the pattern exists, adding edit to each module is mechanical.

**Success criteria:**
- [ ] Shared `FormMode::Create | FormMode::Edit` pattern established
- [ ] First 6 modules have working edit sheets (Attorneys, Cases, Judges, Deadlines, Calendar, Orders)
- [ ] Edit button visible on every detail page, opens pre-populated sheet
- [ ] Dirty close confirmation: closing sheet with unsaved changes prompts AlertDialog
- [ ] Form hydration correct across entity switches (no stale values)
- [ ] Consistent update semantics: PATCH sends only changed fields, PUT sends full payload
- [ ] Field-level server error display (top-of-form error banner + inline where possible)
- [ ] Toast confirmation on save, detail page data refetches automatically
- [ ] Role gating: edit button hidden for roles without write permission

### Initiative 2: Judge Full CRUD + Case Assignment

**Why now:** Judges is the only core entity with no Create UI. Case assignment (which judge handles which case) is foundational to every workflow that follows — orders, opinions, and calendar events all require a judge.

**Success criteria (v1):**
- [ ] Create Judge sheet on judges list page
- [ ] Edit Judge sheet on judge detail page
- [ ] Judge status updates (active/senior/recalled/retired)
- [ ] Case assignments: create/delete on judge detail Caseload tab
- [ ] Conflicts tab: read-only display of existing conflicts
- [ ] Recusals tab: read-only display of existing recusals
- [ ] "Add Conflict" / "Add Recusal" buttons exist but are disabled/hidden (v2 scope)

### Initiative 3: Order Workflow Actions (Sign/Issue/Serve)

**Why now:** Orders exist but can't progress through their lifecycle. Without sign → issue → serve, orders are just database rows.

**Success criteria:**
- [ ] Transition rules enforced in UI: Draft → Signed → Filed → Served
- [ ] UI only shows valid next actions for current status
- [ ] Sign Order: AlertDialog collects `signed_by` (required), displays signature_hash on success
- [ ] Issue Order: AlertDialog collects `issued_by` (required)
- [ ] Serve Order: AlertDialog collects `served_to[]` (party selector) + `service_method` (required)
- [ ] Every action dialog collects actor (required) and reason (optional but visible)
- [ ] Action result shows updated status chip + last action timestamp in detail header
- [ ] Action buttons disabled while request in-flight (no double-click)
- [ ] Idempotent-safe: double click doesn't duplicate action
- [ ] Judge-only actions (Sign) hidden for Clerk/Attorney roles

### Initiative 4: PDF Generation UI

**Why now:** Federal courts run on paper. Court orders, judgments, conditions of release — all need PDF output. 15 PDF endpoints exist but are invisible to users.

**Success criteria (v1 — single-order from detail page):**
- [ ] "Generate PDF" button on order detail pages
- [ ] Format selector: standard vs formatted variant
- [ ] Signed PDF variant available only for signed orders
- [ ] Response handling: bytes → direct download, URL → open/download, job ID → poll until ready
- [ ] Loading state while PDF generates
- [ ] Error toast if generation fails
- [ ] Batch PDF generation is explicitly v2

---

## 2) Reusable Edit Sheet System (the multiplier)

### UX Pattern: Edit Sheet

**Decision: Sheet, not inline edit.** The Cases inline-toggle pattern (CaseEditForm/CaseInfoDisplay swap) won't scale — it requires duplicating every detail page into read/edit variants. Every module uses the same right-side Sheet panel pattern already proven in Create flows.

**Behavior:**
- Detail page shows "Edit" button in `PageActions` (role-gated)
- Click opens right-side `Sheet` pre-populated with current entity data
- User modifies fields, clicks "Save Changes"
- On success: sheet closes, toast notification, detail data refetches via `data.restart()`
- On error: toast error with server message, sheet stays open with fields preserved
- On close with changes: AlertDialog "Discard unsaved changes?" confirmation

### State Model: Hydrate → Dirty → Submit → Error

#### Hydration (Dioxus gotcha)

Dioxus `use_signal` initializes once per component mount. When editing different entities (open sheet for entity A, close, open for entity B), signals retain stale values from A. Solution:

```rust
// Every FormSheet tracks the entity ID it was hydrated from
let mut hydrated_id = use_signal(String::new);

// use_effect re-hydrates when initial data changes
use_effect(move || {
    if let Some(ref data) = initial {
        let id = data.id.clone();
        if *hydrated_id.read() != id {
            hydrated_id.set(id);
            field_a.set(data.field_a.clone());
            field_b.set(data.field_b.clone());
            // ... all fields
        }
    }
});
```

**Acceptance:** "Opening edit sheet for entity A then entity B shows B's data (no stale values)."

#### Dirty Check (v1 — minimal)

No per-field tracking. Serialize initial values as a JSON string on open. On close, serialize current values and compare.

```rust
// On open, snapshot initial state
let initial_snapshot = use_signal(String::new);
use_effect(move || {
    if open && initial.is_some() {
        let snap = serde_json::json!({
            "field_a": field_a.read().clone(),
            "field_b": field_b.read().clone(),
        }).to_string();
        initial_snapshot.set(snap);
    }
});

// On close attempt, compare
fn is_dirty() -> bool {
    let current = serde_json::json!({ ... }).to_string();
    *initial_snapshot.read() != current
}

// If dirty, show AlertDialog before closing
```

**Acceptance:** "Close sheet with unsaved changes prompts discard confirmation."

#### Update Semantics (PUT vs PATCH)

Every form sheet implements two body builders:

```rust
impl AttorneyFormSheet {
    /// POST /api/attorneys — full payload for create
    fn to_create_body(&self) -> serde_json::Value {
        serde_json::json!({
            "first_name": self.first_name.read().clone(),
            "last_name": self.last_name.read().clone(),
            // ALL fields, required + optional
        })
    }

    /// PUT /api/attorneys/{id} — full payload (PUT = full replace)
    fn to_update_body(&self) -> serde_json::Value {
        // Same as create but may include id-related fields
        self.to_create_body()
    }
}

// For PATCH endpoints (Cases, Orders, Opinions):
impl CaseFormSheet {
    /// PATCH /api/cases/{id} — only changed fields
    fn to_update_body(&self) -> serde_json::Value {
        let mut body = serde_json::Map::new();
        if *self.title.read() != self.initial_title {
            body.insert("title".into(), json!(self.title.read().clone()));
        }
        if *self.status.read() != self.initial_status {
            body.insert("status".into(), json!(self.status.read().clone()));
        }
        // ... only include changed fields
        serde_json::Value::Object(body)
    }
}
```

**Rule:** PUT endpoints → `to_update_body()` sends full payload. PATCH endpoints → `to_update_body()` sends only changed fields.

**Endpoint type by module:**

| Module | Update Method | Endpoint |
|--------|--------------|----------|
| Attorneys | PUT | PUT /api/attorneys/{id} |
| Judges | PUT | PUT /api/judges/{id} |
| Cases | PATCH | PATCH /api/cases/{id} |
| Deadlines | PUT | PUT /api/deadlines/{id} |
| Calendar | (verify) | (verify endpoint exists) |
| Orders | PATCH | PATCH /api/orders/{order_id} |
| Opinions | PATCH | PATCH /api/opinions/{opinion_id} |
| Parties | PUT | PUT /api/parties/{id} |
| Defendants | PUT | PUT /api/defendants/{id} |
| Evidence | PUT | PUT /api/evidence/{id} |
| Sentencing | PUT | PUT /api/sentencing/{id} |
| Rules | PUT | PUT /api/rules/{id} |

#### Error Handling

- Server returns error string → displayed as toast AND as top-of-form error banner
- Field validation errors (e.g. "invalid status") → map to inline error below the field when possible
- Network errors → toast with retry suggestion
- Sheet stays open on error — user can fix and resubmit

### Role Gating

Build on existing `auth.rs` infrastructure (`use_user_role()`, `UserRole` enum, `SidebarVisibility`).

Add a `can()` helper:

```rust
// crates/app/src/auth.rs

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Create,
    Edit,
    Delete,
    Sign,      // Judge-only
    Issue,     // Clerk-only
    Serve,     // Clerk-only
    Seal,      // Judge/Clerk
    GeneratePdf,
}

/// Check if the current user can perform an action.
/// v1: permissive — but the structure exists so we don't hardcode buttons.
pub fn can(role: UserRole, action: Action) -> bool {
    match action {
        Action::Sign => matches!(role, UserRole::Judge | UserRole::Admin),
        Action::Issue | Action::Serve => matches!(role, UserRole::Clerk | UserRole::Admin),
        Action::Seal => matches!(role, UserRole::Judge | UserRole::Clerk | UserRole::Admin),
        Action::Create | Action::Edit | Action::Delete => {
            !matches!(role, UserRole::Public)
        }
        Action::GeneratePdf => !matches!(role, UserRole::Public),
    }
}
```

Usage in detail pages:

```rust
let role = use_user_role();
if can(role, Action::Edit) {
    Button { onclick: open_edit, "Edit" }
}
if can(role, Action::Sign) && order.status == "Draft" {
    Button { onclick: open_sign_dialog, "Sign Order" }
}
```

**Acceptance:** "Judge-only actions (Sign) are hidden for Clerk/Attorney roles."

### Workflow Action Dialog Pattern

For irreversible court-record actions (sign, issue, serve, seal, strike, unseal):

```rust
/// Reusable pattern for workflow action buttons
/// Shows AlertDialog with actor + optional reason collection
///
/// Usage:
///   WorkflowAction {
///     label: "Sign Order",
///     description: "This will digitally sign the order.",
///     requires_actor: true,    // collects signed_by / issued_by
///     requires_reason: false,  // true for seal/strike
///     on_confirm: move |actor, reason| { ... },
///     disabled: in_flight,
///   }
```

Every workflow action:
1. Button click opens AlertDialog
2. Dialog shows description + collects actor_id (required) + reason (optional or required)
3. Confirm button disabled while in-flight
4. On success: close dialog, update status badge + last action timestamp in detail header
5. On error: toast error, dialog stays open

### Shared Field Patterns

Not new components — standardized code snippets that each module copies and adapts:

| Pattern | Renders | Data Source |
|---------|---------|-------------|
| CaseSelector | `<select>` with case_number + title | `search_cases(court, ..., limit: 100)` → `CaseSearchResponse.cases` |
| JudgeSelector | `<select>` with judge name | `list_judges(court)` → `Vec<serde_json::Value>` |
| AttorneySelector | `<select>` with attorney name | `search_attorneys(court, ..., limit: 100)` |
| PartySelector | `<select>` with name + role | `list_all_parties(court, ..., limit: 100)` → `PaginatedResponse<PartyResponse>` |
| DocumentSelector | `<select>` with title + type | `list_all_documents(court, ..., limit: 100)` → `PaginatedResponse<DocumentResponse>` |
| StatusSelect | `FormSelect` with enum constants | From shared-types constants (e.g. `CASE_STATUSES`, `ORDER_STATUSES`) |
| DateTimeInput | `Input { input_type: "datetime-local" }` | Local signal, formatted as ISO 8601 |

### Audit Log Surface (Future-Proofing)

Every detail page gets a "History" section (or tab). In v1:
- Populated where the API already returns events (Documents: `GET /api/documents/{id}/events`, Cases: `GET /api/cases/{case_id}/timeline`)
- For modules without event endpoints: show `created_at` / `updated_at` as minimal history
- Structure exists so we don't redesign detail pages when audit logging ships

### Module Plug-in Pattern

Each module implements a single `*FormSheet` component:

```rust
// crates/app/src/routes/attorneys/form_sheet.rs

#[derive(Clone, Copy, PartialEq)]
pub enum FormMode { Create, Edit }

#[component]
pub fn AttorneyFormSheet(
    mode: FormMode,
    initial: Option<AttorneyResponse>,  // None = create, Some = edit
    open: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,         // caller does data.restart()
) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // --- Hydration ---
    let mut hydrated_id = use_signal(String::new);
    let mut first_name = use_signal(String::new);
    let mut last_name = use_signal(String::new);
    // ... more fields

    use_effect(move || {
        if let Some(ref data) = initial {
            if *hydrated_id.read() != data.id {
                hydrated_id.set(data.id.clone());
                first_name.set(data.first_name.clone());
                last_name.set(data.last_name.clone());
                // ... all fields
            }
        } else if mode == FormMode::Create {
            // Reset to defaults for create
            hydrated_id.set(String::new());
            first_name.set(String::new());
            last_name.set(String::new());
        }
    });

    // --- Dirty snapshot ---
    let initial_snapshot = use_signal(String::new);
    use_effect(move || {
        if open {
            let snap = serde_json::json!({
                "first_name": first_name.read().clone(),
                "last_name": last_name.read().clone(),
            }).to_string();
            initial_snapshot.set(snap);
        }
    });

    let is_dirty = move || {
        let current = serde_json::json!({
            "first_name": first_name.read().clone(),
            "last_name": last_name.read().clone(),
        }).to_string();
        *initial_snapshot.read() != current
    };

    // --- Submit ---
    let mut in_flight = use_signal(|| false);

    let handle_save = move |_: FormEvent| {
        if *in_flight.read() { return; }
        let court = ctx.court_id.read().clone();

        // Build body based on mode
        let body = match mode {
            FormMode::Create => serde_json::json!({
                "first_name": first_name.read().clone(),
                "last_name": last_name.read().clone(),
            }),
            FormMode::Edit => serde_json::json!({
                // PUT = full payload
                "first_name": first_name.read().clone(),
                "last_name": last_name.read().clone(),
            }),
        };

        let id = initial.as_ref().map(|d| d.id.clone()).unwrap_or_default();

        spawn(async move {
            in_flight.set(true);
            let result = match mode {
                FormMode::Create => server::api::create_attorney(court, body.to_string()).await,
                FormMode::Edit => server::api::update_attorney(court, id, body.to_string()).await,
            };
            match result {
                Ok(_) => {
                    on_saved.call(());
                    on_close.call(());
                    toast.success("Attorney saved.".into(), ToastOptions::new());
                }
                Err(e) => {
                    toast.error(format!("{e}"), ToastOptions::new());
                }
            }
            in_flight.set(false);
        });
    };

    // --- Render ---
    rsx! {
        Sheet { open, on_close: move |_| {
            if is_dirty() {
                // show_discard_dialog.set(true);  // AlertDialog
            } else {
                on_close.call(());
            }
        }, side: SheetSide::Right,
            SheetContent {
                SheetHeader {
                    SheetTitle {
                        match mode {
                            FormMode::Create => "New Attorney",
                            FormMode::Edit => "Edit Attorney",
                        }
                    }
                    SheetClose { on_close: move |_| on_close.call(()) }
                }
                Form { onsubmit: handle_save,
                    div { class: "sheet-form",
                        Input {
                            label: "First Name *",
                            value: first_name.read().clone(),
                            on_input: move |e: FormEvent| first_name.set(e.value()),
                        }
                        Input {
                            label: "Last Name *",
                            value: last_name.read().clone(),
                            on_input: move |e: FormEvent| last_name.set(e.value()),
                        }
                        // ... remaining fields
                    }
                    SheetFooter {
                        div { class: "sheet-footer-actions",
                            SheetClose { on_close: move |_| on_close.call(()) }
                            button {
                                class: "button",
                                "data-style": "primary",
                                r#type: "submit",
                                disabled: *in_flight.read(),
                                match mode {
                                    FormMode::Create => "Create Attorney",
                                    FormMode::Edit => "Save Changes",
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

Then both `list.rs` and `detail.rs` use the same component:

```rust
// list.rs
AttorneyFormSheet {
    mode: FormMode::Create,
    initial: None,
    open: show_sheet(),
    on_close: move |_| show_sheet.set(false),
    on_saved: move |_| data.restart(),
}

// detail.rs
AttorneyFormSheet {
    mode: FormMode::Edit,
    initial: Some(attorney_data.clone()),
    open: show_edit(),
    on_close: move |_| show_edit.set(false),
    on_saved: move |_| data.restart(),
}
```

---

## 3) 30/60/90 Plan

### Days 0–30: Edit System Foundation + Core CRUD

**Week 1–2: Pattern + Attorneys (reference implementation)**
- Implement `FormMode` enum + hydration + dirty guard on Attorneys
- Extract `AttorneyFormSheet` from existing list create sheet
- Add edit button to attorney detail page
- Add `can()` helper in `auth.rs`
- This becomes the template every other module follows

**Week 2–3: Judges full CRUD**
- `JudgeFormSheet` (Create + Edit) — closes biggest CRUD gap
- Wire "New Judge" button on list page
- Edit button on judge detail page
- Judge status update via sheet
- Case assignment create/delete on Caseload tab

**Week 3–4: Cases, Deadlines, Calendar, Orders edit sheets**
- Cases: Replace inline CaseEditForm with `CaseFormSheet` (PATCH semantics)
- Deadlines: `DeadlineFormSheet` (Create + Edit)
- Calendar: Merge create.rs into `CalendarFormSheet`, keep `/calendar/new` route as redirect
- Orders: `OrderFormSheet` (Create + Edit)

**Deliverable:** 6 modules with full Create+Edit, consistent UX, role gating, dirty guard.

### Days 31–60: Remaining Modules + Order Workflows

**Week 5–6: Roll edit sheets to remaining modules**
- Opinions, Parties, Defendants, Evidence, Sentencing, Victims, Service Records, Rules
- Mechanical work following established pattern (~1 day per module)

**Week 6–7: Documents operational**
- Create document from attachment (`promote_attachment` endpoint)
- Edit document metadata
- Seal/unseal/strike as WorkflowActionButtons on detail page
- Delete document

**Week 7–8: Order workflow actions**
- Sign Order (Judge-only, AlertDialog with `signed_by`)
- Issue Order (Clerk-only, AlertDialog with `issued_by`)
- Serve Order (party selector + service method)
- Status-aware: only show valid next actions
- Display action metadata (timestamp, actor) in detail header

**Deliverable:** All 15 modules have edit. Documents fully operational. Order lifecycle complete.

### Days 61–90: PDF Generation + Advanced Workflows

**Week 9–10: PDF generation UI**
- "Generate PDF" on Order detail page (one endpoint + one download path first)
- Format selector (standard / formatted)
- Signed PDF variant for signed orders
- Extend to other types: minute entry, waiver, conditions of release, criminal judgment
- Response adapter: handles bytes, URL, or job ID response formats

**Week 11–12: Advanced workflows**
- Opinion file/publish actions + draft display
- Judge conflict create/delete (upgrade from read-only)
- Judge recusal motion creation + processing
- Sentencing guidelines calculator integration
- Speedy trial deadline check display

**Deliverable:** PDF generation, opinion lifecycle, judge conflict/recusal management, sentencing calculations.

---

## 4) First 8 PR Slices

### PR 1: Reusable Edit Sheet Pattern + Attorney Edit
**Title:** `feat(ui): reusable edit sheet pattern + attorney edit`

**Scope:** Attorneys module (list + detail), auth.rs

**UI work:**
- New: `attorneys/form_sheet.rs` — `AttorneyFormSheet` with `FormMode`, hydration, dirty guard
- New: `auth.rs` additions — `Action` enum, `can()` helper
- Modify: `attorneys/list.rs` — replace inline create Sheet with `AttorneyFormSheet { mode: Create }`
- Modify: `attorneys/detail.rs` — add Edit button (role-gated), wire `AttorneyFormSheet { mode: Edit }`

**API work:**
- Create: `server::api::create_attorney(court, body)` — POST (already called)
- Update: `server::api::update_attorney(court, id, body)` — PUT /api/attorneys/{id}
- No new server endpoints

**Acceptance tests:**
- [ ] Edit button visible on attorney detail page (hidden for Public role)
- [ ] Clicking Edit opens sheet pre-populated with current attorney data
- [ ] Opening edit for attorney A then attorney B shows B's data (no stale values)
- [ ] Modifying first_name and saving calls PUT with correct body
- [ ] Detail page refreshes showing updated data after save
- [ ] Toast success notification appears on save
- [ ] Error response shows toast error, sheet stays open
- [ ] Close sheet with unsaved changes prompts discard confirmation
- [ ] Create on list page still works via same form component
- [ ] Save button disabled while request in-flight

**Risk/unknowns:**
- `use_effect` hydration timing — need to verify Dioxus fires effect when `initial` prop changes with sheet already mounted
- Attorney has 15+ fields — sheet may need scroll; verify sheet CSS handles overflow

---

### PR 2: Judge Create + Edit
**Title:** `feat(judges): create and edit judge UI`

**Scope:** Judges module (list + detail)

**UI work:**
- New: `judges/form_sheet.rs` — `JudgeFormSheet` (Create + Edit)
- Modify: `judges/list.rs` — wire "New Judge" button to open create sheet
- Modify: `judges/detail.rs` — add Edit button, wire edit sheet

**API work:**
- Create: `server::api::create_judge(court, body)` — POST /api/judges
- Update: `server::api::update_judge(court, id, body)` — PUT /api/judges/{id}
- Status: `server::api::update_judge_status(court, id, body)` — PATCH /api/judges/{id}/status

**Acceptance tests:**
- [ ] "New Judge" button on list page opens create sheet
- [ ] Fields: name, title, status, district, courtroom, max_caseload
- [ ] Title validates against server-allowed values (District Judge, Magistrate Judge, etc.)
- [ ] Status validates against allowed values (active, senior, recalled, retired)
- [ ] Creating judge adds it to list, toast success
- [ ] Edit on detail page opens pre-populated sheet
- [ ] Status change reflected in badge on detail page
- [ ] Hydration + dirty guard work correctly

**Risk/unknowns:**
- Need to verify `create_judge` server function signature and required fields
- Judge title/status enum values must match server validation constants

---

### PR 2b: Judge Case Assignments
**Title:** `feat(judges): case assignment management on detail page`

**Scope:** Judge detail Caseload tab

**UI work:**
- Modify: `judges/tabs/caseload.rs` — add "Assign Case" button, CaseSelector dropdown, assignment list with delete
- Conflicts tab: read-only display
- Recusals tab: read-only display

**API work:**
- Create assignment: `server::api::create_assignment(court, body)` — POST /api/judges/assignments
- Delete assignment: `server::api::delete_assignment(court, id)` — DELETE /api/assignments/{id}
- List: `server::api::list_assignments_by_case(court, case_id)` (already used)

**Acceptance tests:**
- [ ] "Assign Case" opens inline form with CaseSelector
- [ ] Creating assignment shows case in Caseload tab
- [ ] Deleting assignment removes it (AlertDialog confirmation)
- [ ] Conflicts tab displays existing conflicts (read-only)
- [ ] Recusals tab displays existing recusals (read-only)

**Risk/unknowns:**
- Assignment endpoint may require additional fields (assignment_type, assigned_date)
- Need to verify response format from POST /api/judges/assignments

---

### PR 3: Cases Edit Sheet (Replace Inline Edit)
**Title:** `feat(cases): replace inline edit with edit sheet`

**Scope:** Cases module (detail + list)

**UI work:**
- New: `cases/form_sheet.rs` — `CaseFormSheet` with PATCH semantics (only changed fields)
- Modify: `cases/detail.rs` — remove `CaseEditForm`/`CaseInfoDisplay` toggle, add Edit button + sheet
- Modify: `cases/list.rs` — replace inline create sheet with `CaseFormSheet { mode: Create }`

**API work:**
- Update: `server::api::update_case(court, id, body)` — PATCH /api/cases/{id} (only changed fields)
- No new endpoints

**Acceptance tests:**
- [ ] Inline edit toggle removed from detail page
- [ ] Edit button in PageActions opens sheet
- [ ] Sheet pre-populated with current case data
- [ ] PATCH body contains only changed fields (not full payload)
- [ ] crime_type dropdown shows all values from `CRIME_TYPES`
- [ ] status dropdown shows all values from `CASE_STATUSES`
- [ ] Save calls PATCH, detail page refreshes
- [ ] Create on list page still works

**Risk/unknowns:**
- Removing inline edit is a behavior change — verify no workflow depends on in-place editing
- PATCH field-diff logic needs careful testing (edge case: user changes field back to original)

---

### PR 4: Deadlines + Calendar Edit Sheets
**Title:** `feat(deadlines,calendar): add edit sheets`

**Scope:** Deadlines + Calendar (detail + list pages)

**UI work:**
- New: `deadlines/form_sheet.rs` — `DeadlineFormSheet`
- New: `calendar/form_sheet.rs` — `CalendarFormSheet`
- Modify: detail + list pages for both modules
- Keep `calendar/create.rs` route — make it render the list page with sheet auto-opened (or redirect)

**API work:**
- Deadline update: `server::api::update_deadline(court, id, body)` — PUT /api/deadlines/{id}
- Calendar update: verify endpoint exists (may need to confirm)

**Acceptance tests:**
- [ ] Deadline detail: Edit opens sheet with due_at, status, rule_code, notes pre-populated
- [ ] Calendar detail: Edit opens sheet with event_type, scheduled_date, courtroom pre-populated
- [ ] Case/Judge selectors work in calendar edit (pre-selected to current values)
- [ ] `/calendar/new` route still works (uses shared form sheet)
- [ ] Both modules: create from list still works

**Risk/unknowns:**
- Calendar event update endpoint may not exist — need to verify before starting
- Deadline `due_at` is ISO datetime — `datetime-local` input formatting needs care

---

### PR 5a: Orders Edit Sheet
**Title:** `feat(orders): edit sheet for order metadata`

**Scope:** Orders module (detail + list)

**UI work:**
- New: `orders/form_sheet.rs` — `OrderFormSheet` with PATCH semantics
- Modify: `orders/detail.rs` — add Edit button + sheet
- Modify: `orders/list.rs` — use shared form for create

**API work:**
- Update: `server::api::update_order(court, id, body)` — PATCH /api/orders/{order_id}

**Acceptance tests:**
- [ ] Edit sheet opens with title, content, status, is_sealed pre-populated
- [ ] PATCH sends only changed fields
- [ ] Save updates detail page
- [ ] Create from list still works

**Risk/unknowns:**
- Order has `effective_date` and `expiration_date` — date picker formatting

---

### PR 5b: Order Workflow Actions (Sign/Issue/Serve)
**Title:** `feat(orders): sign, issue, serve workflow actions`

**Scope:** Orders detail page

**UI work:**
- Add WorkflowAction pattern: AlertDialog with actor + reason collection
- "Sign Order" (Judge-only): collects `signed_by`
- "Issue Order" (Clerk-only): collects `issued_by`
- "Serve Order" (Clerk-only): collects `served_to[]` + `service_method`
- Status-aware rendering: only show valid next action
- Action metadata display in detail header (status + timestamp)

**API work:**
- Sign: `server::api::sign_order(court, id, body)` — POST /api/orders/{order_id}/sign
- Issue: `server::api::issue_order(court, id, body)` — POST /api/orders/{order_id}/issue
- Serve: `server::api::serve_order(court, id, body)` — POST /api/orders/{order_id}/service

**Acceptance tests:**
- [ ] Draft order shows "Sign Order", no Issue/Serve
- [ ] Signed order shows "Issue Order", no Sign/Serve
- [ ] Filed order shows "Serve Order", no Sign/Issue
- [ ] Sign requires `signed_by`, shows signature_hash after success
- [ ] Serve requires party selection + service method
- [ ] All actions: button disabled while in-flight
- [ ] All actions: double-click safe (idempotent)
- [ ] Role gating: Sign hidden for non-Judge roles
- [ ] Action result: updated status badge + timestamp in detail header

**Risk/unknowns:**
- Exact order status state machine transitions need server-side verification
- `serve_order` response may not include all service details — check what's returned

---

### PR 6: Batch Edit Sheets (Opinions, Parties, Defendants, Evidence)
**Title:** `feat(ui): edit sheets for opinions, parties, defendants, evidence`

**Scope:** 4 modules following established pattern

**UI work:**
- New form_sheet.rs for each: opinions, parties, defendants, evidence
- Edit button + sheet on each detail page
- Shared form on each list page

**API work:**
- `update_opinion()` — PATCH /api/opinions/{id} (only changed fields)
- `update_party()` — PUT /api/parties/{id}
- `update_defendant()` — PUT /api/defendants/{id}
- `update_evidence()` — PUT /api/evidence/{id}

**Acceptance tests:**
- [ ] Each module: edit button on detail, pre-populated sheet, correct endpoint called
- [ ] Opinions: PATCH semantics (changed fields only)
- [ ] Parties/Defendants/Evidence: PUT semantics (full payload)
- [ ] All: toast on success/error, data refresh, dirty guard, hydration

**Risk/unknowns:**
- 4 modules in one PR may be too large — split into 2x2 if review is heavy
- Need to verify all `update_*` server functions exist

---

### PR 7: Documents Operational
**Title:** `feat(documents): CRUD + seal/unseal/strike workflow actions`

**Scope:** Documents module (list + detail)

**UI work:**
- New: `documents/form_sheet.rs` — document metadata form
- Modify detail: add Edit, Seal/Unseal/Strike/Replace as WorkflowActions
- Modify detail: add document events timeline ("History" section)
- Modify list: add create document option
- Add Delete with AlertDialog confirmation

**API work:**
- Promote: POST /api/documents/from-attachment
- Seal: POST /api/documents/{id}/seal
- Unseal: POST /api/documents/{id}/unseal
- Strike: POST /api/documents/{id}/strike (reason required)
- Replace: POST /api/documents/{id}/replace
- Events: GET /api/documents/{id}/events

**Acceptance tests:**
- [ ] Document detail shows Seal/Unseal/Strike/Replace actions (role-gated)
- [ ] Seal requires confirmation, updates is_sealed badge
- [ ] Strike requires reason (mandatory), confirmation
- [ ] Document events timeline displayed in History section
- [ ] Create from list works (promote attachment flow)

**Risk/unknowns:**
- Promote from attachment depends on existing docket attachment — may simplify to metadata-only create
- Replace document needs file reference — may defer upload piece

---

### PR 8: PDF Generation (Single-Order, v1)
**Title:** `feat(pdf): single-order PDF generation from detail page`

**Scope:** Orders detail page + PDF response adapter

**UI work:**
- Add "Generate PDF" button on order detail (with format dropdown)
- PDF response adapter: handles bytes (download), URL (open), job ID (poll)
- Loading state during generation
- Signed PDF option visible only for signed orders

**API work:**
- Court order: POST /api/pdf/court-order
- Court order formatted: POST /api/pdf/court-order/{format}
- Signed: POST /api/pdf/signed/rule16b (if order is signed)

**Acceptance tests:**
- [ ] "Generate PDF" visible on order detail
- [ ] Format selector: standard vs formatted
- [ ] PDF generation triggers browser download
- [ ] Signed variant only shown for signed orders
- [ ] Loading spinner while generating
- [ ] Error toast on failure
- [ ] Response adapter handles bytes or URL response

**Risk/unknowns:**
- PDF download in Dioxus may need `web_sys` / JS interop for triggering file save
- PDF endpoint request body format needs investigation
- Large PDFs could timeout — may need progress indication

---

## 5) Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Dioxus signal hydration doesn't fire on prop change | `use_effect` with ID comparison gate; test with rapid entity switching |
| PATCH sends full payload accidentally (silent data reset) | `to_update_body()` enforces diff; integration tests verify partial update |
| Calendar update endpoint may not exist | Verify before PR 4; if missing, create one (small server PR) |
| PDF endpoint response format unknown | Build adapter that handles bytes/URL/job-id; test one endpoint first before expanding |
| 4-module batch PR too large for review | Split into 2x2 if needed (Opinions+Parties, Defendants+Evidence) |
| Role gating `can()` too permissive in v1 | Structure exists; tighten rules as auth system matures |
| Order status machine not well-documented | Verify transitions against server code before PR 5b |
| Sheet overflow for entities with many fields (Attorneys: 15+) | CSS `overflow-y: auto` on sheet-form; test on standard viewport |

---

## Module Rollout Table

| Module | PR | Edit | Create | Workflow Actions | Status |
|--------|----|------|--------|-----------------|--------|
| Attorneys | 1 | Edit Sheet | Sheet (refactored) | — | Day 0-30 |
| Judges | 2, 2b | Edit Sheet | Sheet (new) | Status update, assignments | Day 0-30 |
| Cases | 3 | Edit Sheet (PATCH) | Sheet (refactored) | — | Day 0-30 |
| Deadlines | 4 | Edit Sheet | Sheet (refactored) | — | Day 0-30 |
| Calendar | 4 | Edit Sheet | Sheet (refactored) | — | Day 0-30 |
| Orders | 5a, 5b | Edit Sheet (PATCH) | Sheet (refactored) | Sign/Issue/Serve | Day 0-30, 31-60 |
| Opinions | 6 | Edit Sheet (PATCH) | Sheet (exists) | — (file/publish in day 61-90) | Day 31-60 |
| Parties | 6 | Edit Sheet | Sheet (exists) | — | Day 31-60 |
| Defendants | 6 | Edit Sheet | Sheet (exists) | — | Day 31-60 |
| Evidence | 6 | Edit Sheet | Sheet (exists) | — | Day 31-60 |
| Sentencing | 7+ | Edit Sheet | Sheet (exists) | — | Day 31-60 |
| Victims | 7+ | Edit Sheet | Sheet (exists) | — | Day 31-60 |
| Service Records | 7+ | Edit Sheet | Sheet (exists) | — | Day 31-60 |
| Rules | 7+ | Edit Sheet | Sheet (exists) | — | Day 31-60 |
| Documents | 7 | Edit Sheet | Promote attachment | Seal/Unseal/Strike/Replace | Day 31-60 |
| PDF | 8 | — | — | Generate PDF | Day 61-90 |
