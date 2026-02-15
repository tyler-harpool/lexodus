CREATE TABLE IF NOT EXISTS conflict_checks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL REFERENCES courts(id),
    attorney_id     UUID NOT NULL REFERENCES attorneys(id) ON DELETE CASCADE,
    check_date      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    case_id         UUID REFERENCES criminal_cases(id) ON DELETE SET NULL,
    party_names     TEXT[] NOT NULL DEFAULT '{}',
    adverse_parties TEXT[] NOT NULL DEFAULT '{}',
    cleared         BOOLEAN NOT NULL DEFAULT FALSE,
    waiver_obtained BOOLEAN NOT NULL DEFAULT FALSE,
    notes           TEXT
);
CREATE INDEX idx_conflict_checks_court ON conflict_checks(court_id);
CREATE INDEX idx_conflict_checks_court_attorney ON conflict_checks(court_id, attorney_id);
CREATE INDEX idx_conflict_checks_court_case ON conflict_checks(court_id, case_id);
CREATE INDEX idx_conflict_checks_court_date ON conflict_checks(court_id, check_date);
CREATE INDEX idx_conflict_checks_court_cleared ON conflict_checks(court_id, cleared);
