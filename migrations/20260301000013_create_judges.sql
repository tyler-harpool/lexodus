CREATE TABLE IF NOT EXISTS judges (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id           TEXT NOT NULL REFERENCES courts(id),
    name               TEXT NOT NULL,
    title              TEXT NOT NULL DEFAULT 'Judge'
        CHECK (title IN ('Chief Judge','Judge','Senior Judge','Magistrate Judge','Visiting Judge')),
    district           TEXT NOT NULL,
    appointed_date     TIMESTAMPTZ,
    status             TEXT NOT NULL DEFAULT 'Active'
        CHECK (status IN ('Active','Senior','Inactive','Retired','Deceased')),
    senior_status_date TIMESTAMPTZ,
    courtroom          TEXT,
    current_caseload   INT NOT NULL DEFAULT 0,
    max_caseload       INT NOT NULL DEFAULT 150,
    specializations    TEXT[] NOT NULL DEFAULT '{}',
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_judges_court ON judges(court_id);
CREATE INDEX idx_judges_court_status ON judges(court_id, status);
CREATE INDEX idx_judges_court_district ON judges(court_id, district);
CREATE INDEX idx_judges_court_name ON judges(court_id, lower(name));
CREATE INDEX idx_judges_court_caseload ON judges(court_id, current_caseload);
