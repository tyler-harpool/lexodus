use axum::http::StatusCode;
use crate::common::*;
use uuid::Uuid;

#[tokio::test]
async fn create_queue_item_success() {
    let (app, _pool, _guard) = test_app().await;
    let source_id = Uuid::new_v4().to_string();
    let body = serde_json::json!({
        "queue_type": "filing",
        "priority": 2,
        "title": "Motion to Dismiss - USA v. Test",
        "description": "Urgent motion requiring review",
        "source_type": "filing",
        "source_id": source_id,
        "case_number": "2:24-cr-00142",
    });

    let (status, resp) = post_json(&app, "/api/queue", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["queue_type"], "filing");
    assert_eq!(resp["priority"], 2);
    assert_eq!(resp["status"], "pending");
    assert_eq!(resp["current_step"], "review");
    assert_eq!(resp["title"], "Motion to Dismiss - USA v. Test");
    assert!(resp["id"].as_str().is_some());
}

#[tokio::test]
async fn create_queue_item_defaults() {
    let (app, _pool, _guard) = test_app().await;
    let source_id = Uuid::new_v4().to_string();
    let body = serde_json::json!({
        "queue_type": "order",
        "title": "Scheduling Order",
        "source_type": "order",
        "source_id": source_id,
    });

    let (status, resp) = post_json(&app, "/api/queue", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["priority"], 3);
    assert_eq!(resp["status"], "pending");
    // Order pipeline starts at "docket", not "review"
    assert_eq!(resp["current_step"], "docket");
}

#[tokio::test]
async fn create_queue_item_invalid_type() {
    let (app, _pool, _guard) = test_app().await;
    let body = serde_json::json!({
        "queue_type": "invalid",
        "title": "Bad item",
        "source_type": "filing",
        "source_id": Uuid::new_v4().to_string(),
    });

    let (status, _resp) = post_json(&app, "/api/queue", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_queue_item_empty_title() {
    let (app, _pool, _guard) = test_app().await;
    let body = serde_json::json!({
        "queue_type": "filing",
        "title": "",
        "source_type": "filing",
        "source_id": Uuid::new_v4().to_string(),
    });

    let (status, _resp) = post_json(&app, "/api/queue", &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_queue_item_success() {
    let (app, _pool, _guard) = test_app().await;
    let source_id = Uuid::new_v4().to_string();
    let item = create_test_queue_item(&app, "district9", "Test Item", &source_id).await;
    let id = item["id"].as_str().unwrap();

    let (status, resp) = get_with_court(&app, &format!("/api/queue/{id}"), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["title"], "Test Item");
}

#[tokio::test]
async fn get_queue_item_not_found() {
    let (app, _pool, _guard) = test_app().await;
    let fake_id = Uuid::new_v4();
    let (status, _resp) = get_with_court(&app, &format!("/api/queue/{fake_id}"), "district9").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_queue_items_empty() {
    let (app, _pool, _guard) = test_app().await;
    let (status, resp) = get_with_court(&app, "/api/queue", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["items"].as_array().unwrap().len(), 0);
    assert_eq!(resp["total"], 0);
}

#[tokio::test]
async fn list_queue_items_with_filters() {
    let (app, _pool, _guard) = test_app().await;
    create_test_queue_item(&app, "district9", "Item 1", &Uuid::new_v4().to_string()).await;
    create_test_queue_item(&app, "district9", "Item 2", &Uuid::new_v4().to_string()).await;

    let (status, resp) = get_with_court(&app, "/api/queue?status=pending", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["items"].as_array().unwrap().len(), 2);
    assert_eq!(resp["total"], 2);
}

#[tokio::test]
async fn queue_tenant_isolation() {
    let (app, _pool, _guard) = test_app().await;
    create_test_queue_item(&app, "district9", "D9 Item", &Uuid::new_v4().to_string()).await;
    create_test_queue_item(&app, "district12", "D12 Item", &Uuid::new_v4().to_string()).await;

    let (status, resp) = get_with_court(&app, "/api/queue", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["total"], 1);
    assert_eq!(resp["items"][0]["title"], "D9 Item");
}
