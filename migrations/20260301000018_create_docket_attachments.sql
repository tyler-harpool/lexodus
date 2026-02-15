CREATE TABLE IF NOT EXISTS docket_attachments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL REFERENCES courts(id),
    docket_entry_id UUID NOT NULL REFERENCES docket_entries(id) ON DELETE CASCADE,
    filename        TEXT NOT NULL,
    file_size       BIGINT NOT NULL,
    content_type    TEXT NOT NULL,
    storage_key     TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_docket_attachments_court ON docket_attachments(court_id);
CREATE INDEX idx_docket_attachments_court_entry ON docket_attachments(court_id, docket_entry_id);
