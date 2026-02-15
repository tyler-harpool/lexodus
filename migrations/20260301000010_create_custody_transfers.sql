CREATE TABLE IF NOT EXISTS custody_transfers (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id         TEXT NOT NULL REFERENCES courts(id),
    evidence_id      UUID NOT NULL REFERENCES evidence(id) ON DELETE CASCADE,
    transferred_from TEXT NOT NULL,
    transferred_to   TEXT NOT NULL,
    date             TIMESTAMPTZ NOT NULL,
    location         TEXT,
    condition        TEXT,
    notes            TEXT
);
CREATE INDEX idx_custody_transfers_court ON custody_transfers(court_id);
CREATE INDEX idx_custody_transfers_court_evidence ON custody_transfers(court_id, evidence_id);
CREATE INDEX idx_custody_transfers_court_date ON custody_transfers(court_id, date);
