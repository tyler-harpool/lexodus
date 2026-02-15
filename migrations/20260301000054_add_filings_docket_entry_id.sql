-- Add tenant-scoped docket_entry_id FK to filings + filing_uploads staging table

-- 1. Add UNIQUE(court_id, id) on docket_entries for compound FK support
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'docket_entries_court_id_id_key'
    ) THEN
        ALTER TABLE docket_entries
            ADD CONSTRAINT docket_entries_court_id_id_key UNIQUE (court_id, id);
    END IF;
END $$;

-- 2. Add docket_entry_id column with tenant-scoped FK
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'filings' AND column_name = 'docket_entry_id'
    ) THEN
        ALTER TABLE filings
            ADD COLUMN docket_entry_id UUID;
        ALTER TABLE filings
            ADD CONSTRAINT filings_court_docket_entry_fk
            FOREIGN KEY (court_id, docket_entry_id)
            REFERENCES docket_entries (court_id, id)
            ON DELETE SET NULL;
        CREATE INDEX idx_filings_docket_entry
            ON filings(court_id, docket_entry_id)
            WHERE docket_entry_id IS NOT NULL;
    END IF;
END $$;

-- 3. Staging table for filing uploads (presign + finalize, no docket entry needed)
CREATE TABLE IF NOT EXISTS filing_uploads (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id      TEXT NOT NULL REFERENCES courts(id),
    filename      TEXT NOT NULL,
    file_size     BIGINT NOT NULL,
    content_type  TEXT NOT NULL,
    storage_key   TEXT NOT NULL,
    sha256        TEXT,
    uploaded_at   TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_filing_uploads_court ON filing_uploads(court_id);
CREATE INDEX IF NOT EXISTS idx_filing_uploads_court_uploaded ON filing_uploads(court_id, uploaded_at)
    WHERE uploaded_at IS NOT NULL;
