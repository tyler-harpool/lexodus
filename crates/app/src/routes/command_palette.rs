use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdSearch;
use dioxus_free_icons::Icon;
use serde::Deserialize;

use crate::routes::Route;
use crate::CourtContext;

/// Maximum number of search results to fetch from the server.
const SEARCH_RESULT_LIMIT: usize = 10;

/// Client-side mirror of `server::search::SearchResult`.
/// Deserialized from the JSON string returned by `global_search`.
#[derive(Debug, Clone, Deserialize)]
struct SearchResult {
    id: String,
    entity_type: String,
    title: String,
    subtitle: String,
}

/// Human-readable group labels keyed by entity_type values from the search index.
const ENTITY_TYPE_LABELS: &[(&str, &str)] = &[
    ("case", "Criminal Cases"),
    ("civil_case", "Civil Cases"),
    ("attorney", "Attorneys"),
    ("judge", "Judges"),
    ("docket", "Docket Entries"),
    ("calendar", "Calendar Events"),
    ("deadline", "Deadlines"),
    ("order", "Orders"),
    ("opinion", "Opinions"),
];

/// Converts a `SearchResult` into the `Route` it should navigate to.
fn route_for_result(result: &SearchResult) -> Route {
    match result.entity_type.as_str() {
        "case" => Route::CaseDetail {
            id: result.id.clone(),
        },
        // Civil cases route to CaseDetail for now; a dedicated CivilCaseDetail
        // route will be added when the civil case UI is built.
        "civil_case" => Route::CaseDetail {
            id: result.id.clone(),
        },
        "attorney" => Route::AttorneyDetail {
            id: result.id.clone(),
        },
        "judge" => Route::JudgeDetail {
            id: result.id.clone(),
        },
        "opinion" => Route::OpinionDetail {
            id: result.id.clone(),
        },
        // Docket entries, calendar events, deadlines, orders all link to their parent case.
        // The search result `id` is the entity's own id; for now we navigate to CaseDetail
        // since these entity detail pages are integrated into the case view.
        _ => Route::CaseDetail {
            id: result.id.clone(),
        },
    }
}

/// Groups results by entity_type while preserving the display ordering defined in
/// `ENTITY_TYPE_LABELS`. Returns a vec of (group_label, items_in_group).
fn group_results(results: &[SearchResult]) -> Vec<(&str, Vec<&SearchResult>)> {
    let mut groups: Vec<(&str, Vec<&SearchResult>)> = Vec::new();

    for (type_key, label) in ENTITY_TYPE_LABELS {
        let items: Vec<&SearchResult> = results
            .iter()
            .filter(|r| r.entity_type == *type_key)
            .collect();
        if !items.is_empty() {
            groups.push((label, items));
        }
    }

    // Include any entity types not in our predefined list at the end.
    let known_types: Vec<&str> = ENTITY_TYPE_LABELS.iter().map(|(k, _)| *k).collect();
    let other: Vec<&SearchResult> = results
        .iter()
        .filter(|r| !known_types.contains(&r.entity_type.as_str()))
        .collect();
    if !other.is_empty() {
        groups.push(("Other", other));
    }

    groups
}

/// Global command palette overlay, toggled with Cmd+K / Ctrl+K or the search icon.
///
/// Provides full-text search across all court entities (cases, attorneys, judges,
/// docket entries, calendar events, deadlines, orders, opinions) powered by Tantivy.
#[component]
pub fn CommandPalette(show: Signal<bool>) -> Element {
    let ctx = use_context::<CourtContext>();
    let nav = navigator();

    let mut query = use_signal(String::new);
    let mut results: Signal<Vec<SearchResult>> = use_signal(Vec::new);
    let mut active_index = use_signal(|| 0usize);
    let mut loading = use_signal(|| false);

    // Close helper: reset state when closing the palette.
    let mut close = move || {
        show.set(false);
        query.set(String::new());
        results.set(Vec::new());
        active_index.set(0);
    };

    // Navigate to the given result and close the palette.
    let mut navigate_to = {
        let nav = nav.clone();
        move |result: &SearchResult| {
            let route = route_for_result(result);
            nav.push(route);
            close();
        }
    };

    if !show() {
        return rsx! {};
    }

    // Flatten all results for keyboard navigation indexing.
    let all_results = results.read().clone();
    let total_results = all_results.len();
    let groups = group_results(&all_results);
    let current_query = query.read().clone();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./command_palette.css") }

        // Backdrop
        div {
            class: "cmd-palette-backdrop",
            onclick: move |_| close(),
        }

        // Modal
        div {
            class: "cmd-palette-modal",
            // Keyboard handling inside the palette
            onkeydown: move |e: KeyboardEvent| {
                let key = e.key();
                match key {
                    Key::Escape => {
                        e.prevent_default();
                        close();
                    }
                    Key::ArrowDown => {
                        e.prevent_default();
                        if total_results > 0 {
                            let current = *active_index.read();
                            active_index.set((current + 1) % total_results);
                        }
                    }
                    Key::ArrowUp => {
                        e.prevent_default();
                        if total_results > 0 {
                            let current = *active_index.read();
                            if current == 0 {
                                active_index.set(total_results - 1);
                            } else {
                                active_index.set(current - 1);
                            }
                        }
                    }
                    Key::Enter => {
                        e.prevent_default();
                        let idx = *active_index.read();
                        let results_snapshot = results.read().clone();
                        if let Some(result) = results_snapshot.get(idx) {
                            navigate_to(result);
                        }
                    }
                    _ => {}
                }
            },

            // Search input
            div {
                class: "cmd-palette-input-wrap",
                span {
                    class: "cmd-palette-search-icon",
                    Icon::<LdSearch> { icon: LdSearch, width: 18, height: 18 }
                }
                input {
                    class: "cmd-palette-input",
                    placeholder: "Search cases, people, docket entries...",
                    value: "{query}",
                    autofocus: true,
                    oninput: move |e: FormEvent| {
                        let value = e.value();
                        query.set(value.clone());
                        active_index.set(0);

                        if value.trim().is_empty() {
                            results.set(Vec::new());
                            return;
                        }

                        let court_id = ctx.court_id.read().clone();
                        loading.set(true);
                        spawn(async move {
                            match server::api::global_search(
                                court_id,
                                value,
                                Some(SEARCH_RESULT_LIMIT),
                            )
                            .await
                            {
                                Ok(json) => {
                                    let parsed: Vec<SearchResult> =
                                        serde_json::from_str(&json).unwrap_or_default();
                                    results.set(parsed);
                                }
                                Err(_) => {
                                    results.set(Vec::new());
                                }
                            }
                            loading.set(false);
                        });
                    },
                }
                span { class: "cmd-palette-kbd", "ESC" }
            }

            // Results area
            div {
                class: "cmd-palette-results",

                if current_query.trim().is_empty() {
                    // Empty state: no query typed yet
                    div {
                        class: "cmd-palette-empty",
                        "Search cases, people, docket entries..."
                    }
                } else if *loading.read() && all_results.is_empty() {
                    // Loading spinner
                    div {
                        class: "cmd-palette-loading",
                        "Searching..."
                    }
                } else if all_results.is_empty() {
                    // No results
                    div {
                        class: "cmd-palette-empty",
                        "No results found for '{current_query}'"
                    }
                } else {
                    // Grouped results
                    {
                        // Build a flat index counter across all groups so keyboard
                        // navigation maps to the correct item.
                        let mut flat_idx: usize = 0;
                        let current_active = *active_index.read();

                        rsx! {
                            for (label, items) in groups.iter() {
                                div {
                                    class: "cmd-palette-group-header",
                                    "{label}"
                                }
                                for item in items.iter() {
                                    {
                                        let idx = flat_idx;
                                        flat_idx += 1;
                                        let is_active = idx == current_active;
                                        let result = (*item).clone();
                                        let result_for_click = result.clone();
                                        rsx! {
                                            button {
                                                class: if is_active { "cmd-palette-result cmd-palette-result-active" } else { "cmd-palette-result" },
                                                onmouseenter: move |_| {
                                                    active_index.set(idx);
                                                },
                                                onclick: move |_| {
                                                    navigate_to(&result_for_click);
                                                },
                                                span {
                                                    class: "cmd-palette-result-title",
                                                    "{result.title}"
                                                }
                                                span {
                                                    class: "cmd-palette-result-subtitle",
                                                    "{result.subtitle}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
