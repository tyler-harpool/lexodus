CREATE TABLE IF NOT EXISTS judge_signatures (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id       TEXT NOT NULL REFERENCES courts(id),
    judge_id       UUID NOT NULL REFERENCES judges(id) ON DELETE CASCADE,
    signature_data TEXT NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(court_id, judge_id)
);
CREATE INDEX idx_judge_signatures_court ON judge_signatures(court_id);
CREATE INDEX idx_judge_signatures_court_judge ON judge_signatures(court_id, judge_id);
