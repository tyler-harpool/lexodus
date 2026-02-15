-- Court role admission requests: request-and-approve workflow for court memberships.
CREATE TABLE IF NOT EXISTS court_role_requests (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id        BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    court_id       TEXT NOT NULL,
    requested_role TEXT NOT NULL CHECK (requested_role IN ('attorney', 'clerk', 'judge')),
    status         TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'denied')),
    requested_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_by    BIGINT REFERENCES users(id),
    reviewed_at    TIMESTAMPTZ,
    notes          TEXT,
    UNIQUE (user_id, court_id, status)
);

CREATE INDEX IF NOT EXISTS idx_court_role_requests_user ON court_role_requests(user_id);
CREATE INDEX IF NOT EXISTS idx_court_role_requests_status ON court_role_requests(status);
CREATE INDEX IF NOT EXISTS idx_court_role_requests_court ON court_role_requests(court_id);
