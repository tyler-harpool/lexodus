-- Audit trail for document lifecycle actions (seal, unseal, replace, strike).
CREATE TABLE IF NOT EXISTS document_events (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id      TEXT NOT NULL REFERENCES courts(id),
    document_id   UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    event_type    TEXT NOT NULL CHECK (event_type IN ('sealed','unsealed','replaced','stricken')),
    actor         TEXT NOT NULL,
    detail        JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_doc_events_court_doc ON document_events(court_id, document_id);
