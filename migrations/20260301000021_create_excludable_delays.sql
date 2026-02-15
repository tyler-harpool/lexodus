CREATE TABLE IF NOT EXISTS excludable_delays (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id            TEXT NOT NULL REFERENCES courts(id),
    case_id             UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    start_date          TIMESTAMPTZ NOT NULL,
    end_date            TIMESTAMPTZ,
    reason              TEXT NOT NULL,
    statutory_reference TEXT,
    days_excluded       BIGINT NOT NULL DEFAULT 0,
    order_reference     TEXT
);
CREATE INDEX idx_excludable_delays_court ON excludable_delays(court_id);
CREATE INDEX idx_excludable_delays_court_case ON excludable_delays(court_id, case_id);
CREATE INDEX idx_excludable_delays_court_dates ON excludable_delays(court_id, start_date, end_date);
