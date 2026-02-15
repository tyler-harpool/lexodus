CREATE TABLE IF NOT EXISTS attorney_ecf_registrations (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id           TEXT NOT NULL REFERENCES courts(id),
    attorney_id        UUID NOT NULL REFERENCES attorneys(id) ON DELETE CASCADE,
    registration_date  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status             TEXT NOT NULL DEFAULT 'Active'
        CHECK (status IN ('Active', 'Suspended', 'Revoked', 'Pending')),
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (court_id, attorney_id)
);

CREATE INDEX IF NOT EXISTS idx_ecf_reg_court ON attorney_ecf_registrations(court_id);
CREATE INDEX IF NOT EXISTS idx_ecf_reg_attorney ON attorney_ecf_registrations(attorney_id);
CREATE INDEX IF NOT EXISTS idx_ecf_reg_court_status ON attorney_ecf_registrations(court_id, status);
