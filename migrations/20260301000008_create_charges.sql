CREATE TABLE IF NOT EXISTS charges (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id              TEXT NOT NULL REFERENCES courts(id),
    defendant_id          UUID NOT NULL REFERENCES defendants(id) ON DELETE CASCADE,
    count_number          INT NOT NULL,
    statute               TEXT NOT NULL,
    offense_description   TEXT NOT NULL,
    statutory_max_months  INT,
    statutory_min_months  INT,
    plea                  TEXT
        CHECK (plea IS NULL OR plea IN ('Not Guilty','Guilty','No Contest','Alford','Not Yet Entered')),
    plea_date             TIMESTAMPTZ,
    verdict               TEXT
        CHECK (verdict IS NULL OR verdict IN ('Guilty','Not Guilty','Dismissed','Mistrial','Acquitted','Hung Jury')),
    verdict_date          TIMESTAMPTZ
);
CREATE INDEX idx_charges_court ON charges(court_id);
CREATE INDEX idx_charges_court_defendant ON charges(court_id, defendant_id);
CREATE INDEX idx_charges_court_statute ON charges(court_id, statute);
CREATE INDEX idx_charges_court_plea ON charges(court_id, plea);
CREATE INDEX idx_charges_court_verdict ON charges(court_id, verdict);
