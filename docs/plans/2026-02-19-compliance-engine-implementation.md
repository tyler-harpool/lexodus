# Federal Rules Compliance Engine — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build an event-driven compliance engine that encodes FRCP, FRCrP, and Speedy Trial Act as compiled Rust logic — automatically creating deadlines, enforcing status transitions, blocking non-compliant actions, auto-assigning judges via weighted wheel, and managing fee schedules.

**Architecture:** Inline event hooks within Axum handlers. Every state-changing action evaluates federal rules (compiled Rust) + local rules (DB JSONB) in the same database transaction. Command pattern (`ComplianceEffect`) for deferred effects. Postgres `case_events` table for audit trail. No external message queue.

**Tech Stack:** Rust, Axum, sqlx, PostgreSQL, chrono (business day math)

**Design Doc:** `docs/plans/2026-02-19-compliance-engine-design.md`

---

## Phase 1: Foundation (Tasks 1-4)

### Task 1: Case Events Table

**Files:**
- Create: `migrations/20260301000092_create_case_events.sql`
- Create: `migrations/20260301000092_create_case_events.down.sql`
- Create: `crates/server/src/repo/case_event.rs`
- Modify: `crates/server/src/repo/mod.rs`
- Modify: `crates/tests/src/common.rs` (add `case_events` to TRUNCATE)

**Step 1: Create migration**

`migrations/20260301000092_create_case_events.sql`:

```sql
CREATE TABLE IF NOT EXISTS case_events (
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

Down: `DROP TABLE IF EXISTS case_events;`

**Step 2: Run migration**

Run: `sqlx migrate run`

**Step 3: Add case_events to test TRUNCATE**

In `crates/tests/src/common.rs`, add `case_events` to the TRUNCATE query (before CASCADE).

**Step 4: Create repo module**

Create `crates/server/src/repo/case_event.rs`:

```rust
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub async fn insert(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
    case_type: &str,
    event_type: &str,
    actor_id: Option<i64>,
    payload: &serde_json::Value,
) -> Result<Uuid, sqlx::Error> {
    let row = sqlx::query_scalar!(
        r#"INSERT INTO case_events (court_id, case_id, case_type, event_type, actor_id, payload)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id"#,
        court_id,
        case_id,
        case_type,
        event_type,
        actor_id,
        payload,
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<CaseEventRow>, sqlx::Error> {
    sqlx::query_as!(
        CaseEventRow,
        r#"SELECT id, court_id, case_id, case_type, event_type,
                  actor_id, payload, created_at
           FROM case_events
           WHERE court_id = $1 AND case_id = $2
           ORDER BY created_at"#,
        court_id,
        case_id,
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone)]
pub struct CaseEventRow {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub case_type: String,
    pub event_type: String,
    pub actor_id: Option<i64>,
    pub payload: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

Add `pub mod case_event;` to `crates/server/src/repo/mod.rs` (under `#[cfg(feature = "server")]`).

**Step 5: Verify compilation**

Run: `cargo check -p server`

**Step 6: Commit**

```bash
git add migrations/20260301000092* crates/server/src/repo/case_event.rs crates/server/src/repo/mod.rs crates/tests/src/common.rs
git commit -m "feat(compliance): case_events table and repo module"
```

---

### Task 2: Compliance Engine Core Types

**Files:**
- Create: `crates/server/src/compliance/mod.rs`
- Create: `crates/server/src/compliance/event.rs`
- Modify: `crates/server/src/lib.rs` (add `pub mod compliance;`)

**Step 1: Create event types module**

Create `crates/server/src/compliance/event.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A case lifecycle event that triggers compliance evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseEvent {
    pub event_type: String,
    pub case_id: Uuid,
    pub case_type: CaseType,
    pub court_id: String,
    pub actor_id: Option<i64>,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl CaseEvent {
    pub fn new(
        event_type: &str,
        case_id: Uuid,
        case_type: CaseType,
        court_id: &str,
        actor_id: Option<i64>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_type: event_type.to_string(),
            case_id,
            case_type,
            court_id: court_id.to_string(),
            actor_id,
            payload,
            timestamp: Utc::now(),
        }
    }

    /// Convenience: get a string field from payload.
    pub fn payload_str(&self, key: &str) -> Option<&str> {
        self.payload.get(key)?.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseType {
    Criminal,
    Civil,
}

impl CaseType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CaseType::Criminal => "criminal",
            CaseType::Civil => "civil",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "criminal" => Some(CaseType::Criminal),
            "civil" => Some(CaseType::Civil),
            _ => None,
        }
    }
}

/// Current state of a case, loaded once per compliance evaluation.
#[derive(Debug, Clone)]
pub struct CaseContext {
    pub case_id: Uuid,
    pub case_type: CaseType,
    pub case_number: String,
    pub status: String,
    pub date_filed: DateTime<Utc>,
    pub assigned_judge_id: Option<Uuid>,
    pub trial_date: Option<DateTime<Utc>>,
    pub fee_status: String,
    pub is_sealed: bool,
}

/// An effect to be applied within the same DB transaction.
#[derive(Debug, Clone)]
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
}

/// A compliance violation that blocks the action.
#[derive(Debug, Clone, Serialize)]
pub struct ComplianceViolation {
    pub rule_citation: String,
    pub message: String,
    pub override_allowed: bool,
    pub override_role: String,
}

impl std::fmt::Display for ComplianceViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.rule_citation, self.message)
    }
}

/// A clerk or judge override for a compliance violation.
#[derive(Debug, Clone, Deserialize)]
pub struct ClerkOverride {
    pub user_id: i64,
    pub role: String,
    pub reason: String,
}

/// Result of evaluating a rule set.
#[derive(Debug, Default)]
pub struct RuleResult {
    pub effects: Vec<ComplianceEffect>,
    pub violations: Vec<ComplianceViolation>,
}

impl RuleResult {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: RuleResult) {
        self.effects.extend(other.effects);
        self.violations.extend(other.violations);
    }
}
```

**Step 2: Create compliance module**

Create `crates/server/src/compliance/mod.rs`:

```rust
pub mod event;

pub use event::*;
```

**Step 3: Register module**

In `crates/server/src/lib.rs`, add `pub mod compliance;`

**Step 4: Verify compilation**

Run: `cargo check -p server`

**Step 5: Commit**

```bash
git add crates/server/src/compliance/ crates/server/src/lib.rs
git commit -m "feat(compliance): core types — CaseEvent, ComplianceEffect, ComplianceViolation"
```

---

### Task 3: Business Day Calculator (FRCP 6(a))

**Files:**
- Create: `crates/server/src/compliance/calendar.rs`
- Modify: `crates/server/src/compliance/mod.rs`
- Create: `crates/tests/src/compliance_calendar_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Write tests**

Create `crates/tests/src/compliance_calendar_tests.rs`:

```rust
use chrono::{NaiveDate, TimeZone, Utc};
use server::compliance::calendar::*;

#[test]
fn is_federal_holiday_detects_new_years() {
    assert!(is_federal_holiday(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()));
}

#[test]
fn is_federal_holiday_detects_mlk_2026() {
    // MLK Day 2026 = third Monday of January = Jan 19
    assert!(is_federal_holiday(NaiveDate::from_ymd_opt(2026, 1, 19).unwrap()));
}

#[test]
fn is_federal_holiday_detects_juneteenth() {
    assert!(is_federal_holiday(NaiveDate::from_ymd_opt(2026, 6, 19).unwrap()));
}

#[test]
fn is_federal_holiday_normal_day_is_not() {
    assert!(!is_federal_holiday(NaiveDate::from_ymd_opt(2026, 3, 4).unwrap()));
}

#[test]
fn add_business_days_skips_weekends() {
    // Friday March 6 + 1 business day = Monday March 9
    let fri = Utc.with_ymd_and_hms(2026, 3, 6, 12, 0, 0).unwrap();
    let result = add_business_days(fri, 1);
    assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 9).unwrap());
}

#[test]
fn add_business_days_skips_holidays() {
    // Day before MLK Day (Fri Jan 16 2026) + 1 business day
    // Jan 17 = Sat, Jan 18 = Sun, Jan 19 = MLK (holiday), so next = Jan 20 (Tue)
    let fri = Utc.with_ymd_and_hms(2026, 1, 16, 12, 0, 0).unwrap();
    let result = add_business_days(fri, 1);
    assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 1, 20).unwrap());
}

#[test]
fn compute_deadline_under_11_uses_business_days() {
    // FRCP 6(a)(1)(B): periods < 11 days count business days only
    let start = Utc.with_ymd_and_hms(2026, 3, 2, 12, 0, 0).unwrap(); // Monday
    let result = compute_deadline(start, 7, false); // 7 calendar days
    // 7 days under 11 → count business days: Mon-Fri = 5, + 2 more = next Tue+Wed
    // March 2 (Mon) + 7 business days = March 11 (Wed)
    assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 11).unwrap());
}

#[test]
fn compute_deadline_11_plus_uses_calendar_days() {
    // FRCP 6(a)(1)(A): periods >= 11 days count calendar days
    let start = Utc.with_ymd_and_hms(2026, 3, 2, 12, 0, 0).unwrap(); // Monday
    let result = compute_deadline(start, 14, false);
    // 14 calendar days from March 2 = March 16 (Monday)
    assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 16).unwrap());
}

#[test]
fn compute_deadline_extends_past_weekend() {
    // FRCP 6(a)(1)(C): if deadline falls on weekend, extend to Monday
    let start = Utc.with_ymd_and_hms(2026, 3, 2, 12, 0, 0).unwrap(); // Monday
    let result = compute_deadline(start, 12, false);
    // 12 calendar days from March 2 = March 14 (Saturday) → extends to March 16 (Monday)
    assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 16).unwrap());
}

#[test]
fn compute_deadline_adds_3_for_mail_service() {
    // FRCP 6(d): add 3 days if served by mail
    let start = Utc.with_ymd_and_hms(2026, 3, 2, 12, 0, 0).unwrap();
    let result = compute_deadline(start, 14, true); // mail service
    // 14 calendar days = March 16 (Mon) + 3 = March 19 (Thu)
    assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 19).unwrap());
}
```

Add `mod compliance_calendar_tests;` to `crates/tests/src/lib.rs`.

**Step 2: Run tests to verify they fail**

Run: `cargo test -p tests compliance_calendar -- --test-threads=1`
Expected: FAIL — module doesn't exist

**Step 3: Implement calendar module**

Create `crates/server/src/compliance/calendar.rs`:

```rust
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};

/// Check if a date is a US federal holiday.
/// Covers all 11 federal holidays per 5 U.S.C. § 6103.
pub fn is_federal_holiday(date: NaiveDate) -> bool {
    let year = date.year();
    let month = date.month();
    let day = date.day();

    // Fixed-date holidays (with weekend shifting)
    let fixed_holidays = [
        (1, 1),   // New Year's Day
        (6, 19),  // Juneteenth
        (7, 4),   // Independence Day
        (11, 11), // Veterans Day
        (12, 25), // Christmas Day
    ];

    for (m, d) in fixed_holidays {
        let holiday = NaiveDate::from_ymd_opt(year, m, d).unwrap();
        let observed = if holiday.weekday() == Weekday::Sat {
            holiday - Duration::days(1) // Friday before
        } else if holiday.weekday() == Weekday::Sun {
            holiday + Duration::days(1) // Monday after
        } else {
            holiday
        };
        if date == observed {
            return true;
        }
    }

    // Monday-anchored holidays (Nth Monday of month)
    match (month, date.weekday()) {
        // MLK Day: 3rd Monday of January
        (1, Weekday::Mon) if nth_weekday_of_month(date) == 3 => return true,
        // Presidents' Day: 3rd Monday of February
        (2, Weekday::Mon) if nth_weekday_of_month(date) == 3 => return true,
        // Memorial Day: last Monday of May
        (5, Weekday::Mon) if is_last_weekday_of_month(date) => return true,
        // Labor Day: 1st Monday of September
        (9, Weekday::Mon) if nth_weekday_of_month(date) == 1 => return true,
        // Columbus Day: 2nd Monday of October
        (10, Weekday::Mon) if nth_weekday_of_month(date) == 2 => return true,
        _ => {}
    }

    // Thanksgiving: 4th Thursday of November
    if month == 11 && date.weekday() == Weekday::Thu && nth_weekday_of_month(date) == 4 {
        return true;
    }

    false
}

fn nth_weekday_of_month(date: NaiveDate) -> u32 {
    (date.day() - 1) / 7 + 1
}

fn is_last_weekday_of_month(date: NaiveDate) -> bool {
    let next_week = date + Duration::days(7);
    next_week.month() != date.month()
}

/// Check if a date is a weekend.
pub fn is_weekend(date: NaiveDate) -> bool {
    matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
}

/// Check if a date is a non-business day (weekend or federal holiday).
pub fn is_non_business_day(date: NaiveDate) -> bool {
    is_weekend(date) || is_federal_holiday(date)
}

/// Add N business days to a date (skipping weekends and federal holidays).
pub fn add_business_days(from: DateTime<Utc>, days: i64) -> DateTime<Utc> {
    let mut current = from;
    let mut remaining = days;
    while remaining > 0 {
        current = current + Duration::days(1);
        if !is_non_business_day(current.date_naive()) {
            remaining -= 1;
        }
    }
    current
}

/// Extend a date forward past weekends and holidays (FRCP 6(a)(1)(C)).
pub fn extend_past_non_business_days(date: DateTime<Utc>) -> DateTime<Utc> {
    let mut current = date;
    while is_non_business_day(current.date_naive()) {
        current = current + Duration::days(1);
    }
    current
}

/// Compute a deadline per FRCP 6(a).
///
/// - Periods < 11 days: count business days only (FRCP 6(a)(1)(B))
/// - Periods >= 11 days: count calendar days (FRCP 6(a)(1)(A))
/// - If deadline falls on weekend/holiday: extend to next business day (FRCP 6(a)(1)(C))
/// - If mail_service: add 3 calendar days (FRCP 6(d))
pub fn compute_deadline(from: DateTime<Utc>, days: i64, mail_service: bool) -> DateTime<Utc> {
    let target = if days < 11 {
        add_business_days(from, days)
    } else {
        let raw = from + Duration::days(days);
        extend_past_non_business_days(raw)
    };

    if mail_service {
        let with_mail = target + Duration::days(3);
        extend_past_non_business_days(with_mail)
    } else {
        target
    }
}
```

Add `pub mod calendar;` to `crates/server/src/compliance/mod.rs`.

**Step 4: Run tests**

Run: `cargo test -p tests compliance_calendar -- --test-threads=1`
Expected: All pass

**Step 5: Commit**

```bash
git add crates/server/src/compliance/calendar.rs crates/server/src/compliance/mod.rs crates/tests/src/compliance_calendar_tests.rs crates/tests/src/lib.rs
git commit -m "feat(compliance): FRCP 6(a) business day calculator with federal holidays"
```

---

### Task 4: Status Transition Validator

**Files:**
- Create: `crates/server/src/compliance/status.rs`
- Modify: `crates/server/src/compliance/mod.rs`
- Create: `crates/tests/src/compliance_status_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Write tests**

Create `crates/tests/src/compliance_status_tests.rs`:

```rust
use server::compliance::status::*;

#[test]
fn criminal_filed_to_arraigned_is_valid() {
    assert!(is_valid_criminal_transition("filed", "arraigned"));
}

#[test]
fn criminal_filed_to_in_trial_is_invalid() {
    assert!(!is_valid_criminal_transition("filed", "in_trial"));
}

#[test]
fn criminal_sentenced_to_on_appeal_is_valid() {
    assert!(is_valid_criminal_transition("sentenced", "on_appeal"));
}

#[test]
fn criminal_dismissed_to_on_appeal_is_valid() {
    assert!(is_valid_criminal_transition("dismissed", "on_appeal"));
}

#[test]
fn civil_filed_to_served_is_valid() {
    assert!(is_valid_civil_transition("filed", "served"));
}

#[test]
fn civil_filed_to_in_trial_is_invalid() {
    assert!(!is_valid_civil_transition("filed", "in_trial"));
}

#[test]
fn civil_judgment_to_on_appeal_is_valid() {
    assert!(is_valid_civil_transition("judgment", "on_appeal"));
}

#[test]
fn get_violation_message_includes_citation() {
    let v = transition_violation("criminal", "filed", "in_trial");
    assert!(v.is_some());
    let v = v.unwrap();
    assert!(v.rule_citation.contains("FRCrP"));
    assert!(v.message.contains("filed"));
    assert!(v.message.contains("in_trial"));
}
```

Add `mod compliance_status_tests;` to `crates/tests/src/lib.rs`.

**Step 2: Implement status transitions**

Create `crates/server/src/compliance/status.rs`:

```rust
use crate::compliance::event::ComplianceViolation;

/// Valid criminal case status transitions.
/// Key = current status, Value = list of valid next statuses.
fn criminal_transitions() -> &'static [(&'static str, &'static [&'static str])] {
    &[
        ("filed", &["arraigned", "dismissed"]),
        ("arraigned", &["discovery", "pretrial_motions", "plea_negotiations", "dismissed"]),
        ("discovery", &["pretrial_motions", "plea_negotiations", "dismissed"]),
        ("pretrial_motions", &["plea_negotiations", "trial_ready", "dismissed"]),
        ("plea_negotiations", &["trial_ready", "awaiting_sentencing", "dismissed"]),
        ("trial_ready", &["in_trial", "plea_negotiations", "dismissed"]),
        ("in_trial", &["awaiting_sentencing", "dismissed"]),
        ("awaiting_sentencing", &["sentenced", "dismissed"]),
        ("sentenced", &["on_appeal"]),
        ("dismissed", &["on_appeal"]),
        ("on_appeal", &["sentenced", "dismissed", "arraigned"]),
    ]
}

/// Valid civil case status transitions.
fn civil_transitions() -> &'static [(&'static str, &'static [&'static str])] {
    &[
        ("filed", &["served", "dismissed"]),
        ("served", &["answer_due", "dismissed"]),
        ("answer_due", &["at_issue", "dismissed"]),
        ("at_issue", &["discovery", "dismissed", "settled"]),
        ("discovery", &["pretrial", "dismissed", "settled"]),
        ("pretrial", &["trial_ready", "dismissed", "settled"]),
        ("trial_ready", &["in_trial", "dismissed", "settled"]),
        ("in_trial", &["judgment", "dismissed", "settled"]),
        ("judgment", &["on_appeal"]),
        ("dismissed", &["on_appeal"]),
        ("settled", &[]),
        ("on_appeal", &["judgment", "dismissed"]),
    ]
}

pub fn is_valid_criminal_transition(from: &str, to: &str) -> bool {
    criminal_transitions()
        .iter()
        .find(|(status, _)| *status == from)
        .map(|(_, valid)| valid.contains(&to))
        .unwrap_or(false)
}

pub fn is_valid_civil_transition(from: &str, to: &str) -> bool {
    civil_transitions()
        .iter()
        .find(|(status, _)| *status == from)
        .map(|(_, valid)| valid.contains(&to))
        .unwrap_or(false)
}

/// Check if a status transition is valid. Returns a ComplianceViolation if not.
pub fn transition_violation(
    case_type: &str,
    from: &str,
    to: &str,
) -> Option<ComplianceViolation> {
    let valid = match case_type {
        "criminal" => is_valid_criminal_transition(from, to),
        "civil" => is_valid_civil_transition(from, to),
        _ => false,
    };

    if valid {
        None
    } else {
        let citation = match case_type {
            "criminal" => "FRCrP 10, 11, 32",
            "civil" => "FRCP 12, 16, 56",
            _ => "Unknown",
        };
        Some(ComplianceViolation {
            rule_citation: citation.to_string(),
            message: format!(
                "Invalid status transition: '{}' → '{}'. Case must follow the prescribed procedural sequence.",
                from, to
            ),
            override_allowed: true,
            override_role: "judge".to_string(),
        })
    }
}
```

Add `pub mod status;` to `crates/server/src/compliance/mod.rs`.

**Step 3: Run tests**

Run: `cargo test -p tests compliance_status -- --test-threads=1`
Expected: All pass

**Step 4: Commit**

```bash
git add crates/server/src/compliance/status.rs crates/server/src/compliance/mod.rs crates/tests/src/compliance_status_tests.rs crates/tests/src/lib.rs
git commit -m "feat(compliance): status transition validator for criminal and civil cases"
```

---

## Phase 2: Federal Rules in Rust (Tasks 5-7)

### Task 5: FRCP Civil Rules

**Files:**
- Create: `crates/server/src/compliance/federal/mod.rs`
- Create: `crates/server/src/compliance/federal/frcp.rs`
- Modify: `crates/server/src/compliance/mod.rs`
- Create: `crates/tests/src/compliance_frcp_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Write tests**

Create `crates/tests/src/compliance_frcp_tests.rs`:

```rust
use chrono::{Duration, Utc};
use server::compliance::{CaseEvent, CaseType, CaseContext, RuleResult};
use server::compliance::federal::frcp::FrcpRules;
use server::compliance::federal::RuleSet;
use uuid::Uuid;

fn civil_context() -> CaseContext {
    CaseContext {
        case_id: Uuid::new_v4(),
        case_type: CaseType::Civil,
        case_number: "9:26-cv-00001".into(),
        status: "filed".into(),
        date_filed: Utc::now(),
        assigned_judge_id: None,
        trial_date: None,
        fee_status: "paid".into(),
        is_sealed: false,
    }
}

#[test]
fn case_filed_creates_service_deadline() {
    let rules = FrcpRules;
    let ctx = civil_context();
    let event = CaseEvent::new("case_filed", ctx.case_id, CaseType::Civil, "district9", None, serde_json::json!({}));
    let result = rules.evaluate(&event, &ctx);
    let deadlines: Vec<_> = result.effects.iter().filter(|e| matches!(e, server::compliance::ComplianceEffect::CreateDeadline { .. })).collect();
    assert!(!deadlines.is_empty(), "Should create service of process deadline");
    // Check one has FRCP 4(m) citation
    let has_4m = result.effects.iter().any(|e| {
        if let server::compliance::ComplianceEffect::CreateDeadline { rule_citation, .. } = e {
            rule_citation.contains("FRCP 4(m)")
        } else { false }
    });
    assert!(has_4m, "Should cite FRCP 4(m)");
}

#[test]
fn service_completed_creates_answer_deadline() {
    let rules = FrcpRules;
    let mut ctx = civil_context();
    ctx.status = "served".into();
    let event = CaseEvent::new("service_completed", ctx.case_id, CaseType::Civil, "district9", None, serde_json::json!({}));
    let result = rules.evaluate(&event, &ctx);
    let has_answer = result.effects.iter().any(|e| {
        if let server::compliance::ComplianceEffect::CreateDeadline { rule_citation, .. } = e {
            rule_citation.contains("FRCP 12(a)")
        } else { false }
    });
    assert!(has_answer, "Should cite FRCP 12(a)");
}

#[test]
fn answer_filed_creates_26f_and_jury_deadlines() {
    let rules = FrcpRules;
    let mut ctx = civil_context();
    ctx.status = "answer_due".into();
    let event = CaseEvent::new("answer_filed", ctx.case_id, CaseType::Civil, "district9", None, serde_json::json!({}));
    let result = rules.evaluate(&event, &ctx);
    assert!(result.effects.len() >= 2, "Should create 26(f) conference + jury demand deadlines");
}

#[test]
fn motion_filed_creates_response_deadline() {
    let rules = FrcpRules;
    let mut ctx = civil_context();
    ctx.status = "discovery".into();
    let event = CaseEvent::new("motion_filed", ctx.case_id, CaseType::Civil, "district9", None, serde_json::json!({"motion_type": "dismiss"}));
    let result = rules.evaluate(&event, &ctx);
    let has_response = result.effects.iter().any(|e| {
        if let server::compliance::ComplianceEffect::CreateDeadline { title, .. } = e {
            title.to_lowercase().contains("response") || title.to_lowercase().contains("opposition")
        } else { false }
    });
    assert!(has_response, "Should create response deadline for motion");
}

#[test]
fn trial_date_set_creates_multiple_deadlines() {
    let rules = FrcpRules;
    let mut ctx = civil_context();
    ctx.status = "pretrial".into();
    ctx.trial_date = Some(Utc::now() + Duration::days(120));
    let event = CaseEvent::new("trial_date_set", ctx.case_id, CaseType::Civil, "district9", None,
        serde_json::json!({"trial_date": (Utc::now() + Duration::days(120)).to_rfc3339()}));
    let result = rules.evaluate(&event, &ctx);
    // Should create expert disclosures (90 days before), pretrial disclosures (30 days before), SJ (30 days before)
    assert!(result.effects.len() >= 3, "Should create expert, pretrial, and SJ deadlines");
}

#[test]
fn criminal_event_ignored_by_frcp() {
    let rules = FrcpRules;
    let mut ctx = civil_context();
    ctx.case_type = CaseType::Criminal;
    let event = CaseEvent::new("case_filed", ctx.case_id, CaseType::Criminal, "district9", None, serde_json::json!({}));
    let result = rules.evaluate(&event, &ctx);
    assert!(result.effects.is_empty(), "FRCP should not fire for criminal cases");
}
```

Add `mod compliance_frcp_tests;` to `crates/tests/src/lib.rs`.

**Step 2: Create federal rules module structure**

Create `crates/server/src/compliance/federal/mod.rs`:

```rust
pub mod frcp;

use crate::compliance::event::{CaseEvent, CaseContext, RuleResult};

/// Trait for a set of federal rules.
pub trait RuleSet: Send + Sync {
    fn evaluate(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult;
}
```

**Step 3: Implement FRCP rules**

Create `crates/server/src/compliance/federal/frcp.rs`:

```rust
use chrono::{Duration, Utc};
use crate::compliance::calendar::compute_deadline;
use crate::compliance::event::*;
use super::RuleSet;

pub struct FrcpRules;

impl RuleSet for FrcpRules {
    fn evaluate(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        if case.case_type != CaseType::Civil {
            return RuleResult::empty();
        }

        match event.event_type.as_str() {
            "case_filed" => self.on_case_filed(event, case),
            "service_completed" => self.on_service_completed(event, case),
            "answer_filed" => self.on_answer_filed(event, case),
            "motion_filed" => self.on_motion_filed(event, case),
            "response_filed" => self.on_response_filed(event, case),
            "discovery_opened" => self.on_discovery_opened(event, case),
            "trial_date_set" => self.on_trial_date_set(event, case),
            "status_changed" => self.on_status_changed(event, case),
            _ => RuleResult::empty(),
        }
    }
}

impl FrcpRules {
    fn on_case_filed(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        // FRCP 4(m): 90 days to serve process
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Service of process deadline".into(),
            case_id: case.case_id,
            rule_citation: "FRCP 4(m)".into(),
            due_at: compute_deadline(event.timestamp, 90, false),
            notes: Some("Complaint must be served within 90 days of filing or case may be dismissed without prejudice.".into()),
        });
        result
    }

    fn on_service_completed(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        // Check if government party (60-day answer period)
        let is_govt = event.payload_str("party_type")
            .map(|t| t == "government" || t == "us_government")
            .unwrap_or(false);

        let (days, citation) = if is_govt {
            (60, "FRCP 12(a)(2)")
        } else {
            (21, "FRCP 12(a)(1)(A)(i)")
        };

        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Answer due".into(),
            case_id: case.case_id,
            rule_citation: citation.into(),
            due_at: compute_deadline(event.timestamp, days, false),
            notes: Some(format!("Defendant must answer or file responsive motion within {} days of service.", days)),
        });
        result
    }

    fn on_answer_filed(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        // FRCP 26(f): discovery conference within 21 days
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Rule 26(f) discovery conference".into(),
            case_id: case.case_id,
            rule_citation: "FRCP 26(f)".into(),
            due_at: compute_deadline(event.timestamp, 21, false),
            notes: Some("Parties must confer regarding discovery plan.".into()),
        });
        // FRCP 38(b): jury demand within 14 days of last pleading
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Jury demand deadline".into(),
            case_id: case.case_id,
            rule_citation: "FRCP 38(b)".into(),
            due_at: compute_deadline(event.timestamp, 14, false),
            notes: Some("Any party may demand a jury trial within 14 days after the last pleading is served.".into()),
        });
        result
    }

    fn on_motion_filed(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        let motion_type = event.payload_str("motion_type").unwrap_or("other");

        // FRCP 6(d): response due 21 days after service for dispositive motions,
        // 14 days for non-dispositive
        let (days, title) = match motion_type {
            "dismiss" | "summary_judgment" => (21, "Opposition to dispositive motion due"),
            "interrogatories" => (30, "Interrogatory responses due (FRCP 33(b)(2))"),
            "doc_request" | "document_production" => (30, "Document production due (FRCP 34(b)(2)(A))"),
            _ => (14, "Response to motion due"),
        };

        let citation = match motion_type {
            "dismiss" => "FRCP 12(b)",
            "summary_judgment" => "FRCP 56",
            "interrogatories" => "FRCP 33(b)(2)",
            "doc_request" | "document_production" => "FRCP 34(b)(2)(A)",
            _ => "FRCP 6(d)",
        };

        result.effects.push(ComplianceEffect::CreateDeadline {
            title: title.into(),
            case_id: case.case_id,
            rule_citation: citation.into(),
            due_at: compute_deadline(event.timestamp, days, false),
            notes: None,
        });
        result
    }

    fn on_response_filed(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        // Reply brief: 14 days after response
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Reply brief due".into(),
            case_id: case.case_id,
            rule_citation: "FRCP 6(d)".into(),
            due_at: compute_deadline(event.timestamp, 14, false),
            notes: None,
        });
        result
    }

    fn on_discovery_opened(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        // FRCP 26(a)(1): initial disclosures within 14 days
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Initial disclosures due".into(),
            case_id: case.case_id,
            rule_citation: "FRCP 26(a)(1)".into(),
            due_at: compute_deadline(event.timestamp, 14, false),
            notes: Some("Parties must exchange initial disclosures without awaiting a discovery request.".into()),
        });
        result
    }

    fn on_trial_date_set(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        let trial_date = case.trial_date.unwrap_or_else(|| {
            event.payload_str("trial_date")
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now() + Duration::days(120))
        });

        // FRCP 26(a)(2): expert disclosures 90 days before trial
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Expert disclosures due".into(),
            case_id: case.case_id,
            rule_citation: "FRCP 26(a)(2)".into(),
            due_at: trial_date - Duration::days(90),
            notes: Some("Expert witness disclosures must be made at least 90 days before trial.".into()),
        });

        // FRCP 26(a)(3): pretrial disclosures 30 days before trial
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Pretrial disclosures due".into(),
            case_id: case.case_id,
            rule_citation: "FRCP 26(a)(3)".into(),
            due_at: trial_date - Duration::days(30),
            notes: Some("Pretrial disclosures including witness lists and exhibits.".into()),
        });

        // FRCP 56(b): summary judgment at least 30 days before trial
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Summary judgment motion deadline".into(),
            case_id: case.case_id,
            rule_citation: "FRCP 56(b)".into(),
            due_at: trial_date - Duration::days(30),
            notes: Some("Motions for summary judgment should be filed at least 30 days before trial.".into()),
        });

        result
    }

    fn on_status_changed(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        let new_status = event.payload_str("new_status").unwrap_or("");

        match new_status {
            "judgment" | "dismissed" => {
                // FRCP 59(b): post-judgment motion 28 days
                result.effects.push(ComplianceEffect::CreateDeadline {
                    title: "Post-judgment motion deadline".into(),
                    case_id: case.case_id,
                    rule_citation: "FRCP 59(b)".into(),
                    due_at: compute_deadline(event.timestamp, 28, false),
                    notes: Some("Motion for new trial or to alter/amend judgment must be filed within 28 days.".into()),
                });
                // FRAP 4(a)(1): appeal 30 days
                result.effects.push(ComplianceEffect::CreateDeadline {
                    title: "Notice of appeal deadline".into(),
                    case_id: case.case_id,
                    rule_citation: "FRAP 4(a)(1)".into(),
                    due_at: compute_deadline(event.timestamp, 30, false),
                    notes: Some("Notice of appeal must be filed within 30 days of entry of judgment.".into()),
                });
            }
            _ => {}
        }
        result
    }
}
```

Add `pub mod federal;` to `crates/server/src/compliance/mod.rs`.

**Step 4: Run tests**

Run: `cargo test -p tests compliance_frcp -- --test-threads=1`
Expected: All pass

**Step 5: Commit**

```bash
git add crates/server/src/compliance/federal/ crates/server/src/compliance/mod.rs crates/tests/src/compliance_frcp_tests.rs crates/tests/src/lib.rs
git commit -m "feat(compliance): FRCP civil rules — service, answer, discovery, motions, trial deadlines"
```

---

### Task 6: FRCrP + Speedy Trial Act Rules

**Files:**
- Create: `crates/server/src/compliance/federal/frcrp.rs`
- Create: `crates/server/src/compliance/federal/speedy_trial.rs`
- Modify: `crates/server/src/compliance/federal/mod.rs`
- Create: `crates/tests/src/compliance_frcrp_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Write tests**

Create `crates/tests/src/compliance_frcrp_tests.rs`:

```rust
use server::compliance::{CaseEvent, CaseType, CaseContext, ComplianceEffect};
use server::compliance::federal::frcrp::FrcrpRules;
use server::compliance::federal::speedy_trial::SpeedyTrialRules;
use server::compliance::federal::RuleSet;
use uuid::Uuid;
use chrono::Utc;

fn criminal_context() -> CaseContext {
    CaseContext {
        case_id: Uuid::new_v4(),
        case_type: CaseType::Criminal,
        case_number: "9:26-cr-00001".into(),
        status: "filed".into(),
        date_filed: Utc::now(),
        assigned_judge_id: None,
        trial_date: None,
        fee_status: "pending".into(),
        is_sealed: false,
    }
}

#[test]
fn criminal_case_filed_starts_speedy_trial_clock() {
    let rules = SpeedyTrialRules;
    let ctx = criminal_context();
    let event = CaseEvent::new("case_filed", ctx.case_id, CaseType::Criminal, "district9", None, serde_json::json!({}));
    let result = rules.evaluate(&event, &ctx);
    let has_clock = result.effects.iter().any(|e| matches!(e, ComplianceEffect::StartSpeedyTrialClock { .. }));
    assert!(has_clock, "Should start speedy trial clock on criminal case filing");
}

#[test]
fn criminal_case_filed_creates_indictment_deadline() {
    let rules = SpeedyTrialRules;
    let ctx = criminal_context();
    let event = CaseEvent::new("case_filed", ctx.case_id, CaseType::Criminal, "district9", None, serde_json::json!({}));
    let result = rules.evaluate(&event, &ctx);
    let has_indictment = result.effects.iter().any(|e| {
        if let ComplianceEffect::CreateDeadline { rule_citation, .. } = e {
            rule_citation.contains("3161(b)")
        } else { false }
    });
    assert!(has_indictment, "Should create 30-day indictment deadline");
}

#[test]
fn arraignment_creates_trial_deadline() {
    let rules = SpeedyTrialRules;
    let mut ctx = criminal_context();
    ctx.status = "arraigned".into();
    let event = CaseEvent::new("arraignment_held", ctx.case_id, CaseType::Criminal, "district9", None, serde_json::json!({}));
    let result = rules.evaluate(&event, &ctx);
    let has_trial = result.effects.iter().any(|e| {
        if let ComplianceEffect::CreateDeadline { rule_citation, .. } = e {
            rule_citation.contains("3161(c)")
        } else { false }
    });
    assert!(has_trial, "Should create 70-day trial deadline");
}

#[test]
fn motion_filed_tolls_speedy_trial() {
    let rules = SpeedyTrialRules;
    let mut ctx = criminal_context();
    ctx.status = "pretrial_motions".into();
    let event = CaseEvent::new("motion_filed", ctx.case_id, CaseType::Criminal, "district9", None, serde_json::json!({}));
    let result = rules.evaluate(&event, &ctx);
    let has_toll = result.effects.iter().any(|e| matches!(e, ComplianceEffect::TollSpeedyTrialClock { .. }));
    assert!(has_toll, "Should toll speedy trial clock on pretrial motion");
}

#[test]
fn frcrp_arraignment_creates_pretrial_motion_deadline() {
    let rules = FrcrpRules;
    let mut ctx = criminal_context();
    ctx.status = "arraigned".into();
    let event = CaseEvent::new("arraignment_held", ctx.case_id, CaseType::Criminal, "district9", None, serde_json::json!({}));
    let result = rules.evaluate(&event, &ctx);
    let has_motion = result.effects.iter().any(|e| {
        if let ComplianceEffect::CreateDeadline { rule_citation, .. } = e {
            rule_citation.contains("FRCrP 12(b)")
        } else { false }
    });
    assert!(has_motion, "Should create pretrial motion deadline");
}

#[test]
fn civil_event_ignored_by_frcrp() {
    let rules = FrcrpRules;
    let mut ctx = criminal_context();
    ctx.case_type = CaseType::Civil;
    let event = CaseEvent::new("case_filed", ctx.case_id, CaseType::Civil, "district9", None, serde_json::json!({}));
    let result = rules.evaluate(&event, &ctx);
    assert!(result.effects.is_empty());
}
```

Add `mod compliance_frcrp_tests;` to `crates/tests/src/lib.rs`.

**Step 2: Implement FRCrP rules**

Create `crates/server/src/compliance/federal/frcrp.rs`:

```rust
use crate::compliance::calendar::compute_deadline;
use crate::compliance::event::*;
use super::RuleSet;

pub struct FrcrpRules;

impl RuleSet for FrcrpRules {
    fn evaluate(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        if case.case_type != CaseType::Criminal {
            return RuleResult::empty();
        }
        match event.event_type.as_str() {
            "case_filed" => self.on_case_filed(event, case),
            "indictment_returned" => self.on_indictment(event, case),
            "arraignment_held" => self.on_arraignment(event, case),
            "motion_filed" => self.on_motion_filed(event, case),
            "status_changed" => self.on_status_changed(event, case),
            _ => RuleResult::empty(),
        }
    }
}

impl FrcrpRules {
    fn on_case_filed(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        // FRCrP 5(a): initial appearance without unnecessary delay
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Initial appearance deadline".into(),
            case_id: case.case_id,
            rule_citation: "FRCrP 5(a)".into(),
            due_at: compute_deadline(event.timestamp, 2, false), // 48 hours if arrested
            notes: Some("Defendant must be brought before a magistrate judge without unnecessary delay.".into()),
        });
        result
    }

    fn on_indictment(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        // FRCrP 10: arraignment
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Arraignment deadline".into(),
            case_id: case.case_id,
            rule_citation: "FRCrP 10".into(),
            due_at: compute_deadline(event.timestamp, 14, false),
            notes: Some("Arraignment must be held without unnecessary delay after indictment.".into()),
        });
        result
    }

    fn on_arraignment(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        // FRCrP 12(b): pretrial motions deadline
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Pretrial motions deadline".into(),
            case_id: case.case_id,
            rule_citation: "FRCrP 12(b)".into(),
            due_at: compute_deadline(event.timestamp, 21, false),
            notes: Some("Pretrial motions must be filed per the court's scheduling order.".into()),
        });
        // FRCrP 16 + Brady: discovery disclosure
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Government discovery/Brady disclosure".into(),
            case_id: case.case_id,
            rule_citation: "FRCrP 16 / Brady v. Maryland".into(),
            due_at: compute_deadline(event.timestamp, 14, false),
            notes: Some("Government must disclose discoverable material and exculpatory evidence.".into()),
        });
        result
    }

    fn on_motion_filed(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        // Response deadline for motions
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Response to motion due".into(),
            case_id: case.case_id,
            rule_citation: "FRCrP 47 / Local Rule".into(),
            due_at: compute_deadline(event.timestamp, 14, false),
            notes: None,
        });
        result
    }

    fn on_status_changed(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        let new_status = event.payload_str("new_status").unwrap_or("");
        match new_status {
            "awaiting_sentencing" => {
                // FRCrP 32(b)(1): presentence investigation and sentencing
                result.effects.push(ComplianceEffect::CreateDeadline {
                    title: "Sentencing hearing deadline".into(),
                    case_id: case.case_id,
                    rule_citation: "FRCrP 32(b)(1)".into(),
                    due_at: compute_deadline(event.timestamp, 90, false),
                    notes: Some("Sentencing must occur within 90 days. Presentence report due 35 days before.".into()),
                });
            }
            "sentenced" | "dismissed" => {
                // FRAP 4(b): criminal appeal 14 days
                result.effects.push(ComplianceEffect::CreateDeadline {
                    title: "Notice of appeal deadline".into(),
                    case_id: case.case_id,
                    rule_citation: "FRAP 4(b)(1)".into(),
                    due_at: compute_deadline(event.timestamp, 14, false),
                    notes: Some("Notice of appeal in a criminal case must be filed within 14 days.".into()),
                });
            }
            _ => {}
        }
        result
    }
}
```

**Step 3: Implement Speedy Trial rules**

Create `crates/server/src/compliance/federal/speedy_trial.rs`:

```rust
use crate::compliance::calendar::compute_deadline;
use crate::compliance::event::*;
use super::RuleSet;

pub struct SpeedyTrialRules;

impl RuleSet for SpeedyTrialRules {
    fn evaluate(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        if case.case_type != CaseType::Criminal {
            return RuleResult::empty();
        }
        match event.event_type.as_str() {
            "case_filed" => self.on_case_filed(event, case),
            "arraignment_held" => self.on_arraignment(event, case),
            "motion_filed" => self.on_motion_filed(event, case),
            "order_entered" => self.on_order_entered(event, case),
            _ => RuleResult::empty(),
        }
    }
}

impl SpeedyTrialRules {
    fn on_case_filed(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        // Start the Speedy Trial clock
        result.effects.push(ComplianceEffect::StartSpeedyTrialClock {
            case_id: case.case_id,
            trigger_date: event.timestamp,
        });
        // 18 U.S.C. § 3161(b): indictment within 30 days of arrest/summons
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Indictment deadline (Speedy Trial Act)".into(),
            case_id: case.case_id,
            rule_citation: "18 U.S.C. § 3161(b)".into(),
            due_at: compute_deadline(event.timestamp, 30, false),
            notes: Some("Indictment must be filed within 30 days of arrest or service of summons.".into()),
        });
        result
    }

    fn on_arraignment(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        // 18 U.S.C. § 3161(c)(1): trial within 70 days of indictment filing
        // or initial appearance, whichever is later
        result.effects.push(ComplianceEffect::CreateDeadline {
            title: "Trial must commence (Speedy Trial Act)".into(),
            case_id: case.case_id,
            rule_citation: "18 U.S.C. § 3161(c)(1)".into(),
            due_at: compute_deadline(event.timestamp, 70, false),
            notes: Some("Trial must begin within 70 days of arraignment, excluding excludable delays.".into()),
        });
        result
    }

    fn on_motion_filed(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        // 18 U.S.C. § 3161(h)(1)(D): pretrial motions toll the clock
        result.effects.push(ComplianceEffect::TollSpeedyTrialClock {
            case_id: case.case_id,
            reason: "Pending pretrial motion".into(),
            statutory_ref: "18 U.S.C. § 3161(h)(1)(D)".into(),
        });
        result
    }

    fn on_order_entered(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult {
        let mut result = RuleResult::empty();
        let order_type = event.payload_str("order_type").unwrap_or("");
        // If order resolves a motion, resume the clock
        if order_type == "motion_ruling" || order_type == "motion_denied" || order_type == "motion_granted" {
            result.effects.push(ComplianceEffect::ResumeSpeedyTrialClock {
                case_id: case.case_id,
            });
        }
        // If continuance granted under ends-of-justice
        if order_type == "continuance" {
            result.effects.push(ComplianceEffect::TollSpeedyTrialClock {
                case_id: case.case_id,
                reason: "Continuance granted (ends of justice)".into(),
                statutory_ref: "18 U.S.C. § 3161(h)(7)".into(),
            });
        }
        result
    }
}
```

Update `crates/server/src/compliance/federal/mod.rs`:

```rust
pub mod frcp;
pub mod frcrp;
pub mod speedy_trial;

use crate::compliance::event::{CaseEvent, CaseContext, RuleResult};

/// Trait for a set of federal rules.
pub trait RuleSet: Send + Sync {
    fn evaluate(&self, event: &CaseEvent, case: &CaseContext) -> RuleResult;
}
```

**Step 4: Run tests**

Run: `cargo test -p tests compliance_frcrp -- --test-threads=1`
Expected: All pass

**Step 5: Commit**

```bash
git add crates/server/src/compliance/federal/ crates/tests/src/compliance_frcrp_tests.rs crates/tests/src/lib.rs
git commit -m "feat(compliance): FRCrP + Speedy Trial Act rules — indictment, arraignment, trial deadlines, clock tolling"
```

---

### Task 7: Local Rules Evaluator

**Files:**
- Create: `crates/server/src/compliance/local.rs`
- Modify: `crates/server/src/compliance/mod.rs`
- Create: `crates/tests/src/compliance_local_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Write tests**

Create `crates/tests/src/compliance_local_tests.rs`:

```rust
use crate::common::*;
use server::compliance::{CaseEvent, CaseType, CaseContext};
use server::compliance::local::evaluate_local_rules;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn local_rule_with_matching_conditions_fires() {
    let (_app, pool, _guard) = test_app().await;
    // Insert a local rule
    sqlx::query(
        "INSERT INTO rules (court_id, name, description, source, category, priority, status, conditions, actions)
         VALUES ('district9', 'L.R. 7.1 Motion Response', 'Local response deadline', 'Local Rules', 'Deadline', 1, 'Active',
                 '{\"trigger\": \"motion_filed\", \"case_type\": \"civil\"}'::jsonb,
                 '{\"create_deadline\": {\"days\": 14, \"title\": \"Motion response due (L.R. 7.1)\"}}'::jsonb)"
    ).execute(&pool).await.unwrap();

    let ctx = CaseContext {
        case_id: Uuid::new_v4(),
        case_type: CaseType::Civil,
        case_number: "9:26-cv-00001".into(),
        status: "discovery".into(),
        date_filed: Utc::now(),
        assigned_judge_id: None,
        trial_date: None,
        fee_status: "paid".into(),
        is_sealed: false,
    };
    let event = CaseEvent::new("motion_filed", ctx.case_id, CaseType::Civil, "district9", None, serde_json::json!({}));
    let result = evaluate_local_rules(&pool, "district9", &event, &ctx).await;
    assert!(!result.effects.is_empty(), "Local rule should fire and produce a deadline");
}

#[tokio::test]
async fn local_rule_with_non_matching_conditions_does_not_fire() {
    let (_app, pool, _guard) = test_app().await;
    sqlx::query(
        "INSERT INTO rules (court_id, name, description, source, category, priority, status, conditions, actions)
         VALUES ('district9', 'Criminal only rule', 'Test', 'Local Rules', 'Deadline', 1, 'Active',
                 '{\"trigger\": \"case_filed\", \"case_type\": \"criminal\"}'::jsonb,
                 '{\"create_deadline\": {\"days\": 7, \"title\": \"Test\"}}'::jsonb)"
    ).execute(&pool).await.unwrap();

    let ctx = CaseContext {
        case_id: Uuid::new_v4(),
        case_type: CaseType::Civil,
        case_number: "9:26-cv-00001".into(),
        status: "filed".into(),
        date_filed: Utc::now(),
        assigned_judge_id: None,
        trial_date: None,
        fee_status: "paid".into(),
        is_sealed: false,
    };
    let event = CaseEvent::new("case_filed", ctx.case_id, CaseType::Civil, "district9", None, serde_json::json!({}));
    let result = evaluate_local_rules(&pool, "district9", &event, &ctx).await;
    assert!(result.effects.is_empty(), "Rule should not fire for civil when conditions require criminal");
}
```

Add `mod compliance_local_tests;` to `crates/tests/src/lib.rs`.

**Step 2: Implement local rules evaluator**

Create `crates/server/src/compliance/local.rs`:

```rust
use chrono::Duration;
use sqlx::{Pool, Postgres};

use crate::compliance::calendar::compute_deadline;
use crate::compliance::event::*;

/// Evaluate local (DB-stored) rules against an event.
/// Local rules are stored in the `rules` table with JSONB conditions and actions.
pub async fn evaluate_local_rules(
    pool: &Pool<Postgres>,
    court_id: &str,
    event: &CaseEvent,
    case: &CaseContext,
) -> RuleResult {
    let rules = match crate::repo::rule::list_active(pool, court_id, None).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(%e, "Failed to load local rules");
            return RuleResult::empty();
        }
    };

    let mut result = RuleResult::empty();

    for rule in rules {
        if matches_conditions(&rule.conditions, event, case) {
            parse_actions(&rule.actions, event, case, &rule.citation.unwrap_or_default(), &mut result);
        }
    }

    result
}

/// Check if a rule's conditions match the current event + case context.
fn matches_conditions(conditions: &serde_json::Value, event: &CaseEvent, case: &CaseContext) -> bool {
    let cond = match conditions.as_object() {
        Some(m) => m,
        None => return true, // empty conditions = always match
    };

    if cond.is_empty() {
        return true;
    }

    for (key, expected) in cond {
        let actual = match key.as_str() {
            "trigger" => Some(serde_json::Value::String(event.event_type.clone())),
            "case_type" => Some(serde_json::Value::String(case.case_type.as_str().to_string())),
            "case_status" => Some(serde_json::Value::String(case.status.clone())),
            k if k.starts_with("payload.") => {
                let payload_key = &k["payload.".len()..];
                event.payload.get(payload_key).cloned()
            }
            _ => event.payload.get(key).cloned(),
        };

        let matches = match (actual, expected) {
            (Some(actual), serde_json::Value::Array(arr)) => arr.contains(&actual),
            (Some(actual), expected) => &actual == expected,
            (None, _) => false,
        };

        if !matches {
            return false;
        }
    }

    true
}

/// Parse a rule's actions JSONB and add effects to the result.
fn parse_actions(
    actions: &serde_json::Value,
    event: &CaseEvent,
    case: &CaseContext,
    citation: &str,
    result: &mut RuleResult,
) {
    let actions_obj = match actions.as_object() {
        Some(m) => m,
        None => return,
    };

    if let Some(dl) = actions_obj.get("create_deadline") {
        let title = dl.get("title").and_then(|v| v.as_str()).unwrap_or("Deadline").to_string();
        let mail_service = dl.get("mail_service").and_then(|v| v.as_bool()).unwrap_or(false);

        let due_at = if let Some(days) = dl.get("days").and_then(|v| v.as_i64()) {
            compute_deadline(event.timestamp, days, mail_service)
        } else if let Some(days_before) = dl.get("days_before_trial").and_then(|v| v.as_i64()) {
            let trial = case.trial_date.unwrap_or(event.timestamp + Duration::days(120));
            trial - Duration::days(days_before)
        } else {
            return;
        };

        result.effects.push(ComplianceEffect::CreateDeadline {
            title,
            case_id: case.case_id,
            rule_citation: citation.to_string(),
            due_at,
            notes: dl.get("notes").and_then(|v| v.as_str()).map(String::from),
        });
    }

    if let Some(block) = actions_obj.get("block_action") {
        let message = block.get("message").and_then(|v| v.as_str()).unwrap_or("Action blocked by local rule").to_string();
        let override_role = block.get("override_role").and_then(|v| v.as_str()).unwrap_or("clerk").to_string();
        result.violations.push(ComplianceViolation {
            rule_citation: citation.to_string(),
            message,
            override_allowed: true,
            override_role,
        });
    }
}
```

Add `pub mod local;` to `crates/server/src/compliance/mod.rs`.

**Step 3: Run tests**

Run: `cargo test -p tests compliance_local -- --test-threads=1`
Expected: All pass

**Step 4: Commit**

```bash
git add crates/server/src/compliance/local.rs crates/server/src/compliance/mod.rs crates/tests/src/compliance_local_tests.rs crates/tests/src/lib.rs
git commit -m "feat(compliance): local rules evaluator — DB-driven JSONB conditions/actions"
```

---

## Phase 3: Engine Orchestrator + Judge Assignment (Tasks 8-10)

### Task 8: ComplianceEngine Orchestrator

**Files:**
- Create: `crates/server/src/compliance/engine.rs`
- Modify: `crates/server/src/compliance/mod.rs`
- Create: `crates/tests/src/compliance_engine_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Write integration tests**

Create `crates/tests/src/compliance_engine_tests.rs`:

```rust
use crate::common::*;
use server::compliance::{CaseEvent, CaseType, ComplianceEffect};
use server::compliance::engine::ComplianceEngine;
use uuid::Uuid;

#[tokio::test]
async fn engine_evaluates_frcp_on_civil_case_filed() {
    let (_app, pool, _guard) = test_app().await;
    let engine = ComplianceEngine::new(&pool, "district9");
    let case_id = Uuid::new_v4();
    let event = CaseEvent::new("case_filed", case_id, CaseType::Civil, "district9", None, serde_json::json!({}));
    let ctx = engine.build_context_for_new_case(case_id, CaseType::Civil, "9:26-cv-00001").await;
    let result = engine.evaluate(&event, &ctx, None).await.unwrap();
    // Should have FRCP 4(m) service deadline + judge assignment
    let has_deadline = result.iter().any(|e| matches!(e, ComplianceEffect::CreateDeadline { .. }));
    assert!(has_deadline, "Should produce at least one deadline");
}

#[tokio::test]
async fn engine_evaluates_speedy_trial_on_criminal_case_filed() {
    let (_app, pool, _guard) = test_app().await;
    let engine = ComplianceEngine::new(&pool, "district9");
    let case_id = Uuid::new_v4();
    let event = CaseEvent::new("case_filed", case_id, CaseType::Criminal, "district9", None, serde_json::json!({}));
    let ctx = engine.build_context_for_new_case(case_id, CaseType::Criminal, "9:26-cr-00001").await;
    let result = engine.evaluate(&event, &ctx, None).await.unwrap();
    let has_clock = result.iter().any(|e| matches!(e, ComplianceEffect::StartSpeedyTrialClock { .. }));
    assert!(has_clock, "Should start speedy trial clock");
}

#[tokio::test]
async fn engine_returns_violation_for_invalid_transition() {
    let (_app, pool, _guard) = test_app().await;
    let engine = ComplianceEngine::new(&pool, "district9");
    let case_id = create_test_case(&pool, "district9", "9:26-cr-00099").await;
    let event = CaseEvent::new("status_changed", case_id, CaseType::Criminal, "district9", None,
        serde_json::json!({"old_status": "filed", "new_status": "in_trial"}));
    let mut ctx = engine.build_context_for_new_case(case_id, CaseType::Criminal, "9:26-cr-00099").await;
    ctx.status = "filed".into();
    let result = engine.evaluate(&event, &ctx, None).await;
    assert!(result.is_err(), "Should reject invalid status transition");
}
```

Add `mod compliance_engine_tests;` to `crates/tests/src/lib.rs`.

**Step 2: Implement engine**

Create `crates/server/src/compliance/engine.rs`:

```rust
use chrono::Utc;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::compliance::event::*;
use crate::compliance::federal::{frcp::FrcpRules, frcrp::FrcrpRules, speedy_trial::SpeedyTrialRules, RuleSet};
use crate::compliance::local::evaluate_local_rules;
use crate::compliance::status::transition_violation;

pub struct ComplianceEngine<'a> {
    pool: &'a Pool<Postgres>,
    court_id: &'a str,
}

impl<'a> ComplianceEngine<'a> {
    pub fn new(pool: &'a Pool<Postgres>, court_id: &'a str) -> Self {
        Self { pool, court_id }
    }

    /// Build a CaseContext for a brand-new case (no DB state yet).
    pub async fn build_context_for_new_case(
        &self,
        case_id: Uuid,
        case_type: CaseType,
        case_number: &str,
    ) -> CaseContext {
        CaseContext {
            case_id,
            case_type,
            case_number: case_number.to_string(),
            status: "filed".into(),
            date_filed: Utc::now(),
            assigned_judge_id: None,
            trial_date: None,
            fee_status: "pending".into(),
            is_sealed: false,
        }
    }

    /// Load CaseContext from database for an existing case.
    pub async fn load_case_context(
        &self,
        case_id: Uuid,
        case_type: CaseType,
    ) -> Result<CaseContext, shared_types::AppError> {
        match case_type {
            CaseType::Criminal => {
                let row = sqlx::query!(
                    r#"SELECT case_number, status, date_opened, assigned_judge_id, is_sealed
                       FROM criminal_cases WHERE id = $1 AND court_id = $2"#,
                    case_id, self.court_id,
                )
                .fetch_one(self.pool)
                .await
                .map_err(|_| shared_types::AppError::not_found("Case not found"))?;

                Ok(CaseContext {
                    case_id,
                    case_type: CaseType::Criminal,
                    case_number: row.case_number,
                    status: row.status,
                    date_filed: row.date_opened,
                    assigned_judge_id: row.assigned_judge_id,
                    trial_date: None,
                    fee_status: "pending".into(),
                    is_sealed: row.is_sealed.unwrap_or(false),
                })
            }
            CaseType::Civil => {
                let row = sqlx::query!(
                    r#"SELECT case_number, status, date_filed, assigned_judge_id, is_sealed
                       FROM civil_cases WHERE id = $1 AND court_id = $2"#,
                    case_id, self.court_id,
                )
                .fetch_one(self.pool)
                .await
                .map_err(|_| shared_types::AppError::not_found("Case not found"))?;

                Ok(CaseContext {
                    case_id,
                    case_type: CaseType::Civil,
                    case_number: row.case_number,
                    status: row.status,
                    date_filed: row.date_filed,
                    assigned_judge_id: row.assigned_judge_id,
                    trial_date: None,
                    fee_status: "pending".into(),
                    is_sealed: row.is_sealed.unwrap_or(false),
                })
            }
        }
    }

    /// Evaluate all rule sets against an event.
    /// Returns Ok(effects) or Err(violation) if a hard block is triggered.
    pub async fn evaluate(
        &self,
        event: &CaseEvent,
        case: &CaseContext,
        clerk_override: Option<&ClerkOverride>,
    ) -> Result<Vec<ComplianceEffect>, ComplianceViolation> {
        let mut all_effects = Vec::new();
        let mut all_violations = Vec::new();

        // 1. Status transition validation (if applicable)
        if event.event_type == "status_changed" {
            if let Some(new_status) = event.payload_str("new_status") {
                if let Some(violation) = transition_violation(
                    case.case_type.as_str(),
                    &case.status,
                    new_status,
                ) {
                    all_violations.push(violation);
                }
            }
        }

        // 2. Federal rules (compiled Rust)
        let rule_sets: Vec<Box<dyn RuleSet>> = vec![
            Box::new(FrcpRules),
            Box::new(FrcrpRules),
            Box::new(SpeedyTrialRules),
        ];

        for rule_set in &rule_sets {
            let result = rule_set.evaluate(event, case);
            all_effects.extend(result.effects);
            all_violations.extend(result.violations);
        }

        // 3. Local rules (DB-stored)
        let local_result = evaluate_local_rules(self.pool, self.court_id, event, case).await;
        all_effects.extend(local_result.effects);
        all_violations.extend(local_result.violations);

        // 4. Check violations — allow override if clerk/judge provides one
        if let Some(violation) = all_violations.into_iter().next() {
            if let Some(override_info) = clerk_override {
                if violation.override_allowed {
                    tracing::warn!(
                        rule = %violation.rule_citation,
                        reason = %override_info.reason,
                        user_id = override_info.user_id,
                        "Compliance violation overridden"
                    );
                    // Override accepted — continue with effects
                } else {
                    return Err(violation);
                }
            } else {
                return Err(violation);
            }
        }

        Ok(all_effects)
    }

    /// Apply compliance effects within a transaction.
    pub async fn apply_effects(
        &self,
        pool: &Pool<Postgres>,
        effects: &[ComplianceEffect],
        court_id: &str,
    ) -> Result<(), shared_types::AppError> {
        for effect in effects {
            match effect {
                ComplianceEffect::CreateDeadline { title, case_id, rule_citation, due_at, notes } => {
                    sqlx::query!(
                        r#"INSERT INTO deadlines (court_id, case_id, title, rule_code, due_at, status, notes)
                           VALUES ($1, $2, $3, $4, $5, 'open', $6)"#,
                        court_id,
                        case_id,
                        title,
                        rule_citation,
                        due_at,
                        notes.as_deref(),
                    )
                    .execute(pool)
                    .await
                    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;
                }
                ComplianceEffect::StartSpeedyTrialClock { case_id, trigger_date } => {
                    let deadline = *trigger_date + chrono::Duration::days(70);
                    sqlx::query!(
                        r#"INSERT INTO speedy_trial_clocks (case_id, court_id, trial_start_deadline, days_elapsed, days_remaining, is_tolled, waived)
                           VALUES ($1, $2, $3, 0, 70, false, false)
                           ON CONFLICT (case_id) DO NOTHING"#,
                        case_id,
                        court_id,
                        deadline,
                    )
                    .execute(pool)
                    .await
                    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;
                }
                ComplianceEffect::TollSpeedyTrialClock { case_id, reason, statutory_ref } => {
                    sqlx::query!(
                        "UPDATE speedy_trial_clocks SET is_tolled = true WHERE case_id = $1 AND court_id = $2",
                        case_id, court_id,
                    )
                    .execute(pool)
                    .await
                    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

                    sqlx::query!(
                        r#"INSERT INTO excludable_delays (court_id, case_id, start_date, reason, statutory_reference, days_excluded)
                           VALUES ($1, $2, NOW(), $3, $4, 0)"#,
                        court_id, case_id, reason, statutory_ref,
                    )
                    .execute(pool)
                    .await
                    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;
                }
                ComplianceEffect::ResumeSpeedyTrialClock { case_id } => {
                    sqlx::query!(
                        "UPDATE speedy_trial_clocks SET is_tolled = false WHERE case_id = $1 AND court_id = $2",
                        case_id, court_id,
                    )
                    .execute(pool)
                    .await
                    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;
                }
                ComplianceEffect::AssignJudge { case_id, judge_id, reason } => {
                    crate::repo::case_assignment::create(
                        pool, court_id, *case_id, *judge_id,
                        "Initial", reason, None, None,
                    ).await?;
                }
                ComplianceEffect::CreateQueueItem { case_id, queue_type, title, priority } => {
                    sqlx::query!(
                        r#"INSERT INTO clerk_queue (court_id, case_id, queue_type, title, description, priority, status, current_step)
                           VALUES ($1, $2, $3, $4, 'Auto-created by compliance engine', $5, 'pending', 'review')"#,
                        court_id, case_id, queue_type, title, priority,
                    )
                    .execute(pool)
                    .await
                    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;
                }
            }
        }
        Ok(())
    }
}
```

Add `pub mod engine;` to `crates/server/src/compliance/mod.rs`.

Update final `crates/server/src/compliance/mod.rs`:

```rust
pub mod calendar;
pub mod engine;
pub mod event;
pub mod federal;
pub mod local;
pub mod status;

pub use event::*;
```

**Step 3: Run tests**

Run: `cargo test -p tests compliance_engine -- --test-threads=1`
Expected: All pass

**Step 4: Commit**

```bash
git add crates/server/src/compliance/engine.rs crates/server/src/compliance/mod.rs crates/tests/src/compliance_engine_tests.rs crates/tests/src/lib.rs
git commit -m "feat(compliance): engine orchestrator — evaluates federal + local rules, applies effects"
```

---

### Task 9: Judge Assignment Wheel

**Files:**
- Create: `migrations/20260301000093_create_judge_assignment_config.sql`
- Create: `migrations/20260301000093_create_judge_assignment_config.down.sql`
- Create: `crates/server/src/compliance/assignment.rs`
- Modify: `crates/server/src/compliance/mod.rs`
- Create: `crates/tests/src/compliance_assignment_tests.rs`
- Modify: `crates/tests/src/lib.rs`
- Modify: `crates/tests/src/common.rs` (add `judge_assignment_config` to TRUNCATE)

**Step 1: Create migration**

```sql
CREATE TABLE IF NOT EXISTS judge_assignment_config (
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

CREATE INDEX idx_judge_assignment_config_court ON judge_assignment_config(court_id, case_type, active);
```

Down: `DROP TABLE IF EXISTS judge_assignment_config;`

**Step 2: Write tests**

Create `crates/tests/src/compliance_assignment_tests.rs`:

```rust
use crate::common::*;
use server::compliance::assignment::assign_judge;
use server::compliance::event::CaseType;
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::test]
async fn assign_judge_with_config_returns_assignment() {
    let (_app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Assignment Test").await;
    // Insert assignment config
    sqlx::query(
        "INSERT INTO judge_assignment_config (court_id, judge_id, case_type, weight) VALUES ('district9', $1, 'criminal', 50)"
    ).bind(judge_id).execute(&pool).await.unwrap();

    let case_id = Uuid::new_v4();
    let result = assign_judge(&pool, "district9", CaseType::Criminal, case_id).await;
    assert!(result.is_ok());
    let effect = result.unwrap();
    assert!(effect.is_some(), "Should produce an AssignJudge effect");
}

#[tokio::test]
async fn assign_judge_without_config_returns_none() {
    let (_app, pool, _guard) = test_app().await;
    let case_id = Uuid::new_v4();
    let result = assign_judge(&pool, "district9", CaseType::Civil, case_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none(), "No config = no auto-assignment");
}

#[tokio::test]
async fn assign_judge_respects_weights() {
    let (_app, pool, _guard) = test_app().await;
    let judge_a = create_test_judge(&pool, "district9", "Judge A Weight").await;
    let judge_b = create_test_judge(&pool, "district9", "Judge B Weight").await;
    sqlx::query("INSERT INTO judge_assignment_config (court_id, judge_id, case_type, weight) VALUES ('district9', $1, 'civil', 80)")
        .bind(judge_a).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO judge_assignment_config (court_id, judge_id, case_type, weight) VALUES ('district9', $1, 'civil', 20)")
        .bind(judge_b).execute(&pool).await.unwrap();

    // Run 100 assignments and check distribution
    let mut counts: HashMap<Uuid, u32> = HashMap::new();
    for _ in 0..100 {
        let case_id = Uuid::new_v4();
        if let Ok(Some(effect)) = assign_judge(&pool, "district9", CaseType::Civil, case_id).await {
            if let server::compliance::ComplianceEffect::AssignJudge { judge_id, .. } = effect {
                *counts.entry(judge_id).or_insert(0) += 1;
            }
        }
    }

    let a_count = counts.get(&judge_a).copied().unwrap_or(0);
    let b_count = counts.get(&judge_b).copied().unwrap_or(0);
    // With 80/20 weights over 100 trials, judge A should get significantly more
    assert!(a_count > b_count, "Judge A (weight 80) should get more assignments than Judge B (weight 20): A={}, B={}", a_count, b_count);
}
```

Add `mod compliance_assignment_tests;` to `crates/tests/src/lib.rs`.

**Step 3: Implement assignment module**

Create `crates/server/src/compliance/assignment.rs`:

```rust
use rand::Rng;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::compliance::event::{CaseType, ComplianceEffect};

struct AssignmentConfig {
    judge_id: Uuid,
    weight: i32,
}

/// Attempt to auto-assign a judge using the weighted random wheel.
/// Returns Some(AssignJudge effect) if config exists, None otherwise.
pub async fn assign_judge(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_type: CaseType,
    case_id: Uuid,
) -> Result<Option<ComplianceEffect>, shared_types::AppError> {
    let case_type_str = case_type.as_str();

    // Fetch active configs, join with judges to check caseload
    let configs: Vec<AssignmentConfig> = sqlx::query_as!(
        AssignmentConfig,
        r#"SELECT jac.judge_id, jac.weight
           FROM judge_assignment_config jac
           JOIN judges j ON j.id = jac.judge_id
           WHERE jac.court_id = $1
             AND jac.case_type = $2
             AND jac.active = true
             AND j.status = 'Active'
             AND j.current_caseload < j.max_caseload"#,
        court_id,
        case_type_str,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error_convert::SqlxErrorExt::into_app_error)?;

    if configs.is_empty() {
        return Ok(None);
    }

    let total_weight: i32 = configs.iter().map(|c| c.weight).sum();
    if total_weight == 0 {
        return Ok(None);
    }

    let mut rng = rand::thread_rng();
    let mut roll = rng.gen_range(0..total_weight);

    for config in &configs {
        if roll < config.weight {
            return Ok(Some(ComplianceEffect::AssignJudge {
                case_id,
                judge_id: config.judge_id,
                reason: format!("Random wheel assignment (weight: {}/{})", config.weight, total_weight),
            }));
        }
        roll -= config.weight;
    }

    // Fallback to first (shouldn't reach here)
    Ok(Some(ComplianceEffect::AssignJudge {
        case_id,
        judge_id: configs[0].judge_id,
        reason: "Random wheel assignment (fallback)".into(),
    }))
}
```

Add `pub mod assignment;` to `crates/server/src/compliance/mod.rs`.

**Step 4: Run migration and tests**

Run: `sqlx migrate run && cargo test -p tests compliance_assignment -- --test-threads=1`

**Step 5: Commit**

```bash
git add migrations/20260301000093* crates/server/src/compliance/assignment.rs crates/server/src/compliance/mod.rs crates/tests/src/compliance_assignment_tests.rs crates/tests/src/lib.rs crates/tests/src/common.rs
git commit -m "feat(compliance): weighted random judge assignment wheel with caseload awareness"
```

---

### Task 10: Wire Compliance Engine into Case Creation

**Files:**
- Modify: `crates/server/src/rest/case.rs` (create_case handler)
- Modify: `crates/server/src/rest/civil_case.rs` (create_civil_case handler)
- Create: `crates/tests/src/compliance_wiring_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Write tests**

Create `crates/tests/src/compliance_wiring_tests.rs`:

```rust
use crate::common::*;

#[tokio::test]
async fn creating_criminal_case_auto_creates_speedy_trial_deadline() {
    let (app, pool, _guard) = test_app().await;
    // Create a judge + assignment config so the wheel works
    let judge_id = create_test_judge(&pool, "district9", "Judge Wiring Test").await;
    sqlx::query("INSERT INTO judge_assignment_config (court_id, judge_id, case_type, weight) VALUES ('district9', $1, 'criminal', 100)")
        .bind(judge_id).execute(&pool).await.unwrap();

    let body = serde_json::json!({
        "title": "United States v. Wiring Test",
        "crime_type": "fraud"
    });
    let (status, resp) = post_json(&app, "/api/cases", &body.to_string(), "district9").await;
    assert_eq!(status, 201);

    let case_id = resp["id"].as_str().unwrap();
    // Check deadlines were auto-created
    let (dl_status, dl_body) = get_with_court(&app, &format!("/api/cases/{}/deadlines", case_id), "district9").await;
    assert_eq!(dl_status, 200);
    let deadlines = dl_body.as_array().unwrap();
    assert!(!deadlines.is_empty(), "Should auto-create deadlines for criminal case");

    // Check at least one has speedy trial citation
    let has_speedy = deadlines.iter().any(|d| {
        d["rule_code"].as_str().unwrap_or("").contains("3161")
    });
    assert!(has_speedy, "Should have Speedy Trial Act deadline");
}

#[tokio::test]
async fn creating_civil_case_auto_creates_service_deadline() {
    let (app, pool, _guard) = test_app().await;
    let judge_id = create_test_judge(&pool, "district9", "Judge Civil Wiring").await;
    sqlx::query("INSERT INTO judge_assignment_config (court_id, judge_id, case_type, weight) VALUES ('district9', $1, 'civil', 100)")
        .bind(judge_id).execute(&pool).await.unwrap();

    let body = serde_json::json!({
        "title": "Smith v. Jones (Wiring Test)",
        "nature_of_suit": "440",
        "jurisdiction_basis": "federal_question"
    });
    let (status, resp) = post_json(&app, "/api/civil-cases", &body.to_string(), "district9").await;
    assert_eq!(status, 201);

    let case_id = resp["id"].as_str().unwrap();
    let (dl_status, dl_body) = get_with_court(&app, &format!("/api/cases/{}/deadlines", case_id), "district9").await;
    assert_eq!(dl_status, 200);
    let deadlines = dl_body.as_array().unwrap();
    let has_service = deadlines.iter().any(|d| {
        d["rule_code"].as_str().unwrap_or("").contains("FRCP 4(m)")
    });
    assert!(has_service, "Should have FRCP 4(m) service of process deadline");
}

#[tokio::test]
async fn creating_case_logs_event() {
    let (app, pool, _guard) = test_app().await;
    let body = serde_json::json!({
        "title": "United States v. Event Log Test",
        "crime_type": "fraud"
    });
    let (status, resp) = post_json(&app, "/api/cases", &body.to_string(), "district9").await;
    assert_eq!(status, 201);
    let case_id = resp["id"].as_str().unwrap();

    // Check case_events table
    let events = sqlx::query!(
        "SELECT event_type FROM case_events WHERE case_id = $1::uuid",
        case_id,
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(!events.is_empty(), "Should log case_filed event");
    assert_eq!(events[0].event_type, "case_filed");
}
```

Add `mod compliance_wiring_tests;` to `crates/tests/src/lib.rs`.

**Step 2: Modify criminal case creation**

In `crates/server/src/rest/case.rs`, modify `create_case`:

After `let case = crate::repo::case::create(&pool, &court.0, body).await?;`, add:

```rust
// Compliance engine: evaluate rules and apply effects
let engine = crate::compliance::engine::ComplianceEngine::new(&pool, &court.0);
let event = crate::compliance::CaseEvent::new(
    "case_filed",
    case.id,
    crate::compliance::CaseType::Criminal,
    &court.0,
    None,
    serde_json::json!({"case_type": "criminal", "crime_type": &case.crime_type}),
);
let ctx = engine.build_context_for_new_case(case.id, crate::compliance::CaseType::Criminal, &case.case_number).await;

// Judge assignment
if let Ok(Some(assign_effect)) = crate::compliance::assignment::assign_judge(&pool, &court.0, crate::compliance::CaseType::Criminal, case.id).await {
    engine.apply_effects(&pool, &[assign_effect], &court.0).await.ok();
}

// Federal + local rules
if let Ok(effects) = engine.evaluate(&event, &ctx, None).await {
    engine.apply_effects(&pool, &effects, &court.0).await.ok();
}

// Log event
crate::repo::case_event::insert(&pool, &court.0, case.id, "criminal", "case_filed", None, &serde_json::json!({})).await.ok();
```

**Step 3: Modify civil case creation**

Similar changes in `crates/server/src/rest/civil_case.rs` `create_civil_case`, using `CaseType::Civil`.

**Step 4: Run tests**

Run: `cargo test -p tests compliance_wiring -- --test-threads=1`
Expected: All pass

Also run full suite: `cargo test -p tests -- --test-threads=1`

**Step 5: Commit**

```bash
git add crates/server/src/rest/case.rs crates/server/src/rest/civil_case.rs crates/tests/src/compliance_wiring_tests.rs crates/tests/src/lib.rs
git commit -m "feat(compliance): wire engine into case creation — auto-deadlines, judge assignment, event logging"
```

---

## Phase 4: Admin + Remaining Wiring (Tasks 11-13)

### Task 11: Wire into Status Change + Filing Submission

**Files:**
- Modify: `crates/server/src/rest/case.rs` (update_case_status handler)
- Modify: `crates/server/src/rest/civil_case.rs` (update_civil_case_status)
- Modify: `crates/server/src/rest/filing.rs` (submit_filing)
- Create: `crates/tests/src/compliance_status_wiring_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Modify status change handler**

In `update_case_status`, before calling `repo::case::update_status`:

```rust
// Compliance check: validate status transition
let engine = crate::compliance::engine::ComplianceEngine::new(&pool, &court.0);
let current_case = crate::repo::case::find_by_id(&pool, &court.0, uuid).await?
    .ok_or(AppError::not_found("Case not found"))?;
let event = crate::compliance::CaseEvent::new(
    "status_changed", uuid, crate::compliance::CaseType::Criminal, &court.0, None,
    serde_json::json!({"old_status": &current_case.status, "new_status": &body.status}),
);
let ctx = engine.load_case_context(uuid, crate::compliance::CaseType::Criminal).await?;
let effects = engine.evaluate(&event, &ctx, None).await
    .map_err(|v| AppError::bad_request(v.to_string()))?;

// Proceed with status update
let case = crate::repo::case::update_status(&pool, &court.0, uuid, &body.status).await?;

// Apply effects (new deadlines from status change)
engine.apply_effects(&pool, &effects, &court.0).await?;
crate::repo::case_event::insert(&pool, &court.0, uuid, "criminal", "status_changed", None, &serde_json::json!({"old_status": &current_case.status, "new_status": &body.status})).await.ok();
```

**Step 2: Modify filing submission**

In `submit_filing`, after the filing is created, detect the event type from the document type and fire the engine:

```rust
let event_type = match document_type {
    "motion" => "motion_filed",
    "answer" => "answer_filed",
    "response" | "reply" => "response_filed",
    _ => "filing_submitted",
};
```

Then evaluate and apply effects.

**Step 3: Write tests, run, commit**

```bash
git commit -m "feat(compliance): wire engine into status changes and filing submission"
```

---

### Task 12: Admin API — Assignment Config + Fee Schedule

**Files:**
- Create: `crates/server/src/rest/admin_assignment.rs`
- Create: `crates/server/src/rest/admin_fees.rs`
- Modify: `crates/server/src/rest/mod.rs`
- Create: `crates/tests/src/admin_assignment_tests.rs`
- Create: `crates/tests/src/admin_fee_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Implement assignment config API**

Endpoints:
- `GET /api/admin/assignment-config` — list all weights for court
- `PUT /api/admin/assignment-config` — bulk update weights `[{judge_id, case_type, weight, active}]`
- `GET /api/admin/assignment-stats` — actual vs target stats

All require `RoleRequired("admin")` or equivalent.

**Step 2: Implement fee schedule admin API**

Endpoints (from CM/ECF plan, adapt filing_fee_schedule table if already created by that plan):
- `GET /api/admin/fee-schedule` — list fees
- `POST /api/admin/fee-schedule` — create fee entry
- `PUT /api/admin/fee-schedule/:id` — update fee
- `DELETE /api/admin/fee-schedule/:id` — deactivate fee

**Step 3: Tests, commit**

```bash
git commit -m "feat(admin): assignment config and fee schedule REST endpoints"
```

---

### Task 13: Admin UI Pages

**Files:**
- Create: `crates/app/src/routes/admin/assignment.rs`
- Create: `crates/app/src/routes/admin/fees.rs`
- Modify: `crates/app/src/routes/admin/mod.rs` (or create if needed)
- Modify: `crates/app/src/routes/mod.rs` (add routes)

**Step 1: Assignment wheel admin page**

Components:
- Tabs for Criminal / Civil
- DataTable with judge name, weight slider, active toggle
- Save button calls `PUT /api/admin/assignment-config`
- Stats section showing actual distribution

**Step 2: Fee schedule admin page**

Components:
- DataTable with entry_type, fee_cents (displayed as dollars), description, active
- Add/Edit via Sheet component
- Save calls POST/PUT to `/api/admin/fee-schedule`

**Step 3: Verify compilation, commit**

```bash
git commit -m "feat(admin): UI pages for judge assignment wheel and fee schedule management"
```

---

## Phase 5: Cleanup (Task 14)

### Task 14: Final Integration Test and Cleanup

**Files:**
- Modify: `crates/tests/src/common.rs` (ensure all new tables in TRUNCATE)
- Run full test suite

**Step 1: Update TRUNCATE**

Add to the TRUNCATE list: `case_events, judge_assignment_config`

**Step 2: Run full test suite**

Run: `cargo test -p tests -- --test-threads=1`
Expected: All tests pass (existing + new compliance tests)

**Step 3: Verify workspace compilation**

Run: `cargo check --workspace`

**Step 4: Commit**

```bash
git commit -m "chore: final cleanup — TRUNCATE updates, full test suite passing"
```

---

## Summary

| Phase | Tasks | What It Builds |
|-------|-------|---------------|
| 1. Foundation | 1-4 | case_events table, core types, FRCP 6(a) calendar, status transitions |
| 2. Federal Rules | 5-7 | FRCP civil rules, FRCrP + Speedy Trial, local rules evaluator |
| 3. Engine + Assignment | 8-10 | Orchestrator, judge wheel, wire into case creation |
| 4. Admin + Wiring | 11-13 | Status/filing wiring, admin APIs, admin UI |
| 5. Cleanup | 14 | TRUNCATE, full test suite |

**Total: 14 tasks, 1 new table (case_events) + 1 new table (judge_assignment_config), ~8 new Rust modules, 16+ new tests**
