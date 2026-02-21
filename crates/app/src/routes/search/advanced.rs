use dioxus::prelude::*;
use shared_types::{UnifiedSearchResponse, UnifiedSearchResult};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, FormSelect, PageHeader,
    PageTitle, Skeleton,
};

use crate::routes::Route;
use crate::CourtContext;

/// Number of results per page for the advanced search view.
const PER_PAGE: i64 = 20;

/// Entity type options available for filtering.
const ENTITY_TYPE_OPTIONS: &[(&str, &str)] = &[
    ("", "All Types"),
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

/// Navigates to the appropriate route for a given search result.
fn route_for_result(result: &UnifiedSearchResult) -> Route {
    match result.entity_type.as_str() {
        "case" | "civil_case" => Route::CaseDetail {
            id: result.id.clone(),
            tab: None,
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
        "docket" | "calendar" | "deadline" | "order" => Route::CaseDetail {
            id: result
                .parent_id
                .clone()
                .unwrap_or_else(|| result.id.clone()),
            tab: match result.entity_type.as_str() {
                "docket" => Some("docket".to_string()),
                "calendar" => Some("scheduling".to_string()),
                "order" => Some("docket".to_string()),
                _ => None,
            },
        },
        _ => Route::CaseDetail {
            id: result.id.clone(),
            tab: None,
        },
    }
}

/// Returns a `BadgeVariant` for a given entity type string.
fn entity_badge_variant(entity_type: &str) -> BadgeVariant {
    match entity_type {
        "case" | "civil_case" => BadgeVariant::Primary,
        "attorney" | "judge" => BadgeVariant::Secondary,
        "docket" | "calendar" | "deadline" => BadgeVariant::Outline,
        "order" | "opinion" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}

/// Advanced search page with filters, faceted results, and pagination.
#[component]
pub fn AdvancedSearchPage() -> Element {
    let ctx = use_context::<CourtContext>();

    let mut search_query = use_signal(String::new);
    let mut court_filter = use_signal(String::new);
    let mut entity_type_filter = use_signal(String::new);
    let mut offset = use_signal(|| 0i64);
    let mut loading = use_signal(|| false);
    let mut response: Signal<Option<UnifiedSearchResponse>> = use_signal(|| None);

    // Compute the current page from offset
    let current_page = (*offset.read() / PER_PAGE) + 1;

    // Perform the search
    let do_search = move |page: i64| {
        let q = search_query.read().clone();
        if q.trim().is_empty() {
            return;
        }

        let courts = court_filter.read().clone();
        let entity_types = entity_type_filter.read().clone();

        loading.set(true);
        spawn(async move {
            let courts_param = if courts.is_empty() {
                Some("all".to_string())
            } else {
                Some(courts)
            };
            let entity_types_param = if entity_types.is_empty() {
                None
            } else {
                Some(entity_types)
            };

            match server::api::unified_search(
                q,
                courts_param,
                entity_types_param,
                Some(page),
                Some(PER_PAGE),
            )
            .await
            {
                Ok(resp) => {
                    response.set(Some(resp));
                }
                Err(_) => {
                    response.set(None);
                }
            }
            loading.set(false);
        });
    };

    // Clone for use in the search button handler
    let mut do_search_for_btn = do_search.clone();
    let mut do_search_for_enter = do_search.clone();

    let resp_data = response.read().clone();
    let total = resp_data.as_ref().map(|r| r.total).unwrap_or(0);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./advanced.css") }

        PageHeader {
            PageTitle { "Advanced Search" }
        }

        // Search controls
        Card {
            CardContent {
                div {
                    class: "adv-search-controls",
                    // Search input
                    div {
                        class: "adv-search-input-row",
                        input {
                            class: "adv-search-input",
                            placeholder: "Search across all courts and entities...",
                            value: "{search_query}",
                            oninput: move |e: FormEvent| {
                                search_query.set(e.value());
                            },
                            onkeydown: move |e: KeyboardEvent| {
                                if matches!(e.key(), Key::Enter) {
                                    e.prevent_default();
                                    offset.set(0);
                                    do_search_for_enter(1);
                                }
                            },
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                offset.set(0);
                                do_search_for_btn(1);
                            },
                            "Search"
                        }
                    }

                    // Filter dropdowns
                    div {
                        class: "adv-search-filters",
                        FormSelect {
                            label: "Court".to_string(),
                            value: court_filter.read().clone(),
                            onchange: move |e: Event<FormData>| {
                                court_filter.set(e.value());
                            },
                            option { value: "", "All Courts" }
                            option { value: "{ctx.court_id}", "{ctx.court_id}" }
                        }

                        FormSelect {
                            label: "Entity Type".to_string(),
                            value: entity_type_filter.read().clone(),
                            onchange: move |e: Event<FormData>| {
                                entity_type_filter.set(e.value());
                            },
                            for (value, label) in ENTITY_TYPE_OPTIONS.iter() {
                                option { value: *value, "{label}" }
                            }
                        }
                    }
                }
            }
        }

        // Facets
        if let Some(ref resp) = resp_data {
            if !resp.facets.by_court.is_empty() || !resp.facets.by_entity_type.is_empty() {
                div {
                    class: "adv-search-facets",
                    // Court facets
                    if !resp.facets.by_court.is_empty() {
                        div {
                            class: "adv-search-facet-group",
                            span { class: "adv-search-facet-label", "Courts:" }
                            for (court, count) in resp.facets.by_court.iter() {
                                Badge {
                                    variant: BadgeVariant::Outline,
                                    "{court} ({count})"
                                }
                            }
                        }
                    }
                    // Entity type facets
                    if !resp.facets.by_entity_type.is_empty() {
                        div {
                            class: "adv-search-facet-group",
                            span { class: "adv-search-facet-label", "Types:" }
                            for (entity_type, count) in resp.facets.by_entity_type.iter() {
                                Badge {
                                    variant: entity_badge_variant(entity_type),
                                    "{entity_type} ({count})"
                                }
                            }
                        }
                    }
                }
            }
        }

        // Loading state
        if *loading.read() {
            div {
                class: "adv-search-loading",
                Skeleton { width: "100%", height: "60px" }
                Skeleton { width: "100%", height: "60px" }
                Skeleton { width: "100%", height: "60px" }
            }
        } else if let Some(ref resp) = resp_data {
            if resp.results.is_empty() {
                div {
                    class: "adv-search-empty",
                    "No results found. Try a different query or adjust your filters."
                }
            } else {
                // Results list
                div {
                    class: "adv-search-results",
                    for result in resp.results.iter() {
                        {
                            let result_route = route_for_result(result);
                            let entity_type = result.entity_type.clone();
                            let court_id = result.court_id.clone();
                            rsx! {
                                Link {
                                    to: result_route,
                                    class: "adv-search-result-link",
                                    div {
                                        class: "adv-search-result",
                                        div {
                                            class: "adv-search-result-badges",
                                            Badge {
                                                variant: entity_badge_variant(&entity_type),
                                                "{entity_type}"
                                            }
                                            Badge {
                                                variant: BadgeVariant::Outline,
                                                "{court_id}"
                                            }
                                        }
                                        div {
                                            class: "adv-search-result-body",
                                            span {
                                                class: "adv-search-result-title",
                                                "{result.title}"
                                            }
                                            span {
                                                class: "adv-search-result-subtitle",
                                                "{result.subtitle}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Pagination
                {
                    let mut do_search_for_pagination = do_search.clone();
                    rsx! {
                        div {
                            class: "adv-search-pagination",
                            if current_page > 1 {
                                Button {
                                    variant: ButtonVariant::Outline,
                                    onclick: move |_| {
                                        let new_offset = (*offset.read() - PER_PAGE).max(0);
                                        offset.set(new_offset);
                                        let new_page = (new_offset / PER_PAGE) + 1;
                                        do_search_for_pagination(new_page);
                                    },
                                    "Previous"
                                }
                            }
                            span {
                                class: "adv-search-pagination-info",
                                "Page {current_page} of {((total + PER_PAGE - 1) / PER_PAGE).max(1)} ({total} results)"
                            }
                            if current_page < ((total + PER_PAGE - 1) / PER_PAGE) {
                                {
                                    let mut do_search_for_next = do_search.clone();
                                    rsx! {
                                        Button {
                                            variant: ButtonVariant::Outline,
                                            onclick: move |_| {
                                                let new_offset = *offset.read() + PER_PAGE;
                                                offset.set(new_offset);
                                                let new_page = (new_offset / PER_PAGE) + 1;
                                                do_search_for_next(new_page);
                                            },
                                            "Next"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Initial state: no search performed yet
            div {
                class: "adv-search-empty",
                "Enter a search query and press Search to find results across all courts."
            }
        }
    }
}
