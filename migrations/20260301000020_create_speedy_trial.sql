CREATE TABLE IF NOT EXISTS speedy_trial (
    case_id              UUID PRIMARY KEY REFERENCES criminal_cases(id) ON DELETE CASCADE,
    court_id             TEXT NOT NULL REFERENCES courts(id),
    arrest_date          TIMESTAMPTZ,
    indictment_date      TIMESTAMPTZ,
    arraignment_date     TIMESTAMPTZ,
    trial_start_deadline TIMESTAMPTZ,
    days_elapsed         BIGINT NOT NULL DEFAULT 0,
    days_remaining       BIGINT NOT NULL DEFAULT 70,
    is_tolled            BOOLEAN NOT NULL DEFAULT FALSE,
    waived               BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX idx_speedy_trial_court ON speedy_trial(court_id);
CREATE INDEX idx_speedy_trial_court_deadline ON speedy_trial(court_id, trial_start_deadline);
CREATE INDEX idx_speedy_trial_court_tolled ON speedy_trial(court_id, is_tolled);
CREATE INDEX idx_speedy_trial_court_remaining ON speedy_trial(court_id, days_remaining);
