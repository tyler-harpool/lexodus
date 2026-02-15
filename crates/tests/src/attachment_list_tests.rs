use axum::http::StatusCode;

use crate::common::{
    create_test_case, create_test_docket_entry, get_no_court, get_with_court, test_app,
};

#[tokio::test]
async fn list_attachments_empty() {
    let (app, pool, _guard) = test_app().await;
    let case_id = create_test_case(&pool, "district9", "ATT-LIST-001").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "motion").await;
    let entry_id = entry["id"].as_str().unwrap();

    let uri = format!("/api/docket/entries/{}/attachments", entry_id);
    let (status, body) = get_with_court(&app, &uri, "district9").await;

    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().expect("response should be an array");
    assert!(arr.is_empty(), "should return empty list");
}

#[tokio::test]
async fn list_attachments_missing_court_header() {
    let (app, _pool, _guard) = test_app().await;
    let fake_uuid = uuid::Uuid::new_v4();
    let uri = format!("/api/docket/entries/{}/attachments", fake_uuid);

    let (status, _body) = get_no_court(&app, &uri).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_attachments_invalid_uuid() {
    let (app, _pool, _guard) = test_app().await;

    let (status, body) =
        get_with_court(&app, "/api/docket/entries/not-a-uuid/attachments", "district9").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    let msg = body["message"].as_str().unwrap_or_default();
    assert!(msg.contains("UUID"), "error should mention UUID: {}", msg);
}
