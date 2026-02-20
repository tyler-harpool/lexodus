CREATE TABLE IF NOT EXISTS fee_schedule (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL REFERENCES courts(id),
    fee_id          TEXT NOT NULL,
    category        TEXT NOT NULL,
    description     TEXT NOT NULL,
    amount_cents    INT NOT NULL,
    statute         TEXT,
    waivable        BOOLEAN NOT NULL DEFAULT false,
    waiver_form     TEXT,
    cap_cents       INT,
    cap_description TEXT,
    effective_date  DATE NOT NULL DEFAULT CURRENT_DATE,
    active          BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(court_id, fee_id, effective_date)
);

CREATE INDEX idx_fee_schedule_court ON fee_schedule(court_id);
CREATE INDEX idx_fee_schedule_court_active ON fee_schedule(court_id, active);
