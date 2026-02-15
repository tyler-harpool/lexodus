use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Opinion validation constants ───────────────────────────────────

/// Valid opinion type values matching the DB CHECK constraint.
pub const OPINION_TYPES: &[&str] = &[
    "Majority", "Concurrence", "Dissent", "Per Curiam",
    "Memorandum", "En Banc", "Summary", "Other",
];

/// Valid opinion disposition values matching the DB CHECK constraint.
pub const OPINION_DISPOSITIONS: &[&str] = &[
    "Affirmed", "Reversed", "Remanded", "Vacated",
    "Dismissed", "Modified", "Certified",
];

/// Valid opinion status values matching the DB CHECK constraint.
pub const OPINION_STATUSES: &[&str] = &[
    "Draft", "Under Review", "Circulated", "Filed",
    "Published", "Withdrawn", "Superseded",
];

/// Valid vote type values matching the DB CHECK constraint.
pub const VOTE_TYPES: &[&str] = &[
    "Join", "Concur", "Concur in Part", "Dissent",
    "Dissent in Part", "Recused", "Not Participating",
];

/// Valid citation type values matching the DB CHECK constraint.
pub const CITATION_TYPES: &[&str] = &[
    "Followed", "Distinguished", "Overruled", "Cited",
    "Discussed", "Criticized", "Questioned", "Harmonized",
    "Parallel", "Other",
];

/// Valid draft status values matching the DB CHECK constraint.
pub const DRAFT_STATUSES: &[&str] = &[
    "Draft", "Under Review", "Approved", "Rejected", "Superseded",
];

/// Check whether an opinion type string is valid.
pub fn is_valid_opinion_type(s: &str) -> bool {
    OPINION_TYPES.contains(&s)
}

/// Check whether an opinion disposition string is valid.
pub fn is_valid_opinion_disposition(s: &str) -> bool {
    OPINION_DISPOSITIONS.contains(&s)
}

/// Check whether an opinion status string is valid.
pub fn is_valid_opinion_status(s: &str) -> bool {
    OPINION_STATUSES.contains(&s)
}

/// Check whether a vote type string is valid.
pub fn is_valid_vote_type(s: &str) -> bool {
    VOTE_TYPES.contains(&s)
}

/// Check whether a citation type string is valid.
pub fn is_valid_citation_type(s: &str) -> bool {
    CITATION_TYPES.contains(&s)
}

/// Check whether a draft status string is valid.
pub fn is_valid_draft_status(s: &str) -> bool {
    DRAFT_STATUSES.contains(&s)
}

// ── Opinion list query params ─────────────────────────────────────

/// Query parameters for listing opinions with optional filters.
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct OpinionListParams {
    pub case_id: Option<String>,
    pub author_judge_id: Option<String>,
    pub is_published: Option<bool>,
    pub is_precedential: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ── JudicialOpinion DB struct ──────────────────────────────────────

/// A written judicial opinion for a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct JudicialOpinion {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub case_name: String,
    pub docket_number: String,
    pub author_judge_id: Uuid,
    pub author_judge_name: String,
    /// OpinionType enum stored as text (e.g. "Majority", "Concurring", "Dissenting").
    pub opinion_type: String,
    /// Disposition enum stored as text (e.g. "Affirmed", "Reversed", "Remanded").
    pub disposition: String,
    pub title: String,
    pub syllabus: String,
    pub content: String,
    /// OpinionStatus enum stored as text (e.g. "Draft", "Filed", "Published").
    pub status: String,
    pub is_published: bool,
    pub is_precedential: bool,
    pub citation_volume: Option<String>,
    pub citation_reporter: Option<String>,
    pub citation_page: Option<String>,
    pub filed_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub keywords: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── JudicialOpinion API response ───────────────────────────────────

/// API response shape for a judicial opinion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JudicialOpinionResponse {
    pub id: String,
    pub court_id: String,
    pub case_id: String,
    pub case_name: String,
    pub docket_number: String,
    pub author_judge_id: String,
    pub author_judge_name: String,
    pub opinion_type: String,
    pub disposition: String,
    pub title: String,
    pub syllabus: String,
    pub content: String,
    pub status: String,
    pub is_published: bool,
    pub is_precedential: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_reporter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_page: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    pub keywords: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<JudicialOpinion> for JudicialOpinionResponse {
    fn from(o: JudicialOpinion) -> Self {
        Self {
            id: o.id.to_string(),
            court_id: o.court_id,
            case_id: o.case_id.to_string(),
            case_name: o.case_name,
            docket_number: o.docket_number,
            author_judge_id: o.author_judge_id.to_string(),
            author_judge_name: o.author_judge_name,
            opinion_type: o.opinion_type,
            disposition: o.disposition,
            title: o.title,
            syllabus: o.syllabus,
            content: o.content,
            status: o.status,
            is_published: o.is_published,
            is_precedential: o.is_precedential,
            citation_volume: o.citation_volume,
            citation_reporter: o.citation_reporter,
            citation_page: o.citation_page,
            filed_at: o.filed_at.map(|dt| dt.to_rfc3339()),
            published_at: o.published_at.map(|dt| dt.to_rfc3339()),
            keywords: o.keywords,
            created_at: o.created_at.to_rfc3339(),
            updated_at: o.updated_at.to_rfc3339(),
        }
    }
}

// ── JudicialOpinion request types ──────────────────────────────────

/// Request to create a new judicial opinion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateJudicialOpinionRequest {
    pub case_id: Uuid,
    pub case_name: String,
    pub docket_number: String,
    pub author_judge_id: Uuid,
    pub author_judge_name: String,
    pub opinion_type: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub disposition: Option<String>,
    #[serde(default)]
    pub syllabus: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
}

/// Request to update a judicial opinion (all fields optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateJudicialOpinionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub syllabus: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
}

// ── OpinionVote DB struct ──────────────────────────────────────────

/// A judge's vote on a panel opinion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct OpinionVote {
    pub id: Uuid,
    pub court_id: String,
    pub opinion_id: Uuid,
    pub judge_id: Uuid,
    /// VoteType enum stored as text (e.g. "Join", "Concur", "Dissent").
    pub vote_type: String,
    pub joined_at: DateTime<Utc>,
    pub notes: Option<String>,
}

// ── OpinionVote API response ───────────────────────────────────────

/// API response shape for an opinion vote.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OpinionVoteResponse {
    pub id: String,
    pub opinion_id: String,
    pub judge_id: String,
    pub vote_type: String,
    pub joined_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl From<OpinionVote> for OpinionVoteResponse {
    fn from(v: OpinionVote) -> Self {
        Self {
            id: v.id.to_string(),
            opinion_id: v.opinion_id.to_string(),
            judge_id: v.judge_id.to_string(),
            vote_type: v.vote_type,
            joined_at: v.joined_at.to_rfc3339(),
            notes: v.notes,
        }
    }
}

// ── OpinionVote request types ──────────────────────────────────────

/// Request to create a new opinion vote.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateOpinionVoteRequest {
    pub judge_id: Uuid,
    pub vote_type: String,
    #[serde(default)]
    pub notes: Option<String>,
}

// ── OpinionCitation DB struct ──────────────────────────────────────

/// A legal citation within an opinion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct OpinionCitation {
    pub id: Uuid,
    pub court_id: String,
    pub opinion_id: Uuid,
    pub cited_opinion_id: Option<Uuid>,
    pub citation_text: String,
    /// CitationType enum stored as text (e.g. "Followed", "Distinguished", "Overruled").
    pub citation_type: String,
    pub context: Option<String>,
    pub pinpoint_cite: Option<String>,
}

// ── OpinionCitation API response ───────────────────────────────────

/// API response shape for an opinion citation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OpinionCitationResponse {
    pub id: String,
    pub opinion_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cited_opinion_id: Option<String>,
    pub citation_text: String,
    pub citation_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinpoint_cite: Option<String>,
}

impl From<OpinionCitation> for OpinionCitationResponse {
    fn from(c: OpinionCitation) -> Self {
        Self {
            id: c.id.to_string(),
            opinion_id: c.opinion_id.to_string(),
            cited_opinion_id: c.cited_opinion_id.map(|u| u.to_string()),
            citation_text: c.citation_text,
            citation_type: c.citation_type,
            context: c.context,
            pinpoint_cite: c.pinpoint_cite,
        }
    }
}

// ── OpinionCitation request types ──────────────────────────────────

/// Request to create a new opinion citation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateOpinionCitationRequest {
    pub citation_text: String,
    pub citation_type: String,
    #[serde(default)]
    pub cited_opinion_id: Option<Uuid>,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub pinpoint_cite: Option<String>,
}

// ── Headnote DB struct ─────────────────────────────────────────────

/// A headnote summarizing a legal point in an opinion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Headnote {
    pub id: Uuid,
    pub court_id: String,
    pub opinion_id: Uuid,
    pub headnote_number: i32,
    pub topic: String,
    pub text: String,
    pub key_number: Option<String>,
}

// ── Headnote API response ──────────────────────────────────────────

/// API response shape for a headnote.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HeadnoteResponse {
    pub id: String,
    pub opinion_id: String,
    pub headnote_number: i32,
    pub topic: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_number: Option<String>,
}

impl From<Headnote> for HeadnoteResponse {
    fn from(h: Headnote) -> Self {
        Self {
            id: h.id.to_string(),
            opinion_id: h.opinion_id.to_string(),
            headnote_number: h.headnote_number,
            topic: h.topic,
            text: h.text,
            key_number: h.key_number,
        }
    }
}

// ── Headnote request types ─────────────────────────────────────────

/// Request to create a new headnote.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateHeadnoteRequest {
    pub headnote_number: i32,
    pub topic: String,
    pub text: String,
    #[serde(default)]
    pub key_number: Option<String>,
}

// ── OpinionDraft DB struct ─────────────────────────────────────────

/// A draft version of a judicial opinion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct OpinionDraft {
    pub id: Uuid,
    pub court_id: String,
    pub opinion_id: Uuid,
    pub version: i32,
    pub content: String,
    /// DraftStatus enum stored as text (e.g. "InProgress", "UnderReview", "Final").
    pub status: String,
    pub author_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ── OpinionDraft API response ──────────────────────────────────────

/// API response shape for an opinion draft.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OpinionDraftResponse {
    pub id: String,
    pub opinion_id: String,
    pub version: i32,
    pub content: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<String>,
    pub created_at: String,
}

impl From<OpinionDraft> for OpinionDraftResponse {
    fn from(d: OpinionDraft) -> Self {
        Self {
            id: d.id.to_string(),
            opinion_id: d.opinion_id.to_string(),
            version: d.version,
            content: d.content,
            status: d.status,
            author_id: d.author_id,
            created_at: d.created_at.to_rfc3339(),
        }
    }
}

// ── OpinionDraft request types ─────────────────────────────────────

/// Request to create a new opinion draft.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateOpinionDraftRequest {
    pub content: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub author_id: Option<String>,
}

// ── DraftComment DB struct ─────────────────────────────────────────

/// A comment on an opinion draft during the review process.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct DraftComment {
    pub id: Uuid,
    pub court_id: String,
    pub draft_id: Uuid,
    pub author: String,
    pub content: String,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ── DraftComment API response ──────────────────────────────────────

/// API response shape for a draft comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DraftCommentResponse {
    pub id: String,
    pub draft_id: String,
    pub author: String,
    pub content: String,
    pub resolved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    pub created_at: String,
}

impl From<DraftComment> for DraftCommentResponse {
    fn from(c: DraftComment) -> Self {
        Self {
            id: c.id.to_string(),
            draft_id: c.draft_id.to_string(),
            author: c.author,
            content: c.content,
            resolved: c.resolved,
            resolved_at: c.resolved_at.map(|dt| dt.to_rfc3339()),
            created_at: c.created_at.to_rfc3339(),
        }
    }
}

// ── DraftComment request types ─────────────────────────────────────

/// Request to create a new draft comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateDraftCommentRequest {
    pub author: String,
    pub content: String,
}

// ── Opinion statistics types ───────────────────────────────────────

/// Aggregate statistics about opinions in a court.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OpinionStatistics {
    pub total: i64,
    pub by_type: serde_json::Value,
    pub precedential_count: i64,
    pub avg_days_to_publish: Option<f64>,
}

/// Citation statistics for opinions in a court.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CitationStatistics {
    pub total_citations: i64,
    pub most_cited: Vec<serde_json::Value>,
}
