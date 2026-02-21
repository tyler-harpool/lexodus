use axum::http::StatusCode;

use crate::common::{test_app_with_search, get_with_court, post_json};

/// Helper to build a valid attorney JSON body with required fields.
fn attorney_body(first: &str, last: &str, bar: &str) -> String {
    serde_json::json!({
        "first_name": first,
        "last_name": last,
        "bar_number": bar,
        "email": format!("{}@test.com", bar.to_lowercase()),
        "phone": "555-0100",
        "address": {
            "street1": "100 Test St",
            "city": "Testville",
            "state": "CA",
            "zip_code": "90210",
            "country": "USA"
        }
    })
    .to_string()
}

/// Unified search returns results across both test courts.
#[tokio::test]
async fn unified_search_returns_cross_court_results() {
    let (app, pool, _guard, search) = test_app_with_search().await;

    // Create an attorney in district9
    let (status, _) =
        post_json(&app, "/api/attorneys", &attorney_body("Cross", "Court", "CC-001"), "district9")
            .await;
    assert!(
        status == StatusCode::OK || status == StatusCode::CREATED,
        "expected 200/201, got {status}"
    );

    // Create an attorney in district12
    let (status, _) = post_json(
        &app,
        "/api/attorneys",
        &attorney_body("Cross", "District", "CD-001"),
        "district12",
    )
    .await;
    assert!(
        status == StatusCode::OK || status == StatusCode::CREATED,
        "expected 200/201, got {status}"
    );

    // Rebuild the search index so the new attorneys are indexed
    server::search::build_index(&pool, &search).await;

    // Search across all courts via REST
    let (status, resp) = get_with_court(
        &app,
        "/api/search/unified?q=Cross&courts=all",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let results = resp["results"].as_array().expect("results should be array");
    assert!(
        results.len() >= 2,
        "should find attorneys in both courts, got {}",
        results.len()
    );

    // Verify facets include both courts
    let by_court = resp["facets"]["by_court"].as_array().expect("by_court facets");
    let court_ids: Vec<&str> = by_court
        .iter()
        .filter_map(|f| f.as_array().and_then(|a| a[0].as_str()))
        .collect();
    assert!(court_ids.contains(&"district9"), "facets should include district9");
    assert!(court_ids.contains(&"district12"), "facets should include district12");
}

/// Unified search filters by court when specified.
#[tokio::test]
async fn unified_search_filters_by_court() {
    let (app, pool, _guard, search) = test_app_with_search().await;

    let (status, _) = post_json(
        &app,
        "/api/attorneys",
        &attorney_body("FilterTest", "Attorney", "FT-001"),
        "district9",
    )
    .await;
    assert!(
        status == StatusCode::OK || status == StatusCode::CREATED,
        "expected 200/201, got {status}"
    );

    let (status, _) = post_json(
        &app,
        "/api/attorneys",
        &attorney_body("FilterTest", "Other", "FT-002"),
        "district12",
    )
    .await;
    assert!(
        status == StatusCode::OK || status == StatusCode::CREATED,
        "expected 200/201, got {status}"
    );

    server::search::build_index(&pool, &search).await;

    let (status, resp) = get_with_court(
        &app,
        "/api/search/unified?q=FilterTest&courts=district9",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let results = resp["results"].as_array().expect("results");
    for r in results {
        assert_eq!(r["court_id"].as_str().unwrap(), "district9");
    }
}

/// Unified search filters by entity type.
#[tokio::test]
async fn unified_search_filters_by_entity_type() {
    let (app, pool, _guard, search) = test_app_with_search().await;

    let (status, _) = post_json(
        &app,
        "/api/attorneys",
        &attorney_body("TypeFilter", "Atty", "TF-001"),
        "district9",
    )
    .await;
    assert!(
        status == StatusCode::OK || status == StatusCode::CREATED,
        "expected 200/201, got {status}"
    );

    // Create a case too
    let case_body = serde_json::json!({
        "title": "TypeFilter Case",
        "crime_type": "fraud",
        "district_code": "9",
        "priority": "medium"
    })
    .to_string();
    let (status, _) = post_json(&app, "/api/cases", &case_body, "district9").await;
    assert!(
        status == StatusCode::OK || status == StatusCode::CREATED,
        "expected 200/201 for case, got {status}"
    );

    server::search::build_index(&pool, &search).await;

    let (status, resp) = get_with_court(
        &app,
        "/api/search/unified?q=TypeFilter&entity_types=attorney",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let results = resp["results"].as_array().expect("results");
    for r in results {
        assert_eq!(r["entity_type"].as_str().unwrap(), "attorney");
    }
}

/// Unified search returns empty for no matches.
#[tokio::test]
async fn unified_search_empty_results() {
    let (app, _pool, _guard, _search) = test_app_with_search().await;

    let (status, resp) = get_with_court(
        &app,
        "/api/search/unified?q=xyznonexistent99999",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let results = resp["results"].as_array().expect("results");
    assert!(results.is_empty(), "no results for nonsense query");
    assert_eq!(resp["total"].as_i64().unwrap(), 0);
}

/// Unified search paginates correctly.
#[tokio::test]
async fn unified_search_pagination() {
    let (app, pool, _guard, search) = test_app_with_search().await;

    // Create 5 attorneys with same prefix
    for i in 1..=5 {
        let (status, _) = post_json(
            &app,
            "/api/attorneys",
            &attorney_body("Paginator", &format!("Number{i}"), &format!("PG-{i:03}")),
            "district9",
        )
        .await;
        assert!(
            status == StatusCode::OK || status == StatusCode::CREATED,
            "expected 200/201 for attorney {i}, got {status}"
        );
    }

    server::search::build_index(&pool, &search).await;

    let (status, resp) = get_with_court(
        &app,
        "/api/search/unified?q=Paginator&per_page=2&page=1",
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let results = resp["results"].as_array().expect("results");
    assert_eq!(results.len(), 2, "page 1 should have 2 results");
    assert!(resp["total"].as_i64().unwrap() >= 5, "total should be >= 5");
}
