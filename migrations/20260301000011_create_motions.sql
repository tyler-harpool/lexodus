CREATE TABLE IF NOT EXISTS motions (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id     TEXT NOT NULL REFERENCES courts(id),
    case_id      UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    motion_type  TEXT NOT NULL
        CHECK (motion_type IN ('Dismiss','Suppress','Compel','Summary Judgment','Continuance','Change of Venue','Reconsideration','Limine','Severance','Joinder','Discovery','New Trial','Other')),
    filed_by     TEXT NOT NULL,
    description  TEXT NOT NULL,
    filed_date   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status       TEXT NOT NULL DEFAULT 'Pending'
        CHECK (status IN ('Pending','Granted','Denied','Withdrawn','Moot','Deferred','Partially Granted')),
    ruling_date  TIMESTAMPTZ,
    ruling_text  TEXT
);
CREATE INDEX idx_motions_court ON motions(court_id);
CREATE INDEX idx_motions_court_case ON motions(court_id, case_id);
CREATE INDEX idx_motions_court_status ON motions(court_id, status);
CREATE INDEX idx_motions_court_type ON motions(court_id, motion_type);
CREATE INDEX idx_motions_court_filed_date ON motions(court_id, filed_date);
