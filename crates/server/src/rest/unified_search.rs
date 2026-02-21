use axum::extract::{Query, State};
use axum::Json;
use shared_types::{UnifiedSearchParams, UnifiedSearchResponse};

use crate::db::AppState;

/// GET /api/search/unified?q=...&courts=...&entity_types=...&page=1&per_page=20
///
/// Cross-court full-text search. Pass `courts=all` or omit to search every court.
/// Pass `courts=district9,district12` to search specific courts.
/// Pass `entity_types=case,attorney` to filter by entity type.
pub async fn unified_search(
    State(state): State<AppState>,
    Query(params): Query<UnifiedSearchParams>,
) -> Json<UnifiedSearchResponse> {
    let court_ids: Vec<String> = params
        .courts
        .as_deref()
        .filter(|c| !c.is_empty() && *c != "all")
        .map(|c| c.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let entity_types: Vec<String> = params
        .entity_types
        .as_deref()
        .filter(|e| !e.is_empty())
        .map(|e| e.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let response = state.search.unified_search(
        &params.q,
        &court_ids,
        &entity_types,
        page,
        per_page,
    );

    Json(response)
}
