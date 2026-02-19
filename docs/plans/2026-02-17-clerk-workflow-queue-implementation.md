# Clerk Workflow Queue Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build an event-driven clerk work queue that mirrors CM/ECF filing processing — inbox dashboard, claim/release, step-by-step pipeline (review → docket → NEF → route judge → serve).

**Architecture:** New `clerk_queue` table + repo + REST handlers + shared types. Queue items auto-created when filings/motions/orders are submitted. Clerk dashboard becomes a filterable work queue. Case detail page gets a workflow panel for step-by-step processing.

**Tech Stack:** Rust, Axum, sqlx (Postgres), Dioxus 0.7, shared-ui components

**Design Doc:** `docs/plans/2026-02-17-clerk-workflow-queue-design.md`

---

## PR 1: Database + Shared Types

### Task 1: Create clerk_queue migration

**Files:**
- Create: `migrations/20260217000074_create_clerk_queue.sql`
- Create: `migrations/20260217000074_create_clerk_queue.down.sql`

**Step 1: Write the up migration**

```sql
CREATE TABLE IF NOT EXISTS clerk_queue (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL REFERENCES courts(id),
    queue_type      TEXT NOT NULL CHECK (queue_type IN ('filing', 'motion', 'order', 'deadline_alert', 'general')),
    priority        INT NOT NULL DEFAULT 3 CHECK (priority BETWEEN 1 AND 4),
    status          TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'in_review', 'processing', 'completed', 'rejected')),
    title           TEXT NOT NULL,
    description     TEXT,
    source_type     TEXT NOT NULL CHECK (source_type IN ('filing', 'motion', 'order', 'document', 'deadline', 'calendar_event')),
    source_id       UUID NOT NULL,
    case_id         UUID REFERENCES criminal_cases(id) ON DELETE SET NULL,
    case_number     TEXT,
    assigned_to     INT REFERENCES users(id) ON DELETE SET NULL,
    submitted_by    INT REFERENCES users(id) ON DELETE SET NULL,
    current_step    TEXT NOT NULL DEFAULT 'review' CHECK (current_step IN ('review', 'docket', 'nef', 'route_judge', 'serve', 'completed')),
    metadata        JSONB DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ
);

-- Main queue listing: clerk opens dashboard, sees pending items sorted by priority then age
CREATE INDEX idx_clerk_queue_court_status ON clerk_queue(court_id, status, priority, created_at);

-- "My items" filter: clerk sees only items assigned to them
CREATE INDEX idx_clerk_queue_court_assigned ON clerk_queue(court_id, assigned_to, status);

-- Queue items by case: see all queue items for a specific case
CREATE INDEX idx_clerk_queue_court_case ON clerk_queue(court_id, case_id);

-- Lookup by source entity: check if a queue item already exists for a filing/motion/order
CREATE INDEX idx_clerk_queue_source ON clerk_queue(source_type, source_id);

-- Prevent duplicate queue items for the same source entity in the same court
CREATE UNIQUE INDEX idx_clerk_queue_unique_source ON clerk_queue(court_id, source_type, source_id)
    WHERE status NOT IN ('completed', 'rejected');
```

**Step 2: Write the down migration**

```sql
DROP TABLE IF EXISTS clerk_queue;
```

**Step 3: Run the migration**

Run: `sqlx migrate run`
Expected: Migration applied successfully.

**Step 4: Commit**

```
git add migrations/20260217000074_*
git commit -m "feat(queue): add clerk_queue table migration"
```

---

### Task 2: Add queue shared types

**Files:**
- Create: `crates/shared-types/src/queue.rs`
- Modify: `crates/shared-types/src/lib.rs`

**Step 1: Create the queue types module**

Create `crates/shared-types/src/queue.rs` with these types:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const QUEUE_TYPES: &[&str] = &["filing", "motion", "order", "deadline_alert", "general"];
pub const QUEUE_STATUSES: &[&str] = &["pending", "in_review", "processing", "completed", "rejected"];
pub const QUEUE_STEPS: &[&str] = &["review", "docket", "nef", "route_judge", "serve", "completed"];
pub const QUEUE_SOURCE_TYPES: &[&str] = &["filing", "motion", "order", "document", "deadline", "calendar_event"];

pub fn is_valid_queue_type(s: &str) -> bool {
    QUEUE_TYPES.contains(&s)
}

pub fn is_valid_queue_status(s: &str) -> bool {
    QUEUE_STATUSES.contains(&s)
}

pub fn is_valid_queue_step(s: &str) -> bool {
    QUEUE_STEPS.contains(&s)
}

// ---------------------------------------------------------------------------
// Database Row
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct QueueItem {
    pub id: Uuid,
    pub court_id: String,
    pub queue_type: String,
    pub priority: i32,
    pub status: String,
    pub title: String,
    pub description: Option<String>,
    pub source_type: String,
    pub source_id: Uuid,
    pub case_id: Option<Uuid>,
    pub case_number: Option<String>,
    pub assigned_to: Option<i32>,
    pub submitted_by: Option<i32>,
    pub current_step: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// API Response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct QueueItemResponse {
    pub id: String,
    pub court_id: String,
    pub queue_type: String,
    pub priority: i32,
    pub status: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub source_type: String,
    pub source_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_by: Option<i32>,
    pub current_step: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

impl From<QueueItem> for QueueItemResponse {
    fn from(q: QueueItem) -> Self {
        Self {
            id: q.id.to_string(),
            court_id: q.court_id,
            queue_type: q.queue_type,
            priority: q.priority,
            status: q.status,
            title: q.title,
            description: q.description,
            source_type: q.source_type,
            source_id: q.source_id.to_string(),
            case_id: q.case_id.map(|u| u.to_string()),
            case_number: q.case_number,
            assigned_to: q.assigned_to,
            submitted_by: q.submitted_by,
            current_step: q.current_step,
            metadata: Some(q.metadata),
            created_at: q.created_at.to_rfc3339(),
            updated_at: q.updated_at.to_rfc3339(),
            completed_at: q.completed_at.map(|d| d.to_rfc3339()),
        }
    }
}

// ---------------------------------------------------------------------------
// Search Response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct QueueSearchResponse {
    pub items: Vec<QueueItemResponse>,
    pub total: i64,
}

// ---------------------------------------------------------------------------
// Stats Response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct QueueStats {
    pub pending_count: i64,
    pub my_count: i64,
    pub today_count: i64,
    pub urgent_count: i64,
    pub avg_processing_mins: Option<f64>,
}

// ---------------------------------------------------------------------------
// Request Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateQueueItemRequest {
    pub queue_type: String,
    pub priority: Option<i32>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub source_type: String,
    pub source_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_by: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Search/filter params for GET /api/queue
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct QueueSearchParams {
    pub status: Option<String>,
    pub queue_type: Option<String>,
    pub priority: Option<i32>,
    pub assigned_to: Option<i32>,
    pub case_id: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

/// POST /api/queue/{id}/advance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdvanceQueueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_data: Option<serde_json::Value>,
}

/// POST /api/queue/{id}/reject
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RejectQueueRequest {
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Pipeline Step Mapping
// ---------------------------------------------------------------------------

/// Returns the ordered pipeline steps for a given queue_type.
pub fn pipeline_steps(queue_type: &str) -> Vec<&'static str> {
    match queue_type {
        "filing" => vec!["review", "docket", "nef", "serve"],
        "motion" => vec!["review", "docket", "nef", "route_judge", "serve"],
        "order"  => vec!["docket", "nef", "serve"],
        "deadline_alert" | "general" => vec!["review"],
        _ => vec!["review"],
    }
}

/// Returns the next step after `current` for a given queue_type, or None if at the end.
pub fn next_step(queue_type: &str, current: &str) -> Option<&'static str> {
    let steps = pipeline_steps(queue_type);
    let pos = steps.iter().position(|&s| s == current)?;
    steps.get(pos + 1).copied()
}
```

**Step 2: Register the module in lib.rs**

Add to `crates/shared-types/src/lib.rs` after the `pub mod victim;` line:
```rust
pub mod queue;
```

And add the re-export after `pub use victim::*;`:
```rust
pub use queue::*;
```

**Step 3: Verify compilation**

Run: `cargo check -p shared-types`
Expected: Compiles without errors.

**Step 4: Commit**

```
git add crates/shared-types/src/queue.rs crates/shared-types/src/lib.rs
git commit -m "feat(queue): add queue shared types and pipeline step mapping"
```

---

## PR 2: Repo + REST Handlers

### Task 3: Create queue repo module

**Files:**
- Create: `crates/server/src/repo/queue.rs`
- Modify: `crates/server/src/repo/mod.rs`

**Step 1: Write the repo module**

Create `crates/server/src/repo/queue.rs` with CRUD + search + stats:

```rust
use chrono::{DateTime, Utc};
use shared_types::{AppError, QueueItem, QueueStats};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    queue_type: &str,
    priority: i32,
    title: &str,
    description: Option<&str>,
    source_type: &str,
    source_id: Uuid,
    case_id: Option<Uuid>,
    case_number: Option<&str>,
    submitted_by: Option<i32>,
    metadata: Option<serde_json::Value>,
    first_step: &str,
) -> Result<QueueItem, AppError> {
    sqlx::query_as!(
        QueueItem,
        r#"
        INSERT INTO clerk_queue
            (court_id, queue_type, priority, title, description,
             source_type, source_id, case_id, case_number, submitted_by,
             metadata, current_step)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, court_id, queue_type, priority, status, title,
                  description, source_type, source_id, case_id, case_number,
                  assigned_to, submitted_by, current_step,
                  metadata, created_at, updated_at, completed_at
        "#,
        court_id,
        queue_type,
        priority,
        title,
        description,
        source_type,
        source_id,
        case_id,
        case_number,
        submitted_by,
        metadata.unwrap_or(serde_json::json!({})),
        first_step,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<QueueItem>, AppError> {
    sqlx::query_as!(
        QueueItem,
        r#"
        SELECT id, court_id, queue_type, priority, status, title,
               description, source_type, source_id, case_id, case_number,
               assigned_to, submitted_by, current_step,
               metadata, created_at, updated_at, completed_at
        FROM clerk_queue
        WHERE id = $1 AND court_id = $2
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

pub async fn search(
    pool: &Pool<Postgres>,
    court_id: &str,
    status: Option<&str>,
    queue_type: Option<&str>,
    priority: Option<i32>,
    assigned_to: Option<i32>,
    case_id: Option<Uuid>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<QueueItem>, i64), AppError> {
    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM clerk_queue
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR status = $2)
          AND ($3::TEXT IS NULL OR queue_type = $3)
          AND ($4::INT IS NULL OR priority = $4)
          AND ($5::INT IS NULL OR assigned_to = $5)
          AND ($6::UUID IS NULL OR case_id = $6)
        "#,
        court_id,
        status,
        queue_type,
        priority,
        assigned_to,
        case_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        QueueItem,
        r#"
        SELECT id, court_id, queue_type, priority, status, title,
               description, source_type, source_id, case_id, case_number,
               assigned_to, submitted_by, current_step,
               metadata, created_at, updated_at, completed_at
        FROM clerk_queue
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR status = $2)
          AND ($3::TEXT IS NULL OR queue_type = $3)
          AND ($4::INT IS NULL OR priority = $4)
          AND ($5::INT IS NULL OR assigned_to = $5)
          AND ($6::UUID IS NULL OR case_id = $6)
        ORDER BY priority ASC, created_at ASC
        LIMIT $7 OFFSET $8
        "#,
        court_id,
        status,
        queue_type,
        priority,
        assigned_to,
        case_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, total))
}

pub async fn stats(
    pool: &Pool<Postgres>,
    court_id: &str,
    user_id: Option<i32>,
) -> Result<QueueStats, AppError> {
    let pending_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM clerk_queue WHERE court_id = $1 AND status = 'pending'"#,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let my_count = match user_id {
        Some(uid) => sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM clerk_queue WHERE court_id = $1 AND assigned_to = $2 AND status IN ('in_review', 'processing')"#,
            court_id,
            uid,
        )
        .fetch_one(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?,
        None => 0,
    };

    let today_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM clerk_queue WHERE court_id = $1 AND created_at >= CURRENT_DATE AND status NOT IN ('completed', 'rejected')"#,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let urgent_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM clerk_queue WHERE court_id = $1 AND priority <= 2 AND status NOT IN ('completed', 'rejected')"#,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let avg_processing_mins = sqlx::query_scalar!(
        r#"
        SELECT EXTRACT(EPOCH FROM AVG(completed_at - created_at)) / 60.0 as "avg?"
        FROM clerk_queue
        WHERE court_id = $1 AND status = 'completed' AND completed_at IS NOT NULL
        "#,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(QueueStats {
        pending_count,
        my_count,
        today_count,
        urgent_count,
        avg_processing_mins: avg_processing_mins.map(|f| f as f64),
    })
}

pub async fn claim(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    user_id: i32,
) -> Result<Option<QueueItem>, AppError> {
    sqlx::query_as!(
        QueueItem,
        r#"
        UPDATE clerk_queue SET
            assigned_to = $3,
            status = 'in_review',
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2 AND assigned_to IS NULL AND status = 'pending'
        RETURNING id, court_id, queue_type, priority, status, title,
                  description, source_type, source_id, case_id, case_number,
                  assigned_to, submitted_by, current_step,
                  metadata, created_at, updated_at, completed_at
        "#,
        id,
        court_id,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

pub async fn release(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    user_id: i32,
) -> Result<Option<QueueItem>, AppError> {
    sqlx::query_as!(
        QueueItem,
        r#"
        UPDATE clerk_queue SET
            assigned_to = NULL,
            status = 'pending',
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2 AND assigned_to = $3
        RETURNING id, court_id, queue_type, priority, status, title,
                  description, source_type, source_id, case_id, case_number,
                  assigned_to, submitted_by, current_step,
                  metadata, created_at, updated_at, completed_at
        "#,
        id,
        court_id,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

pub async fn advance(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    next_step: &str,
    new_status: &str,
    completed_at: Option<DateTime<Utc>>,
) -> Result<Option<QueueItem>, AppError> {
    sqlx::query_as!(
        QueueItem,
        r#"
        UPDATE clerk_queue SET
            current_step = $3,
            status = $4,
            completed_at = $5,
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, queue_type, priority, status, title,
                  description, source_type, source_id, case_id, case_number,
                  assigned_to, submitted_by, current_step,
                  metadata, created_at, updated_at, completed_at
        "#,
        id,
        court_id,
        next_step,
        new_status,
        completed_at,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

pub async fn reject(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    reason: &str,
) -> Result<Option<QueueItem>, AppError> {
    let metadata_patch = serde_json::json!({ "reject_reason": reason });
    sqlx::query_as!(
        QueueItem,
        r#"
        UPDATE clerk_queue SET
            status = 'rejected',
            metadata = metadata || $3,
            completed_at = NOW(),
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, queue_type, priority, status, title,
                  description, source_type, source_id, case_id, case_number,
                  assigned_to, submitted_by, current_step,
                  metadata, created_at, updated_at, completed_at
        "#,
        id,
        court_id,
        metadata_patch,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}
```

**Step 2: Register in repo/mod.rs**

Add to `crates/server/src/repo/mod.rs` after the last module:
```rust
#[cfg(feature = "server")]
pub mod queue;
```

**Step 3: Verify compilation**

Run: `cargo check -p server`
Expected: Compiles without errors.

**Step 4: Commit**

```
git add crates/server/src/repo/queue.rs crates/server/src/repo/mod.rs
git commit -m "feat(queue): add queue repo with CRUD, search, stats, claim/release, advance/reject"
```

---

### Task 4: Create queue REST handlers

**Files:**
- Create: `crates/server/src/rest/queue.rs`
- Modify: `crates/server/src/rest/mod.rs`

**Step 1: Write the REST handler module**

Create `crates/server/src/rest/queue.rs`:

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::{
    AdvanceQueueRequest, AppError, CreateQueueItemRequest, QueueItemResponse,
    QueueSearchParams, QueueSearchResponse, QueueStats, RejectQueueRequest,
};
use crate::error_convert::SqlxErrorExt;
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// GET /api/queue
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/api/queue",
    params(QueueSearchParams),
    responses(
        (status = 200, description = "Queue items", body = QueueSearchResponse)
    ),
    tag = "queue"
)]
pub async fn list_queue(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<QueueSearchParams>,
) -> Result<Json<QueueSearchResponse>, AppError> {
    let case_uuid = params.case_id.as_deref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| AppError::bad_request("invalid case_id UUID"))?;

    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);

    let (items, total) = crate::repo::queue::search(
        &pool,
        &court.0,
        params.status.as_deref(),
        params.queue_type.as_deref(),
        params.priority,
        params.assigned_to,
        case_uuid,
        offset,
        limit,
    )
    .await?;

    Ok(Json(QueueSearchResponse {
        items: items.into_iter().map(QueueItemResponse::from).collect(),
        total,
    }))
}

// ---------------------------------------------------------------------------
// GET /api/queue/stats
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/api/queue/stats",
    responses(
        (status = 200, description = "Queue statistics", body = QueueStats)
    ),
    tag = "queue"
)]
pub async fn queue_stats(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Query(params): Query<QueueStatsParams>,
) -> Result<Json<QueueStats>, AppError> {
    let stats = crate::repo::queue::stats(&pool, &court.0, params.user_id).await?;
    Ok(Json(stats))
}

#[derive(Debug, serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct QueueStatsParams {
    pub user_id: Option<i32>,
}

// ---------------------------------------------------------------------------
// GET /api/queue/{id}
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/api/queue/{id}",
    responses(
        (status = 200, description = "Queue item", body = QueueItemResponse),
        (status = 404, description = "Not found")
    ),
    tag = "queue"
)]
pub async fn get_queue_item(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<Uuid>,
) -> Result<Json<QueueItemResponse>, AppError> {
    let item = crate::repo::queue::find_by_id(&pool, &court.0, id)
        .await?
        .ok_or_else(|| AppError::not_found("Queue item not found"))?;
    Ok(Json(QueueItemResponse::from(item)))
}

// ---------------------------------------------------------------------------
// POST /api/queue
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/queue",
    request_body = CreateQueueItemRequest,
    responses(
        (status = 201, description = "Queue item created", body = QueueItemResponse)
    ),
    tag = "queue"
)]
pub async fn create_queue_item(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CreateQueueItemRequest>,
) -> Result<(StatusCode, Json<QueueItemResponse>), AppError> {
    if body.title.trim().is_empty() {
        return Err(AppError::bad_request("title must not be empty"));
    }
    if !shared_types::is_valid_queue_type(&body.queue_type) {
        return Err(AppError::bad_request(format!("invalid queue_type: {}", body.queue_type)));
    }
    if !shared_types::QUEUE_SOURCE_TYPES.contains(&body.source_type.as_str()) {
        return Err(AppError::bad_request(format!("invalid source_type: {}", body.source_type)));
    }

    let source_uuid = Uuid::parse_str(&body.source_id)
        .map_err(|_| AppError::bad_request("invalid source_id UUID"))?;
    let case_uuid = body.case_id.as_deref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| AppError::bad_request("invalid case_id UUID"))?;

    let steps = shared_types::pipeline_steps(&body.queue_type);
    let first_step = steps.first().copied().unwrap_or("review");

    let item = crate::repo::queue::create(
        &pool,
        &court.0,
        &body.queue_type,
        body.priority.unwrap_or(3),
        &body.title,
        body.description.as_deref(),
        &body.source_type,
        source_uuid,
        case_uuid,
        body.case_number.as_deref(),
        body.submitted_by,
        body.metadata.clone(),
        first_step,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(QueueItemResponse::from(item))))
}

// ---------------------------------------------------------------------------
// POST /api/queue/{id}/claim
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/queue/{id}/claim",
    responses(
        (status = 200, description = "Claimed", body = QueueItemResponse),
        (status = 409, description = "Already claimed")
    ),
    tag = "queue"
)]
pub async fn claim_queue_item(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<Uuid>,
    Json(body): Json<ClaimRequest>,
) -> Result<Json<QueueItemResponse>, AppError> {
    let item = crate::repo::queue::claim(&pool, &court.0, id, body.user_id)
        .await?
        .ok_or_else(|| AppError::conflict("Queue item is not available for claiming"))?;
    Ok(Json(QueueItemResponse::from(item)))
}

#[derive(Debug, serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ClaimRequest {
    pub user_id: i32,
}

// ---------------------------------------------------------------------------
// POST /api/queue/{id}/release
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/queue/{id}/release",
    responses(
        (status = 200, description = "Released", body = QueueItemResponse),
        (status = 404, description = "Not found or not assigned to user")
    ),
    tag = "queue"
)]
pub async fn release_queue_item(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<Uuid>,
    Json(body): Json<ClaimRequest>,
) -> Result<Json<QueueItemResponse>, AppError> {
    let item = crate::repo::queue::release(&pool, &court.0, id, body.user_id)
        .await?
        .ok_or_else(|| AppError::not_found("Queue item not found or not assigned to this user"))?;
    Ok(Json(QueueItemResponse::from(item)))
}

// ---------------------------------------------------------------------------
// POST /api/queue/{id}/advance
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/queue/{id}/advance",
    request_body = AdvanceQueueRequest,
    responses(
        (status = 200, description = "Advanced", body = QueueItemResponse),
        (status = 404, description = "Not found"),
        (status = 400, description = "Cannot advance - already at final step")
    ),
    tag = "queue"
)]
pub async fn advance_queue_item(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<Uuid>,
    Json(_body): Json<AdvanceQueueRequest>,
) -> Result<Json<QueueItemResponse>, AppError> {
    let current = crate::repo::queue::find_by_id(&pool, &court.0, id)
        .await?
        .ok_or_else(|| AppError::not_found("Queue item not found"))?;

    match shared_types::next_step(&current.queue_type, &current.current_step) {
        Some(next) => {
            let (status, completed_at) = if shared_types::next_step(&current.queue_type, next).is_none() {
                // This is the last step — completing it finishes the item
                ("completed", Some(Utc::now()))
            } else {
                ("processing", None)
            };
            let item = crate::repo::queue::advance(&pool, &court.0, id, next, status, completed_at)
                .await?
                .ok_or_else(|| AppError::not_found("Queue item not found"))?;
            Ok(Json(QueueItemResponse::from(item)))
        }
        None => {
            // Already at last step — mark as completed
            let item = crate::repo::queue::advance(&pool, &court.0, id, "completed", "completed", Some(Utc::now()))
                .await?
                .ok_or_else(|| AppError::not_found("Queue item not found"))?;
            Ok(Json(QueueItemResponse::from(item)))
        }
    }
}

// ---------------------------------------------------------------------------
// POST /api/queue/{id}/reject
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/queue/{id}/reject",
    request_body = RejectQueueRequest,
    responses(
        (status = 200, description = "Rejected", body = QueueItemResponse),
        (status = 404, description = "Not found")
    ),
    tag = "queue"
)]
pub async fn reject_queue_item(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(id): Path<Uuid>,
    Json(body): Json<RejectQueueRequest>,
) -> Result<Json<QueueItemResponse>, AppError> {
    if body.reason.trim().is_empty() {
        return Err(AppError::bad_request("reason must not be empty"));
    }
    let item = crate::repo::queue::reject(&pool, &court.0, id, &body.reason)
        .await?
        .ok_or_else(|| AppError::not_found("Queue item not found"))?;
    Ok(Json(QueueItemResponse::from(item)))
}
```

**Step 2: Register routes in rest/mod.rs**

Add `pub mod queue;` to the module declarations at the top of `crates/server/src/rest/mod.rs`.

Add these routes in the `api_router()` function:
```rust
        // Queue
        .route("/api/queue", get(queue::list_queue).post(queue::create_queue_item))
        .route("/api/queue/stats", get(queue::queue_stats))
        .route("/api/queue/{id}", get(queue::get_queue_item))
        .route("/api/queue/{id}/claim", post(queue::claim_queue_item))
        .route("/api/queue/{id}/release", post(queue::release_queue_item))
        .route("/api/queue/{id}/advance", post(queue::advance_queue_item))
        .route("/api/queue/{id}/reject", post(queue::reject_queue_item))
```

**Step 3: Verify compilation**

Run: `cargo check -p server`
Expected: Compiles without errors.

**Step 4: Commit**

```
git add crates/server/src/rest/queue.rs crates/server/src/rest/mod.rs
git commit -m "feat(queue): add queue REST handlers with list, stats, claim, advance, reject"
```

---

## PR 3: Integration Tests

### Task 5: Queue CRUD tests

**Files:**
- Create: `crates/tests/src/queue_create_tests.rs`
- Modify: `crates/tests/src/lib.rs`
- Modify: `crates/tests/src/common.rs` (add clerk_queue to TRUNCATE + helper)

**Step 1: Add clerk_queue to TRUNCATE in common.rs**

Update BOTH TRUNCATE statements in `crates/tests/src/common.rs` to include `clerk_queue`:

Add `clerk_queue` to the comma-separated list in both truncate queries. The table should be added before `CASCADE`.

**Step 2: Add create_test_queue_item helper to common.rs**

Add to `crates/tests/src/common.rs`:

```rust
pub async fn create_test_queue_item(
    app: &Router,
    court: &str,
    title: &str,
    source_id: &str,
) -> serde_json::Value {
    let body = serde_json::json!({
        "queue_type": "filing",
        "priority": 3,
        "title": title,
        "source_type": "filing",
        "source_id": source_id,
    });
    let (status, resp) = post_json(app, "/api/queue", &body.to_string(), court).await;
    assert_eq!(status, StatusCode::CREATED, "Failed to create queue item: {resp}");
    resp
}
```

**Step 3: Write queue creation tests**

Create `crates/tests/src/queue_create_tests.rs`:

```rust
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
```

**Step 4: Register in lib.rs**

Add to `crates/tests/src/lib.rs`:
```rust
#[cfg(test)]
mod queue_create_tests;
```

**Step 5: Run tests**

Run: `cargo test -p tests -- queue_create --test-threads=1`
Expected: All tests pass.

**Step 6: Commit**

```
git add crates/tests/src/queue_create_tests.rs crates/tests/src/lib.rs crates/tests/src/common.rs
git commit -m "test(queue): add queue creation, get, list, and isolation tests"
```

---

### Task 6: Queue claim/release/advance/reject tests

**Files:**
- Create: `crates/tests/src/queue_workflow_tests.rs`
- Modify: `crates/tests/src/lib.rs`

**Step 1: Write workflow tests**

Create `crates/tests/src/queue_workflow_tests.rs`:

```rust
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
```

**Step 2: Register in lib.rs**

Add to `crates/tests/src/lib.rs`:
```rust
#[cfg(test)]
mod queue_workflow_tests;
```

**Step 3: Run tests**

Run: `cargo test -p tests -- queue_workflow --test-threads=1`
Expected: All tests pass.

**Step 4: Commit**

```
git add crates/tests/src/queue_workflow_tests.rs crates/tests/src/lib.rs
git commit -m "test(queue): add claim, release, advance pipeline, reject, and stats tests"
```

---

## PR 4: Server Functions + Clerk Dashboard UI

### Task 7: Add queue server functions

**Files:**
- Modify: `crates/server/src/api.rs`

**Step 1: Add queue server functions**

Add these server functions to `crates/server/src/api.rs` (follow the pattern of existing server functions like `search_deadlines`):

```rust
#[server]
pub async fn search_queue(
    court_id: String,
    status: Option<String>,
    queue_type: Option<String>,
    priority: Option<i32>,
    assigned_to: Option<i32>,
    case_id: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<String, ServerFnError> {
    let pool = crate::db::pool().await?;
    let case_uuid = case_id.as_deref()
        .map(|s| uuid::Uuid::parse_str(s))
        .transpose()
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let (items, total) = crate::repo::queue::search(
        &pool,
        &court_id,
        status.as_deref(),
        queue_type.as_deref(),
        priority,
        assigned_to,
        case_uuid,
        offset.unwrap_or(0),
        limit.unwrap_or(20).clamp(1, 100),
    )
    .await
    .map_err(|e| ServerFnError::new(e.message))?;

    let resp = shared_types::QueueSearchResponse {
        items: items.into_iter().map(shared_types::QueueItemResponse::from).collect(),
        total,
    };
    serde_json::to_string(&resp).map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_queue_stats(
    court_id: String,
    user_id: Option<i32>,
) -> Result<String, ServerFnError> {
    let pool = crate::db::pool().await?;
    let stats = crate::repo::queue::stats(&pool, &court_id, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.message))?;
    serde_json::to_string(&stats).map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn claim_queue_item(
    court_id: String,
    id: String,
    user_id: i32,
) -> Result<String, ServerFnError> {
    let pool = crate::db::pool().await?;
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
    let item = crate::repo::queue::claim(&pool, &court_id, uuid, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.message))?
        .ok_or_else(|| ServerFnError::new("Item not available for claiming".to_string()))?;
    serde_json::to_string(&shared_types::QueueItemResponse::from(item))
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn release_queue_item_fn(
    court_id: String,
    id: String,
    user_id: i32,
) -> Result<String, ServerFnError> {
    let pool = crate::db::pool().await?;
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
    let item = crate::repo::queue::release(&pool, &court_id, uuid, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.message))?
        .ok_or_else(|| ServerFnError::new("Item not found or not assigned to user".to_string()))?;
    serde_json::to_string(&shared_types::QueueItemResponse::from(item))
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn advance_queue_item_fn(
    court_id: String,
    id: String,
) -> Result<String, ServerFnError> {
    let pool = crate::db::pool().await?;
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
    let current = crate::repo::queue::find_by_id(&pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.message))?
        .ok_or_else(|| ServerFnError::new("Queue item not found".to_string()))?;

    let (next, status, completed_at) = match shared_types::next_step(&current.queue_type, &current.current_step) {
        Some(next) => {
            if shared_types::next_step(&current.queue_type, next).is_none() {
                (next, "completed", Some(chrono::Utc::now()))
            } else {
                (next, "processing", None)
            }
        }
        None => ("completed", "completed", Some(chrono::Utc::now())),
    };

    let item = crate::repo::queue::advance(&pool, &court_id, uuid, next, status, completed_at)
        .await
        .map_err(|e| ServerFnError::new(e.message))?
        .ok_or_else(|| ServerFnError::new("Queue item not found".to_string()))?;
    serde_json::to_string(&shared_types::QueueItemResponse::from(item))
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn reject_queue_item_fn(
    court_id: String,
    id: String,
    reason: String,
) -> Result<String, ServerFnError> {
    let pool = crate::db::pool().await?;
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
    let item = crate::repo::queue::reject(&pool, &court_id, uuid, &reason)
        .await
        .map_err(|e| ServerFnError::new(e.message))?
        .ok_or_else(|| ServerFnError::new("Queue item not found".to_string()))?;
    serde_json::to_string(&shared_types::QueueItemResponse::from(item))
        .map_err(|e| ServerFnError::new(e.to_string()))
}
```

**Step 2: Verify compilation**

Run: `cargo check -p server`
Expected: Compiles without errors.

**Step 3: Commit**

```
git add crates/server/src/api.rs
git commit -m "feat(queue): add queue server functions for Dioxus frontend"
```

---

### Task 8: Rewrite Clerk Dashboard as Queue View

**Files:**
- Modify: `crates/app/src/routes/dashboard/clerk.rs`

**Step 1: Rewrite the clerk dashboard**

Replace the current clerk dashboard with the queue-driven design. The component should:

1. Fetch queue stats via `server::api::get_queue_stats(court, user_id)`
2. Fetch queue items via `server::api::search_queue(court, status, queue_type, priority, ...)`
3. Show 4 stat cards: Pending, My Items, Today, Urgent
4. Show filter bar with dropdowns for type/priority/assignment
5. Show queue item list sorted by priority then date
6. Each item has Claim button (if unassigned) or Continue button (if assigned to current user)
7. Clicking an item navigates to `/cases/{case_id}?queue={queue_id}`

Use shared-ui components: `Card`, `Badge`, `Button`, `Skeleton` for loading, standard layout patterns.

**Key patterns to follow:**
- Use `use_resource()` for data fetching (matches existing dashboard pattern)
- Use `CourtContext` for court_id
- Use `use_navigator()` for navigation to case detail
- Filter state via `use_signal()` for each filter dropdown
- Priority badge colors: 1=Destructive, 2=Warning (use Outline), 3=Secondary, 4=default

**Step 2: Verify compilation**

Run: `cargo check -p app`
Expected: Compiles without errors (or only pre-existing warnings).

**Step 3: Commit**

```
git add crates/app/src/routes/dashboard/clerk.rs
git commit -m "feat(ui): rewrite clerk dashboard as filing queue inbox"
```

---

### Task 9: Add queue stub to Judge and Attorney dashboards

**Files:**
- Modify: `crates/app/src/routes/dashboard/judge.rs`
- Modify: `crates/app/src/routes/dashboard/attorney.rs`

**Step 1: Add empty queue section to Judge dashboard**

Below the existing stats cards in `judge.rs`, add a "Pending Rulings" section:
- Card with title "Pending Rulings"
- Empty state: icon + "No items pending" text + "Items requiring your attention will appear here"
- This is a placeholder for future judge queue integration

**Step 2: Add empty queue section to Attorney dashboard**

Below the existing stats cards in `attorney.rs`, add a "My Filings" section:
- Card with title "My Filings"
- Empty state: icon + "No pending filings" text + "Track your filed documents here"

**Step 3: Verify compilation**

Run: `cargo check -p app`
Expected: Compiles without errors.

**Step 4: Commit**

```
git add crates/app/src/routes/dashboard/judge.rs crates/app/src/routes/dashboard/attorney.rs
git commit -m "feat(ui): add empty queue stubs to judge and attorney dashboards"
```

---

## PR 5: Workflow Panel on Case Detail

### Task 10: Add workflow panel component to case detail

**Files:**
- Create: `crates/app/src/routes/cases/workflow_panel.rs`
- Modify: `crates/app/src/routes/cases/detail.rs`
- Modify: `crates/app/src/routes/cases/mod.rs`

**Step 1: Create the workflow panel component**

Create `crates/app/src/routes/cases/workflow_panel.rs` — a component that:

1. Takes a `queue_id: String` prop
2. Fetches the queue item via `server::api::search_queue` with the queue ID
3. Shows a banner/panel at the top of the case detail page with:
   - Pipeline step indicator (dots/circles showing progress)
   - Current step name and number (e.g., "Step 1 of 4: Review Filing")
   - Step-specific content and action buttons
4. Review step: Show filing details + Accept/Reject buttons
5. Docket step: Show pre-filled docket entry info + Confirm button
6. NEF step: Show recipient list + Confirm button
7. Route Judge step: Show assigned judge + Confirm/Skip button
8. Serve step: Show parties + Confirm button
9. On final step completion: show success message + "Return to Queue" button

Use:
- `server::api::advance_queue_item_fn(court, id)` on confirm
- `server::api::reject_queue_item_fn(court, id, reason)` on reject
- `use_navigator()` to redirect back to dashboard on completion

**Step 2: Integrate into case detail page**

Modify `crates/app/src/routes/cases/detail.rs` to:
1. Read `queue` query parameter from URL
2. If present, render `WorkflowPanel { queue_id }` above the tabs
3. If not present, render as normal (no workflow panel)

**Step 3: Register module**

Add `pub mod workflow_panel;` to `crates/app/src/routes/cases/mod.rs`.

**Step 4: Verify compilation**

Run: `cargo check -p app`
Expected: Compiles without errors.

**Step 5: Commit**

```
git add crates/app/src/routes/cases/workflow_panel.rs crates/app/src/routes/cases/detail.rs crates/app/src/routes/cases/mod.rs
git commit -m "feat(ui): add workflow panel to case detail for queue-driven processing"
```

---

## PR 6: Auto-Creation Triggers

### Task 11: Auto-create queue items when filings are submitted

**Files:**
- Modify: `crates/server/src/rest/filing.rs`
- Modify: `crates/server/src/rest/motion.rs`

**Step 1: Add queue item creation to filing submission handler**

In `crates/server/src/rest/filing.rs`, after a successful filing submission in the `submit_filing` handler, add a call to create a queue item:

```rust
// After successful filing creation, auto-create queue item
let _ = crate::repo::queue::create(
    &pool,
    &court.0,
    "filing",
    3, // normal priority
    &format!("{} - {}", filing_response.filing_type, /* case title or number */),
    Some("New filing requires clerk review"),
    "filing",
    filing_id,  // the UUID of the created filing
    case_id,    // from the filing
    case_number.as_deref(),
    None, // submitted_by (would come from auth in future)
    None, // metadata
    "review",
).await; // Intentionally ignore errors - queue creation is non-critical
```

**Step 2: Add queue item creation to motion submission handler**

In `crates/server/src/rest/motion.rs`, after a successful motion creation, add similar queue item auto-creation with `queue_type = "motion"` and `priority = 2` (motions are higher priority).

**Step 3: Verify compilation**

Run: `cargo check -p server`
Expected: Compiles without errors.

**Step 4: Run existing filing and motion tests**

Run: `cargo test -p tests -- filing --test-threads=1`
Run: `cargo test -p tests -- motion --test-threads=1` (if motion tests exist)
Expected: All existing tests still pass.

**Step 5: Commit**

```
git add crates/server/src/rest/filing.rs crates/server/src/rest/motion.rs
git commit -m "feat(queue): auto-create queue items on filing/motion submission"
```

---

### Task 12: Run full test suite

**Files:** None (verification only)

**Step 1: Run all tests**

Run: `cargo test -p tests -- --test-threads=1`
Expected: All tests pass (existing + new queue tests).

**Step 2: Run cargo check on all crates**

Run: `cargo check --workspace`
Expected: No compilation errors.

**Step 3: Commit any fixes if needed**

If any tests broke, fix and commit with descriptive message.

---

## Summary

| PR | Tasks | What it delivers |
|----|-------|-----------------|
| PR 1 | 1-2 | Migration + shared types (foundation) |
| PR 2 | 3-4 | Repo + REST handlers (backend complete) |
| PR 3 | 5-6 | Integration tests (~15 tests) |
| PR 4 | 7-9 | Server functions + clerk dashboard queue UI + role stubs |
| PR 5 | 10 | Workflow panel on case detail page |
| PR 6 | 11-12 | Auto-creation triggers + full test verification |
