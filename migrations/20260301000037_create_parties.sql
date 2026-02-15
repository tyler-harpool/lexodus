CREATE TABLE IF NOT EXISTS parties (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id          TEXT NOT NULL REFERENCES courts(id),
    case_id           UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    party_type        TEXT NOT NULL
        CHECK (party_type IN ('Plaintiff','Defendant','Petitioner','Respondent','Intervenor','Amicus Curiae','Third Party','Government','Witness','Other')),
    party_role        TEXT NOT NULL
        CHECK (party_role IN ('Lead','Co-Defendant','Co-Plaintiff','Cross-Claimant','Counter-Claimant','Garnishee','Real Party in Interest','Other')),
    name              TEXT NOT NULL,
    entity_type       TEXT NOT NULL DEFAULT 'Individual'
        CHECK (entity_type IN ('Individual','Corporation','Government','Partnership','LLC','Trust','Estate','Non-Profit','Other')),
    first_name        TEXT,
    middle_name       TEXT,
    last_name         TEXT,
    date_of_birth     DATE,
    organization_name TEXT,
    address_street1   TEXT,
    address_city      TEXT,
    address_state     TEXT,
    address_zip       TEXT,
    address_country   TEXT DEFAULT 'USA',
    phone             TEXT,
    email             TEXT,
    represented       BOOLEAN NOT NULL DEFAULT FALSE,
    pro_se            BOOLEAN NOT NULL DEFAULT FALSE,
    service_method    TEXT
        CHECK (service_method IS NULL OR service_method IN ('Electronic','Mail','Personal Service','Waiver','Publication','Other')),
    status            TEXT NOT NULL DEFAULT 'Active'
        CHECK (status IN ('Active','Terminated','Defaulted','Dismissed','Settled','Deceased','Unknown')),
    joined_date       TIMESTAMPTZ,
    terminated_date   TIMESTAMPTZ,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_parties_court ON parties(court_id);
CREATE INDEX idx_parties_court_case ON parties(court_id, case_id);
CREATE INDEX idx_parties_court_type ON parties(court_id, party_type);
CREATE INDEX idx_parties_court_role ON parties(court_id, party_role);
CREATE INDEX idx_parties_court_status ON parties(court_id, status);
CREATE INDEX idx_parties_court_name ON parties(court_id, lower(name));
CREATE INDEX idx_parties_court_entity ON parties(court_id, entity_type);
