-- Bureau of Prisons (BOP) designations table
CREATE TABLE IF NOT EXISTS bop_designations (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id            TEXT NOT NULL,
    sentencing_id       UUID NOT NULL REFERENCES sentencing(id) ON DELETE CASCADE,
    defendant_id        UUID NOT NULL,
    facility            TEXT NOT NULL,
    security_level      TEXT NOT NULL
                        CHECK (security_level IN ('Minimum', 'Low', 'Medium', 'High', 'Administrative', 'Unassigned')),
    designation_date    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    designation_reason  TEXT,
    rdap_eligible       BOOLEAN NOT NULL DEFAULT false,
    rdap_enrolled       BOOLEAN NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_bop_designations_sentencing
    ON bop_designations(sentencing_id);
CREATE INDEX IF NOT EXISTS idx_bop_designations_court
    ON bop_designations(court_id);
CREATE INDEX IF NOT EXISTS idx_bop_designations_defendant
    ON bop_designations(defendant_id);
CREATE INDEX IF NOT EXISTS idx_bop_designations_rdap
    ON bop_designations(court_id, rdap_eligible) WHERE rdap_eligible = true;
