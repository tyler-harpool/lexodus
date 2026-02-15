CREATE TABLE attorney_federal_admissions (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id         TEXT NOT NULL REFERENCES courts(id),
    attorney_id      UUID NOT NULL REFERENCES attorneys(id) ON DELETE CASCADE,
    court_name       TEXT NOT NULL,
    admission_date   TIMESTAMPTZ NOT NULL,
    sponsor_attorney TEXT,
    status           TEXT NOT NULL DEFAULT 'Active'
        CHECK (status IN ('Active','Inactive','Suspended','Revoked','Expired')),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_federal_admissions_attorney ON attorney_federal_admissions(attorney_id);
CREATE INDEX idx_federal_admissions_court ON attorney_federal_admissions(court_id);
