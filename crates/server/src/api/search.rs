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
