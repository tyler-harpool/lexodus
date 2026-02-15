use axum::http::StatusCode;
use std::collections::HashMap;

use crate::common::{
    create_test_token, create_test_token_with_courts, delete_authed, get_authed,
    post_json_authed, put_json_authed, test_app,
};

const D9: &str = "district9";
const D12: &str = "district12";

/// Insert a pending court role request directly in the DB.
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

// ─── Clerk: Set role for own court ──────────────────────────────────────────

#[tokio::test]
async fn clerk_sets_role_for_own_court() {
    let (app, pool, _guard) = test_app().await;

    sqlx::query!("UPDATE users SET court_roles = '{}' WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();

    let court_roles = HashMap::from([(D9.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let body = serde_json::json!({
        "user_id": 1,
        "court_id": D9,
        "role": "attorney",
    });
    let (status, _) =
        put_json_authed(&app, "/api/admin/court-memberships", &body.to_string(), D9, &token).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify in DB
    let roles: serde_json::Value =
        sqlx::query_scalar!("SELECT court_roles FROM users WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch user");

    assert_eq!(roles[D9], "attorney");
}

// ─── Clerk: Set role for other court => 404 ─────────────────────────────────

#[tokio::test]
async fn clerk_sets_role_for_other_court_returns_not_found() {
    let (app, _pool, _guard) = test_app().await;

    let court_roles = HashMap::from([(D9.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let body = serde_json::json!({
        "user_id": 1,
        "court_id": D12,
        "role": "attorney",
    });
    // Header says D9, body targets D12 — cross-tenant
    let (status, _) =
        put_json_authed(&app, "/api/admin/court-memberships", &body.to_string(), D9, &token).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ─── Admin: Set role for any court ──────────────────────────────────────────

#[tokio::test]
async fn admin_sets_role_for_any_court() {
    let (app, pool, _guard) = test_app().await;

    sqlx::query!("UPDATE users SET court_roles = '{}' WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();

    let token = create_test_token("admin");
    let body = serde_json::json!({
        "user_id": 1,
        "court_id": D12,
        "role": "judge",
    });
    let (status, _) =
        put_json_authed(&app, "/api/admin/court-memberships", &body.to_string(), D12, &token).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    let roles: serde_json::Value =
        sqlx::query_scalar!("SELECT court_roles FROM users WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch user");

    assert_eq!(roles[D12], "judge");
}

// ─── Attorney cannot set role ───────────────────────────────────────────────

#[tokio::test]
async fn attorney_cannot_set_role() {
    let (app, _pool, _guard) = test_app().await;

    let court_roles = HashMap::from([(D9.to_string(), "attorney".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let body = serde_json::json!({
        "user_id": 1,
        "court_id": D9,
        "role": "clerk",
    });
    let (status, _) =
        put_json_authed(&app, "/api/admin/court-memberships", &body.to_string(), D9, &token).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ─── Clerk: Remove role from own court ──────────────────────────────────────

#[tokio::test]
async fn clerk_removes_role_from_own_court() {
    let (app, pool, _guard) = test_app().await;

    let role_obj = serde_json::json!({ D9: "attorney" });
    sqlx::query!(
        "UPDATE users SET court_roles = $1 WHERE id = 1",
        role_obj,
    )
    .execute(&pool)
    .await
    .unwrap();

    let court_roles = HashMap::from([(D9.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/admin/court-memberships/1/{}", D9);
    let (status, _) = delete_authed(&app, &uri, D9, &token).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    let roles: serde_json::Value =
        sqlx::query_scalar!("SELECT court_roles FROM users WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch user");

    assert!(roles.get(D9).is_none(), "Court role should be removed");
}

// ─── Clerk: Remove role from other court => 404 ─────────────────────────────

#[tokio::test]
async fn clerk_removes_role_from_other_court_returns_not_found() {
    let (app, _pool, _guard) = test_app().await;

    let court_roles = HashMap::from([(D9.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/admin/court-memberships/1/{}", D12);
    // Header says D9, path targets D12 — cross-tenant
    let (status, _) = delete_authed(&app, &uri, D9, &token).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ─── Clerk: View user roles (filtered to own court) ─────────────────────────

#[tokio::test]
async fn clerk_views_user_roles_filtered() {
    let (app, pool, _guard) = test_app().await;

    let role_obj = serde_json::json!({ D9: "attorney", D12: "judge" });
    sqlx::query!(
        "UPDATE users SET court_roles = $1 WHERE id = 1",
        role_obj,
    )
    .execute(&pool)
    .await
    .unwrap();

    let court_roles = HashMap::from([(D9.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, response) =
        get_authed(&app, "/api/admin/court-memberships/user/1", D9, &token).await;

    assert_eq!(status, StatusCode::OK, "Get roles failed: {:?}", response);
    // Should only see D9 role, not D12
    assert_eq!(response[D9], "attorney");
    assert!(response.get(D12).is_none() || response[D12].is_null(),
        "Clerk should not see roles for other courts");
}

// ─── Admin: View user roles (all) ───────────────────────────────────────────

#[tokio::test]
async fn admin_views_user_roles_all() {
    let (app, pool, _guard) = test_app().await;

    let role_obj = serde_json::json!({ D9: "clerk", D12: "attorney" });
    sqlx::query!(
        "UPDATE users SET court_roles = $1 WHERE id = 1",
        role_obj,
    )
    .execute(&pool)
    .await
    .unwrap();

    let token = create_test_token("admin");
    let (status, response) =
        get_authed(&app, "/api/admin/court-memberships/user/1", D9, &token).await;

    assert_eq!(status, StatusCode::OK, "Get roles failed: {:?}", response);
    assert_eq!(response[D9], "clerk");
    assert_eq!(response[D12], "attorney");
}

// ─── Clerk: Approve request for own court ───────────────────────────────────

#[tokio::test]
async fn clerk_approves_request_for_own_court() {
    let (app, pool, _guard) = test_app().await;
    let request_id = insert_pending_request(&pool, 1, D9, "attorney").await;

    let court_roles = HashMap::from([(D9.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/admin/court-role-requests/{}/approve", request_id);
    let (status, response) = post_json_authed(&app, &uri, "{}", D9, &token).await;

    assert_eq!(status, StatusCode::OK, "Approve failed: {:?}", response);
    assert_eq!(response["status"], "approved");
}

// ─── Clerk: Approve request for other court => 404 ──────────────────────────

#[tokio::test]
async fn clerk_approves_request_for_other_court_returns_not_found() {
    let (app, pool, _guard) = test_app().await;
    let request_id = insert_pending_request(&pool, 1, D12, "attorney").await;

    // Clerk in D9, request is for D12
    let court_roles = HashMap::from([(D9.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let uri = format!("/api/admin/court-role-requests/{}/approve", request_id);
    let (status, _) = post_json_authed(&app, &uri, "{}", D9, &token).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ─── Clerk: List pending shows only own court ───────────────────────────────

#[tokio::test]
async fn clerk_list_pending_shows_only_own_court() {
    let (app, pool, _guard) = test_app().await;
    let d9_id = insert_pending_request(&pool, 1, D9, "attorney").await;
    insert_pending_request(&pool, 1, D12, "judge").await;

    let court_roles = HashMap::from([(D9.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, response) =
        get_authed(&app, "/api/admin/court-role-requests", D9, &token).await;

    assert_eq!(status, StatusCode::OK, "List pending failed: {:?}", response);
    let arr = response.as_array().expect("Expected JSON array");

    // Should only contain the D9 request
    let d9_found = arr.iter().any(|r| r["id"].as_str() == Some(&d9_id));
    assert!(d9_found, "Should contain request for own court");

    let all_d9 = arr.iter().all(|r| r["court_id"].as_str() == Some(D9));
    assert!(all_d9, "All returned requests should be for the clerk's court");
}

// ─── Clerk: Missing header => 400 ───────────────────────────────────────────

#[tokio::test]
async fn clerk_missing_header_returns_bad_request() {
    let (app, _pool, _guard) = test_app().await;

    let court_roles = HashMap::from([(D9.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let body = serde_json::json!({
        "user_id": 1,
        "court_id": D9,
        "role": "attorney",
    });

    // Send without X-Court-District header by using a raw request
    let req = axum::http::Request::builder()
        .method("PUT")
        .uri("/api/admin/court-memberships")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(axum::body::Body::from(body.to_string()))
        .unwrap();

    let response = tower::ServiceExt::oneshot(app.clone(), req)
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
