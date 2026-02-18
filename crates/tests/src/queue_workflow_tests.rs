use axum::http::StatusCode;
use crate::common::*;
use uuid::Uuid;

#[tokio::test]
async fn claim_queue_item_success() {
    let (app, _pool, _guard) = test_app().await;
    let item = create_test_queue_item(&app, "district9", "Claim Test", &Uuid::new_v4().to_string()).await;
    let id = item["id"].as_str().unwrap();

    let body = serde_json::json!({ "user_id": 1 });
    let (status, resp) = post_json(&app, &format!("/api/queue/{id}/claim"), &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["assigned_to"], 1);
    assert_eq!(resp["status"], "in_review");
}

#[tokio::test]
async fn claim_already_claimed_item_fails() {
    let (app, _pool, _guard) = test_app().await;
    let item = create_test_queue_item(&app, "district9", "Double Claim", &Uuid::new_v4().to_string()).await;
    let id = item["id"].as_str().unwrap();

    let body = serde_json::json!({ "user_id": 1 });
    let (status, _) = post_json(&app, &format!("/api/queue/{id}/claim"), &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);

    // Second claim should fail
    let body2 = serde_json::json!({ "user_id": 2 });
    let (status2, _) = post_json(&app, &format!("/api/queue/{id}/claim"), &body2.to_string(), "district9").await;
    assert_eq!(status2, StatusCode::CONFLICT);
}

#[tokio::test]
async fn release_queue_item_success() {
    let (app, _pool, _guard) = test_app().await;
    let item = create_test_queue_item(&app, "district9", "Release Test", &Uuid::new_v4().to_string()).await;
    let id = item["id"].as_str().unwrap();

    // Claim first
    let body = serde_json::json!({ "user_id": 1 });
    post_json(&app, &format!("/api/queue/{id}/claim"), &body.to_string(), "district9").await;

    // Release
    let (status, resp) = post_json(&app, &format!("/api/queue/{id}/release"), &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert!(resp["assigned_to"].is_null());
    assert_eq!(resp["status"], "pending");
}

#[tokio::test]
async fn advance_filing_through_pipeline() {
    let (app, _pool, _guard) = test_app().await;
    let item = create_test_queue_item(&app, "district9", "Pipeline Test", &Uuid::new_v4().to_string()).await;
    let id = item["id"].as_str().unwrap();
    assert_eq!(item["current_step"], "review");

    let advance_body = serde_json::json!({});

    // review -> docket
    let (s, resp) = post_json(&app, &format!("/api/queue/{id}/advance"), &advance_body.to_string(), "district9").await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(resp["current_step"], "docket");
    assert_eq!(resp["status"], "processing");

    // docket -> nef
    let (s, resp) = post_json(&app, &format!("/api/queue/{id}/advance"), &advance_body.to_string(), "district9").await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(resp["current_step"], "nef");

    // nef -> serve (last step for filing)
    let (s, resp) = post_json(&app, &format!("/api/queue/{id}/advance"), &advance_body.to_string(), "district9").await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(resp["current_step"], "serve");

    // serve -> completed
    let (s, resp) = post_json(&app, &format!("/api/queue/{id}/advance"), &advance_body.to_string(), "district9").await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(resp["status"], "completed");
    assert!(resp["completed_at"].as_str().is_some());
}

#[tokio::test]
async fn advance_order_skips_review() {
    let (app, _pool, _guard) = test_app().await;
    let body = serde_json::json!({
        "queue_type": "order",
        "title": "Order Pipeline",
        "source_type": "order",
        "source_id": Uuid::new_v4().to_string(),
    });
    let (_, item) = post_json(&app, "/api/queue", &body.to_string(), "district9").await;
    let id = item["id"].as_str().unwrap();
    // Order starts at "docket" (skips review)
    assert_eq!(item["current_step"], "docket");

    let advance = serde_json::json!({});
    // docket -> nef
    let (_, resp) = post_json(&app, &format!("/api/queue/{id}/advance"), &advance.to_string(), "district9").await;
    assert_eq!(resp["current_step"], "nef");
    // nef -> serve
    let (_, resp) = post_json(&app, &format!("/api/queue/{id}/advance"), &advance.to_string(), "district9").await;
    assert_eq!(resp["current_step"], "serve");
    // serve -> completed
    let (_, resp) = post_json(&app, &format!("/api/queue/{id}/advance"), &advance.to_string(), "district9").await;
    assert_eq!(resp["status"], "completed");
}

#[tokio::test]
async fn reject_queue_item_success() {
    let (app, _pool, _guard) = test_app().await;
    let item = create_test_queue_item(&app, "district9", "Reject Test", &Uuid::new_v4().to_string()).await;
    let id = item["id"].as_str().unwrap();

    let body = serde_json::json!({ "reason": "Filing does not comply with local rules" });
    let (status, resp) = post_json(&app, &format!("/api/queue/{id}/reject"), &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["status"], "rejected");
    assert!(resp["completed_at"].as_str().is_some());
}

#[tokio::test]
async fn reject_empty_reason_fails() {
    let (app, _pool, _guard) = test_app().await;
    let item = create_test_queue_item(&app, "district9", "Reject Empty", &Uuid::new_v4().to_string()).await;
    let id = item["id"].as_str().unwrap();

    let body = serde_json::json!({ "reason": "" });
    let (status, _) = post_json(&app, &format!("/api/queue/{id}/reject"), &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn queue_stats_counts() {
    let (app, _pool, _guard) = test_app().await;
    create_test_queue_item(&app, "district9", "Stat1", &Uuid::new_v4().to_string()).await;
    create_test_queue_item(&app, "district9", "Stat2", &Uuid::new_v4().to_string()).await;

    let (status, resp) = get_with_court(&app, "/api/queue/stats", "district9").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["pending_count"], 2);
    assert_eq!(resp["today_count"], 2);
}

#[tokio::test]
async fn motion_pipeline_includes_route_judge() {
    let (app, _pool, _guard) = test_app().await;
    let body = serde_json::json!({
        "queue_type": "motion",
        "title": "Motion Pipeline",
        "source_type": "motion",
        "source_id": Uuid::new_v4().to_string(),
    });
    let (_, item) = post_json(&app, "/api/queue", &body.to_string(), "district9").await;
    let id = item["id"].as_str().unwrap();
    assert_eq!(item["current_step"], "review");

    let advance = serde_json::json!({});
    // review -> docket -> nef -> route_judge -> serve -> completed
    let (_, resp) = post_json(&app, &format!("/api/queue/{id}/advance"), &advance.to_string(), "district9").await;
    assert_eq!(resp["current_step"], "docket");
    let (_, resp) = post_json(&app, &format!("/api/queue/{id}/advance"), &advance.to_string(), "district9").await;
    assert_eq!(resp["current_step"], "nef");
    let (_, resp) = post_json(&app, &format!("/api/queue/{id}/advance"), &advance.to_string(), "district9").await;
    assert_eq!(resp["current_step"], "route_judge");
    let (_, resp) = post_json(&app, &format!("/api/queue/{id}/advance"), &advance.to_string(), "district9").await;
    assert_eq!(resp["current_step"], "serve");
    let (_, resp) = post_json(&app, &format!("/api/queue/{id}/advance"), &advance.to_string(), "district9").await;
    assert_eq!(resp["status"], "completed");
}
