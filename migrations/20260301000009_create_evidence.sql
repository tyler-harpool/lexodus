CREATE TABLE IF NOT EXISTS evidence (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id      TEXT NOT NULL REFERENCES courts(id),
    case_id       UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    description   TEXT NOT NULL,
    evidence_type TEXT NOT NULL
        CHECK (evidence_type IN ('Physical','Documentary','Digital','Testimonial','Demonstrative','Forensic','Other')),
    seized_date   TIMESTAMPTZ,
    seized_by     TEXT,
    location      TEXT,
    is_sealed     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_evidence_court ON evidence(court_id);
CREATE INDEX idx_evidence_court_case ON evidence(court_id, case_id);
CREATE INDEX idx_evidence_court_type ON evidence(court_id, evidence_type);
CREATE INDEX idx_evidence_court_sealed ON evidence(court_id, is_sealed);
