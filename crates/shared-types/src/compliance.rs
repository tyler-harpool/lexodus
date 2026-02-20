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
    Statutory,      // weight 10
    FederalRule,    // weight 20
    Administrative, // weight 30
    Local,          // weight 40
    StandingOrder,  // weight 50
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
