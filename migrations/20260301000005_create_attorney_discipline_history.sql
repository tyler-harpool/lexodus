CREATE TABLE attorney_discipline_history (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id       TEXT NOT NULL REFERENCES courts(id),
    attorney_id    UUID NOT NULL REFERENCES attorneys(id) ON DELETE CASCADE,
    action_date    TIMESTAMPTZ NOT NULL,
    jurisdiction   TEXT NOT NULL,
    action_type    TEXT NOT NULL
        CHECK (action_type IN ('Warning','Reprimand','Probation','Suspension','Disbarment','Reinstatement','Other')),
    description    TEXT NOT NULL,
    case_number    TEXT,
    effective_date TIMESTAMPTZ NOT NULL,
    end_date       TIMESTAMPTZ,
    public_record  BOOLEAN NOT NULL DEFAULT TRUE,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_discipline_attorney ON attorney_discipline_history(attorney_id);
CREATE INDEX idx_discipline_court ON attorney_discipline_history(court_id);
