CREATE TABLE IF NOT EXISTS representations (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id            TEXT NOT NULL REFERENCES courts(id),
    attorney_id         UUID NOT NULL REFERENCES attorneys(id),
    party_id            UUID NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
    case_id             UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    representation_type TEXT NOT NULL DEFAULT 'Private'
        CHECK (representation_type IN ('Private','Court Appointed','Pro Bono','Public Defender','CJA Panel','Government','Other')),
    status              TEXT NOT NULL DEFAULT 'Active'
        CHECK (status IN ('Active','Withdrawn','Terminated','Substituted','Suspended')),
    start_date          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    end_date            TIMESTAMPTZ,
    lead_counsel        BOOLEAN NOT NULL DEFAULT FALSE,
    local_counsel       BOOLEAN NOT NULL DEFAULT FALSE,
    court_appointed     BOOLEAN NOT NULL DEFAULT FALSE,
    withdrawal_reason   TEXT,
    notes               TEXT
);
CREATE INDEX idx_representations_court ON representations(court_id);
CREATE INDEX idx_representations_court_attorney ON representations(court_id, attorney_id);
CREATE INDEX idx_representations_court_party ON representations(court_id, party_id);
CREATE INDEX idx_representations_court_case ON representations(court_id, case_id);
CREATE INDEX idx_representations_court_status ON representations(court_id, status);
CREATE INDEX idx_representations_court_type ON representations(court_id, representation_type);
