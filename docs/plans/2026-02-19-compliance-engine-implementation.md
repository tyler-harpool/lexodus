# Federal Rules Compliance Engine — Implementation Plan (v2: Ported Architecture)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Port the proven rules evaluation engine from spin-lexodus to the Lexodus Postgres/Axum stack — bringing the 5-stage pipeline (select → prioritize → evaluate conditions → process actions → report), recursive condition tree evaluator, FRCP 6(a) deadline calculator, 48 trigger event types, and 112 ARWD rules. Add new capabilities: `case_events` audit table, Postgres-backed rule storage, endpoint wiring, fee schedule admin, and judge assignment wheel.

**Architecture:** Port the hexagonal RulesEngine from spin-lexodus (Spin KV → Postgres). Rules stored in `rules` table with JSONB conditions/actions using the recursive tagged-enum format from the old system. Engine is stateless — takes `(FilingContext, Vec<Rule>)` → `ComplianceReport`. Evaluation happens inline in Axum handlers (same DB transaction). `case_events` table provides audit trail. No external message queue.

**Tech Stack:** Rust, Axum, sqlx, PostgreSQL, chrono (FRCP 6(a) business day math), serde_json (recursive condition tree)

**Prior Art:** `spin-lexodus/src/domain/rule.rs`, `spin-lexodus/src/adapters/rules_engine_impl.rs`, `spin-lexodus/src/adapters/deadline_engine_impl.rs`, `spin-lexodus/config/jurisdictions/arwd/rules.toml`

**Design Doc:** `docs/plans/2026-02-19-compliance-engine-design.md`

---

## Phase 1: Schema + Domain Types (Tasks 1–3)

### Task 1: Migrate Rules Table + Add Case Events

**Files:**
- Create: `migrations/20260301000092_add_triggers_to_rules.sql`
- Create: `migrations/20260301000092_add_triggers_to_rules.down.sql`
- Create: `migrations/20260301000093_create_case_events.sql`
- Create: `migrations/20260301000093_create_case_events.down.sql`
- Create: `migrations/20260301000094_create_fee_schedule.sql`
- Create: `migrations/20260301000094_create_fee_schedule.down.sql`
- Modify: `crates/tests/src/common.rs` (add tables to TRUNCATE)

**Step 1: Create migrations**

`migrations/20260301000092_add_triggers_to_rules.sql`:

```sql
-- Add triggers column (JSONB array of trigger event strings)
ALTER TABLE rules ADD COLUMN IF NOT EXISTS triggers JSONB NOT NULL DEFAULT '[]';

-- Expand source CHECK to include all rule sources from spin-lexodus
ALTER TABLE rules DROP CONSTRAINT IF EXISTS rules_source_check;
ALTER TABLE rules ADD CONSTRAINT rules_source_check
    CHECK (source IN (
        'Federal Rules of Civil Procedure',
        'Federal Rules of Criminal Procedure',
        'Federal Rules of Evidence',
        'Federal Rules of Appellate Procedure',
        'Local Rules',
        'Standing Orders',
        'Statutory',
        'Administrative',
        'Custom',
        'General Order'
    ));

-- Expand category CHECK to include all categories
ALTER TABLE rules DROP CONSTRAINT IF EXISTS rules_category_check;
ALTER TABLE rules ADD CONSTRAINT rules_category_check
    CHECK (category IN (
        'Procedural', 'Evidentiary', 'Deadline', 'Filing', 'Discovery',
        'Sentencing', 'Appeal', 'Administrative', 'Other',
        'Fee', 'Assignment', 'Service', 'Sealing', 'Privacy', 'Format'
    ));

-- Index on triggers for rule selection
CREATE INDEX IF NOT EXISTS idx_rules_triggers ON rules USING GIN (triggers);

-- Backfill triggers from existing conditions JSON where trigger key exists
UPDATE rules
SET triggers = jsonb_build_array(conditions->>'trigger')
WHERE conditions ? 'trigger' AND triggers = '[]';
```

Down migration:
```sql
ALTER TABLE rules DROP COLUMN IF EXISTS triggers;
-- Revert CHECK constraints (omitted for brevity — keep expanded set)
DROP INDEX IF EXISTS idx_rules_triggers;
```

`migrations/20260301000093_create_case_events.sql`:

```sql
CREATE TABLE IF NOT EXISTS case_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL REFERENCES courts(id),
    case_id         UUID NOT NULL,
    case_type       TEXT NOT NULL CHECK (case_type IN ('criminal', 'civil')),
    trigger_event   TEXT NOT NULL,
    actor_id        UUID,
    payload         JSONB NOT NULL DEFAULT '{}',
    compliance_report JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_case_events_court_case ON case_events(court_id, case_id);
CREATE INDEX idx_case_events_trigger ON case_events(trigger_event);
CREATE INDEX idx_case_events_created ON case_events(created_at DESC);
```

`migrations/20260301000094_create_fee_schedule.sql`:

```sql
CREATE TABLE IF NOT EXISTS fee_schedule (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL REFERENCES courts(id),
    fee_id          TEXT NOT NULL,
    category        TEXT NOT NULL,
    description     TEXT NOT NULL,
    amount_cents    INT NOT NULL,
    statute         TEXT,
    waivable        BOOLEAN NOT NULL DEFAULT false,
    waiver_form     TEXT,
    cap_cents       INT,
    cap_description TEXT,
    effective_date  DATE NOT NULL DEFAULT CURRENT_DATE,
    active          BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(court_id, fee_id, effective_date)
);

CREATE INDEX idx_fee_schedule_court ON fee_schedule(court_id);
CREATE INDEX idx_fee_schedule_court_active ON fee_schedule(court_id, active);
```

**Step 2: Add to TRUNCATE in test helper**

In `crates/tests/src/common.rs`, add `case_events, fee_schedule` to the TRUNCATE list.

**Step 3: Run migration**

```bash
sqlx migrate run
cargo check -p server
```

**Step 4: Commit**

```bash
git add migrations/20260301000092* migrations/20260301000093* migrations/20260301000094* crates/tests/src/common.rs
git commit -m "feat(compliance): add triggers column, case_events table, fee_schedule table"
```

---

### Task 2: Port Domain Types to shared-types

**Files:**
- Create: `crates/shared-types/src/compliance.rs`
- Modify: `crates/shared-types/src/lib.rs` (add `pub mod compliance;`)
- Modify: `crates/shared-types/src/rule.rs` (add `triggers` field to Rule struct)

**Step 1: Add triggers to Rule struct**

In `crates/shared-types/src/rule.rs`, add to the `Rule` struct:

```rust
/// Trigger events that activate this rule (JSONB array of strings)
pub triggers: serde_json::Value,
```

And add `triggers` to `CreateRuleRequest` and `RuleResponse`:

```rust
// In CreateRuleRequest:
#[serde(default)]
pub triggers: Option<serde_json::Value>,

// In RuleResponse:
pub triggers: serde_json::Value,

// In From<Rule> for RuleResponse:
triggers: r.triggers,
```

**Step 2: Create compliance domain types**

Create `crates/shared-types/src/compliance.rs` — ported directly from `spin-lexodus/src/domain/rule.rs`, `spin-lexodus/src/domain/filing_pipeline.rs`, and `spin-lexodus/src/domain/deadline_calc.rs`:

```rust
//! Compliance engine domain types — ported from spin-lexodus
//!
//! These types define the rule evaluation pipeline: trigger events,
//! recursive condition trees, typed actions, filing context, and
//! compliance reports.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Trigger Events ────────────────────────────────────────────

/// Events that can trigger rule evaluation.
/// Ported from spin-lexodus (48 event types covering full case lifecycle).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TriggerEvent {
    CaseFiled,
    MotionFiled,
    OrderIssued,
    DocumentFiled,
    StatusChanged,
    DeadlineApproaching,
    PleaEntered,
    SentencingScheduled,
    CaseAssigned,
    CaseReassigned,
    AppearanceFiled,
    ExtensionRequested,
    ManualEvaluation,
    ComplaintFiled,
    ServiceComplete,
    DocumentServed,
    AmendedPleadingFiled,
    LeaveToAmendGranted,
    SummaryJudgmentFiled,
    SummaryJudgmentResponseFiled,
    JudgmentEntered,
    MagistrateRecommendationFiled,
    ProHacViceFiled,
    ClassActionFiled,
    DiscoveryRequestServed,
    DiscoveryResponseFiled,
    ProposedOrderSubmitted,
    DocumentUploaded,
    SettlementReached,
    WaiverOfServiceRequested,
    WaiverOfServiceAccepted,
    AnswerFiled,
    MotionDenied,
    ThirdPartyComplaintFiled,
    Rule26fConferenceHeld,
    PartyJoined,
    TrialDateSet,
    DepositionNoticed,
    StatementOfDeathFiled,
    DefendantAppeared,
    TroEntered,
    OfferOfJudgmentServed,
    MagistrateOrderEntered,
    AnswerDeadlinePassed,
    DiscoveryClosed,
    ResponseFiled,
    LastPleadingServed,
    NoActivity,
}

impl TriggerEvent {
    /// Parse from a string (used when reading from JSONB triggers array).
    pub fn from_str_opt(s: &str) -> Option<Self> {
        serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
    }
}

// ─── Recursive Condition Tree ──────────────────────────────────

/// Recursive condition tree for rule evaluation.
/// Supports boolean logic (And/Or/Not) and field-level checks.
/// Stored as JSONB in the `rules.conditions` column.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleCondition {
    And { conditions: Vec<RuleCondition> },
    Or { conditions: Vec<RuleCondition> },
    Not { condition: Box<RuleCondition> },
    FieldEquals { field: String, value: String },
    FieldContains { field: String, value: String },
    FieldExists { field: String },
    FieldGreaterThan { field: String, value: String },
    FieldLessThan { field: String, value: String },
    Always,
}

// ─── Rule Actions ──────────────────────────────────────────────

/// Actions a rule can produce when triggered.
/// Stored as JSONB in the `rules.actions` column.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleAction {
    GenerateDeadline {
        description: String,
        days_from_trigger: i32,
    },
    RequireRedaction {
        fields: Vec<String>,
    },
    SendNotification {
        recipient: String,
        message: String,
    },
    BlockFiling {
        reason: String,
    },
    RequireFee {
        amount_cents: u64,
        description: String,
    },
    FlagForReview {
        reason: String,
    },
    LogCompliance {
        message: String,
    },
}

// ─── Rule Priority ─────────────────────────────────────────────

/// Priority level for rule evaluation ordering.
/// Higher weight = evaluated first.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RulePriority {
    Statutory,          // weight 10
    FederalRule,        // weight 20
    Administrative,     // weight 30
    Local,              // weight 40
    StandingOrder,      // weight 50
}

impl RulePriority {
    pub fn weight(&self) -> u32 {
        match self {
            Self::StandingOrder => 50,
            Self::Local => 40,
            Self::Administrative => 30,
            Self::FederalRule => 20,
            Self::Statutory => 10,
        }
    }

    /// Map from DB priority integer to enum.
    /// Convention: 10=Statutory, 20=Federal, 30=Admin, 40=Local, 50=Standing
    pub fn from_db_priority(p: i32) -> Self {
        match p {
            50.. => Self::StandingOrder,
            40..=49 => Self::Local,
            30..=39 => Self::Administrative,
            20..=29 => Self::FederalRule,
            _ => Self::Statutory,
        }
    }
}

// ─── Service Method ────────────────────────────────────────────

/// Method of service for deadline computation purposes.
/// Per FRCP 6(d): mail/leaving with clerk/other adds 3 days.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceMethod {
    Electronic,
    PersonalDelivery,
    Mail,
    LeavingWithClerk,
    Other,
}

impl ServiceMethod {
    pub fn additional_days(&self) -> i32 {
        match self {
            Self::Electronic | Self::PersonalDelivery => 0,
            Self::Mail | Self::LeavingWithClerk | Self::Other => 3,
        }
    }
}

impl Default for ServiceMethod {
    fn default() -> Self {
        Self::Electronic
    }
}

// ─── Filing Context ────────────────────────────────────────────

/// Context information about a filing/action used during rule evaluation.
/// The engine resolves field values from this struct + metadata JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilingContext {
    pub case_type: String,
    pub document_type: String,
    pub filer_role: String,
    pub jurisdiction_id: String,
    pub division: Option<String>,
    pub assigned_judge: Option<String>,
    pub service_method: Option<ServiceMethod>,
    /// Extensible metadata (case_id, party_count, sealed, etc.)
    pub metadata: serde_json::Value,
}

// ─── Compliance Report ─────────────────────────────────────────

/// Report of compliance check results for a filing/action.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceReport {
    pub results: Vec<RuleResult>,
    pub blocked: bool,
    pub block_reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub deadlines: Vec<DeadlineResult>,
    pub fees: Vec<FeeRequirement>,
}

/// Result of evaluating a single rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    pub rule_id: Uuid,
    pub rule_name: String,
    pub matched: bool,
    pub action_taken: String,
    pub message: String,
}

/// A computed deadline from rule evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadlineResult {
    pub due_date: NaiveDate,
    pub description: String,
    pub rule_citation: String,
    pub computation_notes: String,
    pub is_short_period: bool,
}

/// A fee requirement from rule evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeRequirement {
    pub rule_id: Uuid,
    pub rule_name: String,
    pub amount_cents: u64,
    pub description: String,
}

// ─── Deadline Computation Request ──────────────────────────────

/// Request to compute a deadline per FRCP 6(a).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadlineComputeRequest {
    pub trigger_date: NaiveDate,
    pub period_days: i32,
    pub service_method: ServiceMethod,
    pub jurisdiction: String,
    pub description: String,
    pub rule_citation: String,
}

/// A federal holiday for deadline computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederalHoliday {
    pub date: NaiveDate,
    pub name: String,
}
```

**Step 3: Register module in lib.rs**

Add `pub mod compliance;` to `crates/shared-types/src/lib.rs`.

**Step 4: Verify compilation**

```bash
cargo check -p shared-types
```

**Step 5: Commit**

```bash
git add crates/shared-types/src/compliance.rs crates/shared-types/src/lib.rs crates/shared-types/src/rule.rs
git commit -m "feat(compliance): port domain types from spin-lexodus (triggers, conditions, actions, context, report)"
```

---

### Task 3: Port FRCP 6(a) Deadline Computation Engine

**Files:**
- Create: `crates/server/src/compliance/mod.rs`
- Create: `crates/server/src/compliance/deadline_engine.rs`
- Modify: `crates/server/src/lib.rs` (add `pub mod compliance;`)
- Create: `crates/tests/src/deadline_engine_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Create compliance module directory**

`crates/server/src/compliance/mod.rs`:

```rust
pub mod deadline_engine;
pub mod engine;
pub mod condition_evaluator;
```

Note: `engine` and `condition_evaluator` will be created in later tasks. For now they can be empty files or the mod.rs can just have `pub mod deadline_engine;`.

**Step 2: Port the deadline engine**

`crates/server/src/compliance/deadline_engine.rs` — ported from `spin-lexodus/src/adapters/deadline_engine_impl.rs`:

```rust
//! FRCP Rule 6(a) deadline computation engine
//!
//! Ported from spin-lexodus. Computes deadlines per FRCP 6(a)(1)
//! (effective Dec 1, 2009). All periods use calendar-day counting.
//! Landing day on weekend/holiday extends to next business day.
//! Service method adjustments per FRCP 6(d).

use chrono::{Datelike, NaiveDate, Weekday};
use shared_types::compliance::{
    DeadlineComputeRequest, DeadlineResult, FederalHoliday, ServiceMethod,
};

const SHORT_PERIOD_THRESHOLD: i32 = 14;

/// Compute a deadline per FRCP 6(a).
pub fn compute_deadline(request: &DeadlineComputeRequest) -> Result<DeadlineResult, String> {
    if request.period_days < 0 {
        return Err("Period days cannot be negative".to_string());
    }

    let service_additional = request.service_method.additional_days();
    let total_period = request.period_days + service_additional;
    let is_short = total_period <= SHORT_PERIOD_THRESHOLD;

    // Step 1: Exclude trigger date — start from next day
    let start_date = request
        .trigger_date
        .succ_opt()
        .ok_or("Trigger date overflow")?;

    // Step 2: Count ALL calendar days (FRCP 6(a)(1))
    let raw_due_date = count_calendar_days(start_date, total_period)?;

    // Step 3: If due date falls on weekend or holiday, extend
    let due_date = next_business_day(raw_due_date);

    let mut notes = Vec::new();
    notes.push(format!(
        "Trigger date: {}; counting begins {}",
        request.trigger_date, start_date
    ));

    if service_additional > 0 {
        notes.push(format!(
            "Service method ({:?}): +{} days added to base period of {} days",
            request.service_method, service_additional, request.period_days
        ));
    }

    notes.push(format!(
        "Total period: {} calendar days{}",
        total_period,
        if is_short {
            " (short period per FRCP 6(a)(2))"
        } else {
            ""
        }
    ));

    if due_date != raw_due_date {
        notes.push(format!(
            "Landing day {} falls on weekend/holiday; extended to next business day {}",
            raw_due_date, due_date
        ));
    }

    notes.push(format!("Due date: {}", due_date));

    Ok(DeadlineResult {
        due_date,
        description: request.description.clone(),
        rule_citation: request.rule_citation.clone(),
        computation_notes: notes.join("; "),
        is_short_period: is_short,
    })
}

/// Check if a date is a federal holiday.
pub fn is_federal_holiday(date: NaiveDate) -> bool {
    get_federal_holidays(date.year())
        .iter()
        .any(|h| h.date == date)
}

/// Check if a date is a weekend.
pub fn is_weekend(date: NaiveDate) -> bool {
    matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
}

/// Find the next business day (skipping weekends and federal holidays).
pub fn next_business_day(date: NaiveDate) -> NaiveDate {
    let mut current = date;
    while is_weekend(current) || is_federal_holiday(current) {
        current = current.succ_opt().unwrap_or(current);
    }
    current
}

/// Count calendar days from a start date per FRCP 6(a)(1).
fn count_calendar_days(start: NaiveDate, days: i32) -> Result<NaiveDate, String> {
    if days <= 0 {
        return Ok(start);
    }
    start
        .checked_add_signed(chrono::Duration::days((days - 1) as i64))
        .ok_or_else(|| "Date overflow during calendar day count".to_string())
}

/// Compute the nth occurrence of a given weekday in a month.
fn nth_weekday_of_month(year: i32, month: u32, weekday: Weekday, n: u32) -> NaiveDate {
    let first_of_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let first_weekday = first_of_month.weekday();
    let days_ahead = (weekday.num_days_from_monday() as i32
        - first_weekday.num_days_from_monday() as i32
        + 7) % 7;
    let day = 1 + days_ahead as u32 + (n - 1) * 7;
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

/// Compute the last occurrence of a given weekday in a month.
fn last_weekday_of_month(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
    let last_day = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };
    let last_of_month = last_day.pred_opt().unwrap();
    let last_weekday = last_of_month.weekday();
    let days_back = (last_weekday.num_days_from_monday() as i32
        - weekday.num_days_from_monday() as i32
        + 7) % 7;
    NaiveDate::from_ymd_opt(year, month, last_of_month.day() - days_back as u32).unwrap()
}

/// Apply federal holiday observation rule (Sat→Friday, Sun→Monday).
fn observed_date(date: NaiveDate) -> NaiveDate {
    match date.weekday() {
        Weekday::Sat => date.pred_opt().unwrap(),
        Weekday::Sun => date.succ_opt().unwrap(),
        _ => date,
    }
}

fn add_observed_holiday(holidays: &mut Vec<FederalHoliday>, year: i32, month: u32, day: u32, name: &str) {
    let actual = NaiveDate::from_ymd_opt(year, month, day).unwrap();
    let obs = observed_date(actual);
    holidays.push(FederalHoliday {
        date: obs,
        name: name.to_string(),
    });
}

/// Get all federal holidays for a given year (11 holidays).
pub fn get_federal_holidays(year: i32) -> Vec<FederalHoliday> {
    let mut holidays = Vec::new();

    add_observed_holiday(&mut holidays, year, 1, 1, "New Year's Day");

    holidays.push(FederalHoliday {
        date: nth_weekday_of_month(year, 1, Weekday::Mon, 3),
        name: "Martin Luther King Jr. Day".to_string(),
    });

    holidays.push(FederalHoliday {
        date: nth_weekday_of_month(year, 2, Weekday::Mon, 3),
        name: "Presidents' Day".to_string(),
    });

    holidays.push(FederalHoliday {
        date: last_weekday_of_month(year, 5, Weekday::Mon),
        name: "Memorial Day".to_string(),
    });

    add_observed_holiday(&mut holidays, year, 6, 19, "Juneteenth");
    add_observed_holiday(&mut holidays, year, 7, 4, "Independence Day");

    holidays.push(FederalHoliday {
        date: nth_weekday_of_month(year, 9, Weekday::Mon, 1),
        name: "Labor Day".to_string(),
    });

    holidays.push(FederalHoliday {
        date: nth_weekday_of_month(year, 10, Weekday::Mon, 2),
        name: "Columbus Day".to_string(),
    });

    add_observed_holiday(&mut holidays, year, 11, 11, "Veterans Day");

    holidays.push(FederalHoliday {
        date: nth_weekday_of_month(year, 11, Weekday::Thu, 4),
        name: "Thanksgiving Day".to_string(),
    });

    add_observed_holiday(&mut holidays, year, 12, 25, "Christmas Day");

    holidays.sort_by_key(|h| h.date);
    holidays
}
```

**Step 3: Register in server lib.rs**

Add `pub mod compliance;` to `crates/server/src/lib.rs`.

**Step 4: Write tests**

`crates/tests/src/deadline_engine_tests.rs` — ported from `spin-lexodus/src/adapters/deadline_engine_impl.rs` tests:

```rust
//! FRCP 6(a) deadline computation tests
//! Ported from spin-lexodus

use chrono::NaiveDate;
use shared_types::compliance::{DeadlineComputeRequest, ServiceMethod};
use server::compliance::deadline_engine::*;

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn make_request(trigger: NaiveDate, period: i32, service: ServiceMethod) -> DeadlineComputeRequest {
    DeadlineComputeRequest {
        trigger_date: trigger,
        period_days: period,
        service_method: service,
        jurisdiction: "TEST".to_string(),
        description: "Test deadline".to_string(),
        rule_citation: "FRCP 12(a)".to_string(),
    }
}

#[test]
fn holiday_list_has_eleven_entries() {
    assert_eq!(get_federal_holidays(2025).len(), 11);
}

#[test]
fn mlk_day_2025_is_jan_20() {
    let holidays = get_federal_holidays(2025);
    let mlk = holidays.iter().find(|h| h.name.contains("King")).unwrap();
    assert_eq!(mlk.date, date(2025, 1, 20));
}

#[test]
fn memorial_day_2025_is_may_26() {
    let holidays = get_federal_holidays(2025);
    let mem = holidays.iter().find(|h| h.name.contains("Memorial")).unwrap();
    assert_eq!(mem.date, date(2025, 5, 26));
}

#[test]
fn july_4_2026_saturday_observed_friday() {
    assert!(is_federal_holiday(date(2026, 7, 3)));
    assert!(!is_federal_holiday(date(2026, 7, 4)));
}

#[test]
fn saturday_is_weekend() {
    assert!(is_weekend(date(2025, 10, 4)));
}

#[test]
fn monday_is_not_weekend() {
    assert!(!is_weekend(date(2025, 10, 6)));
}

#[test]
fn next_business_day_on_weekday_unchanged() {
    assert_eq!(next_business_day(date(2025, 10, 8)), date(2025, 10, 8));
}

#[test]
fn next_business_day_on_saturday_goes_to_monday() {
    assert_eq!(next_business_day(date(2025, 10, 4)), date(2025, 10, 6));
}

#[test]
fn next_business_day_on_holiday_skips() {
    // Christmas 2025 is Thursday
    assert_eq!(next_business_day(date(2025, 12, 25)), date(2025, 12, 26));
}

#[test]
fn five_day_period_lands_on_weekend_then_holiday() {
    // 5-day from Mon Oct 6, 2025
    // Start: Oct 7, +4 days = Oct 11 (Sat) → Mon Oct 13 (Columbus Day) → Tue Oct 14
    let req = make_request(date(2025, 10, 6), 5, ServiceMethod::Electronic);
    let result = compute_deadline(&req).unwrap();
    assert_eq!(result.due_date, date(2025, 10, 14));
    assert!(result.is_short_period);
}

#[test]
fn five_day_with_mail_adds_three() {
    // 5 + 3 = 8 days, from Oct 6
    // Start: Oct 7, +7 = Oct 14 (Tue, Columbus Day observed Mon Oct 13)
    let req = make_request(date(2025, 10, 6), 5, ServiceMethod::Mail);
    let result = compute_deadline(&req).unwrap();
    assert_eq!(result.due_date, date(2025, 10, 14));
    assert!(result.is_short_period);
}

#[test]
fn thirty_day_period() {
    // 30-day from Oct 7: start Oct 8, +29 = Nov 6 (Thu)
    let req = make_request(date(2025, 10, 7), 30, ServiceMethod::Electronic);
    let result = compute_deadline(&req).unwrap();
    assert_eq!(result.due_date, date(2025, 11, 6));
    assert!(!result.is_short_period);
}

#[test]
fn landing_on_christmas_extends() {
    // 31-day from Nov 25: start Nov 26, +30 = Dec 25 (Thu Christmas) → Dec 26
    let req = make_request(date(2025, 11, 25), 31, ServiceMethod::Electronic);
    let result = compute_deadline(&req).unwrap();
    assert_eq!(result.due_date, date(2025, 12, 26));
}

#[test]
fn zero_day_period() {
    let req = make_request(date(2025, 10, 6), 0, ServiceMethod::Electronic);
    let result = compute_deadline(&req).unwrap();
    assert_eq!(result.due_date, date(2025, 10, 7));
}

#[test]
fn negative_period_is_error() {
    let req = make_request(date(2025, 10, 6), -1, ServiceMethod::Electronic);
    assert!(compute_deadline(&req).is_err());
}
```

**Step 5: Add module to tests/lib.rs**

```rust
mod deadline_engine_tests;
```

**Step 6: Run tests**

```bash
cargo test -p tests deadline_engine -- --test-threads=1
```

Expected: All tests pass.

**Step 7: Commit**

```bash
git add crates/server/src/compliance/ crates/server/src/lib.rs crates/tests/src/deadline_engine_tests.rs crates/tests/src/lib.rs
git commit -m "feat(compliance): port FRCP 6(a) deadline computation engine with tests"
```

---

## Phase 2: Rules Engine Core (Tasks 4–6)

### Task 4: Port Condition Evaluator

**Files:**
- Create: `crates/server/src/compliance/condition_evaluator.rs`
- Create: `crates/tests/src/condition_evaluator_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Port the condition evaluator**

`crates/server/src/compliance/condition_evaluator.rs` — ported from `spin-lexodus/src/adapters/rules_engine_impl.rs` (the `evaluate_condition` and `get_field_value` methods):

```rust
//! Recursive condition tree evaluator
//!
//! Ported from spin-lexodus. Evaluates RuleCondition trees against
//! a FilingContext by resolving field values from the context struct
//! and falling back to the metadata JSON object.

use shared_types::compliance::{FilingContext, RuleCondition};

/// Evaluate a condition tree against a filing context.
pub fn evaluate_condition(condition: &RuleCondition, context: &FilingContext) -> bool {
    match condition {
        RuleCondition::And { conditions } => {
            conditions.iter().all(|c| evaluate_condition(c, context))
        }
        RuleCondition::Or { conditions } => {
            conditions.iter().any(|c| evaluate_condition(c, context))
        }
        RuleCondition::Not { condition } => !evaluate_condition(condition, context),
        RuleCondition::FieldEquals { field, value } => {
            get_field_value(field, context).map_or(false, |v| v == *value)
        }
        RuleCondition::FieldContains { field, value } => {
            get_field_value(field, context).map_or(false, |v| v.contains(value.as_str()))
        }
        RuleCondition::FieldExists { field } => field_exists(field, context),
        RuleCondition::FieldGreaterThan { field, value } => {
            get_field_value(field, context).map_or(false, |v| {
                match (v.parse::<f64>(), value.parse::<f64>()) {
                    (Ok(field_num), Ok(threshold)) => field_num > threshold,
                    _ => v.as_str() > value.as_str(),
                }
            })
        }
        RuleCondition::FieldLessThan { field, value } => {
            get_field_value(field, context).map_or(false, |v| {
                match (v.parse::<f64>(), value.parse::<f64>()) {
                    (Ok(field_num), Ok(threshold)) => field_num < threshold,
                    _ => v.as_str() < value.as_str(),
                }
            })
        }
        RuleCondition::Always => true,
    }
}

/// Resolve a field value from the filing context.
/// Checks direct struct fields first, falls back to metadata JSON.
fn get_field_value(field: &str, context: &FilingContext) -> Option<String> {
    match field {
        "case_type" => Some(context.case_type.clone()),
        "document_type" => Some(context.document_type.clone()),
        "filer_role" => Some(context.filer_role.clone()),
        "jurisdiction_id" => Some(context.jurisdiction_id.clone()),
        "division" => context.division.clone(),
        "assigned_judge" => context.assigned_judge.clone(),
        _ => context.metadata.get(field).and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            _ => Some(v.to_string()),
        }),
    }
}

/// Check whether a field exists in the filing context.
fn field_exists(field: &str, context: &FilingContext) -> bool {
    match field {
        "case_type" | "document_type" | "filer_role" | "jurisdiction_id" => true,
        "division" => context.division.is_some(),
        "assigned_judge" => context.assigned_judge.is_some(),
        "service_method" => context.service_method.is_some(),
        _ => context
            .metadata
            .get(field)
            .map_or(false, |v| !v.is_null()),
    }
}
```

**Step 2: Write tests** (port from spin-lexodus, 30+ condition tests)

`crates/tests/src/condition_evaluator_tests.rs`:

```rust
//! Condition evaluator tests — ported from spin-lexodus

use server::compliance::condition_evaluator::evaluate_condition;
use shared_types::compliance::{FilingContext, RuleCondition, ServiceMethod};
use serde_json::json;

fn ctx(case_type: &str, doc_type: &str) -> FilingContext {
    FilingContext {
        case_type: case_type.to_string(),
        document_type: doc_type.to_string(),
        filer_role: "attorney".to_string(),
        jurisdiction_id: "district9".to_string(),
        division: None,
        assigned_judge: None,
        service_method: None,
        metadata: json!({}),
    }
}

fn ctx_with_meta(case_type: &str, doc_type: &str, meta: serde_json::Value) -> FilingContext {
    FilingContext {
        metadata: meta,
        ..ctx(case_type, doc_type)
    }
}

#[test]
fn field_equals_match() {
    let cond = RuleCondition::FieldEquals {
        field: "case_type".into(),
        value: "civil".into(),
    };
    assert!(evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn field_equals_no_match() {
    let cond = RuleCondition::FieldEquals {
        field: "case_type".into(),
        value: "civil".into(),
    };
    assert!(!evaluate_condition(&cond, &ctx("criminal", "complaint")));
}

#[test]
fn field_contains_match() {
    let cond = RuleCondition::FieldContains {
        field: "document_type".into(),
        value: "motion".into(),
    };
    assert!(evaluate_condition(&cond, &ctx("civil", "motion_to_dismiss")));
}

#[test]
fn field_exists_direct() {
    let cond = RuleCondition::FieldExists { field: "case_type".into() };
    assert!(evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn field_exists_missing() {
    let cond = RuleCondition::FieldExists { field: "nonexistent".into() };
    assert!(!evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn field_exists_metadata() {
    let cond = RuleCondition::FieldExists { field: "party_count".into() };
    assert!(evaluate_condition(&cond, &ctx_with_meta("civil", "c", json!({"party_count": 3}))));
}

#[test]
fn field_greater_than_numeric() {
    let cond = RuleCondition::FieldGreaterThan {
        field: "page_count".into(),
        value: "20".into(),
    };
    assert!(evaluate_condition(&cond, &ctx_with_meta("civil", "c", json!({"page_count": "25"}))));
    assert!(!evaluate_condition(&cond, &ctx_with_meta("civil", "c", json!({"page_count": "10"}))));
}

#[test]
fn field_less_than_numeric() {
    let cond = RuleCondition::FieldLessThan {
        field: "page_count".into(),
        value: "20".into(),
    };
    assert!(evaluate_condition(&cond, &ctx_with_meta("civil", "c", json!({"page_count": "5"}))));
}

#[test]
fn and_all_true() {
    let cond = RuleCondition::And {
        conditions: vec![
            RuleCondition::FieldEquals { field: "case_type".into(), value: "civil".into() },
            RuleCondition::FieldEquals { field: "document_type".into(), value: "complaint".into() },
        ],
    };
    assert!(evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn and_one_false() {
    let cond = RuleCondition::And {
        conditions: vec![
            RuleCondition::FieldEquals { field: "case_type".into(), value: "civil".into() },
            RuleCondition::FieldEquals { field: "document_type".into(), value: "motion".into() },
        ],
    };
    assert!(!evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn or_one_true() {
    let cond = RuleCondition::Or {
        conditions: vec![
            RuleCondition::FieldEquals { field: "case_type".into(), value: "criminal".into() },
            RuleCondition::FieldEquals { field: "case_type".into(), value: "civil".into() },
        ],
    };
    assert!(evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn or_all_false() {
    let cond = RuleCondition::Or {
        conditions: vec![
            RuleCondition::FieldEquals { field: "case_type".into(), value: "criminal".into() },
            RuleCondition::FieldEquals { field: "case_type".into(), value: "bankruptcy".into() },
        ],
    };
    assert!(!evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn not_negates() {
    let cond = RuleCondition::Not {
        condition: Box::new(RuleCondition::FieldEquals {
            field: "case_type".into(),
            value: "criminal".into(),
        }),
    };
    assert!(evaluate_condition(&cond, &ctx("civil", "complaint")));
}

#[test]
fn always_returns_true() {
    assert!(evaluate_condition(&RuleCondition::Always, &ctx("civil", "c")));
}

#[test]
fn nested_compound() {
    // And(Or(civil, criminal), Not(doc_type == "brief"))
    let cond = RuleCondition::And {
        conditions: vec![
            RuleCondition::Or {
                conditions: vec![
                    RuleCondition::FieldEquals { field: "case_type".into(), value: "civil".into() },
                    RuleCondition::FieldEquals { field: "case_type".into(), value: "criminal".into() },
                ],
            },
            RuleCondition::Not {
                condition: Box::new(RuleCondition::FieldEquals {
                    field: "document_type".into(),
                    value: "brief".into(),
                }),
            },
        ],
    };
    assert!(evaluate_condition(&cond, &ctx("civil", "motion")));
    assert!(!evaluate_condition(&cond, &ctx("civil", "brief")));
}

#[test]
fn metadata_boolean_as_string() {
    let cond = RuleCondition::FieldEquals {
        field: "pro_se".into(),
        value: "true".into(),
    };
    assert!(evaluate_condition(&cond, &ctx_with_meta("civil", "c", json!({"pro_se": true}))));
}
```

**Step 3: Run tests**

```bash
cargo test -p tests condition_evaluator -- --test-threads=1
```

**Step 4: Commit**

```bash
git add crates/server/src/compliance/condition_evaluator.rs crates/tests/src/condition_evaluator_tests.rs crates/tests/src/lib.rs
git commit -m "feat(compliance): port recursive condition evaluator with 17 tests"
```

---

### Task 5: Port Rules Engine (5-Stage Pipeline)

**Files:**
- Create: `crates/server/src/compliance/engine.rs`
- Create: `crates/tests/src/rules_engine_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Port the engine**

`crates/server/src/compliance/engine.rs` — ported from `spin-lexodus/src/adapters/rules_engine_impl.rs`:

```rust
//! Rules evaluation engine — 5-stage pipeline
//!
//! Ported from spin-lexodus. Stateless engine that:
//! 1. Selects rules by jurisdiction + trigger + in-effect status
//! 2. Sorts by priority weight (StandingOrder > Local > Admin > Federal > Statutory)
//! 3. Evaluates recursive condition trees against filing context
//! 4. Processes matched rule actions into compliance report
//! 5. Returns ComplianceReport (blocked, warnings, deadlines, fees)

use chrono::Utc;
use shared_types::compliance::*;
use shared_types::rule::Rule;
use super::condition_evaluator::evaluate_condition;

/// Select applicable rules for a given jurisdiction and trigger event.
/// Filters by: in-effect status, jurisdiction match (or global), trigger match.
pub fn select_rules(jurisdiction: &str, trigger: &TriggerEvent, all_rules: &[Rule]) -> Vec<Rule> {
    let jurisdiction_lower = jurisdiction.to_lowercase();
    let trigger_str = serde_json::to_value(trigger)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    all_rules
        .iter()
        .filter(|rule| {
            // Must be active
            if rule.status != "Active" {
                return false;
            }

            // Check effective/expiration dates
            let now = Utc::now();
            if let Some(eff) = rule.effective_date {
                if now < eff { return false; }
            }
            if let Some(exp) = rule.expiration_date {
                if now > exp { return false; }
            }

            // Jurisdiction: matches if rule has no jurisdiction (global) or matches
            let jurisdiction_match = rule.jurisdiction.as_ref().map_or(true, |j| {
                j.to_lowercase() == jurisdiction_lower
            });
            if !jurisdiction_match {
                return false;
            }

            // Trigger: check if rule's triggers array contains this trigger
            if let Some(triggers) = rule.triggers.as_array() {
                triggers.iter().any(|t| {
                    t.as_str().map_or(false, |s| s == trigger_str)
                })
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

/// Sort rules by priority weight (highest first). Stable sort preserves order within same priority.
pub fn resolve_priority(mut rules: Vec<Rule>) -> Vec<Rule> {
    rules.sort_by(|a, b| {
        let wa = RulePriority::from_db_priority(a.priority).weight();
        let wb = RulePriority::from_db_priority(b.priority).weight();
        wb.cmp(&wa)
    });
    rules
}

/// Evaluate a set of rules against a filing context.
/// Returns a ComplianceReport with all results, blocks, warnings, deadlines, and fees.
pub fn evaluate(context: &FilingContext, rules: &[Rule]) -> ComplianceReport {
    let mut report = ComplianceReport::default();
    let today = Utc::now().date_naive();

    for rule in rules {
        // Parse conditions from JSONB
        let conditions: Vec<RuleCondition> = parse_conditions(&rule.conditions);

        // Evaluate: all conditions must match (AND semantics at top level)
        let all_match = if conditions.is_empty() {
            true // No conditions = always matches
        } else {
            conditions.iter().all(|c| evaluate_condition(c, context))
        };

        if all_match {
            // Parse and process actions
            let actions: Vec<RuleAction> = parse_actions(&rule.actions);
            process_actions(rule, &actions, &mut report, today);
        } else {
            report.results.push(RuleResult {
                rule_id: rule.id,
                rule_name: rule.name.clone(),
                matched: false,
                action_taken: "none".to_string(),
                message: "Conditions not met".to_string(),
            });
        }
    }

    report
}

/// Parse conditions from JSONB. Supports both:
/// - New format: array of RuleCondition objects `[{"type": "field_equals", ...}]`
/// - Legacy format: flat object `{"trigger": "case_filed", "case_type": "civil"}`
fn parse_conditions(value: &serde_json::Value) -> Vec<RuleCondition> {
    // Try array of typed conditions first
    if let Ok(conditions) = serde_json::from_value::<Vec<RuleCondition>>(value.clone()) {
        return conditions;
    }

    // Try single condition object
    if let Ok(condition) = serde_json::from_value::<RuleCondition>(value.clone()) {
        return vec![condition];
    }

    // Legacy format: convert flat object to FieldEquals conditions
    if let Some(obj) = value.as_object() {
        let mut conditions = Vec::new();
        for (key, val) in obj {
            if key == "trigger" {
                continue; // Triggers are now in separate column
            }
            if let Some(v) = val.as_str() {
                conditions.push(RuleCondition::FieldEquals {
                    field: key.clone(),
                    value: v.to_string(),
                });
            }
        }
        return conditions;
    }

    Vec::new()
}

/// Parse actions from JSONB. Supports both:
/// - New format: array of RuleAction objects `[{"type": "generate_deadline", ...}]`
/// - Legacy format: object with action keys `{"create_deadline": {"days": 90, ...}}`
fn parse_actions(value: &serde_json::Value) -> Vec<RuleAction> {
    // Try array of typed actions
    if let Ok(actions) = serde_json::from_value::<Vec<RuleAction>>(value.clone()) {
        return actions;
    }

    // Try single action object
    if let Ok(action) = serde_json::from_value::<RuleAction>(value.clone()) {
        return vec![action];
    }

    // Legacy format: convert known action keys
    if let Some(obj) = value.as_object() {
        let mut actions = Vec::new();
        if let Some(dl) = obj.get("create_deadline") {
            if let Some(days) = dl.get("days").and_then(|d| d.as_i64()) {
                let title = dl.get("title").and_then(|t| t.as_str()).unwrap_or("Deadline");
                actions.push(RuleAction::GenerateDeadline {
                    description: title.to_string(),
                    days_from_trigger: days as i32,
                });
            }
        }
        return actions;
    }

    Vec::new()
}

/// Process matched rule actions into the compliance report.
fn process_actions(
    rule: &Rule,
    actions: &[RuleAction],
    report: &mut ComplianceReport,
    today: chrono::NaiveDate,
) {
    for action in actions {
        match action {
            RuleAction::BlockFiling { reason } => {
                report.blocked = true;
                report.block_reasons.push(format!("[{}] {}", rule.name, reason));
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "block_filing".to_string(),
                    message: reason.clone(),
                });
            }
            RuleAction::FlagForReview { reason } => {
                report.warnings.push(format!("[{}] {}", rule.name, reason));
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "flag_for_review".to_string(),
                    message: reason.clone(),
                });
            }
            RuleAction::GenerateDeadline { description, days_from_trigger } => {
                let due_date = today + chrono::Duration::days(*days_from_trigger as i64);
                report.deadlines.push(DeadlineResult {
                    due_date,
                    description: description.clone(),
                    rule_citation: rule.citation.clone().unwrap_or_default(),
                    computation_notes: format!(
                        "Generated by rule '{}': {} days from trigger",
                        rule.name, days_from_trigger
                    ),
                    is_short_period: *days_from_trigger <= 14,
                });
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "generate_deadline".to_string(),
                    message: format!("{} (due {})", description, due_date),
                });
            }
            RuleAction::RequireRedaction { fields } => {
                let field_list = fields.join(", ");
                report.warnings.push(format!("[{}] Redaction required for: {}", rule.name, field_list));
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "require_redaction".to_string(),
                    message: format!("Redaction required for: {}", field_list),
                });
            }
            RuleAction::SendNotification { recipient, message } => {
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "send_notification".to_string(),
                    message: format!("Notify {}: {}", recipient, message),
                });
            }
            RuleAction::RequireFee { amount_cents, description } => {
                report.fees.push(FeeRequirement {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    amount_cents: *amount_cents,
                    description: description.clone(),
                });
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "require_fee".to_string(),
                    message: format!("{}: ${:.2}", description, *amount_cents as f64 / 100.0),
                });
            }
            RuleAction::LogCompliance { message } => {
                report.results.push(RuleResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    matched: true,
                    action_taken: "log_compliance".to_string(),
                    message: message.clone(),
                });
            }
        }
    }
}
```

**Step 2: Write integration tests** that create rules in the DB and evaluate them

`crates/tests/src/rules_engine_tests.rs`:

```rust
//! Rules engine pipeline tests
//! Tests the full select → prioritize → evaluate → report pipeline

use shared_types::compliance::*;
use shared_types::rule::Rule;
use server::compliance::engine;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

fn make_rule(name: &str, priority: i32, triggers: &[&str], conditions: serde_json::Value, actions: serde_json::Value) -> Rule {
    let now = Utc::now();
    Rule {
        id: Uuid::new_v4(),
        court_id: "district9".to_string(),
        name: name.to_string(),
        description: Some(format!("Test rule: {}", name)),
        source: "Federal Rules of Civil Procedure".to_string(),
        category: "Deadline".to_string(),
        priority,
        status: "Active".to_string(),
        jurisdiction: Some("district9".to_string()),
        citation: Some("Test Citation".to_string()),
        effective_date: None,
        expiration_date: None,
        supersedes_rule_id: None,
        conditions,
        actions,
        triggers: json!(triggers),
        created_at: now,
        updated_at: now,
    }
}

fn ctx(case_type: &str, doc_type: &str) -> FilingContext {
    FilingContext {
        case_type: case_type.to_string(),
        document_type: doc_type.to_string(),
        filer_role: "attorney".to_string(),
        jurisdiction_id: "district9".to_string(),
        division: None,
        assigned_judge: None,
        service_method: None,
        metadata: json!({}),
    }
}

#[test]
fn select_rules_jurisdiction_match() {
    let rule = make_rule("test", 20, &["case_filed"], json!({}), json!({}));
    let selected = engine::select_rules("district9", &TriggerEvent::CaseFiled, &[rule]);
    assert_eq!(selected.len(), 1);
}

#[test]
fn select_rules_jurisdiction_mismatch() {
    let rule = make_rule("test", 20, &["case_filed"], json!({}), json!({}));
    let selected = engine::select_rules("district12", &TriggerEvent::CaseFiled, &[rule]);
    assert_eq!(selected.len(), 0);
}

#[test]
fn select_rules_trigger_mismatch() {
    let rule = make_rule("test", 20, &["case_filed"], json!({}), json!({}));
    let selected = engine::select_rules("district9", &TriggerEvent::MotionFiled, &[rule]);
    assert_eq!(selected.len(), 0);
}

#[test]
fn select_rules_inactive_filtered() {
    let mut rule = make_rule("inactive", 20, &["case_filed"], json!({}), json!({}));
    rule.status = "Inactive".to_string();
    let selected = engine::select_rules("district9", &TriggerEvent::CaseFiled, &[rule]);
    assert_eq!(selected.len(), 0);
}

#[test]
fn resolve_priority_orders_by_weight() {
    let standing = make_rule("standing", 50, &["case_filed"], json!({}), json!({}));
    let local = make_rule("local", 40, &["case_filed"], json!({}), json!({}));
    let federal = make_rule("federal", 20, &["case_filed"], json!({}), json!({}));
    let statutory = make_rule("statutory", 10, &["case_filed"], json!({}), json!({}));

    let sorted = engine::resolve_priority(vec![statutory, federal, local, standing]);
    assert_eq!(sorted[0].name, "standing");
    assert_eq!(sorted[1].name, "local");
    assert_eq!(sorted[2].name, "federal");
    assert_eq!(sorted[3].name, "statutory");
}

#[test]
fn evaluate_matching_rule_generates_deadline() {
    let rule = make_rule(
        "FRCP 4(m)",
        20,
        &["case_filed"],
        json!([{"type": "field_equals", "field": "case_type", "value": "civil"}]),
        json!([{"type": "generate_deadline", "description": "Service of process", "days_from_trigger": 90}]),
    );

    let report = engine::evaluate(&ctx("civil", "complaint"), &[rule]);
    assert_eq!(report.deadlines.len(), 1);
    assert_eq!(report.deadlines[0].description, "Service of process");
    assert!(!report.deadlines[0].is_short_period);
    assert!(report.results[0].matched);
}

#[test]
fn evaluate_non_matching_conditions() {
    let rule = make_rule(
        "civil-only",
        20,
        &["case_filed"],
        json!([{"type": "field_equals", "field": "case_type", "value": "civil"}]),
        json!([{"type": "block_filing", "reason": "Should not fire"}]),
    );

    let report = engine::evaluate(&ctx("criminal", "indictment"), &[rule]);
    assert!(!report.blocked);
    assert!(!report.results[0].matched);
}

#[test]
fn evaluate_block_filing() {
    let rule = make_rule(
        "block-rule",
        20,
        &["document_filed"],
        json!([{"type": "always"}]),
        json!([{"type": "block_filing", "reason": "Missing cover sheet"}]),
    );

    let report = engine::evaluate(&ctx("civil", "complaint"), &[rule]);
    assert!(report.blocked);
    assert!(report.block_reasons[0].contains("Missing cover sheet"));
}

#[test]
fn evaluate_require_fee() {
    let rule = make_rule(
        "filing-fee",
        40,
        &["case_filed"],
        json!([{"type": "field_equals", "field": "case_type", "value": "civil"}]),
        json!([{"type": "require_fee", "amount_cents": 40500, "description": "Civil filing fee"}]),
    );

    let report = engine::evaluate(&ctx("civil", "complaint"), &[rule]);
    assert_eq!(report.fees.len(), 1);
    assert_eq!(report.fees[0].amount_cents, 40500);
}

#[test]
fn evaluate_legacy_condition_format() {
    // Tests backward compatibility with existing seeded rules
    let rule = make_rule(
        "legacy",
        20,
        &["case_filed"],
        json!({"case_type": "civil"}),
        json!({"create_deadline": {"days": 90, "title": "Service of process"}}),
    );

    let report = engine::evaluate(&ctx("civil", "complaint"), &[rule]);
    assert_eq!(report.deadlines.len(), 1);
    assert_eq!(report.deadlines[0].description, "Service of process");
}

#[test]
fn evaluate_empty_rules() {
    let report = engine::evaluate(&ctx("civil", "complaint"), &[]);
    assert!(report.results.is_empty());
    assert!(!report.blocked);
}

#[test]
fn evaluate_multiple_rules_mixed_match() {
    let matching = make_rule(
        "matches",
        20,
        &["case_filed"],
        json!([{"type": "field_equals", "field": "case_type", "value": "civil"}]),
        json!([{"type": "flag_for_review", "reason": "Matched"}]),
    );
    let not_matching = make_rule(
        "no-match",
        20,
        &["case_filed"],
        json!([{"type": "field_equals", "field": "case_type", "value": "criminal"}]),
        json!([{"type": "block_filing", "reason": "Should not fire"}]),
    );

    let report = engine::evaluate(&ctx("civil", "complaint"), &[matching, not_matching]);
    assert_eq!(report.results.len(), 2);
    assert!(report.results[0].matched);
    assert!(!report.results[1].matched);
    assert!(!report.blocked);
    assert_eq!(report.warnings.len(), 1);
}
```

**Step 3: Run tests**

```bash
cargo test -p tests rules_engine -- --test-threads=1
```

**Step 4: Commit**

```bash
git add crates/server/src/compliance/engine.rs crates/tests/src/rules_engine_tests.rs crates/tests/src/lib.rs
git commit -m "feat(compliance): port 5-stage rules engine pipeline with 12 tests"
```

---

### Task 6: Case Event Repo + Evaluate Endpoint Enhancement

**Files:**
- Create: `crates/server/src/repo/case_event.rs`
- Modify: `crates/server/src/repo/mod.rs`
- Modify: `crates/server/src/rest/rule.rs` (enhance evaluate endpoint)

**Step 1: Create case_event repo**

`crates/server/src/repo/case_event.rs`:

```rust
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert(
    pool: &PgPool,
    court_id: &str,
    case_id: Uuid,
    case_type: &str,
    trigger_event: &str,
    actor_id: Option<Uuid>,
    payload: &serde_json::Value,
    compliance_report: Option<&serde_json::Value>,
) -> Result<Uuid, sqlx::Error> {
    let row = sqlx::query_scalar!(
        r#"INSERT INTO case_events (court_id, case_id, case_type, trigger_event, actor_id, payload, compliance_report)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id"#,
        court_id,
        case_id,
        case_type,
        trigger_event,
        actor_id,
        payload,
        compliance_report,
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_by_case(
    pool: &PgPool,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows = sqlx::query_scalar!(
        r#"SELECT row_to_json(e) as "event!: serde_json::Value"
           FROM case_events e
           WHERE court_id = $1 AND case_id = $2
           ORDER BY created_at DESC
           LIMIT 100"#,
        court_id,
        case_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

**Step 2: Register module**

Add `pub mod case_event;` to `crates/server/src/repo/mod.rs`.

**Step 3: Enhance the evaluate endpoint**

In `crates/server/src/rest/rule.rs`, update `evaluate_rules` to use the ported engine instead of simple condition matching. The handler should:
1. Load rules from DB matching the trigger
2. Run through the 5-stage pipeline
3. Return the ComplianceReport

**Step 4: Commit**

```bash
git add crates/server/src/repo/case_event.rs crates/server/src/repo/mod.rs crates/server/src/rest/rule.rs
git commit -m "feat(compliance): case_event repo and enhanced evaluate endpoint"
```

---

## Phase 3: Endpoint Wiring (Tasks 7–9)

### Task 7: Wire Engine into Case Creation

**Files:**
- Modify: `crates/server/src/rest/case.rs`
- Create: `crates/tests/src/compliance_wiring_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Add compliance evaluation to `create_case`**

In `crates/server/src/rest/case.rs`, after the case is created in the DB:

1. Build a `FilingContext` from the case data
2. Load rules from DB: `SELECT * FROM rules WHERE court_id = $1 AND status = 'Active'`
3. Call `engine::select_rules(court_id, &TriggerEvent::CaseFiled, &rules)`
4. Call `engine::resolve_priority(selected)`
5. Call `engine::evaluate(&context, &sorted)`
6. If `report.blocked` and no clerk override → return 422 with block_reasons
7. For each `report.deadlines` → insert into `deadlines` table
8. Log `case_event` with trigger=`case_filed` and compliance_report
9. Return case response (with any warnings)

**Step 2: Write tests**

Test that creating a case with `case_type: "civil"` triggers FRCP 4(m) and creates a 90-day service deadline. Test that creating a case with `case_type: "criminal"` does NOT create civil deadlines.

**Step 3: Commit**

```bash
git commit -m "feat(compliance): wire rules engine into case creation"
```

---

### Task 8: Wire Engine into Docket Entry Creation

**Files:**
- Modify: `crates/server/src/rest/docket.rs`

Similar to Task 7 but for docket entries. Map docket entry types to trigger events:
- `complaint` → `ComplaintFiled`
- `motion` → `MotionFiled`
- `answer` → `AnswerFiled`
- `order` → `OrderIssued`
- `judgment` → `JudgmentEntered`
- etc.

Build FilingContext, select+evaluate rules, create deadlines, log case_event.

**Commit:**

```bash
git commit -m "feat(compliance): wire rules engine into docket entry creation"
```

---

### Task 9: Wire Engine into Status Changes

**Files:**
- Modify: `crates/server/src/rest/case.rs` (update_status handler)

When case status changes:
1. Build FilingContext with metadata `{"old_status": "filed", "new_status": "arraigned"}`
2. Select rules with `TriggerEvent::StatusChanged`
3. Evaluate — rules can block invalid transitions
4. Log case_event

**Commit:**

```bash
git commit -m "feat(compliance): wire rules engine into status changes"
```

---

## Phase 4: Comprehensive Rule Seeds (Tasks 10–11)

### Task 10: Seed ARWD Rules (Port from TOML)

**Files:**
- Create: `migrations/20260301000095_seed_comprehensive_rules.sql`
- Create: `migrations/20260301000095_seed_comprehensive_rules.down.sql`

Port the 112 ARWD rules from `spin-lexodus/config/jurisdictions/arwd/rules.toml` to SQL INSERT statements. Use the new tagged-enum format for conditions and actions. Map to both `district9` and `district12`.

Priority tiers:
1. **Tier 1 (must-have)**: All FRCP deadline rules (30 rules × 2 districts = 60 inserts)
2. **Tier 2 (important)**: Local rules for filing/service/discovery (40 rules × 2 = 80 inserts)
3. **Tier 3 (complete)**: Administrative procedures (27 rules × 2 = 54 inserts)

Also update the existing 15 seeded FRCP rules to use the new conditions/actions format and populate the `triggers` column.

**Commit:**

```bash
git commit -m "feat(compliance): seed 112 ARWD rules per district (ported from spin-lexodus)"
```

---

### Task 11: Seed Fee Schedule

**Files:**
- Create: `migrations/20260301000096_seed_fee_schedule.sql`
- Create: `migrations/20260301000096_seed_fee_schedule.down.sql`

Port the fee schedule from `spin-lexodus/config/jurisdictions/arwd/fees.toml` to SQL. 15 fee entries per district (civil filing, appeals, habeas, pro hac vice, search, certification, etc.).

**Commit:**

```bash
git commit -m "feat(compliance): seed ARWD fee schedule (ported from spin-lexodus)"
```

---

## Phase 5: Admin + Polish (Tasks 12–14)

### Task 12: Fee Schedule Admin API

**Files:**
- Create: `crates/server/src/repo/fee_schedule.rs`
- Create: `crates/server/src/rest/fee_schedule.rs`
- Modify: `crates/server/src/repo/mod.rs`
- Modify: `crates/server/src/rest/mod.rs`
- Modify: `crates/server/src/api.rs`

CRUD endpoints for fee schedule:
- `GET /api/fee-schedule` — list active fees for court
- `GET /api/fee-schedule/:id` — get fee detail
- `POST /api/fee-schedule` — create fee entry
- `PATCH /api/fee-schedule/:id` — update fee
- `DELETE /api/fee-schedule/:id` — soft-delete (set active=false)

**Commit:**

```bash
git commit -m "feat(compliance): fee schedule admin API (CRUD)"
```

---

### Task 13: Case Events API + Audit Endpoint

**Files:**
- Modify: `crates/server/src/rest/mod.rs`
- Modify: `crates/server/src/api.rs`

Add endpoints:
- `GET /api/cases/:id/events` — list case events (audit trail)
- `GET /api/cases/:id/compliance` — latest compliance status

**Commit:**

```bash
git commit -m "feat(compliance): case events audit trail API"
```

---

### Task 14: Full Integration Test Suite

**Files:**
- Create: `crates/tests/src/compliance_integration_tests.rs`
- Modify: `crates/tests/src/lib.rs`

End-to-end tests:
1. Create case → verify FRCP 4(m) deadline auto-created
2. Create docket entry (answer) → verify FRCP 26(f) deadline created
3. Create docket entry (motion) → verify response deadline created
4. Create docket entry (complaint) with blocking rule → verify 422 response
5. Evaluate rules via API → verify ComplianceReport
6. List case events → verify audit trail
7. Fee schedule CRUD → verify responses
8. Legacy rule format backward compatibility

Run all:
```bash
cargo test -p tests -- --test-threads=1
```

**Commit:**

```bash
git commit -m "test(compliance): full integration test suite (8 scenarios)"
```

---

## Migration Numbering

| Number | Content |
|--------|---------|
| 000092 | Add `triggers` column to rules, expand CHECK constraints |
| 000093 | Create `case_events` table |
| 000094 | Create `fee_schedule` table |
| 000095 | Seed comprehensive rules (112 per district, ported from ARWD TOML) |
| 000096 | Seed fee schedule (15 entries per district) |

**Note:** The CM/ECF gaps implementation plan (MFA, filing fees, public portal) should use migration numbers 000097+ to avoid conflicts.

---

## Key Differences from v1 Plan

| Aspect | v1 (from-scratch) | v2 (ported) |
|--------|-------------------|-------------|
| Rule evaluation | Hardcoded Rust match per rule | Data-driven 5-stage pipeline |
| Conditions | Flat if/else per rule type | Recursive `And/Or/Not/FieldEquals` tree |
| Actions | `ComplianceEffect` enum | 7 typed actions from spin-lexodus |
| Triggers | ~6 event types | 48 event types covering full lifecycle |
| Rule storage | Federal rules in Rust, local in DB | ALL rules in DB (JSONB conditions/actions) |
| Priority | Not addressed | 5-level weighted priority system |
| Rule count | ~15 FRCP as functions | 112 rules per district (seeded) |
| Condition format | N/A | Backward-compatible with existing seeds |
| Tests | TDD per task | Ported 80+ tests from spin-lexodus |
| Fee schedule | DB table only | Seeded from verified ARWD data |

## Verification

After all tasks complete:
1. `cargo check -p server -p shared-types -p app`
2. `cargo test -p tests -- --test-threads=1` (all tests pass)
3. `sqlx migrate run` on test DB
4. Create a civil case → verify service deadline auto-created
5. File a motion → verify response deadline auto-created
6. Check case events audit trail
