use axum::http::StatusCode;

use crate::common::{
    create_test_case, create_uploaded_attachment,
    post_json, test_app,
};

#[tokio::test]
async fn promote_cross_tenant_attachment_returns_404() {
    let (app, pool, _guard) = test_app().await;

    // Create attachment in district12
    let case_id = create_test_case(&pool, "district12", "CR-2026-ISO-001").await;
    let entry = crate::common::create_test_docket_entry(&app, "district12", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();
    let att_id = create_uploaded_attachment(&pool, "district12", entry_id).await;

    // Try to promote from district9 — should not see district12's attachment
    let body = serde_json::json!({
        "docket_attachment_id": att_id,
    });

    let (status, _) = post_json(
        &app,
        "/api/documents/from-attachment",
        &body.to_string(),
        "district9",
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
