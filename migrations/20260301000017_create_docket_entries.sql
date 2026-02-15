CREATE TABLE IF NOT EXISTS docket_entries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL REFERENCES courts(id),
    case_id         UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    entry_number    INT NOT NULL,
    date_filed      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    date_entered    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    filed_by        TEXT NOT NULL,
    entry_type      TEXT NOT NULL
        CHECK (entry_type IN ('Motion','Order','Notice','Pleading','Transcript','Exhibit','Minute Entry','Summons','Subpoena','Warrant','Judgment','Other')),
    description     TEXT NOT NULL,
    document_id     UUID,
    is_sealed       BOOLEAN NOT NULL DEFAULT FALSE,
    is_ex_parte     BOOLEAN NOT NULL DEFAULT FALSE,
    page_count      INT,
    related_entries INT[] NOT NULL DEFAULT '{}',
    service_list    TEXT[] NOT NULL DEFAULT '{}',
    UNIQUE(court_id, case_id, entry_number)
);
CREATE INDEX idx_docket_entries_court ON docket_entries(court_id);
CREATE INDEX idx_docket_entries_court_case ON docket_entries(court_id, case_id);
CREATE INDEX idx_docket_entries_court_type ON docket_entries(court_id, entry_type);
CREATE INDEX idx_docket_entries_court_filed ON docket_entries(court_id, date_filed);
CREATE INDEX idx_docket_entries_court_sealed ON docket_entries(court_id, is_sealed);
CREATE INDEX idx_docket_entries_court_case_number ON docket_entries(court_id, case_id, entry_number);
