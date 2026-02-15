CREATE TABLE IF NOT EXISTS judge_conflicts (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id      TEXT NOT NULL REFERENCES courts(id),
    judge_id      UUID NOT NULL REFERENCES judges(id) ON DELETE CASCADE,
    party_name    TEXT,
    law_firm      TEXT,
    corporation   TEXT,
    conflict_type TEXT NOT NULL
        CHECK (conflict_type IN ('Financial','Familial','Professional','Prior Representation','Organizational','Other')),
    start_date    TIMESTAMPTZ NOT NULL,
    end_date      TIMESTAMPTZ,
    notes         TEXT
);
CREATE INDEX idx_judge_conflicts_court ON judge_conflicts(court_id);
CREATE INDEX idx_judge_conflicts_court_judge ON judge_conflicts(court_id, judge_id);
CREATE INDEX idx_judge_conflicts_court_type ON judge_conflicts(court_id, conflict_type);
