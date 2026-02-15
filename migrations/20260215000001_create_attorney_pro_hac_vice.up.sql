CREATE TABLE IF NOT EXISTS attorney_pro_hac_vice (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id                TEXT NOT NULL REFERENCES courts(id),
    attorney_id             UUID NOT NULL REFERENCES attorneys(id) ON DELETE CASCADE,
    case_id                 UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    sponsoring_attorney_id  UUID REFERENCES attorneys(id),
    admission_date          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expiration_date         TIMESTAMPTZ,
    status                  TEXT NOT NULL DEFAULT 'Active'
        CHECK (status IN ('Active', 'Expired', 'Revoked', 'Pending')),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_phv_court ON attorney_pro_hac_vice(court_id);
CREATE INDEX IF NOT EXISTS idx_phv_attorney ON attorney_pro_hac_vice(attorney_id);
CREATE INDEX IF NOT EXISTS idx_phv_case ON attorney_pro_hac_vice(case_id);
CREATE INDEX IF NOT EXISTS idx_phv_court_status ON attorney_pro_hac_vice(court_id, status);
