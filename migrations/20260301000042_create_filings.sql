CREATE TABLE IF NOT EXISTS filings (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id          TEXT NOT NULL REFERENCES courts(id),
    case_id           UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    filing_type       TEXT NOT NULL
        CHECK (filing_type IN ('Initial','Response','Reply','Motion','Notice','Stipulation','Supplement','Amendment','Exhibit','Certificate','Other')),
    filed_by          TEXT NOT NULL,
    filed_date        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status            TEXT NOT NULL DEFAULT 'Pending'
        CHECK (status IN ('Pending','Accepted','Rejected','Under Review','Returned','Filed')),
    validation_errors JSONB NOT NULL DEFAULT '[]',
    document_id       UUID REFERENCES documents(id) ON DELETE SET NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_filings_court ON filings(court_id);
CREATE INDEX idx_filings_court_case ON filings(court_id, case_id);
CREATE INDEX idx_filings_court_type ON filings(court_id, filing_type);
CREATE INDEX idx_filings_court_status ON filings(court_id, status);
CREATE INDEX idx_filings_court_date ON filings(court_id, filed_date);
CREATE INDEX idx_filings_court_document ON filings(court_id, document_id);
