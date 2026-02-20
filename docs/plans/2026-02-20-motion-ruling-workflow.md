# Motion Ruling Workflow — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable judges to rule on pending motions inline from their dashboard, auto-generating orders and queue items through the compliance engine.

**Architecture:** Add a `POST /api/motions/{id}/rule` REST endpoint that updates motion status + ruling fields, fires the compliance engine with a `MotionRuled` trigger, auto-creates a judicial order with PDF via Typst, and enqueues a clerk queue item. The judge dashboard gets inline ruling UI (disposition buttons + ruling text). Permission gating is tightened on case detail (remove Queue/Delete for judges).

**Tech Stack:** Rust, Axum, sqlx, Dioxus 0.7, Typst, PostgreSQL

---

### Task 1: Add `RuleMotionRequest` to shared-types

**Files:**
- Modify: `crates/shared-types/src/case.rs`

**Step 1: Add the request struct and ruling disposition constants**

In `crates/shared-types/src/case.rs`, after the existing `UpdateMotionRequest` struct, add:

```rust
/// Valid ruling dispositions for a motion.
pub const RULING_DISPOSITIONS: &[&str] = &[
    "Granted",
    "Denied",
    "Granted in Part",
    "Taken Under Advisement",
    "Set for Hearing",
    "Moot",
];

pub fn is_valid_ruling_disposition(s: &str) -> bool {
    RULING_DISPOSITIONS.contains(&s)
}

/// Request body for a judge ruling on a motion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMotionRequest {
    /// The disposition: "Granted", "Denied", "Granted in Part", "Taken Under Advisement", "Set for Hearing", "Moot"
    pub disposition: String,
    /// Free-form ruling text from the judge
    pub ruling_text: Option<String>,
    /// The judge's ID (UUID as string)
    pub judge_id: String,
    /// The judge's display name (for order generation)
    pub judge_name: String,
}
```

**Step 2: Verify compilation**

Run: `cargo check -p shared-types`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/shared-types/src/case.rs
git commit -m "feat(types): add RuleMotionRequest and ruling disposition constants"
```

---

### Task 2: Add `POST /api/motions/{id}/rule` REST endpoint

**Files:**
- Modify: `crates/server/src/rest/motion.rs`
- Modify: `crates/server/src/api.rs` or `crates/server/src/api/judge.rs` (server function)

**Step 1: Add the rule_motion handler to `rest/motion.rs`**

After the `delete_motion` handler, add:

```rust
/// POST /api/motions/{id}/rule
/// Judge rules on a pending motion: updates status, creates order, fires compliance engine.
#[utoipa::path(
    post,
    path = "/api/motions/{id}/rule",
    request_body = shared_types::RuleMotionRequest,
    params(
        ("id" = String, Path, description = "Motion UUID"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Motion ruled, order created", body = JudicialOrderResponse),
        (status = 400, description = "Invalid request", body = AppError),
        (status = 404, description = "Not found", body = AppError)
    ),
    tag = "motions"
)]
pub async fn rule_motion(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<String>,
    Json(body): Json<shared_types::RuleMotionRequest>,
) -> Result<Json<shared_types::JudicialOrderResponse>, AppError> {
    use shared_types::{
        is_valid_ruling_disposition, CreateJudicialOrderRequest, JudicialOrderResponse,
        UpdateMotionRequest,
    };

    let motion_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    if !is_valid_ruling_disposition(&body.disposition) {
        return Err(AppError::bad_request(format!(
            "Invalid disposition: {}. Valid values: {}",
            body.disposition,
            shared_types::RULING_DISPOSITIONS.join(", ")
        )));
    }

    let judge_uuid = Uuid::parse_str(&body.judge_id)
        .map_err(|_| AppError::bad_request("Invalid judge_id UUID format"))?;

    // 1. Fetch the motion
    let motion = crate::repo::motion::find_by_id(&pool, &court.0, motion_uuid)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Motion {} not found", id)))?;

    if motion.status != "Pending" {
        return Err(AppError::bad_request(format!(
            "Motion is not pending (current status: {})", motion.status
        )));
    }

    // 2. Map disposition to motion status
    let new_status = match body.disposition.as_str() {
        "Granted" | "Granted in Part" => "Granted",
        "Denied" => "Denied",
        "Moot" => "Moot",
        "Taken Under Advisement" => "Deferred",
        "Set for Hearing" => "Pending",  // stays pending until hearing
        _ => "Pending",
    };

    // 3. Update motion with ruling
    let update_req = UpdateMotionRequest {
        motion_type: None,
        filed_by: None,
        description: None,
        status: Some(new_status.to_string()),
        ruling_date: Some(chrono::Utc::now()),
        ruling_text: body.ruling_text.clone(),
    };
    crate::repo::motion::update(&pool, &court.0, motion_uuid, update_req).await?;

    // 4. Determine order type from motion type
    let order_type = match motion.motion_type.as_str() {
        "Dismiss" => "Dismissal",
        "Suppress" | "Limine" => "Procedural",
        "Compel" | "Discovery" => "Discovery",
        _ => "Procedural",
    };

    // 5. Generate order content from ruling
    let ruling_text = body.ruling_text.clone().unwrap_or_else(|| {
        format!(
            "The Court, having considered the {} filed by {}, and for good cause shown, hereby {} the motion.",
            motion.motion_type, motion.filed_by,
            body.disposition.to_lowercase()
        )
    });

    let order_title = format!(
        "Order on {} ({})",
        motion.motion_type, body.disposition
    );

    // 6. Create judicial order
    let create_order = CreateJudicialOrderRequest {
        case_id: motion.case_id,
        judge_id: judge_uuid,
        order_type: order_type.to_string(),
        title: order_title,
        content: ruling_text,
        status: Some("Pending Signature".to_string()),
        is_sealed: Some(false),
        effective_date: None,
        expiration_date: None,
        related_motions: vec![motion_uuid],
    };

    let order = crate::repo::order::create(&pool, &court.0, create_order).await?;

    // 7. Create clerk queue item for the new order
    let _ = crate::repo::queue::create(
        &pool,
        &court.0,
        "order",
        2,
        &format!("Order on {} - pending judge signature", motion.motion_type),
        Some("Auto-generated from motion ruling"),
        "order",
        order.id,
        Some(motion.case_id),
        None,
        None,
        None,
        "route_judge",
    )
    .await;

    // 8. Fire compliance engine with MotionRuled trigger
    let trigger = shared_types::compliance::TriggerEvent::MotionDenied; // closest existing trigger — we'll use MotionFiled for now
    let context = shared_types::compliance::FilingContext {
        case_type: "criminal".to_string(),
        document_type: format!("motion_ruling_{}", body.disposition.to_lowercase().replace(' ', "_")),
        filer_role: "judge".to_string(),
        jurisdiction_id: court.0.clone(),
        division: None,
        assigned_judge: Some(body.judge_name.clone()),
        service_method: None,
        metadata: serde_json::json!({
            "case_id": motion.case_id.to_string(),
            "motion_id": motion.id.to_string(),
            "motion_type": motion.motion_type,
            "disposition": body.disposition,
            "order_id": order.id.to_string(),
        }),
    };

    let all_rules = crate::repo::rule::list_active(&pool, &court.0, None)
        .await
        .unwrap_or_default();
    let selected = crate::compliance::engine::select_rules(&court.0, &trigger, &all_rules);
    let sorted = crate::compliance::engine::resolve_priority(selected);
    let report = crate::compliance::engine::evaluate(&context, &sorted);

    // Apply deadlines from engine
    for deadline in &report.deadlines {
        let due_at = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            deadline.due_date.and_hms_opt(17, 0, 0).unwrap(),
            chrono::Utc,
        );
        let _ = crate::repo::deadline::create(
            &pool,
            &court.0,
            shared_types::CreateDeadlineRequest {
                title: deadline.description.clone(),
                case_id: Some(motion.case_id),
                rule_code: Some(deadline.rule_citation.clone()),
                due_at,
                notes: Some(deadline.computation_notes.clone()),
            },
        )
        .await;
    }

    // Advance case status if dispositive
    for sc in &report.status_changes {
        tracing::info!(
            case_id = %motion.case_id,
            new_status = %sc.new_status,
            rule = %sc.rule_name,
            "Compliance engine advancing case status from motion ruling"
        );
        let _ = crate::repo::case::update_status(&pool, &court.0, motion.case_id, &sc.new_status).await;
    }

    // Log case event
    let report_json = serde_json::to_value(&report).ok();
    let _ = crate::repo::case_event::insert(
        &pool,
        &court.0,
        motion.case_id,
        "criminal",
        "motion_ruled",
        None,
        &serde_json::json!({
            "motion_id": motion.id.to_string(),
            "motion_type": motion.motion_type,
            "disposition": body.disposition,
            "order_id": order.id.to_string(),
        }),
        report_json.as_ref(),
    )
    .await;

    Ok(Json(JudicialOrderResponse::from(order)))
}
```

**Step 2: Register the route in the API router**

In `crates/server/src/api.rs` (the router setup), add:
```rust
.route("/api/motions/{id}/rule", post(rest::motion::rule_motion))
```

Find where `/api/motions/{id}` routes are registered and add it there.

**Step 3: Add the server function wrapper in `api/judge.rs`**

After `list_pending_motions_for_judge`, add:

```rust
/// Judge rules on a motion — updates status, creates order.
#[server]
pub async fn rule_on_motion(
    court_id: String,
    motion_id: String,
    disposition: String,
    ruling_text: Option<String>,
    judge_id: String,
    judge_name: String,
) -> Result<shared_types::JudicialOrderResponse, ServerFnError> {
    use crate::db::get_db;
    use shared_types::{AppError, RuleMotionRequest};

    let db = get_db().await;
    let client = reqwest::Client::new();
    let base = crate::internal_api_base();

    let req = RuleMotionRequest {
        disposition,
        ruling_text,
        judge_id,
        judge_name,
    };

    let resp = client
        .post(format!("{}/api/motions/{}/rule", base, motion_id))
        .header("x-court-district", &court_id)
        .json(&req)
        .send()
        .await
        .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(AppError::internal(msg).into_server_fn_error());
    }

    let order: shared_types::JudicialOrderResponse = resp
        .json()
        .await
        .map_err(|e| AppError::internal(e.to_string()).into_server_fn_error())?;

    Ok(order)
}
```

NOTE: Check if this project calls REST endpoints internally via HTTP or directly calls repo functions. Look at how existing server functions like `list_pending_motions_for_judge` work — they call `crate::repo::motion::list_pending_for_judge()` directly. **Follow that pattern instead of HTTP calls.** Rewrite the server function to call the repo and rest handler logic directly, matching the existing codebase pattern:

```rust
#[server]
pub async fn rule_on_motion(
    court_id: String,
    motion_id: String,
    disposition: String,
    ruling_text: Option<String>,
    judge_id: String,
    judge_name: String,
) -> Result<shared_types::JudicialOrderResponse, ServerFnError> {
    use crate::db::get_db;

    let db = get_db().await;
    let body = shared_types::RuleMotionRequest {
        disposition,
        ruling_text,
        judge_id,
        judge_name,
    };

    // Reuse the REST handler logic directly
    crate::rest::motion::rule_motion_inner(db, &court_id, &motion_id, body)
        .await
        .map_err(|e| e.into_server_fn_error())
}
```

This means you'll need to extract the core logic from `rule_motion` into a `rule_motion_inner` function that both the REST handler and server function can call. Pattern: REST handler wraps `rule_motion_inner(pool, court, id, body)`.

**Step 4: Verify compilation**

Run: `cargo check -p server`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/server/src/rest/motion.rs crates/server/src/api.rs crates/server/src/api/judge.rs
git commit -m "feat(server): add POST /api/motions/{id}/rule endpoint with engine integration"
```

---

### Task 3: Integration tests for motion ruling

**Files:**
- Modify: `crates/tests/src/lib.rs` (add test module)
- Create: `crates/tests/src/motion_ruling_tests.rs`

**Step 1: Write the test file**

```rust
use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;

/// Test: Rule on a pending motion → creates order + updates motion status
#[tokio::test]
async fn test_rule_motion_granted() {
    let (app, pool) = test_app().await;
    let court = "district9";

    // Setup: create case, judge, motion
    let case_id = create_test_case(&pool, court, "9:26-cr-99001").await;
    let judge_id = create_test_judge(&pool, court, "Test Judge Ruling").await;

    // Create a pending motion
    let motion_resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/motions")
                .header("content-type", "application/json")
                .header("x-court-district", court)
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({
                        "case_id": case_id,
                        "motion_type": "Suppress",
                        "filed_by": "Defense Attorney",
                        "description": "Motion to suppress evidence"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(motion_resp.status(), StatusCode::CREATED);
    let motion: serde_json::Value = parse_body(motion_resp).await;
    let motion_id = motion["id"].as_str().unwrap();

    // Rule on the motion
    let rule_resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(&format!("/api/motions/{}/rule", motion_id))
                .header("content-type", "application/json")
                .header("x-court-district", court)
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({
                        "disposition": "Granted",
                        "ruling_text": "The motion to suppress is GRANTED.",
                        "judge_id": judge_id,
                        "judge_name": "Test Judge Ruling"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rule_resp.status(), StatusCode::OK);
    let order: serde_json::Value = parse_body(rule_resp).await;

    // Verify the order was created
    assert_eq!(order["order_type"].as_str().unwrap(), "Procedural");
    assert!(order["title"].as_str().unwrap().contains("Suppress"));
    assert!(order["title"].as_str().unwrap().contains("Granted"));
    assert_eq!(order["status"].as_str().unwrap(), "Pending Signature");
    assert!(order["content"].as_str().unwrap().contains("GRANTED"));

    // Verify motion status was updated
    let motion_get = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(&format!("/api/motions/{}", motion_id))
                .header("x-court-district", court)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(motion_get.status(), StatusCode::OK);
    let updated_motion: serde_json::Value = parse_body(motion_get).await;
    assert_eq!(updated_motion["status"].as_str().unwrap(), "Granted");
    assert!(updated_motion["ruling_date"].as_str().is_some());
}

/// Test: Cannot rule on a non-pending motion
#[tokio::test]
async fn test_rule_motion_not_pending() {
    let (app, pool) = test_app().await;
    let court = "district9";

    let case_id = create_test_case(&pool, court, "9:26-cr-99002").await;
    let judge_id = create_test_judge(&pool, court, "Judge Non-Pending").await;

    // Create a motion that is already granted
    let motion_resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/motions")
                .header("content-type", "application/json")
                .header("x-court-district", court)
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({
                        "case_id": case_id,
                        "motion_type": "Dismiss",
                        "filed_by": "Defense",
                        "description": "Motion to dismiss",
                        "status": "Granted"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(motion_resp.status(), StatusCode::CREATED);
    let motion: serde_json::Value = parse_body(motion_resp).await;
    let motion_id = motion["id"].as_str().unwrap();

    // Try to rule on it — should fail
    let rule_resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(&format!("/api/motions/{}/rule", motion_id))
                .header("content-type", "application/json")
                .header("x-court-district", court)
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({
                        "disposition": "Denied",
                        "judge_id": judge_id,
                        "judge_name": "Judge Non-Pending"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rule_resp.status(), StatusCode::BAD_REQUEST);
}

/// Test: Invalid disposition is rejected
#[tokio::test]
async fn test_rule_motion_invalid_disposition() {
    let (app, pool) = test_app().await;
    let court = "district9";

    let case_id = create_test_case(&pool, court, "9:26-cr-99003").await;
    let judge_id = create_test_judge(&pool, court, "Judge Invalid").await;

    let motion_resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/motions")
                .header("content-type", "application/json")
                .header("x-court-district", court)
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({
                        "case_id": case_id,
                        "motion_type": "Compel",
                        "filed_by": "Plaintiff",
                        "description": "Motion to compel"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let motion: serde_json::Value = parse_body(motion_resp).await;
    let motion_id = motion["id"].as_str().unwrap();

    let rule_resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(&format!("/api/motions/{}/rule", motion_id))
                .header("content-type", "application/json")
                .header("x-court-district", court)
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({
                        "disposition": "Overruled",
                        "judge_id": judge_id,
                        "judge_name": "Judge Invalid"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rule_resp.status(), StatusCode::BAD_REQUEST);
}

/// Test: Denied motion — order is created with correct content
#[tokio::test]
async fn test_rule_motion_denied() {
    let (app, pool) = test_app().await;
    let court = "district9";

    let case_id = create_test_case(&pool, court, "9:26-cr-99004").await;
    let judge_id = create_test_judge(&pool, court, "Judge Denied").await;

    let motion_resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/motions")
                .header("content-type", "application/json")
                .header("x-court-district", court)
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({
                        "case_id": case_id,
                        "motion_type": "Dismiss",
                        "filed_by": "Defense Counsel",
                        "description": "Motion to dismiss for lack of evidence"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let motion: serde_json::Value = parse_body(motion_resp).await;
    let motion_id = motion["id"].as_str().unwrap();

    let rule_resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(&format!("/api/motions/{}/rule", motion_id))
                .header("content-type", "application/json")
                .header("x-court-district", court)
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({
                        "disposition": "Denied",
                        "ruling_text": "The Court finds insufficient grounds. Motion DENIED.",
                        "judge_id": judge_id,
                        "judge_name": "Judge Denied"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rule_resp.status(), StatusCode::OK);
    let order: serde_json::Value = parse_body(rule_resp).await;
    assert_eq!(order["order_type"].as_str().unwrap(), "Dismissal");
    assert!(order["content"].as_str().unwrap().contains("DENIED"));
}
```

**Step 2: Register the module in `lib.rs`**

Add `mod motion_ruling_tests;` to `crates/tests/src/lib.rs`.

**Step 3: Update TRUNCATE in `common.rs`**

Ensure `motions` table is in the TRUNCATE list.

**Step 4: Run tests**

Run: `cargo test -p tests -- motion_ruling --test-threads=1`
Expected: All 4 tests PASS

**Step 5: Commit**

```bash
git add crates/tests/src/motion_ruling_tests.rs crates/tests/src/lib.rs crates/tests/src/common.rs
git commit -m "test: add integration tests for motion ruling endpoint"
```

---

### Task 4: Judge dashboard inline ruling UI

**Files:**
- Modify: `crates/app/src/routes/dashboard/judge.rs`

**Step 1: Add ruling action UI to `MotionRow`**

Replace the current `MotionRow` component (lines 350-374) with an expanded version that includes ruling action buttons. When a judge clicks a disposition button, a ruling dialog opens with a text area for ruling text, then calls the server function.

The existing `MotionRow` currently just shows motion info and links to the case. Transform it to:

1. Show motion info + an expandable ruling panel
2. Ruling panel has: disposition buttons (Grant, Deny, Grant in Part, Set for Hearing, Under Advisement)
3. Ruling text textarea (pre-filled with template based on disposition)
4. Submit button that calls `server::api::rule_on_motion()`
5. On success: remove the motion from the list (it's no longer pending)

Key changes to `judge.rs`:

```rust
/// A single motion row with inline ruling action.
#[component]
fn MotionRow(motion: MotionResponse, on_ruled: EventHandler<()>) -> Element {
    let ctx = use_context::<CourtContext>();
    let auth = use_auth();

    let case_id = motion.case_id.clone();
    let case_display = motion
        .case_number
        .clone()
        .unwrap_or_else(|| truncate_id(&motion.case_id));
    let filed_date = format_date_short(&motion.filed_date);

    let mut expanded = use_signal(|| false);
    let mut selected_disposition = use_signal(|| Option::<String>::None);
    let mut ruling_text = use_signal(|| String::new());
    let mut submitting = use_signal(|| false);

    let handle_rule = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let motion_id = motion.id.clone();
        let disposition = selected_disposition.read().clone();
        let text = ruling_text.read().clone();
        let user = auth.current_user.read().clone();
        let on_ruled = on_ruled.clone();

        if let (Some(disposition), Some(user)) = (disposition, user) {
            let judge_id = user.linked_judge_id.unwrap_or_default();
            let judge_name = user.display_name.clone();

            spawn(async move {
                submitting.set(true);
                let result = server::api::rule_on_motion(
                    court,
                    motion_id,
                    disposition,
                    if text.is_empty() { None } else { Some(text) },
                    judge_id,
                    judge_name,
                )
                .await;
                submitting.set(false);
                if result.is_ok() {
                    on_ruled.call(());
                }
            });
        }
    };

    rsx! {
        DataTableRow {
            DataTableCell {
                Link { to: Route::CaseDetail { id: case_id.clone(), tab: None },
                    span { class: "judge-link", "{case_display}" }
                }
            }
            DataTableCell {
                Badge { variant: BadgeVariant::Outline, "{motion.motion_type}" }
            }
            DataTableCell { "{filed_date}" }
            DataTableCell { "{motion.filed_by}" }
            DataTableCell {
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| expanded.toggle(),
                    if *expanded.read() { "Cancel" } else { "Rule" }
                }
            }
        }
        if *expanded.read() {
            DataTableRow {
                DataTableCell { colspan: "5",
                    div { class: "judge-ruling-panel",
                        p { class: "judge-ruling-description", "{motion.description}" }

                        div { class: "judge-ruling-dispositions",
                            for disp in ["Granted", "Denied", "Granted in Part", "Set for Hearing", "Taken Under Advisement"] {
                                {
                                    let d = disp.to_string();
                                    let is_selected = selected_disposition.read().as_deref() == Some(disp);
                                    rsx! {
                                        Button {
                                            variant: if is_selected { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                            onclick: move |_| {
                                                selected_disposition.set(Some(d.clone()));
                                            },
                                            "{disp}"
                                        }
                                    }
                                }
                            }
                        }

                        if selected_disposition.read().is_some() {
                            textarea {
                                class: "input judge-ruling-text",
                                placeholder: "Ruling text (optional — a default will be generated)",
                                rows: 4,
                                value: "{ruling_text}",
                                oninput: move |evt: Event<FormData>| ruling_text.set(evt.value().clone()),
                            }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: handle_rule,
                                disabled: *submitting.read(),
                                if *submitting.read() { "Submitting..." } else { "Submit Ruling" }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

**Step 2: Update `PendingMotions` to pass refresh callback**

Change the `PendingMotions` component to accept a restart callback and pass `on_ruled` to each `MotionRow`:

```rust
#[component]
fn PendingMotions(
    data: Resource<Option<Vec<MotionResponse>>>,
) -> Element {
    // ... existing code, but change the MotionRow render:
    for motion in motions.iter() {
        MotionRow {
            motion: motion.clone(),
            on_ruled: move |_| data.restart(),
        }
    }
}
```

**Step 3: Add DataTableColumn "Action" header to Pending Motions table**

```rust
DataTableHeader {
    DataTableColumn { "Case" }
    DataTableColumn { "Motion Type" }
    DataTableColumn { "Filed" }
    DataTableColumn { "Filed By" }
    DataTableColumn { "Action" }
}
```

**Step 4: Add CSS for the ruling panel**

In `crates/app/src/routes/dashboard/judge.css`, add:

```css
.judge-ruling-panel {
    padding: 1rem;
    background: var(--color-surface-secondary, #f8f9fa);
    border-radius: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
}

.judge-ruling-description {
    font-style: italic;
    color: var(--color-text-secondary, #666);
    margin: 0;
}

.judge-ruling-dispositions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
}

.judge-ruling-text {
    width: 100%;
    min-height: 100px;
    resize: vertical;
}
```

**Step 5: Verify compilation**

Run: `cargo check -p app`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/app/src/routes/dashboard/judge.rs crates/app/src/routes/dashboard/judge.css
git commit -m "feat(ui): add inline motion ruling to judge dashboard"
```

---

### Task 5: Permission gating — fix case detail action buttons

**Files:**
- Modify: `crates/app/src/routes/cases/detail.rs`

**Step 1: Gate action buttons by role**

Currently the case detail shows Queue, Cases, Edit, and Delete buttons regardless of role. Fix this:

- **Queue**: Only show for Clerk and Admin (not Judge, not Attorney, not Public)
- **Cases**: Show for all authenticated users
- **Edit**: Only show for Clerk and Admin (Judge should not edit case metadata)
- **Delete**: Only show for Admin

Replace the `PageActions` block (lines 104-123) with:

```rust
PageActions {
    // "Back to Cases" — everyone gets this
    Link { to: Route::CaseList {},
        Button { variant: ButtonVariant::Secondary, "Cases" }
    }

    // Queue link — clerks and admin only
    if matches!(role, UserRole::Clerk | UserRole::Admin) {
        Link { to: Route::Dashboard {},
            Button { variant: ButtonVariant::Secondary, "Queue" }
        }
    }

    // Edit — clerks and admin only (judges don't edit case metadata)
    if matches!(role, UserRole::Clerk | UserRole::Admin) {
        Button {
            variant: ButtonVariant::Primary,
            onclick: move |_| show_edit.set(true),
            "Edit"
        }
    }

    // Delete — admin only
    if can(&role, Action::DeleteCase) {
        Button {
            variant: ButtonVariant::Destructive,
            onclick: move |_| show_delete_confirm.set(true),
            "Delete"
        }
    }
}
```

Add `UserRole` to the import from `shared_types`:
```rust
use shared_types::{CaseResponse, UserRole};
```

**Step 2: Verify compilation**

Run: `cargo check -p app`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/app/src/routes/cases/detail.rs
git commit -m "fix(ui): tighten case detail permission gating per role"
```

---

### Task 6: Add `RuleMotionAction` to judge's `Action` enum

**Files:**
- Modify: `crates/app/src/auth.rs`

**Step 1: Add new action variant**

Add `RuleMotion` to the `Action` enum:

```rust
pub enum Action {
    // ... existing variants ...

    // Judge ruling
    RuleMotion,
}
```

And in the `can()` function:

```rust
// Judge/Admin can rule on motions
Action::RuleMotion => matches!(role, UserRole::Judge | UserRole::Admin),
```

**Step 2: Verify compilation**

Run: `cargo check -p app`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/app/src/auth.rs
git commit -m "feat(auth): add RuleMotion action for judge permission gating"
```

---

### Task 7: Sign order inline from judge dashboard

**Files:**
- Modify: `crates/app/src/routes/dashboard/judge.rs`

**Step 1: Add [Sign] action to `OrderRow`**

Replace the current `OrderRow` component (which only has a "Review" badge) with one that has an inline [Sign] button:

```rust
#[component]
fn OrderRow(order: JudicialOrderResponse, on_signed: EventHandler<()>) -> Element {
    let ctx = use_context::<CourtContext>();
    let auth = use_auth();

    let case_id = order.case_id.clone();
    let case_display = order
        .case_number
        .clone()
        .unwrap_or_else(|| truncate_id(&order.case_id));
    let submitted = format_date_short(&order.created_at);

    let mut signing = use_signal(|| false);

    let handle_sign = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let order_id = order.id.clone();
        let user = auth.current_user.read().clone();
        let on_signed = on_signed.clone();

        if let Some(user) = user {
            let signer_name = user.display_name.clone();
            spawn(async move {
                signing.set(true);
                let result = server::api::sign_order_action(court, order_id, signer_name).await;
                signing.set(false);
                if result.is_ok() {
                    on_signed.call(());
                }
            });
        }
    };

    rsx! {
        DataTableRow {
            DataTableCell {
                Link { to: Route::CaseDetail { id: case_id.clone(), tab: Some("docket".to_string()) },
                    span { class: "judge-link", "{case_display}" }
                }
            }
            DataTableCell { "{order.title}" }
            DataTableCell { "{submitted}" }
            DataTableCell {
                div { class: "judge-order-actions",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: handle_sign,
                        disabled: *signing.read(),
                        if *signing.read() { "Signing..." } else { "Sign" }
                    }
                    Link { to: Route::CaseDetail { id: case_id, tab: Some("docket".to_string()) },
                        Button { variant: ButtonVariant::Secondary, "Review" }
                    }
                }
            }
        }
    }
}
```

**Step 2: Pass on_signed to OrderRow from OrdersPendingSignature**

```rust
for order in orders.iter() {
    OrderRow {
        order: order.clone(),
        on_signed: move |_| data.restart(),
    }
}
```

**Step 3: Add CSS for order actions**

```css
.judge-order-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
}
```

**Step 4: Verify compilation**

Run: `cargo check -p app`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/app/src/routes/dashboard/judge.rs crates/app/src/routes/dashboard/judge.css
git commit -m "feat(ui): add inline order signing to judge dashboard"
```

---

### Task 8: Add order signing → clerk queue item flow

**Files:**
- Modify: `crates/server/src/rest/order.rs`

**Step 1: After signing an order, auto-create clerk queue item**

In the `sign_order` handler (lines 486-537), after the order is updated to "Signed" and returned, add clerk queue item creation:

```rust
// After fetching the updated order:

// Auto-create clerk queue item: order is signed, clerk needs to file it
let _ = crate::repo::queue::create(
    &pool,
    &court.0,
    "order",
    2,
    &format!("File signed order: {}", order.title),
    Some("Signed by judge — ready for filing on docket"),
    "order",
    order.id,
    Some(order.case_id),
    None,
    None,
    None,
    "docket",
)
.await;

tracing::info!(
    order_id = %order.id,
    "Order signed — clerk queue item created for filing"
);
```

**Step 2: Log case event for order signing**

```rust
let _ = crate::repo::case_event::insert(
    &pool,
    &court.0,
    order.case_id,
    "criminal",
    "order_signed",
    None,
    &serde_json::json!({
        "order_id": order.id.to_string(),
        "order_type": order.order_type,
        "title": order.title,
        "signer": order.signer_name,
    }),
    None,
)
.await;
```

**Step 3: Verify compilation**

Run: `cargo check -p server`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/server/src/rest/order.rs
git commit -m "feat(server): auto-create clerk queue item when order is signed"
```

---

### Task 9: Run all tests

**Step 1: Run the full test suite**

Run: `cargo test -p tests -- --test-threads=1`
Expected: All tests PASS (existing + new motion ruling tests)

**Step 2: If any tests fail, fix them**

Common issues:
- TRUNCATE list missing `motions` table
- Queue unique constraint violations (need to add `case_events` to TRUNCATE)
- Compilation errors from import changes

**Step 3: Commit any fixes**

```bash
git add -A
git commit -m "fix: test suite passing with motion ruling workflow"
```

---

### Task 10: Final verification

**Step 1: Verify compilation of all crates**

Run: `cargo check -p server -p app -p shared-types -p shared-ui`
Expected: PASS

**Step 2: Run full test suite**

Run: `cargo test -p tests -- --test-threads=1`
Expected: All tests PASS

**Step 3: Manual smoke test**

1. Log in as judge_test
2. Navigate to Judicial Dashboard
3. Verify "Pending Motions" section shows motions with [Rule] buttons
4. Click [Rule] on a motion → disposition panel opens
5. Select "Granted" → ruling text area appears
6. Submit → motion disappears from list, order appears in "Orders Pending Signature"
7. Click [Sign] on the new order → order disappears
8. Log in as clerk_test → new queue item should appear in clerk dashboard

---
