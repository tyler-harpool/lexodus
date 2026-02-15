-- Add source_attachment_id column to documents table for linking
-- a document to the docket attachment it was promoted from.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'documents'
          AND column_name = 'source_attachment_id'
    ) THEN
        ALTER TABLE documents
            ADD COLUMN source_attachment_id UUID NULL
            REFERENCES docket_attachments(id) ON DELETE SET NULL;

        CREATE UNIQUE INDEX idx_documents_source_attachment
            ON documents(source_attachment_id)
            WHERE source_attachment_id IS NOT NULL;
    END IF;
END $$;
