CREATE TABLE IF NOT EXISTS deadlines (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id          TEXT NOT NULL REFERENCES courts(id),
    case_id           UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    deadline_type     TEXT NOT NULL
        CHECK (deadline_type IN ('Filing','Response','Discovery','Trial','Sentencing','Appeal','Motion','Hearing','Statutory','Administrative','Other')),
    due_date          TIMESTAMPTZ NOT NULL,
    triggering_event  TEXT,
    triggering_date   TIMESTAMPTZ,
    applicable_rule   TEXT,
    description       TEXT NOT NULL,
    responsible_party TEXT,
    is_jurisdictional BOOLEAN NOT NULL DEFAULT FALSE,
    is_extendable     BOOLEAN NOT NULL DEFAULT TRUE,
    status            TEXT NOT NULL DEFAULT 'Pending'
        CHECK (status IN ('Pending','Completed','Overdue','Extended','Waived','Cancelled')),
    completion_date   TIMESTAMPTZ,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_deadlines_court ON deadlines(court_id);
CREATE INDEX idx_deadlines_court_case ON deadlines(court_id, case_id);
CREATE INDEX idx_deadlines_court_due ON deadlines(court_id, due_date);
CREATE INDEX idx_deadlines_court_status ON deadlines(court_id, status);
CREATE INDEX idx_deadlines_court_type ON deadlines(court_id, deadline_type);
CREATE INDEX idx_deadlines_court_jurisdictional ON deadlines(court_id, is_jurisdictional);
