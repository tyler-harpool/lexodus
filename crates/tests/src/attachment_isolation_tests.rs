use axum::http::StatusCode;

use crate::common::{
    create_test_case, create_test_docket_entry, get_with_court, post_json, test_app,
};

#[tokio::test]
async fn district9_cannot_list_district12_attachments() {
    let (app, pool, _guard) = test_app().await;

    // Create entry in district12
    let case_id = create_test_case(&pool, "district12", "ATT-ISO-001").await;
    let entry = create_test_docket_entry(&app, "district12", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();

    // district9 tries to list district12's attachments => 404 (entry not found in tenant)
    let uri = format!("/api/docket/entries/{}/attachments", entry_id);
    let (status, _body) = get_with_court(&app, &uri, "district9").await;

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "cross-tenant list should return 404"
    );
}

#[tokio::test]
async fn district9_cannot_create_attachment_on_district12_entry() {
    let (app, pool, _guard) = test_app().await;

    // Create entry in district12
    let case_id = create_test_case(&pool, "district12", "ATT-ISO-002").await;
    let entry = create_test_docket_entry(&app, "district12", &case_id, "order").await;
    let entry_id = entry["id"].as_str().unwrap();

    let body = serde_json::json!({
        "file_name": "order.pdf",
        "content_type": "application/pdf",
        "file_size": 5000,
    });

    // district9 tries to create attachment on district12's entry => 404
    let uri = format!("/api/docket/entries/{}/attachments", entry_id);
    let (status, _resp) = post_json(&app, &uri, &body.to_string(), "district9").await;

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "cross-tenant create should return 404"
    );
}
