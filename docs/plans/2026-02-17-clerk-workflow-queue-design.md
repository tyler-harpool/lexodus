# Clerk Workflow Queue Design

**Date:** 2026-02-17
**Status:** Approved

## Overview

Transform the clerk experience from isolated CRUD pages into an integrated, filing-centric workflow queue that mirrors real CM/ECF usage. The clerk dashboard becomes a prioritized work inbox. Processing an item walks the clerk through a step-by-step pipeline on the case detail page.

## Architecture: Event-Driven Queue

Every filing submission, motion, or order draft creates a `QueueItem` row in a `clerk_queue` table. The clerk dashboard reads from this table. Processing a filing transitions it through pipeline states, with each transition triggering side effects (auto-create docket entry, generate NEF, route to judge).

This pattern is reusable for judge and attorney queues in future phases.

## Data Model

### `clerk_queue` Table

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `court_id` | TEXT | Tenant identifier |
| `queue_type` | TEXT | `filing`, `motion`, `order`, `deadline_alert`, `general` |
| `priority` | INTEGER | 1=critical, 2=high, 3=normal, 4=low |
| `status` | TEXT | `pending`, `in_review`, `processing`, `completed`, `rejected` |
| `title` | TEXT | Display title (e.g., "Motion to Suppress - USA v. Smith") |
| `description` | TEXT | Brief context |
| `source_type` | TEXT | `filing`, `motion`, `order`, `document`, `deadline`, `calendar_event` |
| `source_id` | UUID | FK to the source entity |
| `case_id` | UUID | FK to `criminal_cases` (nullable for non-case items) |
| `case_number` | TEXT | Denormalized for display |
| `assigned_to` | UUID | Clerk user ID (nullable = unassigned) |
| `submitted_by` | UUID | Who created the source item |
| `current_step` | TEXT | Pipeline position: `review`, `docket`, `nef`, `route_judge`, `serve` |
| `metadata` | JSONB | Flexible data per queue_type |
| `created_at` | TIMESTAMPTZ | Auto-set |
| `updated_at` | TIMESTAMPTZ | Auto-updated |
| `completed_at` | TIMESTAMPTZ | Nullable, set on completion |

### Indexes

- `(court_id, status, priority, created_at)` — main queue listing
- `(court_id, assigned_to, status)` — "my items" filter
- `(court_id, case_id)` — queue items by case
- `(source_type, source_id)` — lookup by source entity

## API Endpoints

### Queue Management

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/queue` | List queue items (filter: status, type, priority, assigned_to) |
| `GET` | `/api/queue/stats` | Queue metrics (counts by status, avg processing time, backlog) |
| `GET` | `/api/queue/{id}` | Get single queue item with source entity details |
| `POST` | `/api/queue/{id}/claim` | Clerk claims an unassigned item |
| `POST` | `/api/queue/{id}/release` | Release a claimed item back to pool |

### Pipeline Progression

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/queue/{id}/advance` | Move to next pipeline step (with step-specific payload) |
| `POST` | `/api/queue/{id}/reject` | Reject item with reason |
| `POST` | `/api/queue/{id}/skip-step` | Skip current step |

### Advance Payloads by Step Transition

- **review → docket:** `{ "accepted": true }` or `{ "accepted": false, "reason": "..." }`
- **docket → nef:** Auto-creates docket entry, returns entry ID in response
- **nef → route_judge:** Auto-generates NEF, optionally sets judge routing
- **route_judge → serve:** Creates routing record for judge queue
- **serve → completed:** Creates service records via bulk endpoint

### Auto-Creation Triggers

Queue items are created automatically when:
- `POST /api/filings` succeeds → `queue_type = 'filing'`
- `POST /api/motions` succeeds → `queue_type = 'motion'`
- `POST /api/orders` creates a draft → `queue_type = 'order'`
- Approaching deadlines (background check) → `queue_type = 'deadline_alert'`

Existing endpoints remain unchanged. The queue is a layer on top.

## Pipeline Steps by Queue Type

| Type | Review | Docket | NEF | Route Judge | Serve |
|------|--------|--------|-----|-------------|-------|
| Filing (general) | Yes | Yes | Yes | No | Yes |
| Motion | Yes | Yes | Yes | Yes | Yes |
| Proposed Order | Yes | Yes | Yes | Yes | No |
| Notice | Yes | Yes | Yes | No | Yes |
| Order (issuance) | No | Yes | Yes | No | Yes |

## UI Design

### Clerk Dashboard (Queue View)

```
┌─────────────────────────────────────────────────────────┐
│  Clerk Dashboard                                         │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐                │
│  │  12  │  │   3  │  │   8  │  │  2   │                │
│  │Pending│  │My Items│ │Today │  │Urgent│                │
│  └──────┘  └──────┘  └──────┘  └──────┘                │
│                                                          │
│  ┌─ Filter Bar ────────────────────────────────────┐    │
│  │ [All Types ▾] [All Priority ▾] [My Items ▾]     │    │
│  └──────────────────────────────────────────────────┘    │
│                                                          │
│  ┌─ Queue Items ───────────────────────────────────┐    │
│  │ CRITICAL  Motion to Dismiss - USA v. Jones       │    │
│  │    Case 2:24-cr-00142 | Motion | 2m ago | [Claim]│    │
│  │──────────────────────────────────────────────────│    │
│  │ HIGH  Plea Agreement Filing - USA v. Smith       │    │
│  │    Case 1:24-cr-00089 | Filing | 15m ago| [Claim]│    │
│  │──────────────────────────────────────────────────│    │
│  │ NORMAL  Scheduling Order - USA v. Davis          │    │
│  │    Case 3:24-cr-00201 | Order  | 1h ago |Claimed │    │
│  │──────────────────────────────────────────────────│    │
│  └──────────────────────────────────────────────────┘    │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Workflow Panel (On Case Detail Page)

When a clerk navigates from queue to case detail, a workflow panel appears:

```
┌─ Workflow: Process Filing ──────────────────────┐
│                                                   │
│  Step 1 of 4: Review Filing                       │
│  ● Review  ○ Docket  ○ NEF  ○ Serve              │
│                                                   │
│  Filing: Motion to Dismiss                        │
│  Filed by: Jane Attorney (Bar #12345)             │
│  Document: motion_dismiss_2024.pdf                │
│                                                   │
│  ┌────────────┐  ┌────────────┐                   │
│  │  Accept ✓  │  │  Reject ✗  │                   │
│  └────────────┘  └────────────┘                   │
│                                                   │
└───────────────────────────────────────────────────┘
```

The panel advances through steps. Each step shows relevant data and actions. On completion, the clerk is returned to the dashboard queue.

### Interaction Flow

1. Clerk opens dashboard → sees prioritized queue
2. Clicks "Claim" on item → assigned to them
3. Clicks item → navigates to `/cases/{id}?queue={queue_id}`
4. Workflow panel opens on case detail page
5. Clerk works through pipeline steps
6. On completion → queue item marked done → redirect to dashboard

### Queue Item Visual States

- **Unassigned:** Neutral card with "Claim" button
- **Claimed by me:** Highlighted card with "Continue" button
- **Claimed by others:** Dimmed card showing assignee name
- **Completed:** Not shown in default view (filterable)

## Other Role Stubs

### Judge Dashboard
- Existing stats cards remain
- Add empty queue shell below: "Pending Rulings" section with "No items pending" empty state
- Badge on sidebar showing queue count (0 for now)

### Attorney Dashboard
- Existing stats cards remain
- Add empty queue shell below: "My Filings" section with "No items pending" empty state
- Badge on sidebar showing queue count (0 for now)

### Public Dashboard
- No changes (remains view-only)

## Shared Types

```rust
// New types in shared-types
pub struct QueueItem { /* mirrors table */ }
pub struct QueueItemResponse { /* with source entity preview */ }
pub struct QueueSearchParams { status, queue_type, priority, assigned_to, case_id }
pub struct QueueStats { pending_count, my_count, today_count, urgent_count, avg_processing_mins }
pub struct AdvanceRequest { step_data: Option<serde_json::Value> }
pub struct RejectRequest { reason: String }
```

## Testing

- Queue CRUD: create, list, filter, get
- Claim/release: concurrency (two clerks claiming same item)
- Pipeline advancement: each step transition + side effects
- Auto-creation: filing → queue item created automatically
- Step skipping: skip NEF for internal items
- Rejection: filing rejected → queue item completed with rejected status
- Stats: accurate counts across states
- Multi-tenant: queue items scoped to court_id

## Future Phases

1. **Judge Queue:** Same table, `queue_type = 'judicial_review'`. Steps: review → rule → sign.
2. **Attorney Queue:** Same table, `queue_type = 'attorney_action'`. Steps: review → respond.
3. **Cross-queue routing:** Clerk completes "route_judge" step → creates judge queue item.
4. **Notification integration:** Queue state changes trigger email/in-app notifications.
5. **Queue analytics:** Processing time trends, clerk workload balancing, bottleneck identification.
