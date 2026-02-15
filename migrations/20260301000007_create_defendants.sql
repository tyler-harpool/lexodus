CREATE TABLE IF NOT EXISTS defendants (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id          TEXT NOT NULL REFERENCES courts(id),
    case_id           UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    name              TEXT NOT NULL,
    aliases           TEXT[] NOT NULL DEFAULT '{}',
    usm_number        TEXT,
    fbi_number        TEXT,
    date_of_birth     DATE,
    citizenship_status TEXT
        CHECK (citizenship_status IS NULL OR citizenship_status IN ('Citizen','Permanent Resident','Visa Holder','Undocumented','Unknown')),
    custody_status    TEXT NOT NULL DEFAULT 'Released'
        CHECK (custody_status IN ('In Custody','Released','Bail','Bond','Fugitive','Supervised Release','Unknown')),
    bail_type         TEXT
        CHECK (bail_type IS NULL OR bail_type IN ('Cash','Surety','Property','Personal Recognizance','Unsecured','Denied','None')),
    bail_amount       NUMERIC(15,2),
    bond_conditions   TEXT[] NOT NULL DEFAULT '{}',
    bond_posted_date  TIMESTAMPTZ,
    surety_name       TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_defendants_court ON defendants(court_id);
CREATE INDEX idx_defendants_court_case ON defendants(court_id, case_id);
CREATE INDEX idx_defendants_court_custody ON defendants(court_id, custody_status);
CREATE INDEX idx_defendants_court_name ON defendants(court_id, lower(name));
