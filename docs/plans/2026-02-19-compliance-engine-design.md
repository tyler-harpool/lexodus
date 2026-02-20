# Federal Rules Compliance Engine — Design Document

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement the corresponding implementation plan task-by-task.

**Goal:** Build an event-driven compliance engine that encodes FRCP, FRCrP, Speedy Trial Act, and local court rules as executable logic — automatically creating deadlines, enforcing valid state transitions, blocking non-compliant actions, auto-assigning judges, and managing fee schedules.

**Architecture:** Inline event hooks within Axum handlers. Every state-changing action evaluates federal rules (compiled Rust) + local rules (DB-stored JSONB) in the same database transaction. Command pattern for deferred effects. Postgres event log for audit trail.

**Audience:** Full CM/ECF replacement — court staff, attorneys, judges, and public.

---

## Core Architecture

### Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Event processing | Inline (same transaction) | Hard-blocking requires synchronous evaluation; no eventual consistency |
| Federal rules | Compiled Rust functions | Rarely change, must be correct, no admin misconfiguration risk |
| Local rules | DB-stored JSONB (rules table) | Court-specific, admin-configurable, layered on top of federal |
| Enforcement mode | Hard block + clerk/judge override | Matches real court operations; override logged to audit trail |
| Judge assignment | Weighted random wheel | Standard federal court practice; chief-judge configurable |
| Event storage | Postgres `case_events` table | Same transaction, queryable, replayable, zero new infrastructure |
| Message queue | None (no Kafka) | Overkill for court scale; Postgres event table sufficient |
| Deadline computation | FRCP 6(a) compliant | Business days for <11, calendar days for >=11, holiday-aware |

### Module Structure

```
crates/server/src/compliance/
├── mod.rs              — ComplianceEngine: evaluate() + apply_effects()
├── event.rs            — CaseEvent, ComplianceEffect, ComplianceViolation
├── federal/
│   ├── mod.rs          — FederalRules trait + registry
│   ├── frcp.rs         — Federal Rules of Civil Procedure
│   ├── frcrp.rs        — Federal Rules of Criminal Procedure
│   ├── speedy_trial.rs — Speedy Trial Act (18 U.S.C. §§ 3161-3174)
│   ├── fre.rs          — Federal Rules of Evidence (deadlines only)
│   └── frap.rs         — Federal Rules of Appellate Procedure
├── local.rs            — DB-driven local rules evaluator
├── assignment.rs       — Weighted random judge assignment wheel
├── calendar.rs         — Business day computation, federal holidays, FRCP 6(a)
└── status.rs           — Valid status transition maps (criminal + civil)
```

---

## Event System

### Case Events Table

```sql
CREATE TABLE case_events (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id    TEXT NOT NULL REFERENCES courts(id),
    case_id     UUID NOT NULL,
    case_type   TEXT NOT NULL CHECK (case_type IN ('criminal', 'civil')),
    event_type  TEXT NOT NULL,
    actor_id    BIGINT REFERENCES users(id),
    payload     JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_case_events_case ON case_events(case_id);
CREATE INDEX idx_case_events_court_type ON case_events(court_id, event_type);
CREATE INDEX idx_case_events_created ON case_events(created_at);
```

### Event Types

| Event | Trigger Point | Payload |
|-------|--------------|---------|
| `case_filed` | Case creation | `{case_type, crime_type/nos_code, priority}` |
| `service_completed` | Clerk records service | `{party_id, method, date}` |
| `answer_filed` | Filing with type=answer | `{filing_id, docket_entry_id}` |
| `motion_filed` | Filing with type=motion | `{filing_id, motion_type}` |
| `response_filed` | Filing with type=response | `{filing_id, responding_to}` |
| `order_entered` | Judge signs order | `{order_id, order_type}` |
| `status_changed` | Case status update | `{old_status, new_status}` |
| `judge_assigned` | Assignment created | `{judge_id, assignment_type}` |
| `arraignment_held` | Criminal milestone | `{date}` |
| `indictment_returned` | Criminal milestone | `{date}` |
| `discovery_opened` | Status → discovery | `{}` |
| `trial_date_set` | Scheduling order | `{trial_date}` |

---

## Compliance Engine Core

### ComplianceEngine

```rust
pub struct ComplianceEngine<'a> {
    pool: &'a Pool<Postgres>,
    court_id: &'a str,
}

impl ComplianceEngine<'_> {
    /// Evaluate federal + local rules against an event.
    /// Returns effects to apply and/or violations that block the action.
    pub async fn evaluate(
        &self,
        event: &CaseEvent,
        case: &CaseContext,
        override_token: Option<&ClerkOverride>,
    ) -> Result<Vec<ComplianceEffect>, ComplianceViolation>;

    /// Apply effects within the same DB transaction.
    pub async fn apply_effects(
        &self,
        tx: &mut Transaction<Postgres>,
        effects: Vec<ComplianceEffect>,
    ) -> Result<(), AppError>;
}
```

### CaseContext (loaded once per evaluation)

```rust
pub struct CaseContext {
    pub case_id: Uuid,
    pub case_type: CaseType,        // Criminal | Civil
    pub status: String,
    pub date_filed: DateTime<Utc>,
    pub assigned_judge_id: Option<Uuid>,
    pub trial_date: Option<DateTime<Utc>>,
    pub speedy_trial: Option<SpeedyTrialClock>,
    pub open_deadlines: Vec<Deadline>,
    pub fee_status: String,
    pub parties: Vec<PartySummary>,
}
```

### ComplianceEffect (Command Pattern)

```rust
pub enum ComplianceEffect {
    CreateDeadline {
        title: String,
        case_id: Uuid,
        rule_citation: String,
        due_at: DateTime<Utc>,
        notes: Option<String>,
    },
    StartSpeedyTrialClock {
        case_id: Uuid,
        trigger_date: DateTime<Utc>,
    },
    TollSpeedyTrialClock {
        case_id: Uuid,
        reason: String,
        statutory_ref: String,
    },
    ResumeSpeedyTrialClock {
        case_id: Uuid,
    },
    AssignJudge {
        case_id: Uuid,
        judge_id: Uuid,
        reason: String,
    },
    CreateQueueItem {
        case_id: Uuid,
        queue_type: String,
        title: String,
        priority: i32,
    },
    LogAuditEvent {
        case_id: Uuid,
        event_type: String,
        details: serde_json::Value,
    },
}
```

### ComplianceViolation (Hard Block)

```rust
pub struct ComplianceViolation {
    pub rule_citation: String,
    pub message: String,
    pub override_allowed: bool,
    pub override_role: String,  // "clerk" or "judge"
}
```

### Clerk Override

```rust
pub struct ClerkOverride {
    pub user_id: i64,
    pub role: String,
    pub reason: String,  // mandatory — logged to audit trail
}
```

### Integration Pattern (every endpoint)

```rust
let mut tx = pool.begin().await?;

// 1. Execute action tentatively
let filing = repo::filing::submit(&mut tx, ...).await?;

// 2. Build event
let event = CaseEvent::new("motion_filed", case_id, "civil", payload);

// 3. Evaluate compliance
let engine = ComplianceEngine::new(&pool, &court_id);
let case_ctx = engine.load_case_context(&tx, case_id).await?;
let effects = engine.evaluate(&event, &case_ctx, override_token).await?;

// 4. Apply effects (deadlines, clocks, assignments)
engine.apply_effects(&mut tx, effects).await?;

// 5. Record event
repo::case_event::insert(&mut tx, &event).await?;

// 6. Commit atomically
tx.commit().await?;
```

If step 3 returns a `ComplianceViolation` and no valid override is provided, the transaction rolls back.

---

## Federal Rules (Compiled Rust)

### RuleSet Trait

```rust
pub trait RuleSet: Send + Sync {
    fn evaluate(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult;
}

pub struct RuleResult {
    pub effects: Vec<ComplianceEffect>,
    pub violations: Vec<ComplianceViolation>,
}
```

### FRCP Rules (Civil Procedure)

| Rule | Citation | Trigger | Effect |
|------|----------|---------|--------|
| Service of Process | FRCP 4(m) | `case_filed` (civil) | Deadline: 90 days |
| Answer | FRCP 12(a)(1)(A)(i) | `service_completed` | Deadline: 21 days |
| Answer (US Govt) | FRCP 12(a)(2) | `service_completed` + govt party | Deadline: 60 days |
| Motion to Dismiss response | FRCP 12(b) | `motion_filed` (dismiss) | Deadline: 21 days |
| Discovery Conference | FRCP 26(f) | `answer_filed` | Deadline: 21 days |
| Initial Disclosures | FRCP 26(a)(1) | `discovery_opened` | Deadline: 14 days |
| Expert Disclosures | FRCP 26(a)(2) | `trial_date_set` | Deadline: 90 days before trial |
| Pretrial Disclosures | FRCP 26(a)(3) | `trial_date_set` | Deadline: 30 days before trial |
| Interrogatory Response | FRCP 33(b)(2) | `motion_filed` (interrogatories) | Deadline: 30 days |
| Document Production | FRCP 34(b)(2)(A) | `motion_filed` (doc_request) | Deadline: 30 days |
| Summary Judgment | FRCP 56(b) | `trial_date_set` | Deadline: 30 days before trial |
| Jury Demand | FRCP 38(b) | `answer_filed` | Deadline: 14 days |
| Motion Response | FRCP 6(d) | `motion_filed` (any) | Deadline: 14 days (+ 3 if mail) |
| Reply Brief | FRCP 6(d) | `response_filed` | Deadline: 14 days |
| Post-judgment Motion | FRCP 59(b) | judgment entered | Deadline: 28 days |
| Civil Appeal | FRAP 4(a)(1) | judgment/dismissal | Deadline: 30 days |
| Discovery block | FRCP 26(d) | `motion_filed` (discovery) | Block if before 26(f) conference |

### FRCrP Rules (Criminal Procedure)

| Rule | Citation | Trigger | Effect |
|------|----------|---------|--------|
| Initial Appearance | FRCrP 5(a) | `case_filed` (criminal) | Deadline: 48 hours (if arrested) |
| Arraignment | FRCrP 10 | `indictment_returned` | Deadline: without delay (flag at 14 days) |
| Pretrial Motions | FRCrP 12(b) | `arraignment_held` | Deadline: per schedule (default 21 days) |
| Brady Disclosure | FRCrP 16 + Brady | `arraignment_held` | Deadline: flag at 14 days, violation at 30 |
| Witness List | FRCrP 12.1 | `trial_date_set` | Deadline: per scheduling order |
| Sentencing | FRCrP 32(b)(1) | status → awaiting_sentencing | Deadline: 14-90 day window |
| Criminal Appeal | FRCrP 32(j) / FRAP 4(b) | sentenced/dismissed | Deadline: 14 days |

### Speedy Trial Act (18 U.S.C. §§ 3161-3174)

| Provision | Trigger | Effect |
|-----------|---------|--------|
| § 3161(b) | `case_filed` (criminal) | Start clock + deadline: indictment within 30 days |
| § 3161(c)(1) | `arraignment_held` | Deadline: trial within 70 days |
| § 3161(h)(1)(D) | `motion_filed` (pretrial) | Toll clock (excludable delay) |
| § 3161(h)(1)(A) | proceeding re: mental competency | Toll clock |
| § 3161(h)(1)(F) | interlocutory appeal | Toll clock |
| § 3161(h)(6) | continuance (ends of justice) | Toll clock (requires judge order) |
| § 3161(h)(7) | deferred prosecution | Toll clock |
| § 3162(a)(1) | clock expires (pre-indictment) | Violation: case subject to dismissal |
| § 3162(a)(2) | clock expires (pre-trial) | Violation: case subject to dismissal |

**Excludable delay reasons** (hardcoded from § 3161(h)):
- Pending pretrial motions
- Mental competency examination
- Interlocutory appeal
- Deferred prosecution agreement
- Defendant absence/unavailability
- Continuance granted by judge (ends of justice)
- Co-defendant joinder delay
- Transportation delay

---

## Status Transition Validation

### Criminal Case States

```
filed → arraigned → discovery → pretrial_motions → plea_negotiations
                                                  → trial_ready → in_trial
                                                                → awaiting_sentencing → sentenced
                                                  → dismissed
                                                  → on_appeal (from sentenced or dismissed)
```

### Civil Case States

```
filed → served → answer_due → at_issue → discovery → pretrial → trial_ready
                                                               → in_trial → judgment
                                                    → settled
                                         → dismissed
                                                    → on_appeal (from judgment or dismissed)
```

Invalid transitions return a ComplianceViolation with the applicable rule citation.

---

## Business Day Calculation (FRCP 6(a))

Rules:
- **< 11 days:** Count only business days (exclude weekends + federal holidays)
- **>= 11 days:** Count calendar days
- **Deadline on weekend/holiday:** Extends to next business day (FRCP 6(a)(1)(C))
- **+3 days for mail service** (FRCP 6(d))

Federal holidays (hardcoded):
New Year's Day, MLK Day, Presidents' Day, Memorial Day, Juneteenth, Independence Day, Labor Day, Columbus Day, Veterans Day, Thanksgiving, Christmas.

---

## Local Rules (DB-Driven)

### Rule Evaluation

Local rules use the existing `rules` table with enhanced `conditions` and `actions` JSONB schemas. Evaluated after federal rules in the same pass.

**Priority:** Federal rules always win. Local rules can only be **more restrictive** (shorter deadlines, additional requirements), never less restrictive than federal rules.

### Enhanced Conditions Schema

```json
{
  "trigger": "motion_filed",
  "case_type": "civil",
  "motion_type": "summary_judgment",
  "case_status": ["discovery", "pretrial_motions"]
}
```

Supports:
- Exact match: `"case_type": "civil"`
- Array match (any of): `"case_status": ["discovery", "pretrial_motions"]`
- Nested payload match: `"payload.motion_type": "dismiss"`

### Enhanced Actions Schema

```json
{ "create_deadline": { "days": 14, "days_from": "event", "title": "...", "business_days": true } }
{ "create_deadline": { "days_before_trial": 60, "title": "...", "business_days": true } }
{ "block_action": { "message": "...", "override_role": "judge" } }
{ "warn": { "message": "..." } }
{ "require_attachment": { "message": "..." } }
{ "require_fee": { "entry_type": "complaint" } }
{ "notify": { "to": "assigned_judge", "message": "..." } }
```

### Example Local Rules

| Rule | Trigger | Action |
|------|---------|--------|
| L.R. 7.1 — Motion page limit | `motion_filed` | `warn` if > 25 pages |
| L.R. 16.1 — Pretrial memo | `trial_date_set` | `create_deadline` 60 days before trial |
| L.R. 5.2 — Sealed filings | `motion_filed` + sealed | `block_action` (judge override) |
| L.R. 26.1 — Corporate disclosure | `case_filed` (civil) + corporate | `create_deadline` 14 days |
| Standing Order — Settlement | `discovery_opened` | `create_deadline` 90 days for ADR |

---

## Judge Assignment Wheel

### Configuration Table

```sql
CREATE TABLE judge_assignment_config (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id    TEXT NOT NULL REFERENCES courts(id),
    judge_id    UUID NOT NULL REFERENCES judges(id),
    case_type   TEXT NOT NULL CHECK (case_type IN ('criminal', 'civil')),
    weight      INTEGER NOT NULL CHECK (weight > 0 AND weight <= 100),
    active      BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(court_id, judge_id, case_type)
);
```

### Algorithm

1. Fetch active configs for court + case type
2. Filter out judges at `max_caseload`
3. Check for recusal/conflict (related cases, same defendant)
4. Weighted random selection from remaining pool
5. If no eligible judges, create queue item for manual assignment
6. Log assignment with weight used to `case_events`

### Related Case Detection

Cases with the same defendant or co-conspirators automatically assigned to the same judge (prevents judge-shopping).

### Admin API

| Endpoint | Role | Description |
|----------|------|-------------|
| `GET /api/admin/assignment-config` | Court Admin | List all judge weights |
| `PUT /api/admin/assignment-config` | Court Admin | Set/update weights (bulk) |
| `GET /api/admin/assignment-stats` | Court Admin | Actual vs. target distribution |

---

## Fee Schedule Administration

### Admin API

| Endpoint | Role | Description |
|----------|------|-------------|
| `GET /api/admin/fee-schedule` | Any auth'd | List all fees for court |
| `POST /api/admin/fee-schedule` | Court Admin | Create fee entry |
| `PUT /api/admin/fee-schedule/:id` | Court Admin | Update fee |
| `DELETE /api/admin/fee-schedule/:id` | Court Admin | Deactivate fee |
| `POST /api/admin/fee-schedule/import` | Court Admin | Bulk import Judicial Conference schedule |

### Compliance Integration

Civil `case_filed` event triggers fee check via `28 U.S.C. § 1914(a)`. If fee required and `fee_status == 'pending'`, returns ComplianceViolation (override allowed by clerk for IFP/government).

### Fee Waiver Flow

1. Attorney files IFP motion → docket entry + queue item
2. Judge grants IFP order → clerk sets `fee_status = 'ifp'`
3. Subsequent filings skip fee check
4. Government parties auto-detected → `fee_status = 'government'`

---

## Admin UI Pages

### `/admin/assignment` — Judge Assignment Wheel

- Per-case-type tabs (Criminal | Civil)
- Judge weight sliders (1-100)
- Visual distribution chart (target vs. actual)
- Activate/deactivate judges
- Stats panel: last 30/90/365 days
- Audit log of config changes

### `/admin/fees` — Fee Schedule Management

- Table of all fee entries
- Add/edit/deactivate fees
- Effective date scheduling
- Import from Judicial Conference standard
- Change history log

### `/admin/rules` — Local Rules Management

- CRUD for local court rules
- Condition builder (trigger + filters)
- Action builder (deadline, block, warn, notify)
- Test mode: evaluate rules against sample events
- Cannot edit federal rules (view-only)

---

## Event Lifecycle (Complete)

### Criminal Case

| Step | Event | Federal Rules | Engine Actions |
|------|-------|---------------|---------------|
| 1 | `case_filed` | § 3161(b): indictment 30 days, FRCrP 5: initial appearance | Start Speedy Trial clock, auto-assign judge, queue item |
| 2 | `indictment_returned` | FRCrP 10: arraignment | Deadline |
| 3 | `arraignment_held` | § 3161(c): trial 70 days, FRCrP 12(b): motions, FRCrP 16: Brady | Multiple deadlines |
| 4 | `motion_filed` | § 3161(h): toll clock, response deadline | Toll clock, create deadline |
| 5 | `order_entered` | Resume clock | Recalculate days_remaining |
| 6 | `trial_date_set` | Verify Speedy Trial window, FRCrP 12.1: witnesses | Validate + deadlines |
| 7 | `status_changed` → sentencing | FRCrP 32(b)(1): PSR, sentencing window | Deadlines |
| 8 | `status_changed` → sentenced | FRCrP 32(j): appeal deadline 14 days | Close clock, deadline |

### Civil Case

| Step | Event | Federal Rules | Engine Actions |
|------|-------|---------------|---------------|
| 1 | `case_filed` | FRCP 4(m): service 90 days, 28 U.S.C. § 1914: fee | Auto-assign judge, fee check, queue item |
| 2 | `service_completed` | FRCP 12(a): answer 21/60 days | Deadline |
| 3 | `answer_filed` | FRCP 26(f): conference 21 days, FRCP 38(b): jury 14 days | Deadlines |
| 4 | `discovery_opened` | FRCP 26(a)(1): disclosures 14 days | Deadlines + local rules |
| 5 | `motion_filed` | FRCP 6(d): response 14 days | Deadline, compliance check |
| 6 | `trial_date_set` | FRCP 26(a)(2/3): disclosures, FRCP 56: SJ | Multiple deadlines |
| 7 | judgment/dismissal | FRCP 59: post-judgment 28 days, FRAP 4(a): appeal 30 days | Deadlines |

---

## Migration of Existing Rules

The 15 FRCP rules currently seeded in the `rules` table (migration 000089) will be:
1. **Removed from `rules` table** — federal rules move to compiled Rust
2. **`rules` table reserved for local rules only** — court admin configurable
3. **Source column** updated to only allow local/standing order sources

---

## Not Building

- Kafka or external message queue
- Real-time WebSocket push for deadline notifications (future enhancement)
- Cross-district federated rule evaluation
- AI-powered rule interpretation
- E-filing portal for pro se litigants (future)
