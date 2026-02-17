-- Sentencing special conditions table
CREATE TABLE IF NOT EXISTS sentencing_special_conditions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL,
    sentencing_id   UUID NOT NULL REFERENCES sentencing(id) ON DELETE CASCADE,
    condition_type  TEXT NOT NULL,
    description     TEXT NOT NULL,
    effective_date  TIMESTAMPTZ,
    status          TEXT NOT NULL DEFAULT 'Active'
                    CHECK (status IN ('Active', 'Modified', 'Terminated', 'Expired')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sentencing_special_conditions_sentencing
    ON sentencing_special_conditions(sentencing_id);
CREATE INDEX IF NOT EXISTS idx_sentencing_special_conditions_court
    ON sentencing_special_conditions(court_id);
CREATE INDEX IF NOT EXISTS idx_sentencing_special_conditions_status
    ON sentencing_special_conditions(court_id, status);
