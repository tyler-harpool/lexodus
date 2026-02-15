CREATE TABLE IF NOT EXISTS documents (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id      TEXT NOT NULL REFERENCES courts(id),
    case_id       UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    title         TEXT NOT NULL,
    document_type TEXT NOT NULL
        CHECK (document_type IN ('Motion','Order','Brief','Memorandum','Declaration','Affidavit','Exhibit','Transcript','Notice','Subpoena','Warrant','Indictment','Plea Agreement','Judgment','Verdict','Other')),
    storage_key   TEXT NOT NULL,
    checksum      TEXT NOT NULL,
    file_size     BIGINT NOT NULL,
    content_type  TEXT NOT NULL,
    is_sealed     BOOLEAN NOT NULL DEFAULT FALSE,
    uploaded_by   TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_documents_court ON documents(court_id);
CREATE INDEX idx_documents_court_case ON documents(court_id, case_id);
CREATE INDEX idx_documents_court_type ON documents(court_id, document_type);
CREATE INDEX idx_documents_court_sealed ON documents(court_id, is_sealed);
CREATE INDEX idx_documents_court_created ON documents(court_id, created_at);
