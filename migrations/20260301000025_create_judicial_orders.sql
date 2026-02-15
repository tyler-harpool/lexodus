CREATE TABLE IF NOT EXISTS judicial_orders (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL REFERENCES courts(id),
    case_id         UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    judge_id        UUID NOT NULL REFERENCES judges(id),
    order_type      TEXT NOT NULL
        CHECK (order_type IN ('Scheduling','Protective','Restraining','Dismissal','Sentencing','Detention','Release','Discovery','Sealing','Contempt','Procedural','Standing','Other')),
    title           TEXT NOT NULL,
    content         TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'Draft'
        CHECK (status IN ('Draft','Pending Signature','Signed','Filed','Vacated','Amended','Superseded')),
    is_sealed       BOOLEAN NOT NULL DEFAULT FALSE,
    signer_name     TEXT,
    signed_at       TIMESTAMPTZ,
    signature_hash  TEXT,
    issued_at       TIMESTAMPTZ,
    effective_date  TIMESTAMPTZ,
    expiration_date TIMESTAMPTZ,
    related_motions UUID[] NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_judicial_orders_court ON judicial_orders(court_id);
CREATE INDEX idx_judicial_orders_court_case ON judicial_orders(court_id, case_id);
CREATE INDEX idx_judicial_orders_court_judge ON judicial_orders(court_id, judge_id);
CREATE INDEX idx_judicial_orders_court_type ON judicial_orders(court_id, order_type);
CREATE INDEX idx_judicial_orders_court_status ON judicial_orders(court_id, status);
CREATE INDEX idx_judicial_orders_court_sealed ON judicial_orders(court_id, is_sealed);
CREATE INDEX idx_judicial_orders_court_issued ON judicial_orders(court_id, issued_at);
