CREATE TABLE IF NOT EXISTS sentencing (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id                    TEXT NOT NULL REFERENCES courts(id),
    case_id                     UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    defendant_id                UUID NOT NULL REFERENCES defendants(id) ON DELETE CASCADE,
    judge_id                    UUID NOT NULL REFERENCES judges(id),
    base_offense_level          INT,
    specific_offense_level      INT,
    adjusted_offense_level      INT,
    total_offense_level         INT,
    criminal_history_category   TEXT
        CHECK (criminal_history_category IS NULL OR criminal_history_category IN ('I','II','III','IV','V','VI')),
    criminal_history_points     INT,
    guidelines_range_low_months INT,
    guidelines_range_high_months INT,
    custody_months              INT,
    probation_months            INT,
    fine_amount                 NUMERIC(15,2),
    restitution_amount          NUMERIC(15,2),
    forfeiture_amount           NUMERIC(15,2),
    special_assessment          NUMERIC(15,2),
    departure_type              TEXT
        CHECK (departure_type IS NULL OR departure_type IN ('Upward','Downward','None')),
    departure_reason            TEXT,
    variance_type               TEXT
        CHECK (variance_type IS NULL OR variance_type IN ('Upward','Downward','None')),
    variance_justification      TEXT,
    supervised_release_months   INT,
    appeal_waiver               BOOLEAN NOT NULL DEFAULT FALSE,
    sentencing_date             TIMESTAMPTZ,
    judgment_date               TIMESTAMPTZ,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_sentencing_court ON sentencing(court_id);
CREATE INDEX idx_sentencing_court_case ON sentencing(court_id, case_id);
CREATE INDEX idx_sentencing_court_defendant ON sentencing(court_id, defendant_id);
CREATE INDEX idx_sentencing_court_judge ON sentencing(court_id, judge_id);
CREATE INDEX idx_sentencing_court_date ON sentencing(court_id, sentencing_date);
CREATE INDEX idx_sentencing_court_offense_level ON sentencing(court_id, total_offense_level);
CREATE INDEX idx_sentencing_court_history_cat ON sentencing(court_id, criminal_history_category);
