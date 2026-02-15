use axum::http::StatusCode;
use std::collections::HashMap;

use crate::common::{
    create_test_token, create_test_token_with_courts, delete_authed, get_authed,
    post_json_authed, put_json_authed, test_app,
};

const COURT: &str = "district9";

/// Insert a pending court role request directly in the DB.
/// Returns the request UUID string.
async fn insert_pending_request(
    pool: &sqlx::Pool<sqlx::Postgres>,
    user_id: i64,
    court_id: &str,
    role: &str,
) -> String {
    let row = sqlx::query_scalar!(
        r#"
        INSERT INTO court_role_requests (user_id, court_id, requested_role)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        user_id,
        court_id,
        role,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert pending request");

    row.to_string()
}

// ─── Admin: List pending requests ───────────────────────────────────────────

#[tokio::test]
async fn list_pending_requests_as_admin() {
    let (app, pool, _guard) = test_app().await;
    let request_id = insert_pending_request(&pool, 1, COURT, "clerk").await;

    let token = create_test_token("admin");
    let (status, response) =
        get_authed(&app, "/api/admin/court-role-requests", COURT, &token).await;

    assert_eq!(status, StatusCode::OK, "List pending failed: {:?}", response);
    let arr = response.as_array().expect("Expected JSON array");
    assert!(!arr.is_empty(), "Should have at least one pending request");

    let found = arr.iter().any(|r| r["id"].as_str() == Some(&request_id));
    assert!(found, "Pending list should contain our request");
}

#[tokio::test]
async fn list_pending_requests_rejects_attorney() {
    let (app, _pool, _guard) = test_app().await;

    // Attorney (not clerk, not admin) should be rejected
    let court_roles = HashMap::from([(COURT.to_string(), "attorney".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, _) =
        get_authed(&app, "/api/admin/court-role-requests", COURT, &token).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ─── Admin: Approve request ─────────────────────────────────────────────────

#[tokio::test]
async fn approve_request_updates_court_roles() {
    let (app, pool, _guard) = test_app().await;
    let request_id = insert_pending_request(&pool, 1, COURT, "clerk").await;

    let token = create_test_token("admin");
    let uri = format!("/api/admin/court-role-requests/{}/approve", request_id);
    let (status, response) = post_json_authed(&app, &uri, "{}", COURT, &token).await;

    assert_eq!(status, StatusCode::OK, "Approve failed: {:?}", response);
    assert_eq!(response["status"], "approved");

    // Verify the user's court_roles in DB were updated
    let roles: serde_json::Value = sqlx::query_scalar!(
        "SELECT court_roles FROM users WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch user");

    assert_eq!(roles[COURT], "clerk", "court_roles should contain the approved role");
}

#[tokio::test]
async fn approve_already_reviewed_returns_not_found() {
    let (app, pool, _guard) = test_app().await;
    let request_id = insert_pending_request(&pool, 1, COURT, "attorney").await;

    let token = create_test_token("admin");
    let uri = format!("/api/admin/court-role-requests/{}/approve", request_id);

    // First approval succeeds
    let (status, _) = post_json_authed(&app, &uri, "{}", COURT, &token).await;
    assert_eq!(status, StatusCode::OK);

    // Second approval of same request fails
    let (status, _) = post_json_authed(&app, &uri, "{}", COURT, &token).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ─── Admin: Deny request ────────────────────────────────────────────────────

#[tokio::test]
async fn deny_request_leaves_court_roles_unchanged() {
    let (app, pool, _guard) = test_app().await;

    // Clear any existing court_roles on the test user
    sqlx::query!("UPDATE users SET court_roles = '{}' WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();

    let request_id = insert_pending_request(&pool, 1, COURT, "judge").await;

    let token = create_test_token("admin");
    let body = serde_json::json!({ "approved": false, "notes": "Insufficient credentials" });
    let uri = format!("/api/admin/court-role-requests/{}/deny", request_id);
    let (status, response) =
        post_json_authed(&app, &uri, &body.to_string(), COURT, &token).await;

    assert_eq!(status, StatusCode::OK, "Deny failed: {:?}", response);
    assert_eq!(response["status"], "denied");
    assert_eq!(response["notes"], "Insufficient credentials");

    // Verify the user's court_roles remain empty
    let roles: serde_json::Value = sqlx::query_scalar!(
        "SELECT court_roles FROM users WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch user");

    assert_eq!(roles, serde_json::json!({}), "court_roles should be unchanged after denial");
}

// ─── Duplicate request rejected ─────────────────────────────────────────────

#[tokio::test]
async fn duplicate_pending_request_rejected() {
    let (_app, pool, _guard) = test_app().await;

    // First request succeeds
    insert_pending_request(&pool, 1, COURT, "clerk").await;

    // Second pending request for the same court should fail (unique constraint)
    let result = sqlx::query_scalar!(
        r#"
        INSERT INTO court_role_requests (user_id, court_id, requested_role)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        1_i64,
        COURT,
        "clerk",
    )
    .fetch_one(&pool)
    .await;

    assert!(result.is_err(), "Duplicate pending request should be rejected by DB constraint");
}

// ─── Admin: Directly set court role ─────────────────────────────────────────

#[tokio::test]
async fn admin_set_court_role() {
    let (app, pool, _guard) = test_app().await;

    // Clear court_roles
    sqlx::query!("UPDATE users SET court_roles = '{}' WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();

    let token = create_test_token("admin");
    let body = serde_json::json!({
        "user_id": 1,
        "court_id": COURT,
        "role": "judge",
    });
    let (status, _) =
        put_json_authed(&app, "/api/admin/court-memberships", &body.to_string(), COURT, &token).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify in DB
    let roles: serde_json::Value = sqlx::query_scalar!(
        "SELECT court_roles FROM users WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch user");

    assert_eq!(roles[COURT], "judge");
}

#[tokio::test]
async fn admin_set_invalid_role_rejected() {
    let (app, _pool, _guard) = test_app().await;

    let token = create_test_token("admin");
    let body = serde_json::json!({
        "user_id": 1,
        "court_id": COURT,
        "role": "superuser",
    });
    let (status, _) =
        put_json_authed(&app, "/api/admin/court-memberships", &body.to_string(), COURT, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ─── Admin: Remove court role ───────────────────────────────────────────────

#[tokio::test]
async fn admin_remove_court_role() {
    let (app, pool, _guard) = test_app().await;

    // Set up a court role first
    let role_obj = serde_json::json!({ COURT: "clerk" });
    sqlx::query!(
        "UPDATE users SET court_roles = $1 WHERE id = 1",
        role_obj,
    )
    .execute(&pool)
    .await
    .unwrap();

    let token = create_test_token("admin");
    let uri = format!("/api/admin/court-memberships/1/{}", COURT);
    let (status, _) = delete_authed(&app, &uri, COURT, &token).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify the role was removed
    let roles: serde_json::Value = sqlx::query_scalar!(
        "SELECT court_roles FROM users WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch user");

    assert!(roles.get(COURT).is_none(), "Court role should be removed");
}

// ─── Admin: Get user court roles ────────────────────────────────────────────

#[tokio::test]
async fn admin_get_user_court_roles() {
    let (app, pool, _guard) = test_app().await;

    let role_obj = serde_json::json!({ COURT: "clerk", "district12": "attorney" });
    sqlx::query!(
        "UPDATE users SET court_roles = $1 WHERE id = 1",
        role_obj,
    )
    .execute(&pool)
    .await
    .unwrap();

    let token = create_test_token("admin");
    let (status, response) =
        get_authed(&app, "/api/admin/court-memberships/user/1", COURT, &token).await;

    assert_eq!(status, StatusCode::OK, "Get roles failed: {:?}", response);
    assert_eq!(response[COURT], "clerk");
    assert_eq!(response["district12"], "attorney");
}

// ─── Non-admin/non-clerk rejected for admin endpoints ───────────────────────

#[tokio::test]
async fn set_court_role_rejects_attorney() {
    let (app, _pool, _guard) = test_app().await;

    // Attorney cannot manage memberships
    let court_roles = HashMap::from([(COURT.to_string(), "attorney".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let body = serde_json::json!({
        "user_id": 1,
        "court_id": COURT,
        "role": "judge",
    });
    let (status, _) =
        put_json_authed(&app, "/api/admin/court-memberships", &body.to_string(), COURT, &token).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn remove_court_role_rejects_attorney() {
    let (app, _pool, _guard) = test_app().await;

    // Attorney cannot manage memberships
    let court_roles = HashMap::from([(COURT.to_string(), "attorney".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/admin/court-memberships/1/{}", COURT);
    let (status, _) = delete_authed(&app, &uri, COURT, &token).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}
