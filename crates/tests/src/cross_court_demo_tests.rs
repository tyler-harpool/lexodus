use axum::http::StatusCode;

use crate::common::{
    create_test_attorney, create_test_case_via_api, get_with_court, test_app_with_search,
};

/// Demo scenario: searching "Rivera" across all courts finds cases in both
/// test districts. This mirrors the multi-court seed migration where the
/// same defendant name appears in multiple jurisdictions.
#[tokio::test]
async fn demo_cross_court_search_rivera() {
    let (app, pool, _guard, search) = test_app_with_search().await;

    // Create a "Rivera" case in district9
    let _case_d9 =
        create_test_case_via_api(&app, "district9", "United States v. Rivera").await;

    // Create a "Rivera" case in district12
    let _case_d12 =
        create_test_case_via_api(&app, "district12", "United States v. Rivera").await;

    // Rebuild the search index so the new cases are indexed
    server::search::build_index(&pool, &search).await;

    // Search across all courts
    let (status, resp) = get_with_court(
        &app,
        "/api/search/unified?q=Rivera&courts=all",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let results = resp["results"]
        .as_array()
        .expect("results should be an array");
    assert!(
        results.len() >= 2,
        "should find Rivera cases in both courts, got {}",
        results.len()
    );

    // Collect the distinct court_ids present in results
    let court_ids: Vec<&str> = results
        .iter()
        .filter_map(|r| r["court_id"].as_str())
        .collect();
    assert!(
        court_ids.contains(&"district9"),
        "results should include a case from district9"
    );
    assert!(
        court_ids.contains(&"district12"),
        "results should include a case from district12"
    );

    // Verify facets reflect both courts
    let by_court = resp["facets"]["by_court"]
        .as_array()
        .expect("by_court facets should be an array");
    let facet_courts: Vec<&str> = by_court
        .iter()
        .filter_map(|f| f.as_array().and_then(|a| a[0].as_str()))
        .collect();
    assert!(
        facet_courts.contains(&"district9"),
        "facets should include district9"
    );
    assert!(
        facet_courts.contains(&"district12"),
        "facets should include district12"
    );
}

/// Single-court search still works: results are limited to the requested court.
#[tokio::test]
async fn existing_single_court_search_unchanged() {
    let (app, pool, _guard, search) = test_app_with_search().await;

    // Create a case in district9 with a unique name
    let _case_d9 = create_test_case_via_api(
        &app,
        "district9",
        "United States v. Singlecourtcheck",
    )
    .await;

    // Also create one in district12 so we can verify it is filtered out
    let _case_d12 = create_test_case_via_api(
        &app,
        "district12",
        "United States v. Singlecourtcheck",
    )
    .await;

    // Rebuild the index
    server::search::build_index(&pool, &search).await;

    // Search only within district9
    let (status, resp) = get_with_court(
        &app,
        "/api/search/unified?q=Singlecourtcheck&courts=district9",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let results = resp["results"]
        .as_array()
        .expect("results should be an array");

    // Every result must belong to district9
    for r in results {
        assert_eq!(
            r["court_id"].as_str().unwrap(),
            "district9",
            "single-court filter should exclude other courts"
        );
    }

    // Should find at least one result
    assert!(
        !results.is_empty(),
        "should find at least the district9 case"
    );
}

/// Cross-court search finds cases with the same crime type across districts.
/// Verifies that results span multiple courts when courts=all is specified.
#[tokio::test]
async fn cross_court_case_search_by_title() {
    let (app, pool, _guard, search) = test_app_with_search().await;

    // Create cases with a unique shared keyword in different courts
    let _case_d9 = create_test_case_via_api(
        &app,
        "district9",
        "United States v. Crossdistrict",
    )
    .await;

    let _case_d12 = create_test_case_via_api(
        &app,
        "district12",
        "United States v. Crossdistrict",
    )
    .await;

    server::search::build_index(&pool, &search).await;

    let (status, resp) = get_with_court(
        &app,
        "/api/search/unified?q=Crossdistrict&courts=all",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let results = resp["results"]
        .as_array()
        .expect("results should be an array");

    assert!(
        results.len() >= 2,
        "should find cases in both courts, got {}",
        results.len()
    );

    // All results should be cases
    for r in results {
        assert_eq!(
            r["entity_type"].as_str().unwrap(),
            "case",
            "results should all be cases"
        );
    }

    // Results span both courts
    let court_ids: Vec<&str> = results
        .iter()
        .filter_map(|r| r["court_id"].as_str())
        .collect();
    assert!(court_ids.contains(&"district9"));
    assert!(court_ids.contains(&"district12"));
}

/// Cross-court search with entity_type filter returns only the requested type.
#[tokio::test]
async fn cross_court_search_with_entity_filter() {
    let (app, pool, _guard, search) = test_app_with_search().await;

    // Create a case in district9
    let _case = create_test_case_via_api(
        &app,
        "district9",
        "United States v. Entityfilter",
    )
    .await;

    // Create an attorney in district12 whose bar_number contains "Entityfilter"
    // so it appears in the subtitle when indexed. The helper uses "Test Attorney"
    // as the name, so bar_number is the only unique searchable field.
    let _atty = create_test_attorney(&app, "district12", "ENTITYFILTER-001").await;

    server::search::build_index(&pool, &search).await;

    // Search for "Entityfilter" but only cases
    let (status, resp) = get_with_court(
        &app,
        "/api/search/unified?q=Entityfilter&courts=all&entity_types=case",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let results = resp["results"]
        .as_array()
        .expect("results should be an array");

    // Only case entities should appear
    for r in results {
        assert_eq!(
            r["entity_type"].as_str().unwrap(),
            "case",
            "entity_types=case filter should exclude non-case results"
        );
    }

    assert!(
        !results.is_empty(),
        "should find at least the district9 case"
    );
}
