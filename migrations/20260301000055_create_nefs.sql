-- 1. Add UNIQUE(court_id, id) on filings for compound FK support
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'filings_court_id_id_key'
    ) THEN
        ALTER TABLE filings
            ADD CONSTRAINT filings_court_id_id_key UNIQUE (court_id, id);
    END IF;
END $$;

-- 2. Add UNIQUE(court_id, id) on documents for compound FK support
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'documents_court_id_id_key'
    ) THEN
        ALTER TABLE documents
            ADD CONSTRAINT documents_court_id_id_key UNIQUE (court_id, id);
    END IF;
END $$;

-- 3. Create nefs table
CREATE TABLE IF NOT EXISTS nefs (
    court_id         TEXT NOT NULL,
    id               UUID NOT NULL DEFAULT gen_random_uuid(),
    filing_id        UUID NOT NULL,
    document_id      UUID NOT NULL,
    case_id          UUID NOT NULL,
    docket_entry_id  UUID NOT NULL,
    recipients       JSONB NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (court_id, id),
    FOREIGN KEY (court_id, filing_id)
        REFERENCES filings(court_id, id)
        ON DELETE CASCADE,
    FOREIGN KEY (court_id, document_id)
        REFERENCES documents(court_id, id)
        ON DELETE CASCADE,
    FOREIGN KEY (court_id, docket_entry_id)
        REFERENCES docket_entries(court_id, id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_nefs_case
    ON nefs(court_id, case_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_nefs_filing
    ON nefs(court_id, filing_id);
