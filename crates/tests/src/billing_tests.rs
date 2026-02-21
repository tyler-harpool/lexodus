use axum::http::StatusCode;
use crate::common::*;

/// GET /api/billing/account should auto-create a new account
/// for the seeded test user (user_id=1) with zero balance.
#[tokio::test]
async fn billing_account_auto_created() {
    let (app, _pool, _guard) = test_app().await;

    let (status, resp) = get_with_court(&app, "/api/billing/account", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["balance_cents"], 0);
    assert_eq!(resp["account_type"], "standard");
    assert_eq!(resp["user_id"], 1);
    assert!(resp["id"].as_str().is_some(), "account should have an id");
}

/// GET /api/billing/transactions with no prior activity should return
/// an empty list with total=0.
#[tokio::test]
async fn billing_transactions_empty() {
    let (app, _pool, _guard) = test_app().await;

    let (status, resp) =
        get_with_court(&app, "/api/billing/transactions?page=1&per_page=10", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 0);
    assert_eq!(resp["page"], 1);
    assert_eq!(resp["per_page"], 10);
    let txns = resp["transactions"].as_array().expect("transactions should be an array");
    assert!(txns.is_empty(), "no transactions should exist yet");
}

/// GET /api/admin/billing/fee-schedule should return the 4 seeded entries
/// (search, document_view, report, export) from the test_app re-seed.
#[tokio::test]
async fn fee_schedule_seeded() {
    let (app, _pool, _guard) = test_app().await;

    let (status, resp) =
        get_with_court(&app, "/api/admin/billing/fee-schedule", "district9").await;
    assert_eq!(status, StatusCode::OK);

    let entries = resp.as_array().expect("response should be an array");
    assert!(
        entries.len() >= 4,
        "expected at least 4 fee schedule entries, got {}",
        entries.len()
    );

    // Verify the seeded action types are present
    let action_types: Vec<&str> = entries
        .iter()
        .filter_map(|e| e["action_type"].as_str())
        .collect();
    assert!(action_types.contains(&"search"), "missing 'search' entry");
    assert!(action_types.contains(&"document_view"), "missing 'document_view' entry");
    assert!(action_types.contains(&"report"), "missing 'report' entry");
    assert!(action_types.contains(&"export"), "missing 'export' entry");

    // Verify fee_cents values
    for entry in entries {
        assert_eq!(entry["fee_cents"], 10, "all seeded fees should be 10 cents");
    }
}

/// GET /api/admin/billing/summary with no activity should return
/// zero revenue and zero searches.
#[tokio::test]
async fn admin_billing_summary() {
    let (app, _pool, _guard) = test_app().await;

    let (status, resp) =
        get_with_court(&app, "/api/admin/billing/summary", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total_revenue_cents"], 0);
    assert_eq!(resp["total_searches"], 0);
    assert_eq!(resp["active_accounts"], 0);
    let top_users = resp["top_users"].as_array().expect("top_users should be an array");
    assert!(top_users.is_empty(), "no top users expected yet");
}
