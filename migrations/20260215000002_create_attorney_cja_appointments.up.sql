CREATE TABLE IF NOT EXISTS attorney_cja_appointments (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id          TEXT NOT NULL REFERENCES courts(id),
    attorney_id       UUID NOT NULL REFERENCES attorneys(id) ON DELETE CASCADE,
    case_id           UUID REFERENCES criminal_cases(id) ON DELETE SET NULL,
    appointment_date  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    termination_date  TIMESTAMPTZ,
    voucher_status    TEXT NOT NULL DEFAULT 'Pending'
        CHECK (voucher_status IN ('Pending', 'Submitted', 'Approved', 'Denied', 'Paid')),
    voucher_amount    FLOAT8,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cja_appt_court ON attorney_cja_appointments(court_id);
CREATE INDEX IF NOT EXISTS idx_cja_appt_attorney ON attorney_cja_appointments(attorney_id);
CREATE INDEX IF NOT EXISTS idx_cja_appt_case ON attorney_cja_appointments(case_id);
CREATE INDEX IF NOT EXISTS idx_cja_appt_voucher_status ON attorney_cja_appointments(court_id, voucher_status);
