-- Add SSE enforcement columns and constraints to docket_attachments
ALTER TABLE docket_attachments
    ADD COLUMN IF NOT EXISTS sealed      BOOLEAN     NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS encryption  TEXT        NOT NULL DEFAULT 'SSE_S3',
    ADD COLUMN IF NOT EXISTS sha256      TEXT,
    ADD COLUMN IF NOT EXISTS uploaded_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Enforce non-empty filename and storage_key
DO $$ BEGIN
    ALTER TABLE docket_attachments ADD CONSTRAINT chk_filename_not_empty  CHECK (filename    <> '');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    ALTER TABLE docket_attachments ADD CONSTRAINT chk_storage_key_not_empty CHECK (storage_key <> '');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

-- Composite index for listing attachments by entry ordered by newest first
CREATE INDEX IF NOT EXISTS idx_docket_attachments_entry_created
    ON docket_attachments (court_id, docket_entry_id, created_at DESC);
