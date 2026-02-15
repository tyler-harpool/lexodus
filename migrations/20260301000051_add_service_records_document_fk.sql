-- Add foreign key constraint on service_records.document_id -> documents.id
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_service_records_document'
    ) THEN
        ALTER TABLE service_records
            ADD CONSTRAINT fk_service_records_document
            FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE RESTRICT;
    END IF;
END $$;
