-- Prior sentences table for criminal-history calculation
CREATE TABLE IF NOT EXISTS prior_sentences (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id                TEXT NOT NULL,
    sentencing_id           UUID NOT NULL REFERENCES sentencing(id) ON DELETE CASCADE,
    defendant_id            UUID NOT NULL,
    prior_case_number       TEXT,
    jurisdiction            TEXT NOT NULL,
    offense                 TEXT NOT NULL,
    conviction_date         TIMESTAMPTZ NOT NULL,
    sentence_length_months  INT,
    points_assigned         INT NOT NULL DEFAULT 0,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_prior_sentences_sentencing
    ON prior_sentences(sentencing_id);
CREATE INDEX IF NOT EXISTS idx_prior_sentences_court
    ON prior_sentences(court_id);
CREATE INDEX IF NOT EXISTS idx_prior_sentences_defendant
    ON prior_sentences(defendant_id);
