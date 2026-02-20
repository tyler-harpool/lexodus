use serde::{Deserialize, Serialize};

/// A single search result returned by the global full-text search.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub entity_type: String,
    pub title: String,
    pub subtitle: String,
    /// For child entities (docket, calendar, deadline, order), the parent case ID.
    /// Top-level entities (case, attorney, judge, opinion) leave this empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}
