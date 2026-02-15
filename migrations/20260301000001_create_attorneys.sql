CREATE TABLE attorneys (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL REFERENCES courts(id),
    bar_number      TEXT NOT NULL,
    first_name      TEXT NOT NULL,
    last_name       TEXT NOT NULL,
    middle_name     TEXT,
    firm_name       TEXT,
    email           TEXT NOT NULL,
    phone           TEXT NOT NULL,
    fax             TEXT,
    address_street1 TEXT NOT NULL,
    address_street2 TEXT,
    address_city    TEXT NOT NULL,
    address_state   TEXT NOT NULL,
    address_zip     TEXT NOT NULL,
    address_country TEXT NOT NULL DEFAULT 'USA',
    status          TEXT NOT NULL DEFAULT 'Active'
        CHECK (status IN ('Active','Inactive','Suspended','Disbarred','Retired','Deceased')),
    cja_panel_member    BOOLEAN NOT NULL DEFAULT FALSE,
    cja_panel_districts TEXT[] NOT NULL DEFAULT '{}',
    languages_spoken    TEXT[] NOT NULL DEFAULT '{English}',
    cases_handled       INT NOT NULL DEFAULT 0,
    win_rate_percentage DOUBLE PRECISION,
    avg_case_duration_days INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(court_id, bar_number)
);
CREATE INDEX idx_attorneys_court ON attorneys(court_id);
CREATE INDEX idx_attorneys_court_bar ON attorneys(court_id, bar_number);
CREATE INDEX idx_attorneys_court_status ON attorneys(court_id, status);
CREATE INDEX idx_attorneys_court_name ON attorneys(court_id, lower(first_name), lower(last_name));
