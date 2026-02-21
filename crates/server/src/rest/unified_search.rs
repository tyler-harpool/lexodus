use axum::extract::{Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::Json;
use shared_types::{UnifiedSearchParams, UnifiedSearchResponse};

use crate::db::AppState;

/// Maximum number of results allowed in a CSV export.
const EXPORT_MAX_RESULTS: i64 = 500;

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

/// GET /api/search/export?q=...&courts=...&entity_types=...
///
/// Exports unified search results as a CSV file. Uses the same query parameters
/// as the unified search endpoint but returns up to 500 results as a downloadable
/// CSV with columns: ID, Entity Type, Court, Title, Subtitle.
pub async fn export_search(
    State(state): State<AppState>,
    Query(params): Query<UnifiedSearchParams>,
) -> impl IntoResponse {
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

    let response = state.search.unified_search(
        &params.q,
        &court_ids,
        &entity_types,
        1,
        EXPORT_MAX_RESULTS,
    );

    // Build CSV content
    let mut csv = String::from("ID,Entity Type,Court,Title,Subtitle\n");
    for result in &response.results {
        // Escape fields that may contain commas or quotes
        csv.push_str(&csv_escape(&result.id));
        csv.push(',');
        csv.push_str(&csv_escape(&result.entity_type));
        csv.push(',');
        csv.push_str(&csv_escape(&result.court_id));
        csv.push(',');
        csv.push_str(&csv_escape(&result.title));
        csv.push(',');
        csv.push_str(&csv_escape(&result.subtitle));
        csv.push('\n');
    }

    (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"search-results.csv\"",
            ),
        ],
        csv,
    )
}

/// Escapes a string value for safe inclusion in a CSV field.
/// Wraps the value in double quotes if it contains commas, quotes, or newlines.
fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
