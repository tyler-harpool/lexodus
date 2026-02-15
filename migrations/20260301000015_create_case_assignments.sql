CREATE TABLE IF NOT EXISTS case_assignments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id            TEXT NOT NULL REFERENCES courts(id),
    case_id             UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    judge_id            UUID NOT NULL REFERENCES judges(id),
    assignment_type     TEXT NOT NULL DEFAULT 'Initial'
        CHECK (assignment_type IN ('Initial','Reassignment','Temporary','Related Case','Emergency')),
    assigned_date       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reason              TEXT,
    previous_judge_id   UUID REFERENCES judges(id),
    reassignment_reason TEXT
);
CREATE INDEX idx_case_assignments_court ON case_assignments(court_id);
CREATE INDEX idx_case_assignments_court_case ON case_assignments(court_id, case_id);
CREATE INDEX idx_case_assignments_court_judge ON case_assignments(court_id, judge_id);
CREATE INDEX idx_case_assignments_court_type ON case_assignments(court_id, assignment_type);
CREATE INDEX idx_case_assignments_court_date ON case_assignments(court_id, assigned_date);
