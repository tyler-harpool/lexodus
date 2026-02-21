use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::db::get_db;

// ── Global Search ──────────────────────────────────────────────────

/// Full-text search across all court entities (cases, attorneys, judges, etc.).
/// Returns a list of SearchResult objects matching the query within the given court.
#[server]
pub async fn global_search(
    court_id: String,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<shared_types::SearchResult>, ServerFnError> {
    // Ensure DB + search index are initialized.
    let _pool = get_db().await;
    let search = crate::search::get_search();
    let max_results = limit.unwrap_or(20);

    let results = search.search(&query, &court_id, max_results);
    Ok(results)
}

/// Cross-court full-text search. Returns unified results with court badges and facets.
/// Pass `courts` as "all" or comma-separated court IDs (e.g., "district9,district12").
/// Pass `entity_types` as comma-separated filters (e.g., "case,attorney").
#[server]
pub async fn unified_search(
    query: String,
    courts: Option<String>,
    entity_types: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<shared_types::UnifiedSearchResponse, ServerFnError> {
    let _pool = get_db().await;
    let search = crate::search::get_search();

    let court_ids: Vec<String> = courts
        .as_deref()
        .filter(|c| !c.is_empty() && *c != "all")
        .map(|c| c.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let entity_types_vec: Vec<String> = entity_types
        .as_deref()
        .filter(|e| !e.is_empty())
        .map(|e| e.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let p = page.unwrap_or(1).max(1);
    let pp = per_page.unwrap_or(20).clamp(1, 100);

    let response = search.unified_search(&query, &court_ids, &entity_types_vec, p, pp);
    Ok(response)
}
