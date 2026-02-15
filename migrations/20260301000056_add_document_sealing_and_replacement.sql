-- Add sealing levels, document replacement, and strike support to documents.

-- 1. Sealing level (application-level policy beyond boolean is_sealed)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'documents' AND column_name = 'sealing_level'
    ) THEN
        ALTER TABLE documents
            ADD COLUMN sealing_level TEXT NOT NULL DEFAULT 'Public'
            CHECK (sealing_level IN ('Public', 'SealedCourtOnly', 'SealedCaseParticipants', 'SealedAttorneysOnly'));
    END IF;
END $$;

-- 2. Sealing reason and linked motion
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'documents' AND column_name = 'seal_reason_code'
    ) THEN
        ALTER TABLE documents
            ADD COLUMN seal_reason_code TEXT;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'documents' AND column_name = 'seal_motion_id'
    ) THEN
        ALTER TABLE documents
            ADD COLUMN seal_motion_id UUID;
    END IF;
END $$;

-- 3. Document replacement / strike support
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'documents' AND column_name = 'replaced_by_document_id'
    ) THEN
        ALTER TABLE documents
            ADD COLUMN replaced_by_document_id UUID REFERENCES documents(id);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'documents' AND column_name = 'is_stricken'
    ) THEN
        ALTER TABLE documents
            ADD COLUMN is_stricken BOOLEAN NOT NULL DEFAULT FALSE;
    END IF;
END $$;

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_documents_sealing_level ON documents(court_id, sealing_level);
CREATE INDEX IF NOT EXISTS idx_documents_replaced_by ON documents(replaced_by_document_id) WHERE replaced_by_document_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_documents_stricken ON documents(court_id, is_stricken) WHERE is_stricken = TRUE;
