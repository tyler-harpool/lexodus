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
    assigned_to     BIGINT REFERENCES users(id) ON DELETE SET NULL,
    submitted_by    BIGINT REFERENCES users(id) ON DELETE SET NULL,
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
