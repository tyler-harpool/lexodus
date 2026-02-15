CREATE TABLE IF NOT EXISTS criminal_cases (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id          TEXT NOT NULL REFERENCES courts(id),
    case_number       TEXT NOT NULL,
    title             TEXT NOT NULL,
    description       TEXT,
    crime_type        TEXT NOT NULL
        CHECK (crime_type IN ('Felony','Misdemeanor','Infraction','Petty Offense')),
    status            TEXT NOT NULL DEFAULT 'Open'
        CHECK (status IN ('Open','Active','Pending','Stayed','Closed','Dismissed','Sealed')),
    priority          TEXT NOT NULL DEFAULT 'Normal'
        CHECK (priority IN ('Low','Normal','High','Urgent','Emergency')),
    assigned_judge_id UUID,
    district_code     TEXT NOT NULL,
    location          TEXT,
    is_sealed         BOOLEAN NOT NULL DEFAULT FALSE,
    sealed_date       TIMESTAMPTZ,
    sealed_by         TEXT,
    seal_reason       TEXT,
    opened_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at         TIMESTAMPTZ,
    UNIQUE(court_id, case_number)
);
CREATE INDEX idx_criminal_cases_court ON criminal_cases(court_id);
CREATE INDEX idx_criminal_cases_court_status ON criminal_cases(court_id, status);
CREATE INDEX idx_criminal_cases_court_crime_type ON criminal_cases(court_id, crime_type);
CREATE INDEX idx_criminal_cases_court_priority ON criminal_cases(court_id, priority);
CREATE INDEX idx_criminal_cases_court_judge ON criminal_cases(court_id, assigned_judge_id);
CREATE INDEX idx_criminal_cases_court_district ON criminal_cases(court_id, district_code);
CREATE INDEX idx_criminal_cases_court_sealed ON criminal_cases(court_id, is_sealed);
CREATE INDEX idx_criminal_cases_opened_at ON criminal_cases(court_id, opened_at);
