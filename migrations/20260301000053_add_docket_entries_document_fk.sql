-- Add FK constraint and index on docket_entries.document_id -> documents(id).
-- The column already exists (nullable UUID); this adds referential integrity.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_docket_entries_document'
    ) THEN
        ALTER TABLE docket_entries
            ADD CONSTRAINT fk_docket_entries_document
            FOREIGN KEY (document_id) REFERENCES documents(id)
            ON DELETE SET NULL;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_docket_entries_document
    ON docket_entries (court_id, document_id)
    WHERE document_id IS NOT NULL;
