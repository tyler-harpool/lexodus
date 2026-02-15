CREATE TABLE attorney_bar_admissions (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id       TEXT NOT NULL REFERENCES courts(id),
    attorney_id    UUID NOT NULL REFERENCES attorneys(id) ON DELETE CASCADE,
    state          TEXT NOT NULL,
    bar_number     TEXT NOT NULL,
    admission_date TIMESTAMPTZ NOT NULL,
    status         TEXT NOT NULL DEFAULT 'Active'
        CHECK (status IN ('Active','Inactive','Suspended','Revoked','Expired')),
    expiration_date TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_bar_admissions_attorney ON attorney_bar_admissions(attorney_id);
CREATE INDEX idx_bar_admissions_court ON attorney_bar_admissions(court_id);
