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

/// A search result from the unified cross-court search endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedSearchResult {
    pub id: String,
    pub entity_type: String,
    pub court_id: String,
    pub title: String,
    pub subtitle: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
    pub score: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

/// Faceted counts returned alongside unified search results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchFacets {
    pub by_court: Vec<(String, i64)>,
    pub by_entity_type: Vec<(String, i64)>,
}

/// Full response from the unified search endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedSearchResponse {
    pub results: Vec<UnifiedSearchResult>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub facets: SearchFacets,
}

/// Parameters for the unified search endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchParams {
    pub q: String,
    #[serde(default)]
    pub courts: Option<String>,
    #[serde(default)]
    pub entity_types: Option<String>,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub per_page: Option<i64>,
}
