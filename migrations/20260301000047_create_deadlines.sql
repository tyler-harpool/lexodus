-- Replace the legacy deadlines table (migration 000022) with a simplified schema.
-- The old table had deadline_type, due_date, triggering_event, etc.
-- The new schema uses title, due_at, rule_code, and a composite PK.
DROP TABLE IF EXISTS deadlines CASCADE;

CREATE TABLE deadlines (
    court_id   TEXT        NOT NULL REFERENCES courts(id),
    id         UUID        NOT NULL DEFAULT gen_random_uuid(),
    case_id    UUID        NULL REFERENCES criminal_cases(id) ON DELETE SET NULL,
    title      TEXT        NOT NULL,
    rule_code  TEXT        NULL,
    due_at     TIMESTAMPTZ NOT NULL,
    status     TEXT        NOT NULL DEFAULT 'open'
                           CHECK (status IN ('open', 'met', 'extended', 'cancelled', 'expired')),
    notes      TEXT        NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (court_id, id)
);

-- Indexes (court_id-leading for tenant-scoped queries)
CREATE INDEX IF NOT EXISTS idx_deadlines_court_due
    ON deadlines (court_id, due_at);

CREATE INDEX IF NOT EXISTS idx_deadlines_court_status_due
    ON deadlines (court_id, status, due_at);

CREATE INDEX IF NOT EXISTS idx_deadlines_court_case_due
    ON deadlines (court_id, case_id, due_at);
