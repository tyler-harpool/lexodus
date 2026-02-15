CREATE TABLE IF NOT EXISTS victims (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id           TEXT NOT NULL REFERENCES courts(id),
    case_id            UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    name               TEXT NOT NULL,
    victim_type        TEXT NOT NULL DEFAULT 'Individual'
        CHECK (victim_type IN ('Individual','Organization','Government','Minor','Deceased','Anonymous')),
    notification_email TEXT,
    notification_mail  BOOLEAN NOT NULL DEFAULT FALSE,
    notification_phone TEXT,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_victims_court ON victims(court_id);
CREATE INDEX idx_victims_court_case ON victims(court_id, case_id);
CREATE INDEX idx_victims_court_type ON victims(court_id, victim_type);
