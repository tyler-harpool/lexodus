# Edit System + Workflows Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Ship reusable Edit Sheet system across all modules, Judge CRUD, Order workflows, and PDF generation.

**Architecture:** Every module gets a `form_sheet.rs` component supporting `FormMode::Create | FormMode::Edit`. Sheets hydrate from entity data via `use_effect`, track dirty state via JSON snapshot comparison, and use `to_create_body()` / `to_update_body()` for correct PUT vs PATCH semantics. Role gating via `can(role, action)` helper controls button visibility.

**Tech Stack:** Dioxus 0.7, shared-ui components (Sheet, Form, Input, FormSelect, AlertDialog, Badge, Toast), serde_json for body building, existing `server::api::*` functions.

**Design doc:** `docs/plans/2026-02-16-edit-system-delivery-plan.md`

---

## PR 1: Reusable Edit Sheet Pattern + Attorney Edit

### Task 1: Add `Action` enum and `can()` helper to auth.rs

**Files:**
- Modify: `crates/app/src/auth.rs`

**Step 1: Add the Action enum and can() function after the existing `use_can_manage_memberships` function**

Add to `crates/app/src/auth.rs` after line 60:

```rust
/// Actions that can be role-gated in the UI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Create,
    Edit,
    Delete,
    Sign,         // Judge-only: sign orders
    Issue,        // Clerk-only: issue orders
    Serve,        // Clerk-only: serve orders
    Seal,         // Judge/Clerk: seal/unseal documents
    GeneratePdf,
}

/// Check if a role is permitted to perform an action.
/// v1: permissive defaults — structure exists so we don't hardcode buttons everywhere.
pub fn can(role: UserRole, action: Action) -> bool {
    match action {
        Action::Sign => matches!(role, UserRole::Judge | UserRole::Admin),
        Action::Issue | Action::Serve => matches!(role, UserRole::Clerk | UserRole::Admin),
        Action::Seal => matches!(role, UserRole::Judge | UserRole::Clerk | UserRole::Admin),
        Action::Create | Action::Edit | Action::Delete | Action::GeneratePdf => {
            !matches!(role, UserRole::Public)
        }
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check -p app`
Expected: Compiles with no new errors (existing `use_is_admin` warning is OK)

**Step 3: Commit**

```bash
git add crates/app/src/auth.rs
git commit -m "feat(auth): add Action enum and can() role-gating helper"
```

---

### Task 2: Create AttorneyFormSheet component

**Files:**
- Create: `crates/app/src/routes/attorneys/form_sheet.rs`
- Modify: `crates/app/src/routes/attorneys/mod.rs` (add `pub mod form_sheet;`)

**Step 1: Add module declaration**

In `crates/app/src/routes/attorneys/mod.rs`, add:
```rust
pub mod form_sheet;
```

**Step 2: Create the form sheet component**

Create `crates/app/src/routes/attorneys/form_sheet.rs`:

```rust
use dioxus::prelude::*;
use shared_types::AttorneyResponse;
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Button, ButtonVariant, Form, Input,
    Separator, Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader,
    SheetSide, SheetTitle,
};
use shared_ui::{use_toast, ToastOptions};

use crate::CourtContext;

/// Controls whether the form is in Create or Edit mode.
#[derive(Clone, Copy, PartialEq)]
pub enum FormMode {
    Create,
    Edit,
}

/// Unified create/edit form for attorneys, rendered inside a Sheet.
///
/// - `mode`: Create or Edit
/// - `initial`: None for create, Some(AttorneyResponse) for edit (pre-populates fields)
/// - `open`: whether the sheet is visible
/// - `on_close`: called when user closes the sheet (after dirty check)
/// - `on_saved`: called after successful save (caller should `data.restart()`)
#[component]
pub fn AttorneyFormSheet(
    mode: FormMode,
    initial: Option<AttorneyResponse>,
    open: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // --- Form field signals ---
    let mut bar_number = use_signal(String::new);
    let mut first_name = use_signal(String::new);
    let mut last_name = use_signal(String::new);
    let mut middle_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut firm_name = use_signal(String::new);
    let mut fax = use_signal(String::new);
    let mut street1 = use_signal(String::new);
    let mut street2 = use_signal(String::new);
    let mut city = use_signal(String::new);
    let mut state = use_signal(String::new);
    let mut zip_code = use_signal(String::new);
    let mut country = use_signal(|| "US".to_string());

    // --- Hydration: sync signals from initial data ---
    let mut hydrated_id = use_signal(String::new);

    use_effect(move || {
        if !open {
            return;
        }
        if let Some(ref data) = initial {
            let id = data.id.clone();
            if *hydrated_id.read() != id {
                hydrated_id.set(id);
                bar_number.set(data.bar_number.clone());
                first_name.set(data.first_name.clone());
                last_name.set(data.last_name.clone());
                middle_name.set(data.middle_name.clone().unwrap_or_default());
                email.set(data.email.clone());
                phone.set(data.phone.clone());
                firm_name.set(data.firm_name.clone().unwrap_or_default());
                fax.set(data.fax.clone().unwrap_or_default());
                street1.set(data.address.street1.clone());
                street2.set(data.address.street2.clone().unwrap_or_default());
                city.set(data.address.city.clone());
                state.set(data.address.state.clone());
                zip_code.set(data.address.zip_code.clone());
                country.set(data.address.country.clone());
            }
        } else if mode == FormMode::Create && hydrated_id.read().is_empty() {
            // Already at defaults for create
        } else if mode == FormMode::Create {
            // Reset for a fresh create
            hydrated_id.set(String::new());
            bar_number.set(String::new());
            first_name.set(String::new());
            last_name.set(String::new());
            middle_name.set(String::new());
            email.set(String::new());
            phone.set(String::new());
            firm_name.set(String::new());
            fax.set(String::new());
            street1.set(String::new());
            street2.set(String::new());
            city.set(String::new());
            state.set(String::new());
            zip_code.set(String::new());
            country.set("US".to_string());
        }
    });

    // --- Dirty state tracking ---
    let mut initial_snapshot = use_signal(String::new);

    use_effect(move || {
        if open {
            let snap = snapshot(
                &bar_number, &first_name, &last_name, &middle_name, &email, &phone, &firm_name,
                &fax, &street1, &street2, &city, &state, &zip_code, &country,
            );
            initial_snapshot.set(snap);
        }
    });

    let is_dirty = move || {
        let current = snapshot(
            &bar_number, &first_name, &last_name, &middle_name, &email, &phone, &firm_name,
            &fax, &street1, &street2, &city, &state, &zip_code, &country,
        );
        *initial_snapshot.read() != current
    };

    let mut show_discard = use_signal(|| false);

    let try_close = move |_| {
        if is_dirty() {
            show_discard.set(true);
        } else {
            on_close.call(());
        }
    };

    // --- Submit ---
    let mut in_flight = use_signal(|| false);

    let handle_save = move |_: FormEvent| {
        if *in_flight.read() {
            return;
        }
        let court = ctx.court_id.read().clone();
        let id = initial.as_ref().map(|d| d.id.clone()).unwrap_or_default();

        let body = serde_json::json!({
            "bar_number": bar_number.read().clone(),
            "first_name": first_name.read().clone(),
            "last_name": last_name.read().clone(),
            "middle_name": opt_str(&middle_name.read()),
            "firm_name": opt_str(&firm_name.read()),
            "email": email.read().clone(),
            "phone": phone.read().clone(),
            "fax": opt_str(&fax.read()),
            "address": {
                "street1": street1.read().clone(),
                "street2": opt_str(&street2.read()),
                "city": city.read().clone(),
                "state": state.read().clone(),
                "zip_code": zip_code.read().clone(),
                "country": country.read().clone(),
            }
        });

        spawn(async move {
            in_flight.set(true);
            let result = match mode {
                FormMode::Create => {
                    server::api::create_attorney(court, body.to_string()).await
                }
                FormMode::Edit => {
                    server::api::update_attorney(court, id, body.to_string()).await
                }
            };
            match result {
                Ok(_) => {
                    on_saved.call(());
                    on_close.call(());
                    let msg = match mode {
                        FormMode::Create => "Attorney created successfully",
                        FormMode::Edit => "Attorney updated successfully",
                    };
                    toast.success(msg.to_string(), ToastOptions::new());
                }
                Err(e) => {
                    toast.error(format!("{e}"), ToastOptions::new());
                }
            }
            in_flight.set(false);
        });
    };

    // --- Render ---
    let title = match mode {
        FormMode::Create => "New Attorney",
        FormMode::Edit => "Edit Attorney",
    };
    let description = match mode {
        FormMode::Create => "Add a new attorney to this court district.",
        FormMode::Edit => "Modify attorney information.",
    };
    let submit_label = match mode {
        FormMode::Create => "Create Attorney",
        FormMode::Edit => "Save Changes",
    };

    rsx! {
        Sheet {
            open,
            on_close: try_close,
            side: SheetSide::Right,
            SheetContent {
                SheetHeader {
                    SheetTitle { "{title}" }
                    SheetDescription { "{description}" }
                    SheetClose { on_close: try_close }
                }

                Form {
                    onsubmit: handle_save,

                    div {
                        class: "sheet-form",

                        // Personal Information
                        Input {
                            label: "Bar Number *",
                            value: bar_number.read().clone(),
                            on_input: move |e: FormEvent| bar_number.set(e.value()),
                            placeholder: "e.g., NY-123456",
                        }
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
                        Input {
                            label: "Middle Name",
                            value: middle_name.read().clone(),
                            on_input: move |e: FormEvent| middle_name.set(e.value()),
                        }
                        Input {
                            label: "Firm Name",
                            value: firm_name.read().clone(),
                            on_input: move |e: FormEvent| firm_name.set(e.value()),
                        }

                        // Contact
                        Input {
                            label: "Email *",
                            value: email.read().clone(),
                            on_input: move |e: FormEvent| email.set(e.value()),
                        }
                        Input {
                            label: "Phone *",
                            value: phone.read().clone(),
                            on_input: move |e: FormEvent| phone.set(e.value()),
                        }
                        Input {
                            label: "Fax",
                            value: fax.read().clone(),
                            on_input: move |e: FormEvent| fax.set(e.value()),
                        }

                        // Address
                        Input {
                            label: "Street Address *",
                            value: street1.read().clone(),
                            on_input: move |e: FormEvent| street1.set(e.value()),
                        }
                        Input {
                            label: "Street Address 2",
                            value: street2.read().clone(),
                            on_input: move |e: FormEvent| street2.set(e.value()),
                        }
                        Input {
                            label: "City *",
                            value: city.read().clone(),
                            on_input: move |e: FormEvent| city.set(e.value()),
                        }
                        Input {
                            label: "State *",
                            value: state.read().clone(),
                            on_input: move |e: FormEvent| state.set(e.value()),
                        }
                        Input {
                            label: "ZIP Code *",
                            value: zip_code.read().clone(),
                            on_input: move |e: FormEvent| zip_code.set(e.value()),
                        }
                        Input {
                            label: "Country *",
                            value: country.read().clone(),
                            on_input: move |e: FormEvent| country.set(e.value()),
                        }
                    }

                    Separator {}

                    SheetFooter {
                        div {
                            class: "sheet-footer-actions",
                            SheetClose { on_close: try_close }
                            button {
                                class: "button",
                                "data-style": "primary",
                                r#type: "submit",
                                disabled: *in_flight.read(),
                                if *in_flight.read() { "Saving..." } else { "{submit_label}" }
                            }
                        }
                    }
                }
            }
        }

        // Discard changes confirmation
        AlertDialogRoot {
            open: *show_discard.read(),
            on_open_change: move |open: bool| show_discard.set(open),
            AlertDialogContent {
                AlertDialogTitle { "Discard changes?" }
                AlertDialogDescription {
                    "You have unsaved changes. Are you sure you want to close without saving?"
                }
                AlertDialogActions {
                    AlertDialogCancel {
                        on_cancel: move |_| show_discard.set(false),
                        "Keep Editing"
                    }
                    AlertDialogAction {
                        on_action: move |_| {
                            show_discard.set(false);
                            on_close.call(());
                        },
                        "Discard"
                    }
                }
            }
        }
    }
}

/// Build a JSON snapshot string for dirty-state comparison.
fn snapshot(
    bar_number: &Signal<String>,
    first_name: &Signal<String>,
    last_name: &Signal<String>,
    middle_name: &Signal<String>,
    email: &Signal<String>,
    phone: &Signal<String>,
    firm_name: &Signal<String>,
    fax: &Signal<String>,
    street1: &Signal<String>,
    street2: &Signal<String>,
    city: &Signal<String>,
    state: &Signal<String>,
    zip_code: &Signal<String>,
    country: &Signal<String>,
) -> String {
    serde_json::json!({
        "bar_number": bar_number.read().clone(),
        "first_name": first_name.read().clone(),
        "last_name": last_name.read().clone(),
        "middle_name": middle_name.read().clone(),
        "email": email.read().clone(),
        "phone": phone.read().clone(),
        "firm_name": firm_name.read().clone(),
        "fax": fax.read().clone(),
        "street1": street1.read().clone(),
        "street2": street2.read().clone(),
        "city": city.read().clone(),
        "state": state.read().clone(),
        "zip_code": zip_code.read().clone(),
        "country": country.read().clone(),
    })
    .to_string()
}

/// Return `serde_json::Value::Null` for empty strings, or the string value.
fn opt_str(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(s.to_string())
    }
}
```

**Step 3: Verify compilation**

Run: `cargo check -p app`
Expected: Compiles (warnings about unused imports OK until wired in next task)

**Step 4: Commit**

```bash
git add crates/app/src/routes/attorneys/form_sheet.rs crates/app/src/routes/attorneys/mod.rs
git commit -m "feat(attorneys): add AttorneyFormSheet with hydration and dirty guard"
```

---

### Task 3: Wire AttorneyFormSheet into list page (replace inline create sheet)

**Files:**
- Modify: `crates/app/src/routes/attorneys/list.rs`

**Step 1: Replace the inline create sheet with AttorneyFormSheet**

At the top of `list.rs`, add the import:
```rust
use super::form_sheet::{AttorneyFormSheet, FormMode};
```

Remove ALL the form signal declarations (form_bar_number, form_first_name, form_last_name, etc.) and the `reset_form` closure, `handle_save` closure, and the entire `Sheet { ... }` block at the bottom.

Replace with:
```rust
// In the component body, keep only:
let mut show_sheet = use_signal(|| false);

// Where the create button is:
Button {
    variant: ButtonVariant::Primary,
    onclick: move |_| show_sheet.set(true),
    "New Attorney"
}

// At the bottom of the rsx!, replace the old Sheet block with:
AttorneyFormSheet {
    mode: FormMode::Create,
    initial: None,
    open: show_sheet(),
    on_close: move |_| show_sheet.set(false),
    on_saved: move |_| data.restart(),
}
```

Also remove the now-unused `opt_str` function from list.rs (it's in form_sheet.rs now).

**Step 2: Verify compilation**

Run: `cargo check -p app`
Expected: Clean compile

**Step 3: Commit**

```bash
git add crates/app/src/routes/attorneys/list.rs
git commit -m "feat(attorneys): use AttorneyFormSheet for create on list page"
```

---

### Task 4: Wire AttorneyFormSheet into detail page (Edit button)

**Files:**
- Modify: `crates/app/src/routes/attorneys/detail.rs`

**Step 1: Add imports and edit state**

Add imports:
```rust
use super::form_sheet::{AttorneyFormSheet, FormMode};
use crate::auth::{can, Action, use_user_role};
```

In the component body, add:
```rust
let role = use_user_role();
let mut show_edit = use_signal(|| false);
```

**Step 2: Add Edit button to PageActions (role-gated)**

In the `PageActions` section, add the Edit button before the Delete button:
```rust
PageActions {
    Link { to: Route::AttorneyList {},
        Button { variant: ButtonVariant::Secondary, "Back to List" }
    }
    if can(role, Action::Edit) {
        Button {
            variant: ButtonVariant::Primary,
            onclick: move |_| show_edit.set(true),
            "Edit"
        }
    }
    // ... existing delete button
}
```

**Step 3: Add AttorneyFormSheet at the bottom of rsx!**

After the `AlertDialogRoot` for delete confirmation, add:
```rust
if let Some(ref att) = attorney_data {
    AttorneyFormSheet {
        mode: FormMode::Edit,
        initial: Some(att.clone()),
        open: show_edit(),
        on_close: move |_| show_edit.set(false),
        on_saved: move |_| data.restart(),
    }
}
```

Where `attorney_data` is the parsed `AttorneyResponse` and `data` is the `use_resource`.

**Step 4: Verify compilation**

Run: `cargo check -p app`
Expected: Clean compile

**Step 5: Commit**

```bash
git add crates/app/src/routes/attorneys/detail.rs
git commit -m "feat(attorneys): add role-gated Edit button with edit sheet on detail page"
```

---

## PR 2: Judge Create + Edit

### Task 5: Create JudgeFormSheet component

**Files:**
- Create: `crates/app/src/routes/judges/form_sheet.rs`
- Modify: `crates/app/src/routes/judges/mod.rs` (add `pub mod form_sheet;`)

**Step 1: Add module declaration to mod.rs**

**Step 2: Create the form sheet**

Create `crates/app/src/routes/judges/form_sheet.rs` following the Attorney pattern but with Judge fields:

Fields:
- `name` (String, required)
- `title` (FormSelect from `JUDGE_TITLES`: Chief Judge, Judge, Senior Judge, Magistrate Judge, Visiting Judge)
- `district` (String, required)
- `status` (FormSelect from `JUDGE_STATUSES`: Active, Senior, Inactive, Retired, Deceased) — only in Edit mode
- `courtroom` (String, optional)
- `max_caseload` (number input, default "150")
- `specializations` (comma-separated string input, parsed to Vec<String>)

Import: `use shared_types::{JudgeResponse, JUDGE_TITLES, JUDGE_STATUSES};`

Key differences from Attorney:
- Uses PUT semantics (full payload) for update
- `specializations` needs split/join for comma-separated input
- `title` and `status` use FormSelect with constants, not free text
- Status field hidden in Create mode (server defaults to "Active")
- Server functions: `server::api::create_judge(court, body)` and `server::api::update_judge(court, id, body)`

**Step 3: Verify compilation**

Run: `cargo check -p app`

**Step 4: Commit**

```bash
git add crates/app/src/routes/judges/form_sheet.rs crates/app/src/routes/judges/mod.rs
git commit -m "feat(judges): add JudgeFormSheet with create and edit modes"
```

---

### Task 6: Wire JudgeFormSheet into list and detail pages

**Files:**
- Modify: `crates/app/src/routes/judges/list.rs`
- Modify: `crates/app/src/routes/judges/detail.rs`

**Step 1: List page — add create sheet**

Import `JudgeFormSheet` and `FormMode`. Add `show_sheet` signal. Wire the existing "New Judge" button (currently has empty onclick) to `show_sheet.set(true)`. Add `JudgeFormSheet { mode: Create, ... }` at bottom of rsx!.

**Step 2: Detail page — add edit button and sheet**

Import `JudgeFormSheet`, `FormMode`, `can`, `Action`, `use_user_role`. Add `show_edit` signal. Add role-gated Edit button in PageActions. Add `JudgeFormSheet { mode: Edit, initial: Some(judge_data) }` at bottom.

**Step 3: Verify compilation**

Run: `cargo check -p app`

**Step 4: Commit**

```bash
git add crates/app/src/routes/judges/list.rs crates/app/src/routes/judges/detail.rs
git commit -m "feat(judges): wire create and edit sheets into list and detail pages"
```

---

## PR 3: Cases Edit Sheet (Replace Inline Edit)

### Task 7: Create CaseFormSheet with PATCH semantics

**Files:**
- Create: `crates/app/src/routes/cases/form_sheet.rs`
- Modify: `crates/app/src/routes/cases/mod.rs`

Key differences from Attorney pattern:
- Uses PATCH semantics — `to_update_body()` sends only changed fields
- Track initial values separately to compute diff
- Fields: title, description, crime_type (FormSelect from CRIME_TYPES), status (FormSelect from CASE_STATUSES — edit only), priority (FormSelect from CASE_PRIORITIES), location, district_code
- Import: `use shared_types::{CaseResponse, CASE_STATUSES, CRIME_TYPES, CASE_PRIORITIES};`
- Server: `server::api::create_case(court, body)`, `server::api::update_case(court, id, body)`

PATCH diff pattern:
```rust
fn to_update_body(&self) -> serde_json::Value {
    let mut body = serde_json::Map::new();
    // Only include fields that differ from initial
    if *self.title.read() != self.initial_title {
        body.insert("title".into(), json!(self.title.read().clone()));
    }
    // ... repeat for each field
    serde_json::Value::Object(body)
}
```

**Commit:** `feat(cases): add CaseFormSheet with PATCH-only-changed-fields semantics`

---

### Task 8: Replace inline case edit with CaseFormSheet

**Files:**
- Modify: `crates/app/src/routes/cases/detail.rs`
- Modify: `crates/app/src/routes/cases/list.rs`

**Detail page changes:**
- Remove `CaseEditForm` component entirely
- Remove `CaseInfoDisplay` component (replace with always-visible read-only display)
- Remove `editing` signal and toggle logic from `CaseInfoTab`
- Add Edit button to PageActions (role-gated)
- Add `CaseFormSheet { mode: Edit }` at bottom

**List page changes:**
- Replace inline create Sheet with `CaseFormSheet { mode: Create }`

**Commit:** `feat(cases): replace inline edit toggle with edit sheet pattern`

---

## PR 4: Deadlines + Calendar Edit Sheets

### Task 9: Create DeadlineFormSheet

**Files:**
- Create: `crates/app/src/routes/deadlines/form_sheet.rs`
- Modify: `crates/app/src/routes/deadlines/mod.rs`
- Modify: `crates/app/src/routes/deadlines/list.rs`
- Modify: `crates/app/src/routes/deadlines/detail.rs`

Fields: title, due_at (datetime-local), status (FormSelect from DEADLINE_STATUSES — edit only), rule_code, case_id (CaseSelector), notes (Textarea)

Server: `create_deadline(court, body)`, `update_deadline(court, id, body)` — PUT semantics

**Commit:** `feat(deadlines): add edit sheet to deadline list and detail pages`

---

### Task 10: Create CalendarFormSheet

**Files:**
- Create: `crates/app/src/routes/calendar/form_sheet.rs`
- Modify: `crates/app/src/routes/calendar/mod.rs`
- Modify: `crates/app/src/routes/calendar/list.rs`
- Modify: `crates/app/src/routes/calendar/detail.rs`
- Modify: `crates/app/src/routes/calendar/create.rs` (redirect to list or use form_sheet internally)

Fields: case_id (CaseSelector), judge_id (JudgeSelector), event_type (FormSelect), scheduled_date (datetime-local), duration_minutes (number), courtroom, description (Textarea), participants (comma-separated), is_public (checkbox)

**Important:** Keep `/calendar/new` route working — have create.rs render the form sheet or redirect to list with sheet auto-opened.

**Verify:** Calendar event update endpoint exists. Check for `update_calendar_event` in api.rs. If missing, this is a server gap that needs a small server PR first.

**Commit:** `feat(calendar): add edit sheet to calendar list and detail pages`

---

## PR 5a: Orders Edit Sheet

### Task 11: Create OrderFormSheet with PATCH semantics

**Files:**
- Create: `crates/app/src/routes/orders/form_sheet.rs`
- Modify: `crates/app/src/routes/orders/mod.rs`
- Modify: `crates/app/src/routes/orders/list.rs`
- Modify: `crates/app/src/routes/orders/detail.rs`

Fields: title, order_type (FormSelect from ORDER_TYPES), case_id (CaseSelector — create only), judge_id (JudgeSelector — create only), content (Textarea), status (FormSelect from ORDER_STATUSES — edit only), is_sealed (checkbox — edit only), effective_date (datetime-local — edit only), expiration_date (datetime-local — edit only)

Uses PATCH semantics for update (only changed fields).

**Commit:** `feat(orders): add edit sheet with PATCH semantics`

---

## PR 5b: Order Workflow Actions

### Task 12: Add Sign/Issue workflow action buttons to order detail

**Files:**
- Modify: `crates/app/src/routes/orders/detail.rs`

**Step 1: Add workflow action buttons to the detail page header or Workflow tab**

Status-aware rendering:
```rust
let status = order.status.as_str();
let role = use_user_role();

// Sign: only for Draft or Pending Signature, Judge-only
if can(role, Action::Sign) && (status == "Draft" || status == "Pending Signature") {
    // Sign Order button + AlertDialog with signed_by input
}

// Issue: only for Signed, Clerk-only
if can(role, Action::Issue) && status == "Signed" {
    // Issue Order button + AlertDialog with issued_by input
}
```

Each action:
1. Button click opens AlertDialog
2. Dialog has Input for actor name (required) and optional reason Textarea
3. Confirm calls the server function
4. On success: refetch order data, toast success, show updated status badge + timestamp
5. On error: toast error
6. Button disabled while in-flight

Server functions:
- `server::api::sign_order_action(court, id, signed_by)` — exists
- `server::api::issue_order_action(court, id)` — exists

**Step 2: Add serve_order server function**

The `serve_order` server function does NOT exist in api.rs. Options:
- Create it in a separate server PR
- OR defer Serve to a follow-up PR

For now, implement Sign + Issue only. Add Serve as a follow-up task.

**Step 3: Display action metadata in detail header**

After each workflow action, the response includes `signer_name`, `signed_at`, `issued_at`. Display these in the detail header area:
```rust
if let Some(ref signer) = order.signer_name {
    Badge { variant: BadgeVariant::Primary, "Signed by {signer}" }
}
if let Some(ref signed_at) = order.signed_at {
    span { class: "text-muted", "on {signed_at}" }
}
```

**Commit:** `feat(orders): add sign and issue workflow actions with role gating`

---

## PR 6: Batch Edit Sheets (Opinions, Parties, Defendants, Evidence)

### Task 13–16: Create form sheets for 4 modules

Follow the established pattern from Tasks 2-4. Each module gets:
1. `form_sheet.rs` with FormMode::Create | Edit
2. Hydration via use_effect
3. Dirty guard with snapshot comparison
4. Correct PUT or PATCH semantics
5. Wired into list (create) and detail (edit) pages
6. Role-gated edit button

**Opinions** (PATCH semantics):
- Fields: title, case_id (CaseSelector), judge_id (JudgeSelector), opinion_type (FormSelect), content (Textarea), status, disposition, syllabus, keywords (comma-separated)
- Server: `create_opinion(court, body)`, `update_opinion(court, id, body)`

**Parties** (PUT semantics):
- Fields: case_id (CaseSelector — create only), party_type, party_role, name, entity_type, first_name, last_name, email, phone, pro_se (checkbox), status
- Server: `create_party(court, body)`, `update_party(court, id, body)`

**Defendants** (PUT semantics):
- Fields: case_id (CaseSelector — create only), first_name, last_name, date_of_birth, custody_status, marshal_id
- Server: `create_defendant(court, body)`, `update_defendant(court, id, body)`

**Evidence** (PUT semantics):
- Fields: case_id (CaseSelector — create only), evidence_type, description, submitted_by, storage_location, chain_of_custody_status
- Server: `create_evidence(court, body)`, `update_evidence(court, id, body)`

**Commit per module or batch:** `feat(ui): add edit sheets for opinions, parties, defendants, evidence`

---

## PR 7: Documents Operational

### Task 17: Add document workflow actions to detail page

**Files:**
- Modify: `crates/app/src/routes/documents/detail.rs`
- Modify: `crates/app/src/routes/documents/list.rs`

Add Seal/Unseal/Strike as WorkflowAction buttons (role-gated):
- Seal: POST /api/documents/{id}/seal (Judge/Clerk only)
- Unseal: POST /api/documents/{id}/unseal (Judge/Clerk only)
- Strike: POST /api/documents/{id}/strike (requires reason — mandatory AlertDialog input)

Add document events timeline in a "History" section:
- Fetch: `server::api::list_document_events(court, id)` — GET /api/documents/{id}/events

**Commit:** `feat(documents): add seal/unseal/strike actions and events timeline`

---

## PR 8: PDF Generation (Single-Order, v1)

### Task 18: Add PDF generation to order detail page

**Files:**
- Modify: `crates/app/src/routes/orders/detail.rs`

Add "Generate PDF" button with format dropdown:
- Standard court order
- Formatted court order
- Signed PDF (only if order status is "Signed" or later)

Response handling pattern:
```rust
// PDF endpoints return raw bytes or base64 — check actual response
// If bytes: trigger download via web_sys
// If URL: window.open()
// If job ID: poll until ready
```

**Risk:** Need to investigate actual PDF endpoint response format before implementing. Test one endpoint manually first.

**Commit:** `feat(pdf): add single-order PDF generation from detail page`

---

## Summary: Execution Order

| Task | PR | Description | Dependencies |
|------|-----|-------------|-------------|
| 1 | PR 1 | `can()` helper in auth.rs | None |
| 2 | PR 1 | AttorneyFormSheet component | Task 1 |
| 3 | PR 1 | Wire into attorney list page | Task 2 |
| 4 | PR 1 | Wire into attorney detail page | Task 2, 3 |
| 5 | PR 2 | JudgeFormSheet component | Task 1 |
| 6 | PR 2 | Wire into judge list + detail | Task 5 |
| 7 | PR 3 | CaseFormSheet (PATCH) | Task 1 |
| 8 | PR 3 | Replace inline case edit | Task 7 |
| 9 | PR 4 | DeadlineFormSheet | Task 1 |
| 10 | PR 4 | CalendarFormSheet | Task 1 |
| 11 | PR 5a | OrderFormSheet (PATCH) | Task 1 |
| 12 | PR 5b | Order Sign/Issue actions | Task 11 |
| 13-16 | PR 6 | 4 module form sheets | Task 1 |
| 17 | PR 7 | Documents operational | Task 1 |
| 18 | PR 8 | PDF generation | Task 12 |

**Known server gaps:**
- `serve_order_action` does not exist in api.rs — needs server PR before Serve workflow
- Calendar event update endpoint — needs verification
- PDF endpoint response format — needs manual testing
