use axum::http::StatusCode;
use crate::common;

#[tokio::test]
async fn test_bulk_update_status_returns_204() {
    let (app, _pool, _guard) = common::test_app().await;
    let id1 = common::create_test_attorney(&app, "district9", "BULK001").await;
    let id2 = common::create_test_attorney(&app, "district9", "BULK002").await;

    let body = serde_json::json!({
        "attorney_ids": [id1, id2],
        "status": "Inactive"
    });

    let (status, _) = common::post_json(
        &app,
        "/api/attorneys/bulk/update-status",
        &body.to_string(),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_bulk_update_status_changes_in_db() {
    let (app, _pool, _guard) = common::test_app().await;
    let id1 = common::create_test_attorney(&app, "district9", "BULKDB1").await;
    let id2 = common::create_test_attorney(&app, "district9", "BULKDB2").await;

    let body = serde_json::json!({
        "attorney_ids": [id1, id2],
        "status": "Suspended"
    });
    let (status, _) = common::post_json(
        &app,
        "/api/attorneys/bulk/update-status",
        &body.to_string(),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify each attorney is now Suspended
    let (s1, r1) = common::get_with_court(&app, &format!("/api/attorneys/{}", id1), "district9").await;
    assert_eq!(s1, StatusCode::OK);
    assert_eq!(r1["status"], "Suspended");

    let (s2, r2) = common::get_with_court(&app, &format!("/api/attorneys/{}", id2), "district9").await;
    assert_eq!(s2, StatusCode::OK);
    assert_eq!(r2["status"], "Suspended");
}

#[tokio::test]
async fn test_bulk_update_status_invalid_status() {
    let (app, _pool, _guard) = common::test_app().await;
    let id = common::create_test_attorney(&app, "district9", "BULKERR").await;

    let body = serde_json::json!({
        "attorney_ids": [id],
        "status": "InvalidStatus"
    });
    let (status, _) = common::post_json(
        &app,
        "/api/attorneys/bulk/update-status",
        &body.to_string(),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_bulk_update_does_not_affect_other_tenant() {
    let (app, _pool, _guard) = common::test_app().await;
    let id_d9 = common::create_test_attorney(&app, "district9", "BULKISO1").await;
    let id_d12 = common::create_test_attorney(&app, "district12", "BULKISO2").await;

    // Bulk update only district9 attorneys
    let body = serde_json::json!({
        "attorney_ids": [id_d9, id_d12],
        "status": "Retired"
    });
    let (status, _) = common::post_json(
        &app,
        "/api/attorneys/bulk/update-status",
        &body.to_string(),
        "district9",
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // district9 attorney should be updated
    let (_, r9) = common::get_with_court(&app, &format!("/api/attorneys/{}", id_d9), "district9").await;
    assert_eq!(r9["status"], "Retired");

    // district12 attorney should remain Active (not affected by district9 bulk op)
    let (_, r12) = common::get_with_court(&app, &format!("/api/attorneys/{}", id_d12), "district12").await;
    assert_eq!(r12["status"], "Active");
}
