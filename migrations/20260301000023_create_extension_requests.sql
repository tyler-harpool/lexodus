CREATE TABLE IF NOT EXISTS extension_requests (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id           TEXT NOT NULL REFERENCES courts(id),
    deadline_id        UUID NOT NULL REFERENCES deadlines(id) ON DELETE CASCADE,
    requested_by       TEXT NOT NULL,
    request_date       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    requested_new_date TIMESTAMPTZ NOT NULL,
    reason             TEXT NOT NULL,
    status             TEXT NOT NULL DEFAULT 'Pending'
        CHECK (status IN ('Pending','Granted','Denied','Withdrawn')),
    ruling_date        TIMESTAMPTZ,
    ruling_by          TEXT,
    new_deadline_date  TIMESTAMPTZ
);
CREATE INDEX idx_extension_requests_court ON extension_requests(court_id);
CREATE INDEX idx_extension_requests_court_deadline ON extension_requests(court_id, deadline_id);
CREATE INDEX idx_extension_requests_court_status ON extension_requests(court_id, status);
