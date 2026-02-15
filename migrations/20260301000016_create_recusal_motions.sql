CREATE TABLE IF NOT EXISTS recusal_motions (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id              TEXT NOT NULL REFERENCES courts(id),
    case_id               UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    judge_id              UUID NOT NULL REFERENCES judges(id),
    filed_by              TEXT NOT NULL,
    filed_date            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reason                TEXT NOT NULL,
    detailed_grounds      TEXT,
    status                TEXT NOT NULL DEFAULT 'Pending'
        CHECK (status IN ('Pending','Granted','Denied','Withdrawn','Moot')),
    ruling_date           TIMESTAMPTZ,
    ruling_text           TEXT,
    replacement_judge_id  UUID REFERENCES judges(id)
);
CREATE INDEX idx_recusal_motions_court ON recusal_motions(court_id);
CREATE INDEX idx_recusal_motions_court_case ON recusal_motions(court_id, case_id);
CREATE INDEX idx_recusal_motions_court_judge ON recusal_motions(court_id, judge_id);
CREATE INDEX idx_recusal_motions_court_status ON recusal_motions(court_id, status);
